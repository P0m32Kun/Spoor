use std::cell::OnceCell;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::finding::{Finding, FindingKind};
use crate::matcher::{FetchMatcher, LiteralCollector, LocationMatcher, MatchContext, XhrMatcher};

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
    outcome: OnceCell<ParseOutcome>,
}

impl<'a> Analyzer<'a> {
    pub fn new(source: &'a str, filename: Option<&str>) -> Self {
        Self {
            source,
            filename: filename.unwrap_or("<input>").to_string(),
            allocator: Allocator::default(),
            outcome: OnceCell::new(),
        }
    }

    pub fn parse_outcome(&self) -> &ParseOutcome {
        self.outcome.get_or_init(|| self.parse_for_outcome())
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Walk AST and return all findings (Phase 1 Task 1: literal paths only).
    pub fn collect_findings(&self) -> Vec<Finding> {
        let ret = Parser::new(&self.allocator, self.source, self.source_type())
            .with_options(ParseOptions {
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();
        let error_count = ret.errors.len();
        let _ = self.outcome.get_or_init(|| ParseOutcome {
            recovered: error_count > 0,
            error_count,
        });

        let mut findings =
            FetchMatcher::new(MatchContext::new(self.source)).collect(&ret.program);
        findings.extend(
            LocationMatcher::new(MatchContext::new(self.source)).collect(&ret.program),
        );
        findings.extend(XhrMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings.extend(
            LiteralCollector::new(MatchContext::new(self.source)).collect(&ret.program),
        );
        findings
    }

    /// Walk AST and return path findings from string literals (Phase 0 baseline).
    pub fn collect_literal_paths(&self) -> Vec<Finding> {
        self.collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Path)
            .collect()
    }

    fn source_type(&self) -> SourceType {
        SourceType::from_path(&self.filename).unwrap_or(SourceType::mjs())
    }

    fn parse_for_outcome(&self) -> ParseOutcome {
        let ret = Parser::new(&self.allocator, self.source, self.source_type())
            .with_options(ParseOptions {
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();
        let error_count = ret.errors.len();
        ParseOutcome {
            recovered: error_count > 0,
            error_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::finding::FindingKind;

    const BROKEN_JS: &str = include_str!("../../../tests/fixtures/broken.js");
    const SAMPLE_JS: &str = include_str!("../../../tests/fixtures/sample.js");
    const EXPECTED_PATH: &str = "/api/broken";

    #[test]
    fn broken_js_still_yields_path_findings() {
        let analyzer = Analyzer::new(BROKEN_JS, Some("broken.js"));
        let outcome = analyzer.parse_outcome();

        assert!(
            outcome.error_count > 0,
            "expected parse errors for broken.js, got error_count={}",
            outcome.error_count
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

        let values: HashSet<_> = findings.iter().map(|f| f.value.as_str()).collect();
        let expected = ["/api/v1", "/users", "https://cdn.example.com/app.js"]
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(values, expected);
    }

    #[test]
    fn collect_findings_reuses_single_parse() {
        let src = include_str!("../../../tests/fixtures/sample.js");
        let a = Analyzer::new(src, Some("sample.js"));
        let literals = a.collect_literal_paths();
        let all = a.collect_findings();
        let literal_values: HashSet<_> = literals.iter().map(|f| f.value.as_str()).collect();
        let path_values: HashSet<_> = all
            .iter()
            .filter(|f| f.kind == FindingKind::Path)
            .map(|f| f.value.as_str())
            .collect();
        assert_eq!(literal_values, path_values);
    }
}
