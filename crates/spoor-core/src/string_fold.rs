use oxc_ast::ast::Expression;

/// Placeholder for non-literal fragments in a concatenated string (jsluice-compatible).
pub const EXPR_PLACEHOLDER: &str = "EXPR";

/// Fold string concatenation and template quasi-literals into one string.
/// Unknown sub-expressions become `EXPR`.
pub fn collapsed_string(expr: &Expression<'_>) -> String {
    match expr {
        Expression::StringLiteral(lit) => lit.value.as_str().to_string(),
        Expression::TemplateLiteral(tpl) => {
            let mut out = String::new();
            for quasi in &tpl.quasis {
                out.push_str(quasi.value.raw.as_str());
            }
            if !tpl.expressions.is_empty() {
                out.push_str(EXPR_PLACEHOLDER);
            }
            out
        }
        Expression::BinaryExpression(bin)
            if bin.operator == oxc_ast::ast::BinaryOperator::Addition =>
        {
            let left = collapsed_string(&bin.left);
            let right = collapsed_string(&bin.right);
            merge_folded(&left, &right)
        }
        _ => EXPR_PLACEHOLDER.to_string(),
    }
}

fn merge_folded(left: &str, right: &str) -> String {
    let left_is_expr = left == EXPR_PLACEHOLDER;
    let right_is_expr = right == EXPR_PLACEHOLDER;
    match (left_is_expr, right_is_expr) {
        (true, true) => EXPR_PLACEHOLDER.to_string(),
        (true, false) => {
            if right.is_empty() {
                EXPR_PLACEHOLDER.to_string()
            } else {
                format!("{EXPR_PLACEHOLDER}{right}")
            }
        }
        (false, true) => {
            if left.is_empty() {
                EXPR_PLACEHOLDER.to_string()
            } else {
                format!("{left}{EXPR_PLACEHOLDER}")
            }
        }
        (false, false) => format!("{left}{right}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::{ParseOptions, Parser};
    use oxc_span::SourceType;

    fn fold_source(source: &str) -> String {
        let allocator = Allocator::default();
        let wrapped = format!("({source})");
        let ret = Parser::new(&allocator, &wrapped, SourceType::mjs())
            .with_options(ParseOptions {
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();
        let program = &ret.program;
        let expr = program.body.first().and_then(|stmt| match stmt {
            oxc_ast::ast::Statement::ExpressionStatement(es) => Some(&es.expression),
            _ => None,
        });
        let expr = expr.and_then(|e| match e {
            Expression::ParenthesizedExpression(p) => Some(&p.expression),
            _ => Some(e),
        });
        expr.map(collapsed_string).unwrap_or_default()
    }

    #[test]
    fn literal_concat() {
        assert_eq!(fold_source(r#"'/api/' + 'v1/users'"#), "/api/v1/users");
    }

    #[test]
    fn unknown_expression_becomes_expr() {
        assert_eq!(fold_source(r#"'/api/' + id"#), "/api/EXPR");
    }

    #[test]
    fn double_expr() {
        assert_eq!(fold_source("a + b"), EXPR_PLACEHOLDER);
    }

    #[test]
    fn template_literal_with_expression() {
        assert_eq!(fold_source("`/api/${id}`"), "/api/EXPR");
    }

    #[test]
    fn single_string_literal() {
        assert_eq!(fold_source(r#"'/api/v2/users'"#), "/api/v2/users");
    }
}
