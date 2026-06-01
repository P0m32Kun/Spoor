use crate::dedup::dedup_findings;
use crate::finding::{Finding, FindingKind, Origin};
use crate::secret_patterns::classify_secret_token;

/// Scan HTML/JSON/other page bodies for secrets (no path/endpoint extraction).
pub fn collect_secrets_from_document(source: &str, label: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    scan_token_like(source, &mut findings);
    findings.extend(collect_inline_script_secrets(source, label));
    dedup_findings(findings)
}

fn scan_token_like(source: &str, findings: &mut Vec<Finding>) {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    let mut push_at = |byte_offset: usize, value: &str, secret_type: &str, severity: &str| {
        let (line, column) = offset_to_line_col(byte_offset, &line_starts);
        findings.push(Finding::secret(
            value,
            secret_type,
            severity,
            Origin {
                pattern: "page_text".into(),
                snippet: Some(snippet_at(source, byte_offset, 80)),
                line: Some(line),
                column: Some(column),
            },
        ));
    };

    let bytes = source.as_bytes();
    for (byte_idx, ch) in source.char_indices() {
        if !ch.is_ascii() {
            continue;
        }
        for len in [20, 39] {
            if byte_idx + len <= bytes.len() && source.is_char_boundary(byte_idx + len) {
                let candidate = &source[byte_idx..byte_idx + len];
                if let Some((ty, sev)) = classify_secret_token(candidate) {
                    push_at(byte_idx, candidate, ty, sev);
                }
            }
        }
        if let Some(token) = extract_variable_token(source, byte_idx) {
            if let Some((ty, sev)) = classify_secret_token(&token) {
                push_at(byte_idx, &token, ty, sev);
            }
        }
    }

    if let Some(pem) = extract_pem_block(source) {
        if let Some((ty, sev)) = classify_secret_token(&pem) {
            let offset = source.find(&pem).unwrap_or(0);
            push_at(offset, &pem, ty, sev);
        }
    }
}

fn extract_variable_token(source: &str, start: usize) -> Option<String> {
    let rest = &source[start..];
    let consumed = if rest.starts_with("sk-") {
        3
    } else if rest.starts_with("ghp_") {
        4
    } else if rest.starts_with("github_pat_") {
        11
    } else {
        return None;
    };
    let mut end = consumed;
    for ch in rest[consumed..].chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    if end > consumed + 8 {
        Some(source[start..start + end].to_string())
    } else {
        None
    }
}

fn extract_pem_block(source: &str) -> Option<String> {
    let start = source.find("-----BEGIN")?;
    let tail = &source[start..];
    let end_marker = tail.find("-----END")?;
    let after_end = tail[end_marker..].find('\n').map(|i| end_marker + i + 1).unwrap_or(tail.len());
    let block_end = tail[..after_end]
        .rfind("-----")
        .map(|i| i + tail[end_marker..].find('\n').unwrap_or(tail.len() - end_marker))
        .unwrap_or(after_end);
    Some(source[start..start + block_end.min(tail.len())].to_string())
}

fn collect_inline_script_secrets(source: &str, label: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lower = source.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(open) = lower[search_from..].find("<script") {
        let abs_open = search_from + open;
        let Some(tag_end) = lower[abs_open..].find('>') else {
            break;
        };
        let tag_end = abs_open + tag_end;
        if lower[abs_open..tag_end].contains("src=") {
            search_from = tag_end + 1;
            continue;
        }
        let Some(close) = lower[tag_end..].find("</script>") else {
            break;
        };
        let body_start = tag_end + 1;
        let body_end = tag_end + close;
        let script = &source[body_start..body_end];
        if !script.trim().is_empty() {
            let js_findings = crate::analyzer::Analyzer::new(script, Some(label)).collect_findings();
            findings.extend(
                js_findings
                    .into_iter()
                    .filter(|f| f.kind == FindingKind::Secret)
                    .map(|mut f| {
                        f.origin.pattern = "inline_script".into();
                        if !f.tags.iter().any(|t| t == "inline_script") {
                            f.tags.push("inline_script".into());
                        }
                        f
                    }),
            );
        }
        search_from = body_end + 9;
    }
    findings
}

fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (u32, u32) {
    let line_idx = line_starts.partition_point(|&start| start <= offset).saturating_sub(1);
    let line_start = line_starts.get(line_idx).copied().unwrap_or(0);
    (
        (line_idx + 1) as u32,
        (offset - line_start + 1) as u32,
    )
}

fn snippet_at(source: &str, offset: usize, max_len: usize) -> String {
    let end = (offset + max_len).min(source.len());
    source[offset..end].replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::FindingKind;

    #[test]
    fn finds_secret_in_html_comment() {
        let html = r#"<!-- AKIAIOSFODNN7EXAMPLE backup -->
        <html><body>ok</body></html>"#;
        let secrets: Vec<_> = collect_secrets_from_document(html, "http://x/page")
            .into_iter()
            .filter(|f| f.kind == FindingKind::Secret)
            .collect();
        assert!(secrets.iter().any(|f| f.value == "AKIAIOSFODNN7EXAMPLE"));
        assert!(secrets.iter().all(|f| f.origin.pattern == "page_text"));
    }

    #[test]
    fn finds_secret_in_inline_script() {
        let html = r#"<html><script>const k="sk-live-test-key-abcdef";</script></html>"#;
        let secrets = collect_secrets_from_document(html, "http://x/page");
        assert!(secrets.iter().any(|f| f.value.contains("sk-live-test")));
    }

    #[test]
    fn page_mode_no_endpoints() {
        let html = r#"<html><script>fetch("/api/admin");</script></html>"#;
        let findings = collect_secrets_from_document(html, "http://x/page");
        assert!(!findings.iter().any(|f| f.kind == FindingKind::Endpoint));
    }
}
