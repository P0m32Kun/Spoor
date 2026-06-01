/// True for `http://` / `https://` URLs that can be probed.
pub fn is_http_url(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("http://") || s.starts_with("https://"))
        && url::Url::parse(s)
            .ok()
            .is_some_and(|u| u.host().is_some())
}

/// Status codes that indicate a URL is reachable / meaningful for recon output.
pub fn is_acceptable_http_status(status: u16) -> bool {
    matches!(
        status,
        200..=299 | 301 | 302 | 307 | 308 | 401 | 403 | 405
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_recon_status_codes() {
        for code in [200, 204, 301, 302, 401, 403, 405] {
            assert!(is_acceptable_http_status(code), "expected {code} ok");
        }
        assert!(!is_acceptable_http_status(404));
        assert!(!is_acceptable_http_status(500));
    }

    #[test]
    fn detects_http_urls() {
        assert!(is_http_url("http://192.168.1.8:18080/api"));
        assert!(!is_http_url("wss://x.com/ws"));
        assert!(!is_http_url("/api"));
    }
}
