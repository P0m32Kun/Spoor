use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_program},
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

pub struct FetchMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<crate::finding::Finding>,
}

impl<'a> FetchMatcher<'a> {
    pub fn new(ctx: MatchContext<'a>) -> Self {
        Self {
            ctx,
            findings: Vec::new(),
        }
    }

    pub fn collect(mut self, program: &oxc_ast::ast::Program<'a>) -> Vec<crate::finding::Finding> {
        walk_program(&mut self, program);
        self.findings
    }
}

impl<'a> Visit<'a> for FetchMatcher<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if call.callee_name() == Some("fetch") {
            if let Some(first) = call.arguments.first() {
                if let Some(expr) = first.as_expression() {
                    let folded = collapsed_string(expr);
                    if let Some(finding) = endpoint_from_url(
                        &self.ctx,
                        folded,
                        extract_method(&call.arguments),
                        "fetch",
                        call.span.start,
                    ) {
                        self.findings.push(finding);
                    }
                }
            }
        }
        walk_call_expression(self, call);
    }
}

fn extract_method(arguments: &[Argument<'_>]) -> String {
    let Some(expr) = arguments.get(1).and_then(|arg| arg.as_expression()) else {
        return "GET".into();
    };
    let Expression::ObjectExpression(obj) = expr else {
        return "GET".into();
    };
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if prop.key.is_specific_id("method") {
            if let Expression::StringLiteral(lit) = &prop.value {
                return lit.value.as_str().to_string();
            }
        }
    }
    "GET".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn fetch_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/fetch.js");
        let a = Analyzer::new(src, Some("fetch.js"));
        let findings = a
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint)
            .collect::<Vec<_>>();
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|f| {
            f.value == "/api/v1/users?id=1&sort=asc"
                && f.params
                    .as_ref()
                    .is_some_and(|p| p.query == vec!["id", "sort"])
        }));
        assert!(findings.iter().any(|f| {
            f.value.contains("api.example.com") && f.method.as_deref() == Some("POST")
        }));
    }
}
