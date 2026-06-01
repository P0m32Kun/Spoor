use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentTarget, CallExpression, SimpleAssignmentTarget,
};
use oxc_ast::match_member_expression;
use oxc_ast_visit::{
    walk::{walk_assignment_expression, walk_call_expression, walk_program},
    Visit,
};

use crate::finding::{Confidence, Finding, FindingKind, Origin};
use crate::matcher::MatchContext;
use crate::string_fold::{collapsed_string, EXPR_PLACEHOLDER};

pub struct LocationMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> LocationMatcher<'a> {
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

    fn push_endpoint(
        &mut self,
        value: String,
        method: Option<String>,
        pattern: &str,
        span_start: u32,
    ) {
        let (line, column) = self.ctx.line_col(span_start);
        let origin = Origin {
            pattern: pattern.into(),
            snippet: Some(self.ctx.snippet(span_start, 80)),
            line: Some(line),
            column: Some(column),
        };
        self.findings.push(Finding {
            file: None,
            kind: FindingKind::Endpoint,
            value,
            raw: None,
            confidence: Confidence::High,
            origin,
            method,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: Vec::new(),
            http_status: None,
        });
    }
}

impl<'a> Visit<'a> for LocationMatcher<'a> {
    fn visit_assignment_expression(&mut self, assign: &AssignmentExpression<'a>) {
        if let Some(pattern) = assignment_pattern(&assign.left) {
            let folded = collapsed_string(&assign.right);
            if is_location_url(&folded) {
                self.push_endpoint(folded, None, pattern, assign.span.start);
            }
        }
        walk_assignment_expression(self, assign);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(pattern) = location_call_pattern(call) {
            if let Some(expr) = call.arguments.first().and_then(Argument::as_expression) {
                let folded = collapsed_string(expr);
                if is_location_url(&folded) {
                    self.push_endpoint(folded, Some("GET".into()), pattern, call.span.start);
                }
            }
        }
        walk_call_expression(self, call);
    }
}

fn is_location_url(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s == EXPR_PLACEHOLDER {
        return false;
    }
    s.starts_with('/') || s.starts_with("http")
}

fn assignment_pattern(left: &AssignmentTarget<'_>) -> Option<&'static str> {
    let simple = left.as_simple_assignment_target()?;
    match simple {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) if id.name == "location" => {
            Some("location")
        }
        match_member_expression!(SimpleAssignmentTarget) => {
            let member = simple.to_member_expression();
            if member.is_specific_member_access("location", "href") {
                Some("location.href")
            } else if member.static_property_name() == Some("location") {
                Some("location")
            } else {
                None
            }
        }
        _ => None,
    }
}

fn location_call_pattern(call: &CallExpression<'_>) -> Option<&'static str> {
    let member = call.callee.as_member_expression()?;
    if !member.object().is_specific_id("location") {
        return None;
    }
    match member.static_property_name()? {
        "replace" => Some("location.replace"),
        "assign" => Some("location.assign"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn location_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/location.js");
        let a = Analyzer::new(src, Some("location.js"));
        let findings = a
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint)
            .collect::<Vec<_>>();
        assert_eq!(findings.len(), 3);
        assert!(findings.iter().any(|f| {
            f.value == "https://cdn.example.com/app.js"
                && f.origin.pattern == "location.href"
                && f.method.is_none()
        }));
        assert!(findings.iter().any(|f| {
            f.value == "/login"
                && f.origin.pattern == "location.replace"
                && f.method.as_deref() == Some("GET")
        }));
        assert!(findings.iter().any(|f| {
            f.value == "/dashboard" && f.origin.pattern == "location" && f.method.is_none()
        }));
    }
}
