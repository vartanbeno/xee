use chumsky::{input::ValueInput, prelude::*};

use crate::ast;
use crate::ast::Span;
use crate::lexer::Token;

use super::types::BoxedParser;

#[derive(Clone)]
pub(crate) struct ParserTypeOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    pub(crate) sequence_type: BoxedParser<'a, I, ast::SequenceType>,
    pub(crate) single_type: BoxedParser<'a, I, ast::SingleType>,
}

pub(crate) fn parser_type<'a, I>(
    eqname: BoxedParser<'a, I, ast::NameS>,
    empty_call: BoxedParser<'a, I, Token<'a>>,
    kind_test: BoxedParser<'a, I, ast::KindTest>,
) -> ParserTypeOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let single_type = eqname
        .clone()
        .then(just(Token::QuestionMark).or_not())
        .map_with_span(|(name, question_mark), _span| ast::SingleType {
            name,
            optional: question_mark.is_some(),
        })
        .boxed();

    let empty = just(Token::EmptySequence)
        .ignore_then(empty_call.clone())
        .to(ast::SequenceType::Empty)
        .boxed();

    let item_type_kind_test = kind_test.clone().map(ast::ItemType::KindTest);
    let item_type_empty = just(Token::Item)
        .ignore_then(empty_call.clone())
        .to(ast::ItemType::Item)
        .boxed();
    let item_type_atomic_or_union = eqname.clone().map(ast::ItemType::AtomicOrUnionType);
    let item_type = recursive(|item_type| {
        let parenthesized_item_type =
            item_type.delimited_by(just(Token::LeftParen), just(Token::RightParen));
        item_type_empty
            .or(item_type_kind_test)
            .or(item_type_atomic_or_union)
            .or(parenthesized_item_type)
    })
    .boxed();

    let occurrence = one_of([Token::QuestionMark, Token::Asterisk, Token::Plus])
        .map(|c| match c {
            Token::QuestionMark => ast::Occurrence::Option,
            Token::Asterisk => ast::Occurrence::Many,
            Token::Plus => ast::Occurrence::NonEmpty,
            _ => unreachable!(),
        })
        .or_not()
        .map(|o| o.unwrap_or(ast::Occurrence::One))
        .boxed();

    let item = item_type
        .clone()
        .then(occurrence)
        .map(|(item_type, occurrence)| ast::Item {
            item_type,
            occurrence,
        })
        .boxed();

    let sequence_type = empty.or(item.map(ast::SequenceType::Item)).boxed();

    ParserTypeOutput {
        sequence_type,
        single_type,
    }
}
