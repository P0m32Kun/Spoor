mod fetch;
mod target;
mod verify;

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use spoor_core::{
    collect_secrets_from_document, is_js_resource_url, Analyzer, FindingKind, OutputOptions,
    ScanResult, prepare_for_output,
};

use crate::fetch::fetch_text;
use crate::target::{resolve_scan_target, ScanTarget};
use crate::verify::{verify_findings, HttpVerifier};

#[derive(Parser)]
#[command(
    name = "spoor",
    about = "Spoor — extract paths, API endpoints, and secrets from JavaScript assets",
    version,
    author
)]
struct Cli {
    /// Override JS source URL when scanning a local file (paths/apis/keys).
    #[arg(long, global = true, value_name = "URL")]
    from_url: Option<String>,

    /// Skip HTTP probing of discovered URLs.
    #[arg(long, global = true)]
    no_verify: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan URL(s): JS assets (path+endpoint+secret) or HTML/pages (secret only)
    Scan {
        /// http(s) URL, URL list file (one per line), or `-`
        #[arg(value_name = "URL|FILE")]
        target: String,
        #[arg(short, long, help = "Write JSON to file instead of stdout")]
        output: Option<PathBuf>,
        #[arg(short, long, help = "Emit JSONL (one finding per line, each with file field)")]
        jsonl: bool,
    },
    /// Extract path-like strings only (local file or stdin `-`)
    Paths {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Extract API endpoints (local file)
    Apis {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Extract secrets (local file)
    Keys {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan {
            target,
            output,
            jsonl,
        } => run_scan_urls(&target, output.as_ref(), jsonl, cli.no_verify)?,
        Commands::Paths { path, output } => {
            run_local_scan(
                &path,
                output.as_ref(),
                false,
                Some(FindingKind::Path),
                cli.from_url.as_deref(),
                cli.from_url.is_some() && !cli.no_verify,
            )?;
        }
        Commands::Apis { path, output } => {
            run_local_scan(
                &path,
                output.as_ref(),
                false,
                Some(FindingKind::Endpoint),
                cli.from_url.as_deref(),
                cli.from_url.is_some() && !cli.no_verify,
            )?;
        }
        Commands::Keys { path, output } => {
            run_local_scan(
                &path,
                output.as_ref(),
                false,
                Some(FindingKind::Secret),
                cli.from_url.as_deref(),
                false,
            )?;
        }
    }
    Ok(())
}

fn run_scan_urls(
    target: &str,
    output: Option<&PathBuf>,
    jsonl: bool,
    no_verify: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve_scan_target(target).map_err(|e| e.to_string())?;
    let verify = !no_verify;

    match resolved {
        ScanTarget::Url(url) => {
            let payload = scan_one_url(&url, None, jsonl, verify)?;
            write_payload(output, &payload)?;
        }
        ScanTarget::UrlList(urls) => {
            let mut blocks = Vec::with_capacity(urls.len());
            for url in urls {
                blocks.push(scan_one_url(&url, None, jsonl, verify)?);
            }
            let payload = if jsonl {
                blocks.join("")
            } else {
                let results: Vec<ScanResult> = blocks
                    .iter()
                    .map(|s| serde_json::from_str(s.trim()))
                    .collect::<Result<_, _>>()?;
                serde_json::to_string_pretty(&results)?
            };
            write_payload(output, &payload)?;
        }
        ScanTarget::LocalFile(path) => {
            run_local_scan(
                &PathBuf::from(path),
                output,
                jsonl,
                None,
                None,
                false,
            )?;
        }
    }
    Ok(())
}

fn scan_one_url(
    page_url: &str,
    kind_filter: Option<FindingKind>,
    jsonl: bool,
    verify: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = HttpVerifier::new(true, Duration::from_secs(12))?;
    let source = fetch_text(client.client(), page_url).map_err(|e| {
        format!("failed to fetch {page_url}: {e}")
    })?;

    let mut findings = if is_js_resource_url(page_url) {
        analyze_and_prepare(&source, page_url, page_url, jsonl, verify, &client)?
    } else {
        let mut findings = collect_secrets_from_document(&source, page_url);
        if jsonl {
            for finding in &mut findings {
                finding.file = Some(page_url.to_string());
            }
        }
        findings
    };

    if let Some(kind) = kind_filter {
        findings.retain(|f| f.kind == kind);
    }

    let result = ScanResult {
        file: page_url.to_string(),
        findings,
    };

    Ok(if jsonl {
        result
            .findings
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n")
            + if result.findings.is_empty() { "" } else { "\n" }
    } else {
        serde_json::to_string_pretty(&result)?
    })
}

fn analyze_and_prepare(
    source: &str,
    label: &str,
    from_url: &str,
    embed_file: bool,
    verify: bool,
    verifier: &HttpVerifier,
) -> Result<Vec<spoor_core::Finding>, Box<dyn std::error::Error>> {
    let analyzer = Analyzer::new(source, Some(label));
    let findings = analyzer.collect_findings();

    let mut findings = prepare_for_output(
        findings,
        &OutputOptions {
            file: label.to_string(),
            source,
            from_url: Some(from_url),
            embed_file,
        },
    );

    if verify {
        findings = verify_findings(findings, verifier);
    }

    Ok(findings)
}

fn run_local_scan(
    path: &PathBuf,
    output: Option<&PathBuf>,
    jsonl: bool,
    kind_filter: Option<FindingKind>,
    from_url: Option<&str>,
    verify: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (source, file_label) = read_local_input(path)?;
    let verifier = if verify {
        Some(HttpVerifier::new(true, Duration::from_secs(8))?)
    } else {
        None
    };

    let mut findings = prepare_for_output(
        Analyzer::new(&source, Some(&file_label)).collect_findings(),
        &OutputOptions {
            file: file_label.clone(),
            source: &source,
            from_url,
            embed_file: jsonl,
        },
    );

    if let Some(v) = &verifier {
        findings = verify_findings(findings, v);
    }

    if let Some(kind) = kind_filter {
        findings.retain(|f| f.kind == kind);
    }

    let result = ScanResult {
        file: file_label,
        findings,
    };

    let payload = if jsonl {
        result
            .findings
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n")
            + if result.findings.is_empty() { "" } else { "\n" }
    } else {
        serde_json::to_string_pretty(&result)?
    };

    if let Some(out_path) = output {
        fs::write(out_path, &payload)?;
    } else {
        print!("{payload}");
    }
    Ok(())
}

fn write_payload(output: Option<&PathBuf>, payload: &str) -> io::Result<()> {
    if let Some(out_path) = output {
        fs::write(out_path, payload)?;
    } else {
        print!("{payload}");
    }
    Ok(())
}

fn read_local_input(path: &PathBuf) -> Result<(String, String), Box<dyn std::error::Error>> {
    if path.as_os_str() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok((buf, "<stdin>".into()));
    }
    let source = fs::read_to_string(path)?;
    let label = fs::canonicalize(path)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());
    Ok((source, label))
}
