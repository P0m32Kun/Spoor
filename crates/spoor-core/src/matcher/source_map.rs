use oxc_ast::ast::Expression;
use oxc_ast_visit::{
    walk::{walk_expression, walk_program},
    Visit,
};

use crate::finding::{Confidence, Finding, FindingKind, Origin};
use crate::matcher::MatchContext;
use crate::string_fold::collapsed_string;
use crate::url::resolved_maybe_url;

const SOURCE_MAP_MARKER: &str = "sourceMappingURL=";

pub struct SourceMapMatcher<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> SourceMapMatcher<'a> {
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

impl<'a> Visit<'a> for SourceMapMatcher<'a> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if let Expression::StringLiteral(lit) = expr {
            let folded = collapsed_string(expr);
            if let Some(url) = extract_source_mapping_url(&folded) {
                if is_source_map_url(&url) {
                    let span = lit.span;
                    let (line, column) = self.ctx.line_col(span.start);
                    self.findings.push(Finding {
                        kind: FindingKind::Path,
                        value: url,
                        confidence: Confidence::Medium,
                        origin: Origin {
                            pattern: "sourceMappingURL".into(),
                            snippet: Some(self.ctx.snippet(span.start, 80)),
                            line: Some(line),
                            column: Some(column),
                        },
                        method: None,
                        params: None,
                        secret_type: None,
                        severity: None,
                        context: None,
                        tags: vec!["sourcemap".into()],
                    });
                }
            }
        }
        walk_expression(self, expr);
    }
}

fn extract_source_mapping_url(s: &str) -> Option<String> {
    let idx = s.find(SOURCE_MAP_MARKER)?;
    let url = s[idx + SOURCE_MAP_MARKER.len()..].trim();
    if url.is_empty() {
        return None;
    }
    Some(url.to_string())
}

fn is_source_map_url(url: &str) -> bool {
    let url = url.trim();
    !url.is_empty()
        && (resolved_maybe_url(url)
            || url.ends_with(".map")
            || url.contains(".map?"))
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[test]
    fn source_map_matcher_extracts_url() {
        let src = include_str!("../../../../tests/fixtures/source_map.js");
        let paths: Vec<_> = Analyzer::new(src, Some("source_map.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.origin.pattern == "sourceMappingURL")
            .collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].value, "app.js.map");
        assert_eq!(paths[0].kind, FindingKind::Path);
    }
}
