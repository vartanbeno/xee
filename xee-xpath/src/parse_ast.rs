use ordered_float::OrderedFloat;
use pest::iterators::Pair;

use crate::ast;
use crate::parse::{parse, Rule};

pub struct Error {}

fn pair_to_ast_node(pair: Pair<Rule>) -> Result<ast::Node, Error> {
    match pair.as_rule() {
        Rule::Literal => {
            let literal = pair.into_inner().next().unwrap();
            Ok(ast::Node::Literal(pair_to_literal(literal)))
        }
        _ => {
            panic!("unhandled rule: {:?}", pair.as_rule())
        }
    }
}

fn pair_to_primary_expr(pair: Pair<Rule>) -> Result<ast::PrimaryExpr, Error> {
    match pair.as_rule() {
        Rule::Literal => {
            let literal = pair.into_inner().next().unwrap();
            Ok(ast::PrimaryExpr::Literal(pair_to_literal(literal)))
        }
        _ => {
            panic!("unhandled PrimaryExpr: {:?}", pair.as_rule())
        }
    }
}

fn pair_to_literal(pair: Pair<Rule>) -> ast::Literal {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::StringLiteral => ast::Literal::String(pair.as_str().to_string()),
        Rule::NumericLiteral => {
            let numeric_literal = pair.into_inner().next().unwrap();
            pair_to_numeric_literal(numeric_literal)
        }
        _ => {
            panic!("unhandled literal: {:?}", pair.as_rule())
        }
    }
}

fn pair_to_numeric_literal(pair: Pair<Rule>) -> ast::Literal {
    println!("pair_to_numeric_literal: {:#?}", pair);
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
        let literal = pair_to_literal(pair);
        assert_eq!(literal, ast::Literal::Integer(1));
    }

    #[test]
    fn test_decimal_literal() {
        let mut pairs = XPathParser::parse(Rule::Literal, "1.5").unwrap();
        let pair = pairs.next().unwrap();
        let literal = pair_to_literal(pair);
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
        let literal = pair_to_literal(pair);
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
        let literal = pair_to_literal(pair);
        assert_eq!(
            literal,
            ast::Literal::Decimal(ast::DecimalLiteral {
                value: 5,
                fraction_digits: 1
            })
        );
    }
}
