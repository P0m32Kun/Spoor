use oxc_ast::ast::{CallExpression, Expression, TaggedTemplateExpression};
use oxc_ast_visit::{
    walk::{
        walk_call_expression, walk_program, walk_tagged_template_expression,
    },
    Visit,
};

use crate::finding::{Confidence, Finding, Origin};
use crate::matcher::{util::endpoint_from_url, MatchContext};
use crate::string_fold::collapsed_string;
use crate::url::resolved_maybe_url;

pub struct GraphqlMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> GraphqlMatcher<'a> {
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

impl<'a> Visit<'a> for GraphqlMatcher<'a> {
    fn visit_tagged_template_expression(&mut self, tagged: &TaggedTemplateExpression<'a>) {
        if is_gql_tag(&tagged.tag) {
            self.push_graphql_hint(tagged.span.start, "gql.template");
        }
        walk_tagged_template_expression(self, tagged);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(url) = graphql_request_url(call) {
            if let Some(finding) = endpoint_from_url(
                &self.ctx,
                url,
                "POST",
                "graphql.request",
                call.span.start,
            ) {
                self.findings.push(finding);
            }
        }
        walk_call_expression(self, call);
    }
}

impl<'a> GraphqlMatcher<'a> {
    fn push_graphql_hint(&mut self, span_start: u32, pattern: &str) {
        let (line, column) = self.ctx.line_col(span_start);
        self.findings.push(Finding {
            kind: crate::finding::FindingKind::Endpoint,
            value: "/graphql".into(),
            confidence: Confidence::Medium,
            origin: Origin {
                pattern: pattern.into(),
                snippet: Some(self.ctx.snippet(span_start, 80)),
                line: Some(line),
                column: Some(column),
            },
            method: Some("POST".into()),
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: vec!["graphql".into()],
        });
    }
}

fn is_gql_tag(tag: &Expression<'_>) -> bool {
    match tag {
        Expression::Identifier(id) => matches!(id.name.as_str(), "gql" | "graphql"),
        _ => tag
            .as_member_expression()
            .and_then(|m| m.static_property_name())
            .is_some_and(|name| name == "gql" || name == "graphql"),
    }
}

fn graphql_request_url(call: &CallExpression<'_>) -> Option<String> {
    let name = call.callee_name()?;
    let url = call
        .arguments
        .first()
        .and_then(|a| a.as_expression())
        .map(collapsed_string)?;
    if !resolved_maybe_url(&url) {
        return None;
    }
    if name == "graphql" {
        return Some(url);
    }
    if name == "request" && url.to_ascii_lowercase().contains("graphql") {
        return Some(url);
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn graphql_matcher_finds_gql_template_and_request() {
        let src = include_str!("../../../../tests/fixtures/graphql.js");
        let findings: Vec<_> = Analyzer::new(src, Some("graphql.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| {
                f.kind == FindingKind::Endpoint
                    && (f.origin.pattern.starts_with("gql")
                        || f.origin.pattern == "graphql.request")
            })
            .collect();
        assert!(findings.iter().any(|f| {
            f.value == "/graphql" && f.origin.pattern == "gql.template"
        }));
        assert!(findings.iter().any(|f| {
            f.value == "https://api.example.com/graphql"
                && f.origin.pattern == "graphql.request"
        }));
    }
}
