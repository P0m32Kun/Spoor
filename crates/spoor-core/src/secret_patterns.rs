const AWS_KEY_PREFIX: &str = "AKIA";
const GCP_API_KEY_PREFIX: &str = "AIza";
const GCP_API_KEY_LEN: usize = 39;
const PEM_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----";
const PEM_RSA_PRIVATE_KEY: &str = "-----BEGIN RSA PRIVATE KEY-----";

pub fn looks_like_aws_access_key(value: &str) -> bool {
    value.starts_with(AWS_KEY_PREFIX)
        && value.len() == 20
        && value
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

pub fn looks_like_gcp_api_key(value: &str) -> bool {
    value.starts_with(GCP_API_KEY_PREFIX)
        && value.len() == GCP_API_KEY_LEN
        && value
            .bytes()
            .skip(GCP_API_KEY_PREFIX.len())
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn looks_like_private_key_pem(value: &str) -> bool {
    value.contains(PEM_PRIVATE_KEY) || value.contains(PEM_RSA_PRIVATE_KEY)
}

pub fn classify_secret_token(value: &str) -> Option<(&'static str, &'static str)> {
    if looks_like_aws_access_key(value) {
        return Some(("aws_access_key", "critical"));
    }
    if looks_like_gcp_api_key(value) {
        return Some(("gcp_api_key", "high"));
    }
    if looks_like_private_key_pem(value) {
        return Some(("gcp_private_key", "critical"));
    }
    if value.starts_with("sk-") && value.len() > 8 {
        return Some(("api_key", "high"));
    }
    if value.starts_with("ghp_") || value.starts_with("github_pat_") {
        return Some(("github_token", "critical"));
    }
    None
}

/// True when URL path looks like a JavaScript/TypeScript asset.
pub fn is_js_resource_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url.trim()) else {
        return false;
    };
    let path = parsed.path().to_ascii_lowercase();
    [
        ".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx", ".vue", ".map",
    ]
    .iter()
    .any(|ext| path.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_url_by_extension() {
        assert!(is_js_resource_url("http://192.168.1.8:18080/1.js"));
        assert!(is_js_resource_url("http://x.com/app.chunk.js?v=1"));
        assert!(!is_js_resource_url("http://192.168.1.8:18080/admin"));
        assert!(!is_js_resource_url("http://192.168.1.8:18080/"));
    }
}
