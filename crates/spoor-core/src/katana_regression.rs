//! Regression fixtures simulating JS files Katana would hand to Spoor.

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    struct KatanaCase {
        name: &'static str,
        source: &'static str,
        paths: &'static [&'static str],
        endpoints: &'static [(&'static str, Option<&'static str>)],
        secrets: &'static [&'static str],
        secret_types: &'static [&'static str],
        forbid_endpoint_substrings: &'static [&'static str],
    }

    fn run_case(case: &KatanaCase) {
        let findings = Analyzer::new(case.source, Some(case.name)).collect_findings();

        let paths: HashSet<_> = findings
            .iter()
            .filter(|f| f.kind == FindingKind::Path)
            .map(|f| f.value.as_str())
            .collect();
        for want in case.paths {
            assert!(
                paths.contains(want),
                "{}: missing path {want:?}, got {paths:?}",
                case.name
            );
        }

        for (value, method) in case.endpoints {
            let matched = findings.iter().any(|f| {
                f.kind == FindingKind::Endpoint
                    && f.value == *value
                    && method.is_none_or(|m| f.method.as_deref() == Some(m))
            });
            assert!(
                matched,
                "{}: missing endpoint {value:?} method {:?}, endpoints: {:?}",
                case.name,
                method,
                findings
                    .iter()
                    .filter(|f| f.kind == FindingKind::Endpoint)
                    .map(|f| (&f.value, f.method.as_deref()))
                    .collect::<Vec<_>>()
            );
        }

        for want in case.secrets {
            assert!(
                findings.iter().any(|f| {
                    f.kind == FindingKind::Secret && (f.value == *want || f.value.contains(want))
                }),
                "{}: missing secret matching {want:?}",
                case.name
            );
        }

        for want in case.secret_types {
            assert!(
                findings.iter().any(|f| {
                    f.kind == FindingKind::Secret && f.secret_type.as_deref() == Some(*want)
                }),
                "{}: missing secret_type {want:?}",
                case.name
            );
        }

        for bad in case.forbid_endpoint_substrings {
            assert!(
                !findings.iter().any(|f| {
                    f.kind == FindingKind::Endpoint && f.value.contains(bad)
                }),
                "{}: forbidden endpoint substring {bad:?}",
                case.name
            );
        }
    }

    #[test]
    fn katana_spa_bundle() {
        run_case(&KatanaCase {
            name: "katana/spa_bundle.js",
            source: include_str!("../../../tests/fixtures/katana/spa_bundle.js"),
            paths: &["/app/dashboard", "/app/settings/:tab"],
            endpoints: &[
                ("/api/v2/users?id=1&role=admin", Some("GET")),
                ("https://billing.example.com/invoices", Some("POST")),
                ("/api/v2/profile", Some("GET")),
                ("/graphql", Some("POST")),
            ],
            secrets: &["AKIAIOSFODNN7EXAMPLE"],
            secret_types: &["aws_access_key"],
            forbid_endpoint_substrings: &["EXPR"],
        });
    }

    #[test]
    fn katana_legacy_admin() {
        run_case(&KatanaCase {
            name: "katana/legacy_admin.js",
            source: include_str!("../../../tests/fixtures/katana/legacy_admin.js"),
            paths: &[],
            endpoints: &[
                ("/legacy/users/list", Some("GET")),
                ("/legacy/users/export", Some("POST")),
                ("/legacy/session/check", Some("GET")),
                ("https://auth.example.com/legacy/login", Some("POST")),
                ("/legacy/logout", None),
                ("/legacy/home", Some("GET")),
                ("/legacy/help-popup", Some("GET")),
            ],
            secrets: &[],
            secret_types: &[],
            forbid_endpoint_substrings: &[],
        });
    }

    #[test]
    fn katana_api_clients() {
        run_case(&KatanaCase {
            name: "katana/api_clients.js",
            source: include_str!("../../../tests/fixtures/katana/api_clients.js"),
            paths: &[],
            endpoints: &[
                ("https://realtime.example.com/api/v1/ping", Some("GET")),
                ("/api/v1/health", Some("GET")),
                ("/api/v1/events", Some("POST")),
                ("https://hooks.example.com/callback", Some("DELETE")),
                ("/api/v1/agents", Some("GET")),
                ("/api/v1/agents/self", Some("PUT")),
                ("wss://realtime.example.com/ws", Some("WS")),
                ("/api/v1/ws", Some("WS")),
                ("https://api.example.com/graphql", Some("POST")),
            ],
            secrets: &[],
            secret_types: &[],
            forbid_endpoint_substrings: &[],
        });
    }

    #[test]
    fn katana_secrets_leak() {
        run_case(&KatanaCase {
            name: "katana/secrets_leak.js",
            source: include_str!("../../../tests/fixtures/katana/secrets_leak.js"),
            paths: &[],
            endpoints: &[],
            secrets: &[
                "AIzaSy000000000000000000000000000000000",
                "AIzaSy1111111111111111111111111111111",
                "sk-live-not-a-real-key-abcdef",
                "ghp_demoTokenNotReal123456789012345678",
            ],
            secret_types: &[
                "gcp_api_key",
                "firebase_api_key",
                "gcp_service_account_key",
                "object_literal_key",
                "github_token",
            ],
            forbid_endpoint_substrings: &[],
        });
    }

    #[test]
    fn katana_negative_controls() {
        run_case(&KatanaCase {
            name: "katana/negative_controls.js",
            source: include_str!("../../../tests/fixtures/katana/negative_controls.js"),
            paths: &["synthetic.bundle.js.map"],
            endpoints: &[],
            secrets: &[],
            secret_types: &[],
            forbid_endpoint_substrings: &["EXPR", "/not-an-xhr-endpoint", "/dynamic/"],
        });
    }
}
