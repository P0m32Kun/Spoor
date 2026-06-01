use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind, PropertyKey};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_program},
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

pub struct JqueryMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<crate::finding::Finding>,
}

impl<'a> JqueryMatcher<'a> {
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

impl<'a> Visit<'a> for JqueryMatcher<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some((pattern, default_method)) = jquery_call_pattern(call) {
            if let Some((url, method)) =
                extract_jquery_url_and_method(call, pattern, default_method)
            {
                if let Some(finding) =
                    endpoint_from_url(&self.ctx, url, method, pattern, call.span.start)
                {
                    self.findings.push(finding);
                }
            }
        }
        walk_call_expression(self, call);
    }
}

fn jquery_call_pattern(call: &CallExpression<'_>) -> Option<(&'static str, &'static str)> {
    let member = call.callee.as_member_expression()?;
    let obj = member.object();
    let prop = member.static_property_name()?;
    let is_jquery = match obj {
        Expression::Identifier(id) => id.name == "$" || id.name == "jQuery",
        _ => false,
    };
    if !is_jquery {
        return None;
    }
    match prop {
        "get" => Some(("jquery.get", "GET")),
        "post" => Some(("jquery.post", "POST")),
        "ajax" => Some(("jquery.ajax", "GET")),
        _ => None,
    }
}

fn extract_jquery_url_and_method(
    call: &CallExpression<'_>,
    pattern: &str,
    default_method: &str,
) -> Option<(String, String)> {
    if pattern == "jquery.ajax" {
        let expr = call.arguments.first()?.as_expression()?;
        let Expression::ObjectExpression(obj) = expr else {
            return None;
        };
        let mut url = None;
        let mut method = default_method.to_string();
        for prop in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop else {
                continue;
            };
            let Some(key) = property_key_name(&p.key) else {
                continue;
            };
            if key == "url" {
                if let Expression::StringLiteral(lit) = &p.value {
                    url = Some(lit.value.as_str().to_string());
                }
            } else if key == "type" || key == "method" {
                if let Expression::StringLiteral(lit) = &p.value {
                    method = lit.value.as_str().to_string();
                }
            }
        }
        return url.map(|u| (u, method));
    }
    let expr = call.arguments.first()?.as_expression()?;
    Some((collapsed_string(expr), default_method.to_string()))
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
    fn jquery_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/jquery.js");
        let endpoints: Vec<_> = Analyzer::new(src, Some("jquery.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint && f.origin.pattern.starts_with("jquery"))
            .collect();
        assert_eq!(endpoints.len(), 3);
        assert!(endpoints
            .iter()
            .any(|f| { f.value == "/api/jquery/get" && f.method.as_deref() == Some("GET") }));
        assert!(endpoints
            .iter()
            .any(|f| { f.value == "/api/jquery/ajax" && f.method.as_deref() == Some("POST") }));
    }
}
