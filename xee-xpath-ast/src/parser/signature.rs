use chumsky::{input::ValueInput, prelude::*};

use crate::ast;
use crate::ast::Span;
use crate::lexer::Token;

use super::types::BoxedParser;

#[derive(Clone)]
pub(crate) struct ParserSignatureOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    pub(crate) param_list: BoxedParser<'a, I, Vec<ast::Param>>,
    pub(crate) signature: BoxedParser<'a, I, ast::Signature>,
}

pub(crate) fn parser_signature<'a, I>(
    eqname: BoxedParser<'a, I, ast::NameS>,
    sequence_type: BoxedParser<'a, I, ast::SequenceType>,
) -> ParserSignatureOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let type_declaration = just(Token::As).ignore_then(sequence_type.clone()).boxed();

    let param = just(Token::Dollar)
        .ignore_then(eqname.clone())
        .then(type_declaration.clone().or_not())
        .map(|(name, type_)| ast::Param {
            name: name.value,
            type_,
        })
        .boxed();

    let param_list = param
        .separated_by(just(Token::Comma))
        .collect::<Vec<_>>()
        .boxed();

    let signature_param = just(Token::Dollar)
        .ignore_then(eqname.clone())
        .then(type_declaration.clone())
        .map(|(name, type_)| ast::SignatureParam {
            name: name.value,
            type_,
        })
        .boxed();

    let signature_param_list = signature_param
        .separated_by(just(Token::Comma))
        .collect::<Vec<_>>()
        .boxed();

    let signature = eqname
        .clone()
        .then(signature_param_list.delimited_by(just(Token::LeftParen), just(Token::RightParen)))
        .then_ignore(just(Token::As))
        .then(sequence_type.clone())
        .map(|((name, params), return_type)| ast::Signature {
            name,
            params,
            return_type,
        })
        .boxed();

    ParserSignatureOutput {
        param_list,
        signature,
    }
}
