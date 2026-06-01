use std::collections::HashMap;

use crate::string_fold::EXPR_PLACEHOLDER;
use url::Url;

/// Regex-free scan for absolute URL strings in raw source (bootstrap when findings lack one).
fn scan_source_origins(source: &str, counts: &mut HashMap<String, u32>, weight: u32) {
    for prefix in ["https://", "http://", "wss://", "ws://"] {
        let mut rest = source;
        while let Some(idx) = rest.find(prefix) {
            let slice = &rest[idx..];
            let end = slice
                .char_indices()
                .find(|(_, c)| matches!(c, ' ' | '\t' | '\n' | '\r' | '"' | '\'' | '`' | ')' | '(' | ',' | ';'))
                .map(|(i, _)| i)
                .unwrap_or(slice.len());
            if let Some(origin) = origin_from_urlish(&slice[..end]) {
                *counts.entry(origin).or_default() += weight;
            }
            rest = &slice[1..];
        }
    }
}

/// Extract `scheme://host` from an absolute or protocol-relative URL string.
pub fn origin_from_urlish(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let normalized = if raw.starts_with("//") {
        format!("https:{raw}")
    } else {
        raw.to_string()
    };
    let url = Url::parse(&normalized).ok()?;
    let host = url.host_str()?;
    Some(format!("{}://{}", url.scheme(), host))
}

/// Guess page origin from file path (Katana-style output dirs often embed the hostname).
fn infer_origin_from_file_path(file: &str) -> Option<String> {
    if file == "<stdin>" {
        return None;
    }
    for segment in file.split(['/', '\\']) {
        let seg = segment.trim();
        if seg.is_empty() || seg.ends_with(".js") || seg.ends_with(".ts") || seg.ends_with(".map") {
            continue;
        }
        if seg.contains('.') && !seg.contains(' ') && seg.len() > 3 {
            // e.g. .../target.example.com/static/app.js
            if seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
                return Some(format!("https://{seg}"));
            }
        }
    }
    None
}

/// Infer the most likely page origin for resolving relative endpoints in this file.
pub fn infer_base_url(source: &str, file: &str, findings: &[crate::finding::Finding]) -> Option<String> {
    use crate::finding::FindingKind;

    if let Some(origin) = infer_origin_from_file_path(file) {
        return Some(origin);
    }

    let mut counts: HashMap<String, u32> = HashMap::new();

    for finding in findings {
        let weight = match finding.kind {
            FindingKind::Endpoint => 3,
            FindingKind::Path => 1,
            FindingKind::Secret => 0,
        };
        if weight == 0 {
            continue;
        }
        if let Some(origin) = origin_from_urlish(&finding.value) {
            *counts.entry(origin).or_default() += weight;
        }
    }

    scan_source_origins(source, &mut counts, 1);

    let has_relative_endpoints = findings.iter().any(|f| {
        f.kind == FindingKind::Endpoint
            && !is_absolute_url(&f.value)
            && !f.value.starts_with("//")
    });

    let mut candidates: Vec<(String, u32)> = counts.into_iter().collect();
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    for (origin, _) in &candidates {
        if !has_relative_endpoints || origin_covers_relative_endpoints(origin, findings) {
            return Some(origin.clone());
        }
    }

    None
}

fn first_path_segment(path_or_url: &str) -> Option<String> {
    let path = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        Url::parse(path_or_url).ok()?.path().to_string()
    } else {
        path_or_url.to_string()
    };
    let seg = path.trim_start_matches('/').split('/').next()?;
    if seg.is_empty() {
        None
    } else {
        Some(seg.to_string())
    }
}

/// True when this origin already appears on an absolute URL whose first path segment
/// matches a relative endpoint in the same file (avoids picking a unrelated external API host).
fn origin_covers_relative_endpoints(origin: &str, findings: &[crate::finding::Finding]) -> bool {
    use crate::finding::FindingKind;

    let relative_segments: Vec<String> = findings
        .iter()
        .filter(|f| {
            f.kind == FindingKind::Endpoint
                && !is_absolute_url(&f.value)
                && !f.value.starts_with("//")
        })
        .filter_map(|f| first_path_segment(&f.value))
        .collect();

    if relative_segments.is_empty() {
        return true;
    }

    let absolute_segments: Vec<String> = findings
        .iter()
        .filter(|f| f.kind == FindingKind::Endpoint && is_absolute_url(&f.value))
        .filter(|f| origin_from_urlish(&f.value).as_deref() == Some(origin))
        .filter_map(|f| first_path_segment(&f.value))
        .collect();

    if absolute_segments.is_empty() {
        // Origin only from source scan — allow
        return true;
    }

    if relative_segments
        .iter()
        .any(|rel| absolute_segments.iter().any(|abs| abs == rel))
    {
        return true;
    }

    // Same host, but e.g. billing `/invoices` vs first-party `/api/*` — do not guess.
    absolute_segments
        .iter()
        .any(|abs| looks_first_party_api_segment(abs))
        && relative_segments
            .iter()
            .any(|rel| looks_first_party_api_segment(rel))
}

fn looks_first_party_api_segment(segment: &str) -> bool {
    matches!(
        segment,
        "api" | "v1" | "v2" | "v3" | "graphql" | "rest" | "socket" | "ws"
    )
}

/// Heuristic: does this folded string look worth treating as a URL or path candidate?
pub fn maybe_url(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s == EXPR_PLACEHOLDER {
        return false;
    }
    if s.chars().all(|c| c.is_ascii_whitespace()) {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("data:")
        || lower.starts_with("tel:")
        || lower.starts_with("mailto:")
        || lower.starts_with("javascript:")
        || lower.starts_with("blob:")
    {
        return false;
    }
    if s == EXPR_PLACEHOLDER
        || s.starts_with(EXPR_PLACEHOLDER) && !s.contains('/') && !s.contains('.')
    {
        // bare EXPR with no path-like chars
        if s.len() <= EXPR_PLACEHOLDER.len() + 2 {
            return false;
        }
    }
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("//") {
        return true;
    }
    if s.starts_with('/') && s.len() > 1 {
        return true;
    }
    if s.contains("://") {
        return true;
    }
    // relative paths like api/v1 or ./assets
    if s.starts_with("./") || s.starts_with("../") {
        return true;
    }
    if s.contains('/') && !s.contains(' ') {
        return true;
    }
    // host-like: example.com/path
    if s.contains('.') && s.contains('/') {
        return true;
    }
    false
}

/// Like [`maybe_url`] but rejects folded strings that still contain unresolved `EXPR`.
pub fn resolved_maybe_url(s: &str) -> bool {
    !s.contains(EXPR_PLACEHOLDER) && maybe_url(s)
}

/// Extract query parameter names from a URL or path string (`?a=1&b=2` → `["a", "b"]`).
pub fn query_param_names(url: &str) -> Vec<String> {
    let Some(query_start) = url.find('?') else {
        return Vec::new();
    };
    let query = &url[query_start + 1..];
    let query = query.split('#').next().unwrap_or(query);
    query
        .split('&')
        .filter_map(|pair| pair.split('=').next())
        .filter(|key| !key.is_empty())
        .map(|key| key.to_string())
        .collect()
}

/// True when the string is an absolute URL with scheme and host.
pub fn is_absolute_url(s: &str) -> bool {
    let s = s.trim();
    if !(s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ws://")
        || s.starts_with("wss://"))
    {
        return false;
    }
    Url::parse(s)
        .ok()
        .is_some_and(|u| u.host().is_some())
}

/// Resolve a folded endpoint string to an absolute URL when possible.
///
/// - `http(s)://` / `ws(s)://` 原样校验
/// - `//host/path` → `https://host/path`
/// - `/path` 或相对路径 → 需要推断出的 page origin
/// - WebSocket 相对路径 → 按 base 的 host 使用 `ws`/`wss`
pub fn resolve_endpoint_url(raw: &str, base_url: Option<&str>, websocket: bool) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.contains(EXPR_PLACEHOLDER) {
        return None;
    }

    if raw.starts_with("//") {
        return Url::parse(&format!("https:{raw}"))
            .ok()
            .filter(|u| u.host().is_some())
            .map(|u| u.to_string());
    }

    if is_absolute_url(raw) {
        return Url::parse(raw).ok().map(|u| u.to_string());
    }

    let base_str = base_url?;
    let mut base = Url::parse(base_str).ok()?;
    if base.host().is_none() {
        return None;
    }

    if websocket {
        let scheme = match base.scheme() {
            "https" | "wss" => "wss",
            "http" | "ws" => "ws",
            _ => "wss",
        };
        base.set_scheme(scheme).ok()?;
    }

    base.join(raw).ok().map(|u| u.to_string())
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

    #[test]
    fn resolves_protocol_relative() {
        assert_eq!(
            resolve_endpoint_url("//cdn.example.com/app.js", None, false).as_deref(),
            Some("https://cdn.example.com/app.js")
        );
    }

    #[test]
    fn resolves_relative_with_js_file_url() {
        assert_eq!(
            resolve_endpoint_url(
                "/api/admin",
                Some("http://192.168.1.8:18080/1.js"),
                false
            )
            .as_deref(),
            Some("http://192.168.1.8:18080/api/admin")
        );
    }

    #[test]
    fn resolves_relative_with_base() {
        assert_eq!(
            resolve_endpoint_url(
                "/api/v2/users?id=1",
                Some("https://app.example.com/static/bundle.js"),
                false
            )
            .as_deref(),
            Some("https://app.example.com/api/v2/users?id=1")
        );
    }

    #[test]
    fn resolves_websocket_relative_with_base() {
        assert_eq!(
            resolve_endpoint_url("/socket", Some("https://api.example.com/"), true).as_deref(),
            Some("wss://api.example.com/socket")
        );
    }

    use crate::finding::{Finding, Origin};

    #[test]
    fn infer_skips_unrelated_external_origin() {
        let findings = vec![
            Finding::endpoint(
                "https://billing.example.com/invoices",
                "POST",
                Origin {
                    pattern: "axios.post".into(),
                    snippet: None,
                    line: None,
                    column: None,
                },
            ),
            Finding::endpoint(
                "/api/v2/users",
                "GET",
                Origin {
                    pattern: "fetch".into(),
                    snippet: None,
                    line: None,
                    column: None,
                },
            ),
        ];
        assert!(infer_base_url("", "app.js", &findings).is_none());
    }

    #[test]
    fn infer_base_from_absolute_finding() {
        let findings = vec![
            Finding::endpoint(
                "https://app.example.com/api/boot.js",
                "GET",
                Origin {
                    pattern: "fetch".into(),
                    snippet: None,
                    line: None,
                    column: None,
                },
            ),
            Finding::endpoint(
                "/api/v1",
                "GET",
                Origin {
                    pattern: "fetch".into(),
                    snippet: None,
                    line: None,
                    column: None,
                },
            ),
        ];
        assert_eq!(
            infer_base_url("", "app.js", &findings).as_deref(),
            Some("https://app.example.com")
        );
    }

    #[test]
    fn infer_base_from_source_scan() {
        let source = r#"const x = "https://cdn.example.com/lib.js"; fetch("/api");"#;
        assert_eq!(
            infer_base_url(source, "x.js", &[]).as_deref(),
            Some("https://cdn.example.com")
        );
    }

    #[test]
    fn relative_without_base_stays_unresolved() {
        assert!(resolve_endpoint_url("/api/v1", None, false).is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_noise() {
        assert!(!maybe_url(""));
        assert!(!maybe_url("EXPR"));
        assert!(!maybe_url("data:text/html,foo"));
        assert!(!maybe_url("javascript:void(0)"));
    }

    #[test]
    fn accepts_paths_and_urls() {
        assert!(maybe_url("/api/v1/users"));
        assert!(maybe_url("https://example.com/x"));
        assert!(maybe_url("//cdn.example.com/app.js"));
        assert!(maybe_url("api/v2/auth"));
    }

    #[test]
    fn resolved_maybe_url_rejects_expr() {
        assert!(!resolved_maybe_url("EXPR/users"));
        assert!(!resolved_maybe_url("/api/EXPR"));
        assert!(resolved_maybe_url("/api/v1/users"));
    }

    #[test]
    fn query_param_names_parses_keys() {
        assert_eq!(
            query_param_names("/api/users?id=1&sort=asc"),
            vec!["id", "sort"]
        );
        assert_eq!(
            query_param_names("https://x.com/a?token=abc#frag"),
            vec!["token"]
        );
        assert!(query_param_names("/no/query").is_empty());
    }
}
