use oxc_ast::ast::Expression;
use oxc_ast_visit::{
    walk::{walk_expression, walk_program},
    Visit,
};

use crate::finding::{Confidence, Finding, FindingKind, Origin};
use crate::matcher::MatchContext;
use crate::string_fold::collapsed_string;
use crate::url::maybe_url;

pub struct LiteralCollector<'a> {
    ctx: MatchContext<'a>,
    findings: Vec<Finding>,
}

impl<'a> LiteralCollector<'a> {
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

impl<'a> Visit<'a> for LiteralCollector<'a> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if let Expression::StringLiteral(lit) = expr {
            let folded = collapsed_string(expr);
            if maybe_url(&folded) {
                let span = lit.span;
                let (line, column) = self.ctx.line_col(span.start);
                self.findings.push(Finding {
                    kind: FindingKind::Path,
                    value: folded,
                    confidence: Confidence::Low,
                    origin: Origin {
                        pattern: "string_literal".into(),
                        snippet: Some(self.ctx.snippet(span.start, 80)),
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
