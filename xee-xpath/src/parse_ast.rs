use ordered_float::OrderedFloat;
use pest::iterators::Pair;

use crate::ast;
use crate::parse::{parse, Rule};

pub struct Error {}

fn struct_wrap<T, W>(pair: Pair<Rule>, outer_rule: Rule, inner_rule: Rule, wrap: W) -> T
where
    W: Fn(Pair<Rule>) -> T,
{
    debug_assert_eq!(pair.as_rule(), outer_rule);
    let pair = pair.into_inner().next().unwrap();
    if pair.as_rule() == inner_rule {
        wrap(pair)
    } else {
        panic!("unhandled {:?}", pair.as_rule())
    }
}

fn pair_to_path_expr(pair: Pair<Rule>) -> ast::PathExpr {
    let expr_single = expr_single(pair);
    match expr_single {
        ast::ExprSingle::Path(path_expr) => path_expr,
        _ => ast::PathExpr {
            steps: vec![ast::StepExpr::PrimaryExpr(ast::PrimaryExpr::Expr(vec![
                expr_single,
            ]))],
        },
    }
}

fn expr_single(pair: Pair<Rule>) -> ast::ExprSingle {
    match pair.as_rule() {
        Rule::PathExpr => ast::ExprSingle::Path(path_expr_to_path_expr(pair)),
        Rule::SimpleMapExpr => {
            let mut pairs = pair.into_inner();
            let path_expr_pair = pairs.next().unwrap();
            let simple_map_path_exprs = pairs.map(pair_to_path_expr).collect::<Vec<_>>();
            if !simple_map_path_exprs.is_empty() {
                let path_expr = pair_to_path_expr(path_expr_pair);
                ast::ExprSingle::Apply(ast::ApplyExpr {
                    path_expr,
                    operator: ast::ApplyOperator::SimpleMap(simple_map_path_exprs),
                })
            } else {
                expr_single(path_expr_pair)
            }
        }
        Rule::UnaryExpr => {
            let mut plus_minus = vec![];
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::Minus => {
                        plus_minus.push(ast::UnaryOperator::Minus);
                    }
                    Rule::Plus => {
                        plus_minus.push(ast::UnaryOperator::Plus);
                    }
                    Rule::ValueExpr => {
                        if plus_minus.is_empty() {
                            return expr_single(pair);
                        }
                        let path_expr = pair_to_path_expr(pair);
                        return ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::Unary(plus_minus),
                        });
                    }
                    _ => {
                        panic!("unhandled unary {:?}", pair.as_rule())
                    }
                }
            }
            unreachable!();
        }
        Rule::ArrowExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::CastExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::CastableExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::TreatExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::InstanceofExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::AdditiveExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let op = pairs.next();
            if let Some(op) = op {
                let operator = match op.as_rule() {
                    Rule::Plus => ast::Operator::Add,
                    Rule::Minus => ast::Operator::Sub,
                    _ => {
                        panic!("unhandled AdditiveExpr {:?}", op.as_rule())
                    }
                };
                let right_pair = pairs.next().unwrap();
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::OrExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let right_pair = pairs.next();
            if let Some(right_pair) = right_pair {
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator: ast::Operator::Or,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::AndExpr
        | Rule::ComparisonExpr
        | Rule::StringConcatExpr
        | Rule::RangeExpr
        | Rule::MultiplicativeExpr
        | Rule::UnionExpr
        | Rule::IntersectExceptExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)
            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::ValueExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)
        }
        Rule::ExprSingle => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)
        }
        _ => {
            panic!("unhandled ExprSingle {:?}", pair.as_rule())
        }
    }
}

// fn apply_expr(pair: Pair<Rule>) -> ast::ApplyExpr {
//     match pair.as_rule() {
//         Rule::SimpleMapExpr => {}
//         Rule::UnaryExpr => {}
//         Rule::ArrowExpr => {}
//         Rule::CastExpr => {}
//         Rule::CastableExpr => {}
//         Rule::TreatExpr => {}
//         Rule::InstanceOfExpr => {}
//         _ => {
//             panic!("unhandled ApplyExpr {:?}", pair.as_rule())
//         }
//     }
// }

fn path_expr_to_path_expr(pair: Pair<Rule>) -> ast::PathExpr {
    debug_assert_eq!(pair.as_rule(), Rule::PathExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::RelativePathExpr => ast::PathExpr {
            steps: relative_path_expr_to_steps(pair),
        },
        _ => {
            panic!("unhandled PathExpr: {:?}", pair.as_rule())
        }
    }
}

fn relative_path_expr_to_steps(pair: Pair<Rule>) -> Vec<ast::StepExpr> {
    debug_assert_eq!(pair.as_rule(), Rule::RelativePathExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::StepExpr => {
            vec![step_expr_to_step_expr(pair)]
        }
        _ => {
            panic!("unhandled RelativePathExpr: {:?}", pair.as_rule())
        }
    }
}

fn step_expr_to_step_expr(pair: Pair<Rule>) -> ast::StepExpr {
    debug_assert_eq!(pair.as_rule(), Rule::StepExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::PostfixExpr => {
            let pair = pair.into_inner().next().unwrap();
            ast::StepExpr::PrimaryExpr(primary_expr_to_primary_expr(pair))
        }
        _ => {
            panic!("unhandled StepExpr: {:?}", pair.as_rule())
        }
    }
}

fn primary_expr_to_primary_expr(pair: Pair<Rule>) -> ast::PrimaryExpr {
    debug_assert_eq!(pair.as_rule(), Rule::PrimaryExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::Literal => ast::PrimaryExpr::Literal(literal_to_literal(pair)),
        _ => {
            panic!("unhandled PrimaryExpr: {:?}", pair.as_rule())
        }
    }
}

fn literal_to_literal(pair: Pair<Rule>) -> ast::Literal {
    debug_assert_eq!(pair.as_rule(), Rule::Literal);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::StringLiteral => ast::Literal::String(pair.as_str().to_string()),
        Rule::NumericLiteral => numeric_literal_to_literal(pair),
        _ => {
            panic!("unhandled literal: {:?}", pair.as_rule())
        }
    }
}

fn numeric_literal_to_literal(pair: Pair<Rule>) -> ast::Literal {
    debug_assert_eq!(pair.as_rule(), Rule::NumericLiteral);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::IntegerLiteral => {
            let s = pair.as_str();
            // parser never delivers negative numbers
            let i = s.parse::<i64>().unwrap();
            ast::Literal::Integer(i)
        }
        Rule::DecimalLiteral => {
            let s = pair.as_str();
            let period_index = s.find('.').unwrap();
            let (before, after) = s.split_at(period_index);
            let after = &after[1..];
            let before_nr = if !before.is_empty() {
                before.parse::<i64>().unwrap()
            } else {
                0
            };
            let after_nr = if !after.is_empty() {
                after.parse::<i64>().unwrap()
            } else {
                0
            };
            let digits = after.len();
            // to get positive number
            let factor = 10i64.pow(digits as u32);
            let before_nr = before_nr * factor;
            let nr = before_nr + after_nr;
            ast::Literal::Decimal(ast::DecimalLiteral {
                value: nr,
                fraction_digits: digits as u8,
            })
        }
        Rule::DoubleLiteral => {
            let s = pair.as_str();
            let f = s.parse::<f64>().unwrap();
            ast::Literal::Double(OrderedFloat(f))
        }
        _ => {
            panic!("unhandled numeric literal: {:?}", pair.as_rule())
        }
    }
}

// fn pair_to_ast(pair: Pair<Rule>) -> Result<ast::XPath, Error> {
//     match pair.as_rule() {
//         Rule::Xpath => {
//             let mut ast = ast::XPath { exprs: vec![] };
//             for pair in pair.into_inner() {
//                 let expr = pair_to_ast(pair)?;
//                 ast.exprs.push(expr);
//             }
//             Ok(ast)
//         }
//         _ => {}
//     }
//     Ok(ast::XPath { exprs: vec![] })
// }

// pub(crate) fn parse_ast(xpath: &str) -> Result<ast::XPath, Error> {
//     match parse(xpath) {
//         Ok(mut pairs) => {
//             let ast = pair_to_ast(pairs.next().unwrap());
//             match ast {
//                 Ok(ast) => Ok(ast),
//                 Err(e) => Err(Error {}),
//             }
//         }
//         Err(e) => Err(Error {}),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::XPathParser;
    use insta::assert_debug_snapshot;
    use pest::Parser;

    fn parse_rule<T, F>(rule: Rule, input: &str, f: F) -> T
    where
        F: Fn(Pair<Rule>) -> T,
    {
        let mut pairs = XPathParser::parse(rule, input).unwrap();
        let pair = pairs.next().unwrap();
        f(pair)
    }

    fn parse_expr_single(input: &str) -> ast::ExprSingle {
        parse_rule(Rule::ExprSingle, input, expr_single)
    }

    fn parse_literal(input: &str) -> ast::Literal {
        parse_rule(Rule::Literal, input, literal_to_literal)
    }

    fn parse_primary_expr(input: &str) -> ast::PrimaryExpr {
        parse_rule(Rule::PrimaryExpr, input, primary_expr_to_primary_expr)
    }

    fn parse_step_expr(input: &str) -> ast::StepExpr {
        parse_rule(Rule::StepExpr, input, step_expr_to_step_expr)
    }

    fn parse_relative_path_expr(input: &str) -> Vec<ast::StepExpr> {
        parse_rule(Rule::RelativePathExpr, input, relative_path_expr_to_steps)
    }

    fn parse_path_expr(input: &str) -> ast::PathExpr {
        parse_rule(Rule::PathExpr, input, path_expr_to_path_expr)
    }

    #[test]
    fn test_integer_literal() {
        assert_debug_snapshot!(parse_literal("1"));
    }

    #[test]
    fn test_decimal_literal() {
        assert_debug_snapshot!(parse_literal("1.5"));
    }

    #[test]
    fn test_decimal_literal_no_after() {
        assert_debug_snapshot!(parse_literal("1."));
    }

    #[test]
    fn test_decimal_literal_no_before() {
        assert_debug_snapshot!(parse_literal(".5"));
    }

    #[test]
    fn test_float_lowercase_e() {
        assert_debug_snapshot!(parse_literal("1.5e0"));
    }

    #[test]
    fn test_float_upper_e() {
        assert_debug_snapshot!(parse_literal("1.5E0"));
    }

    #[test]
    fn test_primary_expr_literal() {
        assert_debug_snapshot!(parse_primary_expr("1"));
    }

    #[test]
    fn test_step_expr() {
        assert_debug_snapshot!(parse_step_expr("1"));
    }

    #[test]
    fn test_relative_path() {
        assert_debug_snapshot!(parse_relative_path_expr("1"));
    }

    #[test]
    fn test_path_expr() {
        assert_debug_snapshot!(parse_path_expr("1"));
    }

    #[test]
    fn test_integer_expr_single() {
        assert_debug_snapshot!(parse_expr_single("1"));
    }

    #[test]
    fn test_simple_map_expr() {
        assert_debug_snapshot!(parse_expr_single("1 ! 2"));
    }

    #[test]
    fn test_unary_expr() {
        assert_debug_snapshot!(parse_expr_single("-1"));
    }

    #[test]
    fn test_additive_expr() {
        assert_debug_snapshot!(parse_expr_single("1 + 2"));
    }

    #[test]
    fn test_or_expr() {
        assert_debug_snapshot!(parse_expr_single("1 or 2"));
    }
}
