use chumsky::{input::ValueInput, prelude::*};
use std::borrow::Cow;

use crate::ast::Span;
use crate::lexer::Token;
use crate::{ast, WithSpan};
use crate::{pattern, Namespaces, ParserError, VariableNames};

use super::name::{parser_name, ParserNameOutput};
use super::parser_core::parser as xpath_parser;
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

    // let name = parser_name().eqname;

    // let rooted_path = name.or(function_call).then(predicate_list).then(just(Token::Slash).or(Token::DoubleSlash))
    // let path_expr = rooted_path
    //     .or(slash_path)
    //     .or(double_slash_path)
    //     .or(relative_path)
    //     .boxed();

    // let intersect_except_expr = path_expr
    //     .then(
    //         (just(Token::Intersect).or(just(Token::Except))).map(|token| match token {
    //             Token::Intersect => pattern::IntersectExceptOperator::Intersect,
    //             Token::Except => pattern::IntersectExceptOperator::Except,
    //             _ => unreachable!(),
    //         }),
    //     )
    //     .then(path_expr)
    //     .map(|((left, operator), right)| pattern::IntersectExceptExpr {
    //         operator,
    //         left,
    //         right,
    //     })
    //     .boxed();

    // let union_expr = intersect_except_expr
    //     .then_ignore(just(Token::Union).or(just(Token::Pipe)))
    //     .then(intersect_except_expr)
    //     .map(|(left, right)| pattern::Pattern::UnionExpr(pattern::UnionExpr { left, right }))
    //     .boxed();

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
