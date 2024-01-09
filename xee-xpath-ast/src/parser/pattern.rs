use chumsky::{input::ValueInput, prelude::*};
use std::borrow::Cow;

use crate::ast::Span;
use crate::lexer::Token;
use crate::{ast, WithSpan, FN_NAMESPACE};
use crate::{pattern, Namespaces, ParserError, VariableNames};

use super::axis_node_test::parser_axis_node_test;
use super::name::parser_name;
use super::parser_core::parser as xpath_parser;
use super::primary::parser_primary;
use super::{parse, tokens};

use super::types::BoxedParser;

#[derive(Clone)]
pub(crate) struct PatternParserOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    pub(crate) pattern: BoxedParser<'a, I, pattern::Pattern>,
}

pub(crate) fn parser<'a, I>() -> PatternParserOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let xpath_parser_output = xpath_parser();
    let expr_single = xpath_parser_output.expr_single_core;
    let name_output = parser_name();
    let name = name_output.eqname;
    let parser_primary_output = parser_primary(name.clone());
    let literal = parser_primary_output.literal;
    let var_ref = parser_primary_output.var_ref;
    let parser_axis_node_test_output = parser_axis_node_test(
        name.clone(),
        name_output.ncname,
        name_output.braced_uri_literal,
        xpath_parser_output.kind_test,
    );
    let node_test = parser_axis_node_test_output.node_test;
    let abbrev_forward_step = parser_axis_node_test_output.abbrev_forward_step;

    // HACK: a bit of repetition here to produce predicate_list, as getting it out
    // of the xpath parser seems to lead to recursive parser errors
    let expr = expr_single
        .clone()
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
        .map_with_span(|exprs, span| ast::Expr(exprs).with_span(span))
        .boxed();
    let predicate = expr
        .clone()
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .boxed();
    let predicate_list = predicate.repeated().collect::<Vec<_>>().boxed();

    let predicate_pattern = (just(Token::Dot).ignore_then(predicate_list.clone()))
        .map(|predicates| pattern::PredicatePattern { predicates })
        .boxed();

    let outer_function_name = name.try_map(|name, span| {
        let name = name.value;
        if name.namespace() == Some(FN_NAMESPACE) || name.namespace().is_none() {
            {
                match name.local_name() {
                    "doc" => Ok(pattern::OuterFunctionName::Doc),
                    "id" => Ok(pattern::OuterFunctionName::Id),
                    "element-with-id" => Ok(pattern::OuterFunctionName::ElementWithId),
                    "key" => Ok(pattern::OuterFunctionName::Key),
                    "root" => Ok(pattern::OuterFunctionName::Root),
                    _ => Err(ParserError::IllegalFunctionInPattern { name, span }),
                }
            }
        } else {
            Err(ParserError::IllegalFunctionInPattern { name, span })
        }
    });

    let argument = var_ref
        .clone()
        .map(|var_ref| {
            if let ast::PrimaryExpr::VarRef(name) = var_ref.value {
                pattern::Argument::VarRef(name)
            } else {
                unreachable!()
            }
        })
        .or(literal.map(|literal| {
            if let ast::PrimaryExpr::Literal(literal) = literal.value {
                pattern::Argument::Literal(literal)
            } else {
                unreachable!()
            }
        }));

    let argument_list = (argument.separated_by(just(Token::Comma)))
        .at_least(1)
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LeftParen), just(Token::RightParen))
        .boxed();

    let function_call = outer_function_name.then(argument_list).boxed();

    let rooted_path_start = (var_ref.map(|var_ref| {
        if let ast::PrimaryExpr::VarRef(name) = var_ref.value {
            pattern::RootExpr::VarRef(name)
        } else {
            unreachable!()
        }
    }))
    .or(function_call
        .map(|(name, args)| pattern::RootExpr::FunctionCall(pattern::FunctionCall { name, args })));

    let slash_or_double_slash = just(Token::Slash).or(just(Token::DoubleSlash));

    let expr_pattern = recursive(|expr_pattern| {
        let parenthesized_expr = expr_pattern
            .delimited_by(just(Token::LeftParen), just(Token::RightParen))
            .boxed();

        let postfix_expr = parenthesized_expr.then(predicate_list.clone()).boxed();

        let forward_axis = (just(Token::Child)
            .or(just(Token::Descendant))
            .or(just(Token::Attribute))
            .or(just(Token::Self_))
            .or(just(Token::DescendantOrSelf))
            .or(just(Token::Namespace)))
        .then_ignore(just(Token::DoubleColon))
        .map(|token| match token {
            Token::Child => pattern::ForwardAxis::Child,
            Token::Descendant => pattern::ForwardAxis::Descendant,
            Token::Attribute => pattern::ForwardAxis::Attribute,
            Token::Self_ => pattern::ForwardAxis::Self_,
            Token::DescendantOrSelf => pattern::ForwardAxis::DescendantOrSelf,
            Token::Namespace => pattern::ForwardAxis::Namespace,
            _ => unreachable!(),
        })
        .boxed();

        let forward_step_axis_node_test = forward_axis.then(node_test);
        let forward_step_abbrev = abbrev_forward_step.map(|(axis, node_test)| {
            let axis = match axis {
                ast::Axis::Attribute => pattern::ForwardAxis::Attribute,
                ast::Axis::Child => pattern::ForwardAxis::Child,
                _ => unreachable!(),
            };
            (axis, node_test)
        });

        let forward_step = forward_step_axis_node_test.or(forward_step_abbrev);

        let axis_step = forward_step.then(predicate_list.clone());

        let step_expr = postfix_expr
            .map(|(expr, predicates)| {
                pattern::StepExpr::PostfixExprP(pattern::PostfixExpr { expr, predicates })
            })
            .or(axis_step.map(|((axis, node_test), predicates)| {
                pattern::StepExpr::AxisStep(pattern::AxisStep {
                    forward: axis,
                    node_test,
                    predicates,
                })
            }))
            .boxed();

        let relative_path_expr = step_expr
            .clone()
            .then(
                (slash_or_double_slash.then(step_expr))
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(first_step, rest_steps)| {
                let mut steps = vec![first_step];
                for (token, step) in rest_steps {
                    match token {
                        Token::Slash => {}
                        Token::DoubleSlash => {
                            let axis_step = pattern::AxisStep {
                                forward: pattern::ForwardAxis::DescendantOrSelf,
                                node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                                predicates: vec![],
                            };
                            steps.push(pattern::StepExpr::AxisStep(axis_step));
                        }
                        _ => unreachable!(),
                    }
                    steps.push(step);
                }
                steps
            })
            .boxed();

        let rooted_path = rooted_path_start
            .then(predicate_list)
            .then(relative_path_expr.clone().or_not())
            .map(|((root, predicates), steps)| pattern::PathExpr {
                root: pattern::PathRoot::Rooted { root, predicates },
                steps: steps.unwrap_or_default(),
            });
        let absolute_slash_path = just(Token::Slash)
            .ignore_then(relative_path_expr.clone().or_not())
            .map(|steps| pattern::PathExpr {
                root: pattern::PathRoot::AbsoluteSlash,
                steps: steps.unwrap_or_default(),
            });
        let absolute_double_slash_path = just(Token::DoubleSlash)
            .ignore_then(relative_path_expr.clone())
            .map(|steps| pattern::PathExpr {
                root: pattern::PathRoot::AbsoluteDoubleSlash,
                steps,
            });
        let relative_path = relative_path_expr.map(|steps| pattern::PathExpr {
            root: pattern::PathRoot::Relative,
            steps,
        });

        let path_expr = rooted_path
            .or(absolute_slash_path)
            .or(absolute_double_slash_path)
            .or(relative_path)
            .boxed();

        let operator = just(Token::Intersect)
            .or(just(Token::Except))
            .or(just(Token::Union))
            .or(just(Token::Pipe))
            .map(|token| match token {
                Token::Intersect => pattern::Operator::Intersect,
                Token::Except => pattern::Operator::Except,
                Token::Union => pattern::Operator::Union,
                Token::Pipe => pattern::Operator::Union,
                _ => unreachable!(),
            });

        let expr_pattern = path_expr
            .clone()
            .map(pattern::ExprPattern::Path)
            .foldl(
                operator.then(path_expr.clone()).repeated(),
                |left, (operator, right)| {
                    pattern::ExprPattern::BinaryExpr(pattern::BinaryExpr {
                        operator,
                        left: Box::new(left),
                        right: Box::new(pattern::ExprPattern::Path(right)),
                    })
                },
            )
            .boxed();

        expr_pattern
    })
    .boxed();

    let predicate_pattern = predicate_pattern
        .then_ignore(end())
        .map(pattern::Pattern::Predicate)
        .boxed();

    let union_pattern = expr_pattern
        .then_ignore(end())
        .map(pattern::Pattern::Expr)
        .boxed();

    let pattern = predicate_pattern.or(union_pattern).boxed();

    PatternParserOutput { pattern }
}

impl pattern::Pattern {
    pub fn parse<'a>(
        input: &'a str,
        namespaces: &'a Namespaces,
        _variable_names: &'a VariableNames,
    ) -> Result<Self, ParserError> {
        let pattern = parse(parser().pattern, tokens(input), Cow::Borrowed(namespaces))?;
        // TODO: do we need to rename variables to unique names? probably
        Ok(pattern)
    }
}

#[cfg(test)]
mod tests {
    use ahash::HashSetExt;
    use insta::assert_ron_snapshot;

    use super::*;

    #[test]
    fn test_predicate_pattern_no_predicates() {
        let namespaces = Namespaces::default();
        let variable_names = VariableNames::new();
        assert_ron_snapshot!(pattern::Pattern::parse(".", &namespaces, &variable_names));
    }

    #[test]
    fn test_predicate_pattern_single_predicate() {
        let namespaces = Namespaces::default();
        let variable_names = VariableNames::new();
        assert_ron_snapshot!(pattern::Pattern::parse(
            ".[1]",
            &namespaces,
            &variable_names
        ));
    }

    // #[test]
    // fn test_union_pattern_
}
