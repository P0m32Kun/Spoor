use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_ast_visit::{
    Visit,
    walk::{walk_expression, walk_program},
};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::finding::{Confidence, Finding, FindingKind, Origin};
use crate::string_fold::collapsed_string;
use crate::url::maybe_url;

#[derive(Debug, Clone)]
pub struct ParseOutcome {
    pub recovered: bool,
    pub error_count: usize,
}

/// Parses JavaScript/TypeScript source and collects literal URL-like paths (Phase 0).
pub struct Analyzer<'a> {
    source: &'a str,
    filename: String,
    allocator: Allocator,
    outcome: ParseOutcome,
}

struct LiteralCollector<'a> {
    source: &'a str,
    findings: Vec<Finding>,
}

impl<'a> Visit<'a> for LiteralCollector<'a> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if let Expression::StringLiteral(lit) = expr {
            let folded = collapsed_string(expr);
            if maybe_url(&folded) {
                let span = lit.span;
                let (line, column) = offset_to_line_col(self.source, span.start);
                self.findings.push(Finding {
                    kind: FindingKind::Path,
                    value: folded,
                    confidence: Confidence::Low,
                    origin: Origin {
                        pattern: "string_literal".into(),
                        snippet: Some(snippet_at_offset(self.source, span.start, 80)),
                        line: Some(line),
                        column: Some(column),
                    },
                    method: None,
                    params: None,
                    secret_type: None,
                    severity: None,
                    context: None,
                    tags: vec!["literal".into()],
                });
            }
        }
        walk_expression(self, expr);
    }
}

impl<'a> Analyzer<'a> {
    pub fn new(source: &'a str, filename: Option<&str>) -> Self {
        let filename = filename.unwrap_or("<input>").to_string();
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&filename).unwrap_or(SourceType::mjs());
        let ret = Parser::new(&allocator, source, source_type)
            .with_options(ParseOptions {
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();
        let error_count = ret.errors.len();
        let recovered = error_count > 0;
        // Program is stored in allocator; we only need outcome metadata here for Phase 0.
        let _program = &ret.program;
        Self {
            source,
            filename,
            allocator,
            outcome: ParseOutcome {
                recovered,
                error_count,
            },
        }
    }

    pub fn parse_outcome(&self) -> &ParseOutcome {
        &self.outcome
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Walk AST and return path findings from string literals (Phase 0 baseline).
    pub fn collect_literal_paths(&self) -> Vec<Finding> {
        let source_type = SourceType::from_path(&self.filename).unwrap_or(SourceType::mjs());
        let ret = Parser::new(&self.allocator, self.source, source_type)
            .with_options(ParseOptions {
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();
        let program = &ret.program;
        let mut visitor = LiteralCollector {
            source: self.source,
            findings: Vec::new(),
        };
        walk_program(&mut visitor, program);
        visitor.findings
    }
}

fn offset_to_line_col(source: &str, offset: u32) -> (u32, u32) {
    let offset = offset as usize;
    let mut line = 1u32;
    let mut last_line_start = 0usize;
    for (i, b) in source.bytes().enumerate() {
        if i >= offset {
            break;
        }
        if b == b'\n' {
            line += 1;
            last_line_start = i + 1;
        }
    }
    let column = (offset.saturating_sub(last_line_start) + 1) as u32;
    (line, column)
}

fn snippet_at_offset(source: &str, offset: u32, max_len: usize) -> String {
    let start = offset as usize;
    let end = (start + max_len).min(source.len());
    source.get(start..end).unwrap_or("").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const BROKEN_JS: &str = include_str!("../../../tests/fixtures/broken.js");
    const SAMPLE_JS: &str = include_str!("../../../tests/fixtures/sample.js");
    const EXPECTED_PATH: &str = "/api/broken";

    #[test]
    fn broken_js_still_yields_path_findings() {
        let analyzer = Analyzer::new(BROKEN_JS, Some("broken.js"));
        let outcome = analyzer.parse_outcome();

        assert!(
            outcome.error_count > 0 || outcome.recovered,
            "expected parse errors or recovery for broken.js"
        );

        let findings = analyzer.collect_literal_paths();
        assert!(
            !findings.is_empty(),
            "expected at least one finding from broken.js despite syntax errors"
        );

        assert!(
            findings.iter().any(|f| f.value == EXPECTED_PATH),
            "expected path {EXPECTED_PATH:?}, got: {:?}",
            findings.iter().map(|f| &f.value).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sample_js_yields_expected_path_literals() {
        let analyzer = Analyzer::new(SAMPLE_JS, Some("sample.js"));
        let findings = analyzer.collect_literal_paths();

        assert_eq!(findings.len(), 3, "expected exactly 3 path findings");

        let values: std::collections::HashSet<_> =
            findings.iter().map(|f| f.value.as_str()).collect();
        let expected = [
            "/api/v1",
            "/users",
            "https://cdn.example.com/app.js",
        ]
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
        assert_eq!(values, expected);
    }
}
