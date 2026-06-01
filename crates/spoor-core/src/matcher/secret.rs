use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, StringLiteral};
use oxc_ast_visit::{
    walk::{walk_expression, walk_program},
    Visit,
};

use crate::finding::{Finding, Origin, SecretContext};
use crate::matcher::MatchContext;

const AWS_KEY_PREFIX: &str = "AKIA";

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
            for prop in &obj.properties {
                let ObjectPropertyKind::ObjectProperty(p) = prop else {
                    continue;
                };
                let Some(key) = property_key_name(&p.key) else {
                    continue;
                };
                if let Expression::StringLiteral(lit) = &p.value {
                    if is_sensitive_object_key(key) {
                        self.push_secret(
                            lit.value.as_str().to_string(),
                            "object_literal_key",
                            "medium",
                            "object_literal",
                            p.span.start,
                            vec![key.to_string()],
                        );
                    }
                    if looks_like_aws_access_key(lit.value.as_str()) {
                        self.push_secret(
                            lit.value.as_str().to_string(),
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
        walk_expression(self, expr);
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

fn looks_like_aws_access_key(value: &str) -> bool {
    value.starts_with(AWS_KEY_PREFIX)
        && value.len() == 20
        && value
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
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
