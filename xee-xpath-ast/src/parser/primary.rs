use chumsky::{input::ValueInput, prelude::*};
use ordered_float::OrderedFloat;
use std::borrow::Cow;

use crate::ast::Span;
use crate::lexer::Token;
use crate::span::WithSpan;
use crate::{ast, error::ParserError};

use super::types::{BoxedParser, State};

#[derive(Clone)]
pub(crate) struct ParserPrimaryOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    pub(crate) literal: BoxedParser<'a, I, ast::PrimaryExprS>,
    pub(crate) var_ref: BoxedParser<'a, I, ast::PrimaryExprS>,
    pub(crate) context_item_expr: BoxedParser<'a, I, ast::PrimaryExprS>,
    pub(crate) named_function_ref: BoxedParser<'a, I, ast::PrimaryExprS>,
    pub(crate) string: BoxedParser<'a, I, Cow<'a, str>>,
}

pub(crate) fn parser_primary<'a, I>(
    eqname: BoxedParser<'a, I, ast::NameS>,
) -> ParserPrimaryOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let string = select! {
        Token::StringLiteral(s) => s,
    }
    .boxed();
    let string_literal = string
        .clone()
        .map(|s| ast::Literal::String(s.to_string()))
        .boxed();

    let integer = select! {
        Token::IntegerLiteral(i) => i,
    };

    let integer_literal = integer.map(ast::Literal::Integer).boxed();

    let decimal_literal = select! {
        Token::DecimalLiteral(d) => d,
    }
    .map(ast::Literal::Decimal)
    .boxed();

    let double_literal = select! {
        Token::DoubleLiteral(d) => d,
    }
    .map(|d| ast::Literal::Double(OrderedFloat(d)))
    .boxed();

    let literal = string_literal
        .clone()
        .or(integer_literal.clone())
        .or(decimal_literal)
        .or(double_literal)
        .map_with_span(|literal, span| ast::PrimaryExpr::Literal(literal).with_span(span))
        .boxed();

    let var_ref = just(Token::Dollar)
        .ignore_then(eqname.clone())
        .map_with_span(|name, span| ast::PrimaryExpr::VarRef(name.value).with_span(span))
        .boxed();

    let context_item_expr = just(Token::Dot)
        .map_with_span(|_, span| ast::PrimaryExpr::ContextItem.with_span(span))
        .boxed();

    let named_function_ref = eqname
        .clone()
        .then_ignore(just(Token::Hash))
        .then(integer)
        .try_map(|(name, arity), span| {
            check_reserved(&name, span)?;
            Ok((name, arity))
        })
        .try_map_with_state(|(name, arity), span, state: &mut State| {
            let arity: u8 = arity
                .try_into()
                .map_err(|_| ParserError::ArityOverflow { span })?;
            Ok(ast::PrimaryExpr::NamedFunctionRef(ast::NamedFunctionRef {
                name: name.map(|name| {
                    name.with_default_namespace(state.namespaces.default_function_namespace)
                }),
                arity,
            })
            .with_span(span))
        })
        .boxed();

    ParserPrimaryOutput {
        literal,
        var_ref,
        context_item_expr,
        named_function_ref,
        string,
    }
}

const RESERVED_FUNCTION_NAMES: [&str; 18] = [
    "array",
    "attribute",
    "comment",
    "document-node",
    "element",
    "empty-sequence",
    "function",
    "if",
    "item",
    "map",
    "namespace-node",
    "node",
    "processing-instruction",
    "schema-attribute",
    "schema-element",
    "switch",
    "text",
    "typeswitch",
];

pub(crate) fn check_reserved(
    name: &ast::NameS,
    span: Span,
) -> std::result::Result<(), ParserError> {
    let local_name = name.value.local_name();
    if RESERVED_FUNCTION_NAMES.contains(&local_name) {
        return Err(ParserError::Reserved {
            name: local_name.to_string(),
            span,
        });
    }
    Ok(())
}
