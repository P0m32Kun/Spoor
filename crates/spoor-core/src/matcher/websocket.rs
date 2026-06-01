use oxc_ast::ast::{Argument, Expression, NewExpression};
use oxc_ast_visit::{
    walk::{walk_new_expression, walk_program},
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

pub struct WebSocketMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<crate::finding::Finding>,
}

impl<'a> WebSocketMatcher<'a> {
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

impl<'a> Visit<'a> for WebSocketMatcher<'a> {
    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        if is_websocket_ctor(&new_expr.callee) {
            if let Some(url_expr) = new_expr.arguments.first().and_then(Argument::as_expression) {
                let folded = collapsed_string(url_expr);
                if let Some(finding) = endpoint_from_url(
                    &self.ctx,
                    folded,
                    "WS",
                    "websocket",
                    new_expr.span.start,
                ) {
                    self.findings.push(finding);
                }
            }
        }
        walk_new_expression(self, new_expr);
    }
}

fn is_websocket_ctor(callee: &Expression<'_>) -> bool {
    if let Expression::Identifier(id) = callee {
        return id.name == "WebSocket";
    }
    if let Some(member) = callee.as_member_expression() {
        return member.static_property_name() == Some("WebSocket");
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn websocket_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/websocket.js");
        let endpoints: Vec<_> = Analyzer::new(src, Some("websocket.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint && f.origin.pattern == "websocket")
            .collect();
        assert_eq!(endpoints.len(), 2);
        assert!(endpoints
            .iter()
            .any(|f| f.value == "wss://api.example.com/ws" && f.method.as_deref() == Some("WS")));
        assert!(endpoints
            .iter()
            .any(|f| f.value == "/socket" && f.method.as_deref() == Some("WS")));
    }
}
