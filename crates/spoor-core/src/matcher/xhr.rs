use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_program},
    Visit,
};

use crate::finding::{Finding, Origin};
use crate::matcher::MatchContext;
use crate::string_fold::collapsed_string;
use crate::url::maybe_url;

pub struct XhrMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> XhrMatcher<'a> {
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
}

impl<'a> Visit<'a> for XhrMatcher<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_xhr_open(call) {
            if let (Some(method), Some(url_expr)) = (
                call.arguments.first().and_then(Argument::as_expression),
                call.arguments.get(1).and_then(Argument::as_expression),
            ) {
                let method = extract_method(method);
                let folded = collapsed_string(url_expr);
                if maybe_url(&folded) {
                    let (line, column) = self.ctx.line_col(call.span.start);
                    let origin = Origin {
                        pattern: "xhr.open".into(),
                        snippet: Some(self.ctx.snippet(call.span.start, 80)),
                        line: Some(line),
                        column: Some(column),
                    };
                    self.findings
                        .push(Finding::endpoint(folded, method, origin));
                }
            }
        }
        walk_call_expression(self, call);
    }
}

fn is_xhr_open(call: &CallExpression<'_>) -> bool {
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };
    member.static_property_name() == Some("open")
}

fn extract_method(expr: &Expression<'_>) -> String {
    if let Expression::StringLiteral(lit) = expr {
        return lit.value.as_str().to_string();
    }
    let folded = collapsed_string(expr);
    if folded == crate::string_fold::EXPR_PLACEHOLDER {
        "GET".into()
    } else {
        folded
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn xhr_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/xhr.js");
        let a = Analyzer::new(src, Some("xhr.js"));
        let findings = a
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint && f.origin.pattern == "xhr.open")
            .collect::<Vec<_>>();
        assert_eq!(findings.len(), 2);
        assert!(findings
            .iter()
            .any(|f| { f.value == "/api/v1/status" && f.method.as_deref() == Some("GET") }));
        assert!(findings.iter().any(|f| {
            f.value == "https://example.com/submit" && f.method.as_deref() == Some("POST")
        }));
    }
}
