use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};
use oxc_ast_visit::{
    walk::{walk_object_expression, walk_program},
    Visit,
};

use crate::finding::{Confidence, Finding, FindingKind, Origin};
use crate::matcher::MatchContext;

const ROUTE_MARKERS: &[&str] = &[
    "element",
    "component",
    "children",
    "loader",
    "lazy",
    "redirect",
];

pub struct RouterMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> RouterMatcher<'a> {
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

impl<'a> Visit<'a> for RouterMatcher<'a> {
    fn visit_object_expression(&mut self, obj: &ObjectExpression<'a>) {
        if let Some(path) = extract_route_path(obj) {
            let (line, column) = self.ctx.line_col(obj.span.start);
            self.findings.push(Finding {
                kind: FindingKind::Path,
                value: path,
                confidence: Confidence::High,
                origin: Origin {
                    pattern: "router.path".into(),
                    snippet: Some(self.ctx.snippet(obj.span.start, 80)),
                    line: Some(line),
                    column: Some(column),
                },
                method: None,
                params: None,
                secret_type: None,
                severity: None,
                context: None,
                tags: vec!["router".into()],
            });
        }
        walk_object_expression(self, obj);
    }
}

fn extract_route_path(obj: &ObjectExpression<'_>) -> Option<String> {
    let mut path: Option<String> = None;
    let mut has_marker = false;
    let mut has_name = false;

    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop else {
            continue;
        };
        let Some(key) = property_key_name(&p.key) else {
            continue;
        };
        if key == "path" {
            if let Expression::StringLiteral(lit) = &p.value {
                let value = lit.value.as_str();
                if is_route_path_value(value) {
                    path = Some(value.to_string());
                }
            }
        } else if ROUTE_MARKERS.contains(&key) {
            has_marker = true;
        } else if key == "name" {
            has_name = true;
        }
    }

    if path.is_some() && (has_marker || has_name) {
        path
    } else {
        None
    }
}

fn is_route_path_value(value: &str) -> bool {
    if value.is_empty() || value == "*" {
        return false;
    }
    if value.starts_with('/') {
        return true;
    }
    // Nested react-router / vue-router segments (e.g. "settings", ":id")
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '/')
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
    fn router_matcher_finds_react_and_vue_paths() {
        let src = include_str!("../../../../tests/fixtures/router.js");
        let paths: Vec<_> = Analyzer::new(src, Some("router.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Path && f.origin.pattern == "router.path")
            .map(|f| f.value)
            .collect();
        for expected in [
            "/home",
            "/users/:id",
            "/admin",
            "settings",
            "/dashboard",
        ] {
            assert!(
                paths.contains(&expected.to_string()),
                "missing router path {expected}, got {paths:?}"
            );
        }
    }
}
