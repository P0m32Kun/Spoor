use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind, PropertyKey};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_program},
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

pub struct AxiosMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<crate::finding::Finding>,
}

impl<'a> AxiosMatcher<'a> {
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

impl<'a> Visit<'a> for AxiosMatcher<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some((pattern, default_method)) = axios_call_pattern(call) {
            let (url, method) = extract_axios_url_and_method(call, pattern, default_method);
            if let Some(finding) =
                endpoint_from_url(&self.ctx, url, method, pattern, call.span.start)
            {
                self.findings.push(finding);
            }
        }
        walk_call_expression(self, call);
    }
}

fn axios_call_pattern(call: &CallExpression<'_>) -> Option<(&'static str, &'static str)> {
    let member = call.callee.as_member_expression()?;
    let obj = member.object();
    let prop = member.static_property_name()?;
    let Expression::Identifier(id) = obj else {
        return None;
    };
    if id.name != "axios" {
        return None;
    }
    match prop {
        "get" => Some(("axios.get", "GET")),
        "post" => Some(("axios.post", "POST")),
        "put" => Some(("axios.put", "PUT")),
        "delete" => Some(("axios.delete", "DELETE")),
        "request" => Some(("axios.request", "GET")),
        _ => None,
    }
}

fn extract_axios_url_and_method(
    call: &CallExpression<'_>,
    pattern: &str,
    default_method: &str,
) -> (String, String) {
    if pattern == "axios.request" {
        if let Some(Expression::ObjectExpression(obj)) =
            call.arguments.first().and_then(|a| a.as_expression())
        {
                let mut url = String::new();
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
                            url = lit.value.as_str().to_string();
                        }
                    } else if key == "method" {
                        if let Expression::StringLiteral(lit) = &p.value {
                            method = lit.value.as_str().to_string();
                        }
                    }
                }
                if !url.is_empty() {
                    return (url, method);
                }
        }
    }
    let url = call
        .arguments
        .first()
        .and_then(|a| a.as_expression())
        .map(collapsed_string)
        .unwrap_or_default();
    (url, default_method.to_string())
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
    fn axios_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/axios.js");
        let endpoints: Vec<_> = Analyzer::new(src, Some("axios.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint && f.origin.pattern.starts_with("axios"))
            .collect();
        assert_eq!(endpoints.len(), 3);
    }
}
