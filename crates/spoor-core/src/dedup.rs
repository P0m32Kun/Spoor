use std::collections::HashMap;

use crate::finding::{Confidence, Finding, FindingKind, Origin};

fn kind_rank(kind: FindingKind) -> u8 {
    match kind {
        FindingKind::Endpoint => 2,
        FindingKind::Path => 1,
        FindingKind::Secret => 0,
    }
}

fn confidence_rank(confidence: Confidence) -> u8 {
    match confidence {
        Confidence::High => 3,
        Confidence::Medium => 2,
        Confidence::Low => 1,
    }
}

fn origin_richness(origin: &Origin) -> u32 {
    origin.snippet.is_some() as u32 + origin.line.is_some() as u32 + origin.column.is_some() as u32
}

fn finding_priority(f: &Finding) -> (u8, u8, u8, u32) {
    (
        kind_rank(f.kind),
        confidence_rank(f.confidence),
        u8::from(f.method.is_some()),
        origin_richness(&f.origin),
    )
}

fn prefer(a: &Finding, b: &Finding) -> bool {
    finding_priority(a) > finding_priority(b)
}

/// Deduplicate findings by `value`, keeping the highest-priority row per URL/path.
pub fn dedup_findings(findings: Vec<Finding>) -> Vec<Finding> {
    let mut best: HashMap<String, Finding> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for finding in findings {
        let key = finding.value.clone();
        match best.get_mut(&key) {
            Some(existing) => {
                if prefer(&finding, existing) {
                    *existing = finding;
                }
            }
            None => {
                order.push(key.clone());
                best.insert(key, finding);
            }
        }
    }

    order
        .into_iter()
        .filter_map(|key| best.remove(&key))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Origin;

    #[test]
    fn dedup_prefers_endpoint_over_literal() {
        let src = r#"fetch("/api/v1"); const x = "/api/v1";"#;
        let findings = crate::analyzer::Analyzer::new(src, Some("t.js")).collect_findings();
        let api_v1: Vec<_> = findings.iter().filter(|f| f.value == "/api/v1").collect();
        assert_eq!(api_v1.len(), 1);
        assert_eq!(api_v1[0].kind, FindingKind::Endpoint);
    }

    #[test]
    fn dedup_prefers_higher_confidence_same_kind() {
        let low = Finding {
            kind: FindingKind::Path,
            value: "/x".into(),
            confidence: Confidence::Low,
            origin: Origin {
                pattern: "a".into(),
                snippet: None,
                line: None,
                column: None,
            },
            method: None,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: vec![],
        };
        let high = Finding {
            confidence: Confidence::High,
            origin: Origin {
                pattern: "b".into(),
                snippet: Some("s".into()),
                line: Some(1),
                column: Some(2),
            },
            ..low.clone()
        };
        let out = dedup_findings(vec![low, high]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].confidence, Confidence::High);
    }

    #[test]
    fn dedup_prefers_method_set() {
        let without_method = Finding {
            kind: FindingKind::Endpoint,
            value: "/api".into(),
            confidence: Confidence::High,
            origin: Origin {
                pattern: "location.href".into(),
                snippet: Some("location.href = \"/api\"".into()),
                line: Some(1),
                column: Some(1),
            },
            method: None,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: vec![],
        };
        let with_method = Finding::endpoint(
            "/api",
            "GET",
            Origin {
                pattern: "fetch".into(),
                snippet: Some("fetch(\"/api\")".into()),
                line: Some(2),
                column: Some(1),
            },
        );

        let out = dedup_findings(vec![without_method, with_method]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].method.as_deref(), Some("GET"));
        assert_eq!(out[0].origin.pattern, "fetch");
    }
}
