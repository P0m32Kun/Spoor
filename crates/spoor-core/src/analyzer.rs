use std::cell::OnceCell;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::dedup::dedup_findings;
use crate::finding::{Finding, FindingKind};
use crate::matcher::{
    AxiosMatcher, FetchMatcher, JqueryMatcher, LiteralCollector, LocationMatcher, MatchContext,
    SecretMatcher, WindowOpenMatcher, XhrMatcher,
};

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

        let ctx = MatchContext::new(self.source);
        let mut findings = FetchMatcher::new(ctx).collect(&ret.program);
        findings.extend(LocationMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings.extend(XhrMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings.extend(AxiosMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings.extend(JqueryMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings
            .extend(WindowOpenMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings.extend(SecretMatcher::new(MatchContext::new(self.source)).collect(&ret.program));
        findings
            .extend(LiteralCollector::new(MatchContext::new(self.source)).collect(&ret.program));
        dedup_findings(findings)
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
    const PHASE1_COMBINED_JS: &str = include_str!("../../../tests/fixtures/phase1/combined.js");
    const EXPECTED_PATH: &str = "/api/broken";

    #[test]
    fn broken_js_still_yields_path_findings() {
        let analyzer = Analyzer::new(BROKEN_JS, Some("broken.js"));
        let findings = analyzer.collect_findings();
        let outcome = analyzer.parse_outcome();
        let path_findings: Vec<_> = findings
            .into_iter()
            .filter(|f| f.kind == FindingKind::Path)
            .collect();

        assert!(
            outcome.error_count > 0,
            "expected parse errors for broken.js, got error_count={}",
            outcome.error_count
        );

        assert!(
            !path_findings.is_empty(),
            "expected at least one finding from broken.js despite syntax errors"
        );

        assert!(
            path_findings.iter().any(|f| f.value == EXPECTED_PATH),
            "expected path {EXPECTED_PATH:?}, got: {:?}",
            path_findings.iter().map(|f| &f.value).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sample_js_dynamic_fetch_no_endpoint() {
        let analyzer = Analyzer::new(SAMPLE_JS, Some("sample.js"));
        let endpoints: Vec<_> = analyzer
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint)
            .collect();
        assert!(
            !endpoints.iter().any(|f| f.value.contains("EXPR")),
            "dynamic fetch must not produce EXPR endpoint, got: {:?}",
            endpoints.iter().map(|f| &f.value).collect::<Vec<_>>()
        );
        assert!(endpoints
            .iter()
            .any(|f| f.value == "https://cdn.example.com/app.js"));
    }

    #[test]
    fn sample_js_yields_expected_path_literals() {
        let analyzer = Analyzer::new(SAMPLE_JS, Some("sample.js"));
        let findings = analyzer.collect_literal_paths();

        // `/users` from literal only — fetch(base + "/users") is dynamic in Phase 1.
        // `https://cdn.example.com/app.js` dedupes to location.href endpoint.
        assert_eq!(findings.len(), 2, "expected 2 path findings after dedup");

        let values: HashSet<_> = findings.iter().map(|f| f.value.as_str()).collect();
        let expected = ["/api/v1", "/users"].into_iter().collect::<HashSet<_>>();
        assert_eq!(values, expected);
    }

    #[test]
    fn phase1_combined_yields_endpoints() {
        let analyzer = Analyzer::new(PHASE1_COMBINED_JS, Some("combined.js"));
        let endpoints: Vec<_> = analyzer
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint)
            .collect();

        assert_eq!(
            endpoints.len(),
            7,
            "expected 7 endpoint findings from combined.js, got: {:?}",
            endpoints.iter().map(|f| &f.value).collect::<Vec<_>>()
        );

        let values: HashSet<_> = endpoints.iter().map(|f| f.value.as_str()).collect();
        let expected = [
            "/api/v1/users",
            "https://api.example.com/data",
            "https://cdn.example.com/app.js",
            "/login",
            "/dashboard",
            "/api/v1/status",
            "https://example.com/submit",
        ]
        .into_iter()
        .collect::<HashSet<_>>();
        assert_eq!(values, expected);

        assert!(endpoints
            .iter()
            .any(|f| { f.value == "/api/v1/users" && f.method.as_deref() == Some("GET") }));
        assert!(endpoints.iter().any(|f| {
            f.value.contains("api.example.com") && f.method.as_deref() == Some("POST")
        }));
        assert!(endpoints.iter().any(|f| {
            f.value == "/login"
                && f.origin.pattern == "location.replace"
                && f.method.as_deref() == Some("GET")
        }));
        assert!(endpoints.iter().any(|f| {
            f.value == "/api/v1/status"
                && f.origin.pattern == "xhr.open"
                && f.method.as_deref() == Some("GET")
        }));
    }

    #[test]
    fn jsluice_subset_endpoint_values() {
        let src = include_str!("../../../tests/fixtures/jsluice_subset.js");
        let endpoints: HashSet<_> = Analyzer::new(src, Some("jsluice_subset.js"))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Endpoint)
            .map(|f| f.value)
            .collect();
        for expected in [
            "https://api.example.com/v1/items",
            "/api/status",
            "https://cdn.example.com/app.js",
            "/api/jquery",
            "/popup",
        ] {
            assert!(endpoints.contains(expected), "missing endpoint {expected}");
        }
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
