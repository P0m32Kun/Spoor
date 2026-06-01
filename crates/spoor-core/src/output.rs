use crate::dedup::dedup_findings;
use crate::finding::{Finding, FindingKind};
use crate::url::{infer_base_url, is_absolute_url, maybe_url, resolve_endpoint_url};

/// Options applied when serializing findings for downstream tools.
#[derive(Debug, Clone)]
pub struct OutputOptions<'a> {
    /// Absolute or caller-provided path to the analyzed JS file.
    pub file: String,
    /// Raw JS source — used to infer page origin when `from_url` is absent.
    pub source: &'a str,
    /// URL the JS file was fetched from (e.g. `http://192.168.1.8:18080/1.js`).
    pub from_url: Option<&'a str>,
    /// Attach `file` on each finding (JSONL mode).
    pub embed_file: bool,
}

fn is_websocket_finding(finding: &Finding) -> bool {
    finding.method.as_deref() == Some("WS")
        || finding.value.starts_with("ws://")
        || finding.value.starts_with("wss://")
}

fn should_resolve_to_url(finding: &Finding) -> bool {
    match finding.kind {
        FindingKind::Secret => false,
        FindingKind::Endpoint => true,
        FindingKind::Path => maybe_url(&finding.value),
    }
}

fn resolve_finding_url(finding: &Finding, base: Option<&str>) -> Option<String> {
    let ws = is_websocket_finding(finding);
    resolve_endpoint_url(&finding.value, base, ws)
}

/// Prepare findings for CLI / pipeline output: attach source file, resolve URLs.
pub fn prepare_for_output(mut findings: Vec<Finding>, opts: &OutputOptions<'_>) -> Vec<Finding> {
    let base_url = opts
        .from_url
        .map(str::to_string)
        .or_else(|| infer_base_url(opts.source, &opts.file, &findings));

    for finding in &mut findings {
        if opts.embed_file {
            finding.file = Some(opts.file.clone());
        }

        if !should_resolve_to_url(finding) {
            continue;
        }

        let raw = finding.value.clone();
        if let Some(resolved) = resolve_finding_url(finding, base_url.as_deref()) {
            if resolved != raw {
                finding.raw = Some(raw);
            }
            finding.value = resolved;
        } else if finding.kind == FindingKind::Endpoint
            && !is_absolute_url(&finding.value)
            && opts.from_url.is_none()
        {
            finding.kind = FindingKind::Path;
            finding.method = None;
            finding.params = None;
        }
    }

    dedup_findings(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Origin;

    fn endpoint(value: &str) -> Finding {
        Finding::endpoint(
            value,
            "GET",
            Origin {
                pattern: "fetch".into(),
                snippet: None,
                line: Some(1),
                column: Some(1),
            },
        )
    }

    fn path(value: &str) -> Finding {
        Finding::path(
            value,
            Origin {
                pattern: "string_literal".into(),
                snippet: None,
                line: Some(1),
                column: Some(1),
            },
        )
    }

    #[test]
    fn from_url_joins_relative_path_and_endpoint() {
        let out = prepare_for_output(
            vec![path("/api/admin"), endpoint("/api/users")],
            &OutputOptions {
                file: "1.js".into(),
                source: "",
                from_url: Some("http://192.168.1.8:18080/1.js"),
                embed_file: false,
            },
        );
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].value, "http://192.168.1.8:18080/api/admin");
        assert_eq!(out[0].raw.as_deref(), Some("/api/admin"));
        assert_eq!(out[1].value, "http://192.168.1.8:18080/api/users");
    }

    #[test]
    fn embeds_file_and_resolves_endpoint_from_same_file_absolute() {
        let out = prepare_for_output(
            vec![
                endpoint("https://target.example.com/api/config.js"),
                endpoint("/api/v1/users"),
            ],
            &OutputOptions {
                file: "/data/katana/app.chunk.js".into(),
                source: r#"fetch("https://target.example.com/api/config.js"); fetch("/api/v1/users");"#,
                from_url: None,
                embed_file: true,
            },
        );
        let api = out
            .iter()
            .find(|f| f.raw.as_deref() == Some("/api/v1/users"))
            .expect("resolved endpoint");
        assert_eq!(api.value, "https://target.example.com/api/v1/users");
    }

    #[test]
    fn relative_endpoint_without_origin_becomes_path() {
        let out = prepare_for_output(
            vec![endpoint("/api/v1")],
            &OutputOptions {
                file: "app.js".into(),
                source: r#"fetch("/api/v1");"#,
                from_url: None,
                embed_file: true,
            },
        );
        assert_eq!(out[0].kind, FindingKind::Path);
    }

    #[test]
    fn secret_untouched() {
        let secret = Finding::secret(
            "AKIAIOSFODNN7EXAMPLE",
            "aws_access_key",
            "critical",
            Origin {
                pattern: "string_literal".into(),
                snippet: None,
                line: Some(2),
                column: None,
            },
        );
        let out = prepare_for_output(
            vec![secret],
            &OutputOptions {
                file: "/abs/path/leak.js".into(),
                source: "",
                from_url: None,
                embed_file: true,
            },
        );
        assert_eq!(out[0].kind, FindingKind::Secret);
        assert_eq!(out[0].value, "AKIAIOSFODNN7EXAMPLE");
    }
}
