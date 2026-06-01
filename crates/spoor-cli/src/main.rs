use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use spoor_core::{Analyzer, FindingKind, ScanResult};

#[derive(Parser)]
#[command(
    name = "spoor",
    about = "Spoor — extract paths, API endpoints, and secrets from JavaScript assets",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for paths, endpoints, and secrets (fetch / location / XHR + literals)
    Scan {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long, help = "Write JSON to file instead of stdout")]
        output: Option<PathBuf>,
        #[arg(short, long, help = "Emit JSONL (one finding per line)")]
        jsonl: bool,
    },
    /// Extract path-like strings only
    Paths {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Extract API endpoints (fetch, XHR, axios, …) — Phase 1+
    Apis {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Extract secrets and API keys — Phase 2+
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
            path,
            output,
            jsonl,
        } => run_scan(path, output, jsonl, None)?,
        Commands::Paths { path, output } => run_scan(path, output, false, Some(FindingKind::Path))?,
        Commands::Apis { path, output } => {
            run_scan(path, output, false, Some(FindingKind::Endpoint))?
        }
        Commands::Keys { path, output } => {
            run_scan(path, output, false, Some(FindingKind::Secret))?
        }
    }
    Ok(())
}

fn run_scan(
    path: PathBuf,
    output: Option<PathBuf>,
    jsonl: bool,
    kind_filter: Option<FindingKind>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (source, file_label) = read_input(&path)?;
    let analyzer = Analyzer::new(&source, Some(&file_label));
    let mut findings = analyzer.collect_findings();
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
            + "\n"
    } else {
        serde_json::to_string_pretty(&result)?
    };

    if let Some(out_path) = output {
        fs::write(&out_path, &payload)?;
    } else {
        print!("{payload}");
    }
    Ok(())
}

fn read_input(path: &PathBuf) -> Result<(String, String), Box<dyn std::error::Error>> {
    if path.as_os_str() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok((buf, "<stdin>".into()));
    }
    let source = fs::read_to_string(path)?;
    Ok((source, path.display().to_string()))
}
