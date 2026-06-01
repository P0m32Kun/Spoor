use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind, PropertyKey};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_program},
    Visit,
};

use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;

macro_rules! http_client_matcher {
    ($name:ident, $visit:ident, $pattern_fn:ident) => {
        pub struct $name<'a> {
            ctx: MatchContext<'a>,
            findings: Vec<crate::finding::Finding>,
        }

        impl<'a> $name<'a> {
            pub fn new(ctx: MatchContext<'a>) -> Self {
                Self {
                    ctx,
                    findings: Vec::new(),
                }
            }

            pub fn collect(
                mut self,
                program: &oxc_ast::ast::Program<'a>,
            ) -> Vec<crate::finding::Finding> {
                walk_program(&mut self, program);
                self.findings
            }
        }

        impl<'a> Visit<'a> for $name<'a> {
            fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
                if let Some((pattern, default_method)) = $pattern_fn(call) {
                    let (url, method) =
                        extract_http_url_and_method(call, pattern, default_method);
                    if let Some(finding) =
                        endpoint_from_url(&self.ctx, url, method, pattern, call.span.start)
                    {
                        self.findings.push(finding);
                    }
                }
                walk_call_expression(self, call);
            }
        }
    };
}

http_client_matcher!(KyMatcher, KyVisit, ky_call_pattern);
http_client_matcher!(GotMatcher, GotVisit, got_call_pattern);
http_client_matcher!(SuperagentMatcher, SuperagentVisit, superagent_call_pattern);

fn ky_call_pattern(call: &CallExpression<'_>) -> Option<(&'static str, &'static str)> {
    if call.callee_name() == Some("ky") {
        return Some(("ky", "GET"));
    }
    member_method_pattern(call, "ky", "ky")
}

fn got_call_pattern(call: &CallExpression<'_>) -> Option<(&'static str, &'static str)> {
    if call.callee_name() == Some("got") {
        return Some(("got", "GET"));
    }
    member_method_pattern(call, "got", "got")
}

fn superagent_call_pattern(call: &CallExpression<'_>) -> Option<(&'static str, &'static str)> {
    if let Some((pattern, method)) = member_method_pattern(call, "superagent", "superagent") {
        return Some((pattern, method));
    }
    member_method_pattern(call, "request", "superagent")
}

fn member_method_pattern(
    call: &CallExpression<'_>,
    object_name: &str,
    prefix: &str,
) -> Option<(&'static str, &'static str)> {
    let member = call.callee.as_member_expression()?;
    let Expression::Identifier(id) = member.object() else {
        return None;
    };
    if id.name != object_name {
        return None;
    }
    let prop = member.static_property_name()?;
    let pattern: &'static str = match prop {
        "get" => match prefix {
            "ky" => "ky.get",
            "got" => "got.get",
            _ => "superagent.get",
        },
        "post" => match prefix {
            "ky" => "ky.post",
            "got" => "got.post",
            _ => "superagent.post",
        },
        "put" => match prefix {
            "ky" => "ky.put",
            "got" => "got.put",
            _ => "superagent.put",
        },
        "delete" => match prefix {
            "ky" => "ky.delete",
            "got" => "got.delete",
            _ => "superagent.delete",
        },
        "patch" => match prefix {
            "ky" => "ky.patch",
            "got" => "got.patch",
            _ => "superagent.patch",
        },
        _ => return None,
    };
    Some((pattern, http_method_for_verb(prop)))
}

fn http_method_for_verb(verb: &str) -> &'static str {
    match verb {
        "post" => "POST",
        "put" => "PUT",
        "delete" => "DELETE",
        "patch" => "PATCH",
        _ => "GET",
    }
}

fn extract_http_url_and_method(
    call: &CallExpression<'_>,
    pattern: &str,
    default_method: &str,
) -> (String, String) {
    let url = call
        .arguments
        .first()
        .and_then(|a| a.as_expression())
        .map(collapsed_string)
        .unwrap_or_default();
    let method = method_from_options_arg(call, default_method);
    if pattern.ends_with(".get")
        || pattern.ends_with(".post")
        || pattern.ends_with(".put")
        || pattern.ends_with(".delete")
        || pattern.ends_with(".patch")
    {
        return (url, default_method.to_string());
    }
    (url, method)
}

fn method_from_options_arg(call: &CallExpression<'_>, default: &str) -> String {
    let Some(expr) = call
        .arguments
        .get(1)
        .and_then(|a| a.as_expression())
    else {
        return default.to_string();
    };
    let Expression::ObjectExpression(obj) = expr else {
        return default.to_string();
    };
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop else {
            continue;
        };
        let Some(key) = property_key_name(&p.key) else {
            continue;
        };
        if key == "method" {
            if let Expression::StringLiteral(lit) = &p.value {
                return lit.value.as_str().to_string();
            }
        }
    }
    default.to_string()
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

    fn endpoints_with_prefix(src: &str, file: &str, prefix: &str) -> Vec<String> {
        Analyzer::new(src, Some(file))
            .collect_findings()
            .into_iter()
            .filter(|f| {
                f.kind == FindingKind::Endpoint && f.origin.pattern.starts_with(prefix)
            })
            .map(|f| f.value)
            .collect()
    }

    #[test]
    fn ky_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/ky.js");
        let urls = endpoints_with_prefix(src, "ky.js", "ky");
        assert!(urls.contains(&"/api/ky/get".to_string()));
        assert!(urls.contains(&"/api/ky/post".to_string()));
        assert!(urls.contains(&"/api/ky".to_string()));
    }

    #[test]
    fn got_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/got.js");
        let urls = endpoints_with_prefix(src, "got.js", "got");
        assert!(urls.contains(&"https://api.example.com/got/get".to_string()));
        assert!(urls.contains(&"/api/got/post".to_string()));
        assert!(urls.contains(&"/api/got/direct".to_string()));
    }

    #[test]
    fn superagent_matcher_finds_endpoints() {
        let src = include_str!("../../../../tests/fixtures/superagent.js");
        let urls = endpoints_with_prefix(src, "superagent.js", "superagent");
        assert!(urls.contains(&"/api/superagent/get".to_string()));
        assert!(urls.contains(&"/api/superagent/post".to_string()));
        assert!(urls.contains(&"/api/request-alias".to_string()));
    }
}
