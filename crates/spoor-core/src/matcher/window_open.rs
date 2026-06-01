use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_program},
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

pub struct WindowOpenMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<crate::finding::Finding>,
}

impl<'a> WindowOpenMatcher<'a> {
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

impl<'a> Visit<'a> for WindowOpenMatcher<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(pattern) = window_open_pattern(call) {
            if let Some(expr) = call.arguments.first().and_then(|a| a.as_expression()) {
                let folded = collapsed_string(expr);
                if let Some(finding) =
                    endpoint_from_url(&self.ctx, folded, "GET", pattern, call.span.start)
                {
                    self.findings.push(finding);
                }
            }
        }
        walk_call_expression(self, call);
    }
}

fn window_open_pattern(call: &CallExpression<'_>) -> Option<&'static str> {
    if call.callee_name() == Some("open") {
        return Some("window.open");
    }
    let member = call.callee.as_member_expression()?;
    if member.static_property_name() == Some("open") {
        if let Expression::Identifier(id) = member.object() {
            if id.name == "window" {
                return Some("window.open");
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn window_open_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/window_open.js");
        let endpoints: Vec<_> = Analyzer::new(src, Some("window_open.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint && f.origin.pattern == "window.open")
            .collect();
        assert_eq!(endpoints.len(), 2);
        assert!(endpoints
            .iter()
            .any(|f| f.value == "https://example.com/popup"));
        assert!(endpoints.iter().any(|f| f.value == "/local/popup"));
    }
}
