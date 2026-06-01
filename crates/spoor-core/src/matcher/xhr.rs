use std::collections::HashSet;

use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentTarget, CallExpression, Expression,
    VariableDeclarator,
};
use oxc_ast_visit::{
    walk::{
        walk_assignment_expression, walk_call_expression, walk_program, walk_variable_declarator,
    },
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

pub struct XhrMatcher<'a> {
    ctx: MatchContext<'a>,
    xhr_bindings: HashSet<String>,
    findings: Vec<crate::finding::Finding>,
}

impl<'a> XhrMatcher<'a> {
    pub fn new(ctx: MatchContext<'a>) -> Self {
        Self {
            ctx,
            xhr_bindings: HashSet::new(),
            findings: Vec::new(),
        }
    }

    pub fn collect(mut self, program: &oxc_ast::ast::Program<'a>) -> Vec<crate::finding::Finding> {
        walk_program(&mut self, program);
        self.findings
    }
}

impl<'a> Visit<'a> for XhrMatcher<'a> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let Some(init) = &decl.init {
            if is_new_xml_http_request(init) {
                if let Some(name) = decl.id.get_binding_identifier() {
                    self.xhr_bindings.insert(name.name.to_string());
                }
            }
        }
        walk_variable_declarator(self, decl);
    }

    fn visit_assignment_expression(&mut self, assign: &AssignmentExpression<'a>) {
        if is_new_xml_http_request(&assign.right) {
            if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                self.xhr_bindings.insert(id.name.to_string());
            }
        }
        walk_assignment_expression(self, assign);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_xhr_open(call, &self.xhr_bindings) {
            if let (Some(method), Some(url_expr)) = (
                call.arguments.first().and_then(Argument::as_expression),
                call.arguments.get(1).and_then(Argument::as_expression),
            ) {
                let method = extract_method_string(method);
                let folded = collapsed_string(url_expr);
                if let Some(finding) =
                    endpoint_from_url(&self.ctx, folded, method, "xhr.open", call.span.start)
                {
                    self.findings.push(finding);
                }
            }
        }
        walk_call_expression(self, call);
    }
}

fn is_new_xml_http_request(expr: &Expression<'_>) -> bool {
    let Expression::NewExpression(new_expr) = expr else {
        return false;
    };
    matches!(&new_expr.callee, Expression::Identifier(id) if id.name == "XMLHttpRequest")
}

fn is_xhr_open(call: &CallExpression<'_>, xhr_bindings: &HashSet<String>) -> bool {
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };
    if member.static_property_name() != Some("open") {
        return false;
    }
    let Expression::Identifier(id) = member.object() else {
        return false;
    };
    xhr_bindings.contains(id.name.as_str())
}

fn extract_method_string(expr: &Expression<'_>) -> String {
    if let Expression::StringLiteral(lit) = expr {
        return lit.value.as_str().to_string();
    }
    "GET".into()
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn xhr_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/xhr.js");
        let findings = Analyzer::new(src, Some("xhr.js"))
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

    #[test]
    fn xhr_ignores_unrelated_open_calls() {
        let src = r#"
const db = {};
db.open("GET", "/not-xhr");
const xhr = new XMLHttpRequest();
xhr.open("GET", "/api/real");
"#;
        let findings = Analyzer::new(src, Some("t.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.origin.pattern == "xhr.open")
            .collect::<Vec<_>>();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].value, "/api/real");
    }
}
