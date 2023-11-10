use chumsky::util::MaybeRef;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use xmlparser::{StrSpan, Token};

use crate::parse::{Span, State};

use crate::ast_core as ast;

type Extra<'a> = Full<ParserError, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a>>;

#[cfg_attr(test, derive(serde::Serialize))]
pub enum ParserError {
    ExpectedFound { span: Span },
    MyError,
    XPath(xee_xpath_ast::ParserError),
}

impl From<xee_xpath_ast::ParserError> for ParserError {
    fn from(e: xee_xpath_ast::ParserError) -> Self {
        Self::XPath(e)
    }
}

impl<'a, I> chumsky::error::Error<'a, I> for ParserError
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    // we don't do anything with expected and found, instead just retaining
    // the span. This is because these contain tokens with a lifetime, and
    // having a lifetime for the ParserError turns out open up a world of trouble
    // as soon as we want to build on it in the XSLT parser. We also don't
    // have a good way to turn a logos token into a human-readable string, so
    // we couldn't really construct good error messages anyway.
    fn expected_found<E: IntoIterator<Item = Option<MaybeRef<'a, Token<'a>>>>>(
        _expected: E,
        _found: Option<MaybeRef<'a, Token<'a>>>,
        span: Span,
    ) -> Self {
        Self::ExpectedFound { span }
    }

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (
                ParserError::ExpectedFound { span: span_a },
                ParserError::ExpectedFound { span: _ },
            ) => ParserError::ExpectedFound { span: span_a },
            (ParserError::ExpectedFound { .. }, a) => a,
            (a, ParserError::ExpectedFound { .. }) => a,
            (a, _) => a,
        }
    }
}

struct Element<'a>(&'a str);

struct Attribute<'a>(&'a str, &'a str);

fn element_start<'a, I>(local: &'a str) -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let element = select! {
        Token::ElementStart { local, .. } => Element(local.as_str()),
    }
    .boxed();
    element
        .validate(move |element, span, emitter| {
            if element.0 != local {
                emitter.emit(ParserError::ExpectedFound { span })
            }
        })
        .boxed()
}

fn element_end<'a, I>() -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::ElementEnd { end: xmlparser::ElementEnd::Open, .. } => (),
    }
    .boxed()
}

fn element_close<'a, I>() -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::ElementEnd { end: xmlparser::ElementEnd::Close(..) | xmlparser::ElementEnd::Empty, .. } => (),
    }.boxed()
}

fn parser<'a, I>() -> BoxedParser<'a, I, ast::If>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let if_ = element_start("if");
    // let element = select! {
    //     Token::ElementStart { local, .. } => Element(local.as_str()),
    // }
    // .boxed();
    // let if_ = element
    //     .validate(|element, span, emitter| {
    //         if element.0 != "if" {
    //             emitter.emit(ParserError::ExpectedFound { span })
    //         }
    //     })
    //     .boxed();
    // let element_end = select! {
    //     Token::ElementEnd { end: xmlparser::ElementEnd::Open, .. } => (),
    // }
    // .boxed();

    // let element_close = select! {
    //     Token::ElementEnd { end: xmlparser::ElementEnd::Close(..) | xmlparser::ElementEnd::Empty, .. } => (),
    // };

    let text = select! {
        Token::Text { text } => text,
    }
    .boxed();

    let sequence_constructor = text
        .map(|text| ast::SequenceConstructor::Text(text.to_string()))
        .boxed();

    let attribute = select! {
        Token::Attribute { local, value, ..} => Attribute(local.as_str(), value.as_str()),
    }
    .boxed();

    let test_attribute_str = attribute
        .validate(|attribute, span, emitter| {
            if attribute.0 != "test" {
                emitter.emit(ParserError::ExpectedFound { span })
            }
            attribute.1
        })
        .boxed();

    let test_attribute =
        test_attribute_str.try_map_with_state(|value, _span, state: &mut State| {
            Ok(xee_xpath_ast::ast::XPath::parse(
                value,
                state.namespaces.as_ref(),
                &[],
            )?)
        });

    let if_attributes = test_attribute.repeated().collect::<Vec<_>>();

    if_.ignore_then(if_attributes)
        .then_ignore(element_end())
        .then(sequence_constructor)
        .try_map_with_state(|(attributes, content), _span, _state: &mut State| {
            let test = attributes.into_iter().next().unwrap();
            Ok(ast::If {
                test,
                content: vec![content],
            })
        })
        .then_ignore(element_close())
        .boxed()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use chumsky::input::Stream;
    use insta::assert_ron_snapshot;
    use xee_xpath_ast::Namespaces;

    // #[test]
    // fn test_tokens() {
    //     let tokens = xmlparser::Tokenizer::from(r#"<if test="true()">Hello</if>"#);

    //     dbg!(tokens.collect::<Vec<_>>());
    // }

    #[test]
    fn test_simple_parse_if() {
        let tokens = xmlparser::Tokenizer::from(r#"<if test="true()">Hello</if>"#);

        let stream = Stream::from_iter(tokens.map(|t| t.unwrap()));
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser().parse_with_state(stream, &mut state).into_result());
    }
}
