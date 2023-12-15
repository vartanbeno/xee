use chumsky::{input::ValueInput, prelude::*};
use std::borrow::Cow;

use crate::ast::Span;
use crate::lexer::Token;
use crate::{ast, WithSpan, FN_NAMESPACE};
use crate::{pattern, Namespaces, ParserError, VariableNames};

use super::axis_node_test::parser_axis_node_test;
use super::name::{parser_name, ParserNameOutput};
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
        name,
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

    let predicate_pattern = (just(Token::Dot).ignore_then(predicate_list))
        .map(|predicates| pattern::PredicatePattern { predicates })
        .boxed();

    // let outer_function_name = name.try_map(|name, span| {
    //     let name = name.value;
    //     if name.namespace() == Some(FN_NAMESPACE) || name.namespace().is_none() {
    //         {
    //             match name.local_name() {
    //                 "doc" => Ok(pattern::OuterFunctionName::Doc),
    //                 "id" => Ok(pattern::OuterFunctionName::Id),
    //                 "element-with-id" => Ok(pattern::OuterFunctionName::ElementWithId),
    //                 "key" => Ok(pattern::OuterFunctionName::Key),
    //                 "root" => Ok(pattern::OuterFunctionName::Root),
    //                 _ => Err(ParserError::IllegalFunctionInPattern { name, span }),
    //             }
    //         }
    //     } else {
    //         Err(ParserError::IllegalFunctionInPattern { name, span })
    //     }
    // });

    // let argument = var_ref
    //     .map(|var_ref| {
    //         if let ast::PrimaryExpr::VarRef(name) = var_ref.value {
    //             pattern::Argument::VarRef(name)
    //         } else {
    //             unreachable!()
    //         }
    //     })
    //     .or(literal.map(|literal| {
    //         if let ast::PrimaryExpr::Literal(literal) = literal.value {
    //             pattern::Argument::Literal(literal)
    //         } else {
    //             unreachable!()
    //         }
    //     }));

    // let argument_list = (argument.separated_by(just(Token::Comma)))
    //     .at_least(1)
    //     .collect::<Vec<_>>()
    //     .delimited_by(just(Token::LeftParen), just(Token::RightParen))
    //     .boxed();

    // let function_call = outer_function_name.then(argument_list).boxed();

    // let rooted_path_start = (var_ref.map(|var_ref| {
    //     if let ast::PrimaryExpr::VarRef(name) = var_ref.value {
    //         pattern::RootedPathStart::VarRef(name)
    //     } else {
    //         unreachable!()
    //     }
    // }))
    // .or(function_call.map(|(name, args)| {
    //     pattern::RootedPathStart::FunctionCall(pattern::FunctionCall { name, args })
    // }));

    // let slash_or_double_slash = just(Token::Slash).or(just(Token::DoubleSlash));

    // let union_expr = recursive(|union_expr| {
    //     let parenthesized_expr =
    //         union_expr.delimited_by(just(Token::LeftParen), just(Token::RightParen));

    //     let postfix_expr = parenthesized_expr.then(predicate_list);

    //     let forward_axis = (just(Token::Child)
    //         .or(just(Token::Descendant))
    //         .or(just(Token::Attribute))
    //         .or(just(Token::Self_))
    //         .or(just(Token::DescendantOrSelf))
    //         .or(just(Token::Namespace)))
    //     .then_ignore(just(Token::DoubleColon))
    //     .map(|token| match token {
    //         Token::Child => pattern::ForwardAxis::Child,
    //         Token::Descendant => pattern::ForwardAxis::Descendant,
    //         Token::Attribute => pattern::ForwardAxis::Attribute,
    //         Token::Self_ => pattern::ForwardAxis::Self_,
    //         Token::DescendantOrSelf => pattern::ForwardAxis::DescendantOrSelf,
    //         Token::Namespace => pattern::ForwardAxis::Namespace,
    //         _ => unreachable!(),
    //     })
    //     .boxed();

    //     let forward_step = (forward_axis.then(node_test))
    //         .map(|(forward_axis, node_test)| (forward_axis, node_test))
    //         .or(abbrev_forward_step.map(|(axis, node_test)| {
    //             let axis = match axis {
    //                 ast::Axis::Attribute => pattern::ForwardAxis::Attribute,
    //                 ast::Axis::Child => pattern::ForwardAxis::Child,
    //                 _ => unreachable!(),
    //             };
    //             (axis, node_test)
    //         }));

    //     let axis_step = forward_step.then(predicate_list);

    //     let step_expr = postfix_expr.or(axis_step);

    //     let relative_path_expr = step_expr.then(
    //         (slash_or_double_slash.then(step_expr))
    //             .repeated()
    //             .collect::<Vec<_>>(),
    //     );

    //     let rooted_path = rooted_path_start
    //         .then(predicate_list)
    //         .then((slash_or_double_slash).then(relative_path_expr).map(
    //             |(token, expr)| match token {
    //                 Token::Slash => pattern::RootedPathRelative::Slash(expr),
    //                 Token::DoubleSlash => pattern::RootedPathRelative::DoubleSlash(expr),
    //                 _ => unreachable!(),
    //             },
    //         ))
    //         .or_not()
    //         .map(
    //             |((rooted_path_start, predicates), relative_path_expr)| pattern::RootedPath {
    //                 start: rooted_path_start,
    //                 predicates,
    //                 relative: relative_path_expr,
    //             },
    //         );

    //     let slash_path = just(Token::Slash).then(relative_path_expr.or_not());
    //     let double_slash_path = just(Token::DoubleSlash).then(relative_path_expr);

    //     let path_expr = rooted_path
    //         .or(slash_path)
    //         .or(double_slash_path)
    //         .or(relative_path_expr)
    //         .boxed();

    //     let intersect_except_expr = path_expr
    //         .then(
    //             (just(Token::Intersect).or(just(Token::Except))).map(|token| match token {
    //                 Token::Intersect => pattern::IntersectExceptOperator::Intersect,
    //                 Token::Except => pattern::IntersectExceptOperator::Except,
    //                 _ => unreachable!(),
    //             }),
    //         )
    //         .then(path_expr)
    //         .map(|((left, operator), right)| pattern::IntersectExceptExpr {
    //             operator,
    //             left,
    //             right,
    //         })
    //         .boxed();

    //     let union_expr = intersect_except_expr
    //         .then_ignore(just(Token::Union).or(just(Token::Pipe)))
    //         .then(intersect_except_expr)
    //         .map(|(left, right)| pattern::Pattern::UnionExpr(pattern::UnionExpr { left, right }))
    //         .boxed();

    //     union_expr
    // });

    let pattern = predicate_pattern
        .then_ignore(end())
        .map(pattern::Pattern::PredicatePattern)
        .boxed();

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
}
