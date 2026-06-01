use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, StringLiteral};
use oxc_ast_visit::{
    walk::{walk_expression, walk_program},
    Visit,
};

use crate::finding::{Finding, Origin, SecretContext};
use crate::matcher::MatchContext;
use crate::secret_patterns::{
    looks_like_aws_access_key, looks_like_gcp_api_key, looks_like_private_key_pem,
};

pub struct SecretMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> SecretMatcher<'a> {
    pub fn new(ctx: MatchContext<'a>) -> Self {
        Self {
            ctx,
            findings: Vec::new(),
        }
    }

    pub fn collect(mut self, program: &oxc_ast::ast::Program<'a>) -> Vec<Finding> {
        walk_program(&mut self, program);
        self.findings
    }

    fn push_secret(
        &mut self,
        value: String,
        secret_type: &str,
        severity: &str,
        pattern: &str,
        span_start: u32,
        nearby_keys: Vec<String>,
    ) {
        let (line, column) = self.ctx.line_col(span_start);
        let origin = Origin {
            pattern: pattern.into(),
            snippet: Some(self.ctx.snippet(span_start, 80)),
            line: Some(line),
            column: Some(column),
        };
        let mut finding = Finding::secret(value, secret_type, severity, origin);
        if !nearby_keys.is_empty() {
            finding.context = Some(SecretContext { nearby_keys });
        }
        self.findings.push(finding);
    }
}

impl<'a> Visit<'a> for SecretMatcher<'a> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if let Expression::StringLiteral(lit) = expr {
            check_string_literal(self, lit);
        }
        if let Expression::ObjectExpression(obj) = expr {
            check_object_expression(self, obj);
        }
        walk_expression(self, expr);
    }
}

fn check_object_expression(matcher: &mut SecretMatcher<'_>, obj: &ObjectExpression<'_>) {
    let props = collect_string_props(obj);
    if is_firebase_config(&props) {
        if let Some(api_key) = props.get("apiKey").copied() {
            matcher.push_secret(
                api_key.to_string(),
                "firebase_api_key",
                "critical",
                "firebase_config",
                obj.span.start,
                vec!["apiKey".into(), "projectId".into()],
            );
        }
    }
    if props.get("type") == Some(&"service_account") {
        if let Some(private_key) = props.get("private_key").copied() {
            matcher.push_secret(
                private_key.to_string(),
                "gcp_service_account_key",
                "critical",
                "gcp_service_account",
                obj.span.start,
                vec!["private_key".into(), "type".into()],
            );
        }
    }

    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop else {
            continue;
        };
        let Some(key) = property_key_name(&p.key) else {
            continue;
        };
        if let Expression::StringLiteral(lit) = &p.value {
            let value = lit.value.as_str();
            if is_firebase_config(&props) && key == "apiKey" {
                continue;
            }
            if props.get("type") == Some(&"service_account") && key == "private_key" {
                continue;
            }
            if is_sensitive_object_key(key) {
                matcher.push_secret(
                    value.to_string(),
                    "object_literal_key",
                    "medium",
                    "object_literal",
                    p.span.start,
                    vec![key.to_string()],
                );
            }
            if looks_like_aws_access_key(value) {
                matcher.push_secret(
                    value.to_string(),
                    "aws_access_key",
                    "critical",
                    "string_literal",
                    lit.span.start,
                    vec![key.to_string()],
                );
            }
        }
    }
}

fn check_string_literal(matcher: &mut SecretMatcher<'_>, lit: &StringLiteral<'_>) {
    let value = lit.value.as_str();
    if looks_like_aws_access_key(value) {
        matcher.push_secret(
            value.to_string(),
            "aws_access_key",
            "critical",
            "string_literal",
            lit.span.start,
            Vec::new(),
        );
    } else if looks_like_gcp_api_key(value) {
        matcher.push_secret(
            value.to_string(),
            "gcp_api_key",
            "high",
            "string_literal",
            lit.span.start,
            Vec::new(),
        );
    } else if looks_like_private_key_pem(value) {
        matcher.push_secret(
            value.to_string(),
            "gcp_private_key",
            "critical",
            "string_literal",
            lit.span.start,
            Vec::new(),
        );
    } else if value.starts_with("sk-") && value.len() > 8 {
        matcher.push_secret(
            value.to_string(),
            "api_key",
            "high",
            "string_literal",
            lit.span.start,
            Vec::new(),
        );
    } else if value.starts_with("ghp_") || value.starts_with("github_pat_") {
        matcher.push_secret(
            value.to_string(),
            "github_token",
            "critical",
            "string_literal",
            lit.span.start,
            Vec::new(),
        );
    }
}

fn collect_string_props<'a>(obj: &'a ObjectExpression<'a>) -> std::collections::HashMap<&'a str, &'a str> {
    let mut map = std::collections::HashMap::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop else {
            continue;
        };
        let Some(key) = property_key_name(&p.key) else {
            continue;
        };
        if let Expression::StringLiteral(lit) = &p.value {
            map.insert(key, lit.value.as_str());
        }
    }
    map
}

fn is_firebase_config(props: &std::collections::HashMap<&str, &str>) -> bool {
    props.contains_key("projectId")
        && props.contains_key("authDomain")
        && props.contains_key("apiKey")
}

fn is_sensitive_object_key(key: &str) -> bool {
    matches!(
        key,
        "apiKey" | "api_key" | "secret" | "token" | "password" | "accessToken" | "access_token"
    )
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn secret_matcher_finds_aws_key() {
        let src = include_str!("../../../../tests/fixtures/secrets.js");
        let secrets: Vec<_> = Analyzer::new(src, Some("secrets.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Secret)
            .collect();
        assert!(secrets
            .iter()
            .any(|f| f.secret_type.as_deref() == Some("aws_access_key")));
        assert!(secrets.iter().any(|f| f.value.starts_with("AKIA")));
    }

    #[test]
    fn secret_matcher_finds_gcp_and_firebase() {
        let src = include_str!("../../../../tests/fixtures/secrets.js");
        let types: Vec<_> = Analyzer::new(src, Some("secrets.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Secret)
            .filter_map(|f| f.secret_type)
            .collect();
        assert!(types.iter().any(|t| t == "gcp_api_key"));
        assert!(types.iter().any(|t| t == "firebase_api_key"));
        assert!(types.iter().any(|t| t == "gcp_service_account_key"));
    }

    #[test]
    fn sample_js_aws_key_detected() {
        let src = include_str!("../../../../tests/fixtures/sample.js");
        let secrets: Vec<_> = Analyzer::new(src, Some("sample.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Secret)
            .collect();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].secret_type.as_deref(), Some("aws_access_key"));
    }
}
