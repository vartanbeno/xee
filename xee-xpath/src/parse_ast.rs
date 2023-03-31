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
            ast::StepExpr::PostfixExpr {
                primary: primary_expr_to_primary_expr(pair),
                postfixes: vec![],
            }
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
    use pest::Parser;

    #[test]
    fn test_integer_literal() {
        let mut pairs = XPathParser::parse(Rule::Literal, "1").unwrap();
        let pair = pairs.next().unwrap();
        let literal = literal_to_literal(pair);
        assert_eq!(literal, ast::Literal::Integer(1));
    }

    #[test]
    fn test_decimal_literal() {
        let mut pairs = XPathParser::parse(Rule::Literal, "1.5").unwrap();
        let pair = pairs.next().unwrap();
        let literal = literal_to_literal(pair);
        assert_eq!(
            literal,
            ast::Literal::Decimal(ast::DecimalLiteral {
                value: 15,
                fraction_digits: 1
            })
        );
    }

    #[test]
    fn test_decimal_literal_no_after() {
        let mut pairs = XPathParser::parse(Rule::Literal, "1.").unwrap();
        let pair = pairs.next().unwrap();
        let literal = literal_to_literal(pair);
        assert_eq!(
            literal,
            ast::Literal::Decimal(ast::DecimalLiteral {
                value: 1,
                fraction_digits: 0
            })
        );
    }

    #[test]
    fn test_decimal_literal_no_before() {
        let mut pairs = XPathParser::parse(Rule::Literal, ".5").unwrap();
        let pair = pairs.next().unwrap();
        let literal = literal_to_literal(pair);
        assert_eq!(
            literal,
            ast::Literal::Decimal(ast::DecimalLiteral {
                value: 5,
                fraction_digits: 1
            })
        );
    }

    #[test]
    fn test_float_lowercase_e() {
        let mut pairs = XPathParser::parse(Rule::Literal, "1.5e0").unwrap();
        let pair = pairs.next().unwrap();
        let literal = literal_to_literal(pair);
        assert_eq!(literal, ast::Literal::Double(OrderedFloat(1.5)));
    }

    #[test]
    fn test_float_upper_e() {
        let mut pairs = XPathParser::parse(Rule::Literal, "1.5E0").unwrap();
        let pair = pairs.next().unwrap();
        let literal = literal_to_literal(pair);
        assert_eq!(literal, ast::Literal::Double(OrderedFloat(1.5)));
    }

    #[test]
    fn test_primary_expr_literal() {
        let mut pairs = XPathParser::parse(Rule::PrimaryExpr, "1").unwrap();
        let pair = pairs.next().unwrap();
        let primary_expr = primary_expr_to_primary_expr(pair);
        assert_eq!(
            primary_expr,
            ast::PrimaryExpr::Literal(ast::Literal::Integer(1))
        );
    }

    #[test]
    fn test_step_expr_postfix() {
        let mut pairs = XPathParser::parse(Rule::StepExpr, "1").unwrap();
        let pair = pairs.next().unwrap();
        let step_expr = step_expr_to_step_expr(pair);
        assert_eq!(
            step_expr,
            ast::StepExpr::PostfixExpr {
                primary: ast::PrimaryExpr::Literal(ast::Literal::Integer(1)),
                postfixes: vec![]
            }
        );
    }

    #[test]
    fn test_relative_path_expr_postfix() {
        let mut pairs = XPathParser::parse(Rule::RelativePathExpr, "1").unwrap();
        let pair = pairs.next().unwrap();
        let steps = relative_path_expr_to_steps(pair);
        assert_eq!(
            steps,
            vec![ast::StepExpr::PostfixExpr {
                primary: ast::PrimaryExpr::Literal(ast::Literal::Integer(1)),
                postfixes: vec![]
            }]
        );
    }

    #[test]
    fn test_path_expr() {
        let mut pairs = XPathParser::parse(Rule::PathExpr, "1").unwrap();
        let pair = pairs.next().unwrap();
        let step_expr = path_expr_to_path_expr(pair);
        assert_eq!(
            step_expr,
            ast::PathExpr {
                steps: vec![ast::StepExpr::PostfixExpr {
                    primary: ast::PrimaryExpr::Literal(ast::Literal::Integer(1)),
                    postfixes: vec![]
                }]
            }
        );
    }
}
