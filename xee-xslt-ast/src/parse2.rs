use chumsky::input::Stream;
use chumsky::util::MaybeRef;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use xmlparser::Token;

use crate::parse::{Span, State};

use crate::ast_core as ast;

type Extra<'a> = Full<ParserError, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a>>;

fn token_span(token: &Token) -> Span {
    let span = match token {
        Token::Attribute { span, .. } => span,
        Token::Cdata { span, .. } => span,
        Token::Comment { span, .. } => span,
        Token::ElementEnd { span, .. } => span,
        Token::ElementStart { span, .. } => span,
        Token::ProcessingInstruction { span, .. } => span,
        Token::Text { text, .. } => text,
        Token::Declaration { span, .. } => span,
        Token::EmptyDtd { span, .. } => span,
        Token::DtdEnd { span, .. } => span,
        Token::DtdStart { span, .. } => span,
        Token::EntityDeclaration { span, .. } => span,
    };
    span.range().into()
}

fn tokens(src: &str) -> impl ValueInput<'_, Token = Token<'_>, Span = Span> {
    Stream::from_iter(xmlparser::Tokenizer::from(src).map(|token| {
        // TODO: we need an Error token
        let token = token.unwrap();
        (token, token_span(&token))
    }))
    .spanned((src.len()..src.len()).into())
}

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
        .try_map(move |element, span| {
            if element.0 == local {
                Ok(())
            } else {
                Err(ParserError::ExpectedFound { span })
            }
        })
        .boxed()
}

fn attribute<'a, I>(local: &'a str) -> BoxedParser<'a, I, &'a str>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let attribute = select! {
        Token::Attribute { local, value, ..} => Attribute(local.as_str(), value.as_str()),
    }
    .boxed();
    attribute
        .try_map(move |attribute, span| {
            if attribute.0 == local {
                Ok(attribute.1)
            } else {
                Err(ParserError::ExpectedFound { span })
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

fn parser<'a, I>() -> BoxedParser<'a, I, ast::Instruction>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let if_start = element_start("if");
    let variable_start = element_start("variable");

    let text = select! {
        Token::Text { text } => text,
    }
    .boxed();

    let sequence_constructor = text
        .map(|text| ast::SequenceConstructorItem::Text(text.to_string()))
        .boxed();

    let test_attribute_str = attribute("test");

    let parse_xpath = |value, _span, state: &mut State| {
        Ok(xee_xpath_ast::ast::XPath::parse(
            value,
            state.namespaces.as_ref(),
            &[],
        )?)
    };

    let test_attribute = test_attribute_str.try_map_with_state(parse_xpath);

    let if_attributes = test_attribute.repeated().collect::<Vec<_>>();

    #[derive(Debug)]
    enum VariableAttribute {
        Name(String),
        Select(xee_xpath_ast::ast::XPath),
    }

    let select_attribute_str = attribute("select");
    let select_attribute = select_attribute_str
        .try_map_with_state(parse_xpath)
        .map(VariableAttribute::Select);
    let name_attribute_str = attribute("name").map(|s| VariableAttribute::Name(s.to_string()));

    let variable_attribute = select_attribute.or(name_attribute_str);

    let variable_attributes = variable_attribute.repeated().collect::<Vec<_>>();

    let if_ = if_start
        .ignore_then(if_attributes)
        .then_ignore(element_end())
        .then(sequence_constructor.clone())
        .try_map_with_state(|(attributes, content), _span, _state: &mut State| {
            let test = attributes.into_iter().next().unwrap();
            Ok(ast::If {
                test,
                content: vec![content],
            })
        })
        .then_ignore(element_close())
        .map(ast::Instruction::If)
        .boxed();
    let variable = variable_start
        .ignore_then(variable_attributes)
        .then_ignore(element_end())
        .then(sequence_constructor)
        .try_map(|(attributes, content), span| {
            let mut select = None;
            let mut name = None;
            for attribute in attributes.into_iter() {
                match attribute {
                    VariableAttribute::Select(v) => {
                        select = Some(v);
                    }
                    VariableAttribute::Name(v) => {
                        name = Some(v);
                    }
                }
            }
            Ok(ast::Variable {
                name: name.ok_or(ParserError::ExpectedFound { span })?,
                select,
                content: vec![content],
                as_: None,
                static_: None,
                visibility: None,
            })
        })
        .then_ignore(element_close())
        .map(ast::Instruction::Variable)
        .boxed();
    if_.or(variable).boxed()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use insta::assert_ron_snapshot;
    use xee_xpath_ast::Namespaces;

    // #[test]
    // fn test_tokens() {
    //     let tokens = xmlparser::Tokenizer::from(r#"<if test="true()">Hello</if>"#);

    //     dbg!(tokens.collect::<Vec<_>>());
    // }

    #[test]
    fn test_simple_parse_if() {
        let stream = tokens(r#"<if test="true()">Hello</if>"#);
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser().parse_with_state(stream, &mut state).into_result());
    }

    #[test]
    fn test_simple_parse_variable() {
        let stream = tokens(r#"<variable name="foo" select="true()">Hello</variable>"#);
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser().parse_with_state(stream, &mut state).into_result());
    }
}
