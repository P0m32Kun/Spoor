use crate::string_fold::EXPR_PLACEHOLDER;

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
}
