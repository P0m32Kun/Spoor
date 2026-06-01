use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use spoor_core::{
    is_acceptable_http_status, is_http_url, Finding, FindingKind,
};

pub struct HttpVerifier {
    client: Client,
}

impl HttpVerifier {
    pub fn new(insecure: bool, timeout: Duration) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(timeout)
            .redirect(Policy::limited(5))
            .danger_accept_invalid_certs(insecure)
            .build()?;
        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn probe(&self, url: &str) -> Option<u16> {
        if !is_http_url(url) {
            return None;
        }
        match self.client.head(url).send() {
            Ok(resp) if resp.status().as_u16() != 405 => Some(resp.status().as_u16()),
            _ => self
                .client
                .get(url)
                .send()
                .ok()
                .map(|resp| resp.status().as_u16()),
        }
    }
}

/// Keep secrets always; drop path/endpoint rows whose resolved URL did not probe successfully.
pub fn verify_findings(findings: Vec<Finding>, verifier: &HttpVerifier) -> Vec<Finding> {
    findings
        .into_iter()
        .filter_map(|mut finding| {
            if finding.kind == FindingKind::Secret {
                return Some(finding);
            }

            if finding.value.starts_with("ws://") || finding.value.starts_with("wss://") {
                return Some(finding);
            }

            if !is_http_url(&finding.value) {
                return None;
            }

            let status = verifier.probe(&finding.value)?;
            if !is_acceptable_http_status(status) {
                return None;
            }
            finding.http_status = Some(status);
            Some(finding)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spoor_core::Origin;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn endpoint(value: &str) -> Finding {
        Finding::endpoint(
            value,
            "GET",
            Origin {
                pattern: "fetch".into(),
                snippet: None,
                line: None,
                column: None,
            },
        )
    }

    #[test]
    fn keeps_verified_endpoint_drops_404() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let base = rt.block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("HEAD"))
                .and(path("/api/admin"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&server)
                .await;
            Mock::given(method("HEAD"))
                .and(path("/missing"))
                .respond_with(ResponseTemplate::new(404))
                .mount(&server)
                .await;
            server.uri()
        });

        let verifier = HttpVerifier::new(true, Duration::from_secs(5)).unwrap();
        let out = verify_findings(
            vec![
                endpoint(&format!("{base}/api/admin")),
                endpoint(&format!("{base}/missing")),
            ],
            &verifier,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].value, format!("{base}/api/admin"));
        assert_eq!(out[0].http_status, Some(200));
    }

    #[test]
    fn keeps_secrets_without_probe() {
        let verifier = HttpVerifier::new(true, Duration::from_secs(5)).unwrap();
        let secret = Finding::secret(
            "AKIAIOSFODNN7EXAMPLE",
            "aws_access_key",
            "critical",
            Origin {
                pattern: "string_literal".into(),
                snippet: None,
                line: None,
                column: None,
            },
        );
        let out = verify_findings(vec![secret], &verifier);
        assert_eq!(out.len(), 1);
        assert!(out[0].http_status.is_none());
    }
}
