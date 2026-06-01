use crate::finding::{EndpointParams, Finding, Origin};
use crate::matcher::MatchContext;
use crate::url::{query_param_names, resolved_maybe_url};

pub fn endpoint_from_url(
    ctx: &MatchContext<'_>,
    value: String,
    method: impl Into<String>,
    pattern: &str,
    span_start: u32,
) -> Option<Finding> {
    if !resolved_maybe_url(&value) {
        return None;
    }
    let (line, column) = ctx.line_col(span_start);
    let query = query_param_names(&value);
    let params = if query.is_empty() {
        None
    } else {
        Some(EndpointParams {
            query,
            body: Vec::new(),
        })
    };
    let origin = Origin {
        pattern: pattern.into(),
        snippet: Some(ctx.snippet(span_start, 80)),
        line: Some(line),
        column: Some(column),
    };
    let mut finding = Finding::endpoint(value, method, origin);
    finding.params = params;
    Some(finding)
}
