use chumsky::input::Stream;
use chumsky::util::MaybeRef;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use xot::Xot;

use crate::parse::{Span, State};

use crate::ast_core as ast;

type Extra<'a> = Full<ParserError, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    ElementStartOpen { name: xot::NameId, span: Span },
    AttributeName { name: xot::NameId, span: Span },
    AttributeValue { value: &'a str, span: Span },
    ElementStartClose { name: xot::NameId, span: Span },
    ElementEnd { name: xot::NameId, span: Span },
    Text { value: &'a str, span: Span },
    Comment { value: &'a str, span: Span },
    ProcessingInstructionTarget { target: &'a str, span: Span },
    ProcessingInstructionContent { content: &'a str, span: Span },
    Error,
}

impl<'a> Token<'a> {
    fn span(&self) -> Span {
        use Token::*;

        match self {
            ElementStartOpen { span, .. } => *span,
            AttributeName { span, .. } => *span,
            AttributeValue { span, .. } => *span,
            ElementStartClose { span, .. } => *span,
            ElementEnd { span, .. } => *span,
            Text { span, .. } => *span,
            Comment { span, .. } => *span,
            ProcessingInstructionTarget { span, .. } => *span,
            ProcessingInstructionContent { span, .. } => *span,
            Error => Span::new(0, 0),
        }
    }
}

struct TokenIterator<'a, I: Iterator<Item = (xot::Node, xot::Output<'a>)>> {
    span_info: xot::SpanInfo,
    output_iterator: I,
    want_extra: bool,
    extra: Option<Token<'a>>,
}

impl<'a, I: Iterator<Item = (xot::Node, xot::Output<'a>)>> TokenIterator<'a, I> {
    fn new(span_info: xot::SpanInfo, output_iterator: I) -> Self {
        Self {
            span_info,
            output_iterator,
            want_extra: false,
            extra: None,
        }
    }
}

impl<'a, I: Iterator<Item = (xot::Node, xot::Output<'a>)>> TokenIterator<'a, I> {
    fn next_result<'b>(
        &mut self,
        node: xot::Node,
        output: &'b xot::Output<'a>,
    ) -> Result<Option<Token<'a>>, ParserError> {
        use xot::Output::*;
        Ok(match output {
            StartTagOpen(element) => Some(Token::ElementStartOpen {
                name: element.name(),
                span: self
                    .span_info
                    .get(xot::SpanInfoKey::ElementStart(node))
                    .ok_or(ParserError::SpanInfoMissing)?
                    .range()
                    .into(),
            }),
            Attribute(_, name, value) => {
                if !self.want_extra {
                    self.want_extra = true;
                    Some(Token::AttributeName {
                        name: *name,
                        span: self
                            .span_info
                            .get(xot::SpanInfoKey::AttributeName(node, *name))
                            .ok_or(ParserError::SpanInfoMissing)?
                            .range()
                            .into(),
                    })
                } else {
                    self.want_extra = false;
                    Some(Token::AttributeValue {
                        value,
                        span: self
                            .span_info
                            .get(xot::SpanInfoKey::AttributeValue(node, *name))
                            .ok_or(ParserError::SpanInfoMissing)?
                            .range()
                            .into(),
                    })
                }
            }
            StartTagClose(element) => Some(Token::ElementStartClose {
                name: element.name(),
                span: self
                    .span_info
                    // reuse use element start span
                    .get(xot::SpanInfoKey::ElementStart(node))
                    .ok_or(ParserError::SpanInfoMissing)?
                    .range()
                    .into(),
            }),
            EndTag(element) => Some(Token::ElementEnd {
                name: element.name(),
                span: self
                    .span_info
                    .get(xot::SpanInfoKey::ElementEnd(node))
                    .ok_or(ParserError::SpanInfoMissing)?
                    .range()
                    .into(),
            }),
            Text(text) => Some(Token::Text {
                value: text,
                span: self
                    .span_info
                    .get(xot::SpanInfoKey::Text(node))
                    .ok_or(ParserError::SpanInfoMissing)?
                    .range()
                    .into(),
            }),
            Comment(comment) => Some(Token::Comment {
                value: comment,
                span: self
                    .span_info
                    .get(xot::SpanInfoKey::Comment(node))
                    .ok_or(ParserError::SpanInfoMissing)?
                    .range()
                    .into(),
            }),
            ProcessingInstruction(target, content) => {
                if !self.want_extra {
                    self.want_extra = true;
                    Some(Token::ProcessingInstructionTarget {
                        target,

                        span: self
                            .span_info
                            .get(xot::SpanInfoKey::PiTarget(node))
                            .ok_or(ParserError::SpanInfoMissing)?
                            .range()
                            .into(),
                    })
                } else {
                    self.want_extra = false;
                    if let Some(content) = content {
                        Some(Token::ProcessingInstructionContent {
                            content,
                            span: self
                                .span_info
                                .get(xot::SpanInfoKey::PiContent(node))
                                .ok_or(ParserError::SpanInfoMissing)?
                                .range()
                                .into(),
                        })
                    } else {
                        None
                    }
                }
            }
            Prefix(..) | PrefixesFinished(..) | AttributesFinished(..) => None,
        })
    }
}

impl<'a, I: Iterator<Item = (xot::Node, xot::Output<'a>)>> Iterator for TokenIterator<'a, I> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(extra) = self.extra.take() {
            return Some(extra);
        }
        while let Some((node, output)) = self.output_iterator.next() {
            if let Ok(token_option) = self.next_result(node, &output) {
                if token_option.is_some() {
                    // stupid protocol for those cases we need two tokens for one output
                    if self.want_extra {
                        if let Ok(extra) = self.next_result(node, &output) {
                            self.extra = extra;
                        } else {
                            return Some(Token::Error);
                        }
                    }
                    return token_option;
                } else {
                    // skip any outputs that don't have representation
                    continue;
                }
            } else {
                return Some(Token::Error);
            }
        }
        None
    }
}

fn tokens<'a>(
    xot: &'a mut Xot,
    src: &'a str,
) -> Result<impl ValueInput<'a, Token = Token<'a>, Span = Span>, ParserError> {
    let (node, span_info) = xot.parse_with_span_info(src)?;
    let iterator = TokenIterator::new(span_info, xot.outputs(node));

    Ok(
        Stream::from_iter(iterator.map(|token| (token, token.span())))
            .spanned(Span::new(src.len(), src.len())),
    )
}

#[cfg_attr(test, derive(serde::Serialize))]
pub enum ParserError {
    ExpectedFound {
        span: Span,
    },
    MyError,
    SpanInfoMissing,
    XPath(xee_xpath_ast::ParserError),
    #[cfg_attr(test, serde(skip))]
    Xot(xot::Error),
}

impl From<xee_xpath_ast::ParserError> for ParserError {
    fn from(e: xee_xpath_ast::ParserError) -> Self {
        Self::XPath(e)
    }
}

impl From<xot::Error> for ParserError {
    fn from(e: xot::Error) -> Self {
        Self::Xot(e)
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

// fn element_start<'a, I>(local: &'a str) -> BoxedParser<'a, I, ()>
// where
//     I: ValueInput<'a, Token = Token<'a>, Span = Span>,
// {
//     let element = select! {
//         Token::ElementStart { local, .. } => Element(local.as_str()),
//     }
//     .boxed();
//     element
//         .try_map(move |element, span| {
//             if element.0 == local {
//                 Ok(())
//             } else {
//                 Err(ParserError::ExpectedFound { span })
//             }
//         })
//         .boxed()
// }

// fn attribute<'a, I>(local: &'a str) -> BoxedParser<'a, I, &'a str>
// where
//     I: ValueInput<'a, Token = Token<'a>, Span = Span>,
// {
//     let attribute = select! {
//         Token::Attribute { local, value, ..} => Attribute(local.as_str(), value.as_str()),
//     }
//     .boxed();
//     attribute
//         .try_map(move |attribute, span| {
//             if attribute.0 == local {
//                 Ok(attribute.1)
//             } else {
//                 Err(ParserError::ExpectedFound { span })
//             }
//         })
//         .boxed()
// }

// fn element_end<'a, I>() -> BoxedParser<'a, I, ()>
// where
//     I: ValueInput<'a, Token = Token<'a>, Span = Span>,
// {
//     select! {
//         Token::ElementEnd { end: xmlparser::ElementEnd::Open, .. } => (),
//     }
//     .boxed()
// }

// fn element_close<'a, I>() -> BoxedParser<'a, I, ()>
// where
//     I: ValueInput<'a, Token = Token<'a>, Span = Span>,
// {
//     select! {
//         Token::ElementEnd { end: xmlparser::ElementEnd::Close(..) | xmlparser::ElementEnd::Empty, .. } => (),
//     }.boxed()
// }

// fn parser<'a, I>() -> BoxedParser<'a, I, ast::Instruction>
// where
//     I: ValueInput<'a, Token = Token<'a>, Span = Span>,
// {
//     let if_start = element_start("if");
//     let variable_start = element_start("variable");

//     let text = select! {
//         Token::Text { text } => text,
//     }
//     .boxed();

//     let sequence_constructor = text
//         .map(|text| ast::SequenceConstructor::Text(text.to_string()))
//         .boxed();

//     let test_attribute_str = attribute("test");

//     let parse_xpath = |value, _span, state: &mut State| {
//         Ok(xee_xpath_ast::ast::XPath::parse(
//             value,
//             state.namespaces.as_ref(),
//             &[],
//         )?)
//     };

//     let test_attribute = test_attribute_str.try_map_with_state(parse_xpath);

//     let if_attributes = test_attribute.repeated().collect::<Vec<_>>();

//     #[derive(Debug)]
//     enum VariableAttribute {
//         Name(String),
//         Select(xee_xpath_ast::ast::XPath),
//     }

//     let select_attribute_str = attribute("select");
//     let select_attribute = select_attribute_str
//         .try_map_with_state(parse_xpath)
//         .map(VariableAttribute::Select);
//     let name_attribute_str = attribute("name").map(|s| VariableAttribute::Name(s.to_string()));

//     let variable_attribute = select_attribute.or(name_attribute_str);

//     let variable_attributes = variable_attribute.repeated().collect::<Vec<_>>();

//     let if_ = if_start
//         .ignore_then(if_attributes)
//         .then_ignore(element_end())
//         .then(sequence_constructor.clone())
//         .try_map_with_state(|(attributes, content), _span, _state: &mut State| {
//             let test = attributes.into_iter().next().unwrap();
//             Ok(ast::If {
//                 test,
//                 content: vec![content],
//             })
//         })
//         .then_ignore(element_close())
//         .map(ast::Instruction::If)
//         .boxed();
//     let variable = variable_start
//         .ignore_then(variable_attributes)
//         .then_ignore(element_end())
//         .then(sequence_constructor)
//         .try_map(|(attributes, content), span| {
//             let mut select = None;
//             let mut name = None;
//             for attribute in attributes.into_iter() {
//                 match attribute {
//                     VariableAttribute::Select(v) => {
//                         select = Some(v);
//                     }
//                     VariableAttribute::Name(v) => {
//                         name = Some(v);
//                     }
//                 }
//             }
//             Ok(ast::Variable {
//                 name: name.ok_or(ParserError::ExpectedFound { span })?,
//                 select,
//                 content: vec![content],
//             })
//         })
//         .then_ignore(element_close())
//         .map(ast::Instruction::Variable)
//         .boxed();
//     if_.or(variable).boxed()
// }

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

    // #[test]
    // fn test_simple_parse_if() {
    //     let mut xot = Xot::new();
    //     let stream = tokens(&mut xot, r#"<if test="true()">Hello</if>"#);
    //     let namespaces = Namespaces::default();
    //     let mut state = State {
    //         namespaces: Cow::Owned(namespaces),
    //     };
    //     assert_ron_snapshot!(parser().parse_with_state(stream, &mut state).into_result());
    // }

    // #[test]
    // fn test_simple_parse_variable() {
    //     let mut xot = Xot::new();
    //     let stream = tokens(
    //         &mut xot,
    //         r#"<variable name="foo" select="true()">Hello</variable>"#,
    //     );
    //     let namespaces = Namespaces::default();
    //     let mut state = State {
    //         namespaces: Cow::Owned(namespaces),
    //     };
    //     assert_ron_snapshot!(parser().parse_with_state(stream, &mut state).into_result());
    // }
}
