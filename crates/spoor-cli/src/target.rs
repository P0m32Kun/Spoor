use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// Returns true when `s` looks like an HTTP(S) URL pointing at a fetchable resource.
pub fn is_http_url(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("http://") || s.starts_with("https://"))
        && url::Url::parse(s)
            .ok()
            .is_some_and(|u| u.host().is_some())
}

/// Parse newline-delimited URL list (blank lines and `#` comments skipped).
pub fn parse_url_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| is_http_url(line))
        .map(str::to_string)
        .collect()
}

/// What `spoor scan <TARGET>` resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanTarget {
    /// One JS URL.
    Url(String),
    /// Many JS URLs (from a list file or stdin).
    UrlList(Vec<String>),
    /// Local JS file (paths/apis/keys or legacy scan).
    LocalFile(String),
}

pub fn resolve_scan_target(target: &str) -> io::Result<ScanTarget> {
    let target = target.trim();
    if target.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "scan target must not be empty",
        ));
    }

    if target == "-" {
        let stdin = io::stdin();
        let mut content = String::new();
        for line in stdin.lock().lines() {
            content.push_str(&line?);
            content.push('\n');
        }
        let urls = parse_url_lines(&content);
        if urls.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "stdin contained no http(s) URLs",
            ));
        }
        return Ok(ScanTarget::UrlList(urls));
    }

    if is_http_url(target) {
        return Ok(ScanTarget::Url(target.to_string()));
    }

    let path = Path::new(target);
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let urls = parse_url_lines(&content);
        if !urls.is_empty() {
            return Ok(ScanTarget::UrlList(urls));
        }
        return Ok(ScanTarget::LocalFile(
            fs::canonicalize(path)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| target.to_string()),
        ));
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("target not a URL and not a readable file: {target}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_url() {
        assert!(is_http_url("http://192.168.1.8:18080/1.js"));
        assert!(!is_http_url("/api/x"));
        assert!(!is_http_url("./app.js"));
    }

    #[test]
    fn parses_url_list_with_comments() {
        let text = "# katana output\nhttp://a/1.js\n\nhttp://b/2.js\n";
        assert_eq!(
            parse_url_lines(text),
            vec![
                "http://a/1.js".to_string(),
                "http://b/2.js".to_string(),
            ]
        );
    }
}
