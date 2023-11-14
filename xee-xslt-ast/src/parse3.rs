use chumsky::input::Stream;
use chumsky::util::MaybeRef;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use xot::Xot;

use crate::parse::{Span, State};

use crate::ast_core as ast;

type Extra<'a> = Full<ParserError, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    done: bool,
}

impl<'a, I: Iterator<Item = (xot::Node, xot::Output<'a>)>> TokenIterator<'a, I> {
    fn new(span_info: xot::SpanInfo, output_iterator: I) -> Self {
        Self {
            span_info,
            output_iterator,
            want_extra: false,
            extra: None,
            done: false,
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
        // to ensure we don't get called after exhaustion; output_iterator doesn't
        // seem to like it
        if self.done {
            return None;
        }
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
        self.done = true;
        None
    }
}

fn tokens<'a>(
    xot: &'a mut Xot,
    src: &'a str,
) -> Result<TokenIterator<'a, impl Iterator<Item = (xot::Node, xot::Output<'a>)>>, ParserError> {
    let (node, span_info) = xot.parse_with_span_info(src)?;
    Ok(TokenIterator::new(span_info, xot.outputs(node)))
}

fn token_stream<'a>(
    xot: &'a mut Xot,
    src: &'a str,
) -> Result<impl ValueInput<'a, Token = Token<'a>, Span = Span>, ParserError> {
    let iterator = tokens(xot, src)?;
    Ok(
        Stream::from_iter(iterator.map(|token| (token, token.span())))
            .spanned(Span::new(src.len(), src.len())),
    )
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ParserError {
    ExpectedFound {
        span: Span,
    },
    MissingRequiredAttribute {
        // TODO: use NameId here to indicate missing attribute?
        // but attributes could be both in xsl namespace or not
        name: String,
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

fn element_start<'a, I>(match_name: xot::NameId) -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::ElementStartOpen { name, .. } if name == match_name => (),
    }
    .boxed()
}

fn attribute_name<'a, I>(match_name: xot::NameId) -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::AttributeName { name, ..} if name == match_name => (),
    }
    .boxed()
}

fn attribute_value<'a, I>() -> BoxedParser<'a, I, &'a str>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::AttributeValue { value, ..} => value,
    }
    .boxed()
}

fn element_close<'a, I>() -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::ElementStartClose { .. } => (),
    }
    .boxed()
}

fn element_end<'a, I>() -> BoxedParser<'a, I, ()>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::ElementEnd { .. } => (),
    }
    .boxed()
}

struct Names {
    if_: xot::NameId,
    test: xot::NameId,
    variable: xot::NameId,
    select: xot::NameId,
    name: xot::NameId,
}

impl Names {
    fn new(xot: &mut Xot) -> Self {
        Self {
            if_: xot.add_name("if"),
            test: xot.add_name("test"),
            variable: xot.add_name("variable"),
            select: xot.add_name("select"),
            name: xot.add_name("name"),
        }
    }
}

fn parser<'a, I>(names: &Names) -> BoxedParser<'a, I, ast::Instruction>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let text = select! {
        Token::Text { value, .. } => value,
    }
    .boxed();

    let sequence_constructor = text
        .map(|text| ast::SequenceConstructorItem::Text(text.to_string()))
        .boxed();

    let parse_xpath = |value, span: Span, state: &mut State| {
        Ok(
            xee_xpath_ast::ast::XPath::parse(value, state.namespaces.as_ref(), &[])
                .map_err(|e| e.adjust(span.start))?,
        )
    };

    let xpath_value = attribute_value().try_map_with_state(parse_xpath);

    let if_attributes = (attribute_name(names.test).ignore_then(xpath_value))
        .repeated()
        .collect::<Vec<_>>();

    let if_ = element_start(names.if_)
        .ignore_then(if_attributes)
        .then_ignore(element_close())
        .then(sequence_constructor.clone())
        .try_map_with_state(|(attributes, content), _span, _state: &mut State| {
            let test = attributes.into_iter().next().unwrap();
            Ok(ast::If {
                test,
                content: vec![content],
            })
        })
        .then_ignore(element_end())
        .map(ast::Instruction::If)
        .boxed();

    #[derive(Debug)]
    enum VariableAttribute {
        Name(String),
        Select(xee_xpath_ast::ast::XPath),
    }

    let select_attribute = attribute_name(names.select)
        .ignore_then(attribute_value().try_map_with_state(parse_xpath))
        .map(VariableAttribute::Select);
    let name_attribute = attribute_name(names.name)
        .ignore_then(attribute_value())
        .map(|name| VariableAttribute::Name(name.to_string()));
    let variable_attribute = select_attribute.or(name_attribute);

    let variable_attributes = variable_attribute.repeated().collect::<Vec<_>>();

    let variable = element_start(names.variable)
        .map_with_span(|_, span| span)
        .then(variable_attributes)
        .then_ignore(element_close())
        .then(sequence_constructor)
        .try_map(|((element_span, attributes), content), _| {
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
                name: name.ok_or(ParserError::MissingRequiredAttribute {
                    name: "name".to_string(),
                    span: element_span,
                })?,
                select,
                content: vec![content],
            })
        })
        .then_ignore(element_end())
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

    #[test]
    fn test_tokens() {
        let mut xot = Xot::new();
        let if_name = xot.add_name("if");
        let test_name = xot.add_name("test");

        let tokens = tokens(&mut xot, r#"<if test="true()">Hello</if>"#).unwrap();
        //                                  0123456789012345678901234567
        assert_eq!(
            tokens.collect::<Vec<_>>(),
            vec![
                Token::ElementStartOpen {
                    name: if_name,
                    span: Span::new(1, 3)
                },
                Token::AttributeName {
                    name: test_name,
                    span: Span::new(4, 8)
                },
                Token::AttributeValue {
                    value: "true()",
                    span: Span::new(10, 16)
                },
                Token::ElementStartClose {
                    name: if_name,
                    span: Span::new(1, 3)
                },
                Token::Text {
                    value: "Hello",
                    span: Span::new(18, 23)
                },
                Token::ElementEnd {
                    name: if_name,
                    span: Span::new(23, 28)
                },
            ]
        );
    }

    #[test]
    fn test_simple_parse_if() {
        let mut xot = Xot::new();

        let names = Names::new(&mut xot);

        let stream = token_stream(&mut xot, r#"<if test="true()">Hello</if>"#).unwrap();
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };

        assert_ron_snapshot!(parser(&names)
            .parse_with_state(stream, &mut state)
            .into_result());
    }

    #[test]
    fn test_simple_parse_variable() {
        let mut xot = Xot::new();

        let names = Names::new(&mut xot);

        let stream = token_stream(
            &mut xot,
            r#"<variable name="foo" select="true()">Hello</variable>"#,
        )
        .unwrap();
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser(&names)
            .parse_with_state(stream, &mut state)
            .into_result());
    }

    #[test]
    fn test_simple_parse_variable_missing_required_name_attribute() {
        let mut xot = Xot::new();

        let names = Names::new(&mut xot);

        let stream =
            token_stream(&mut xot, r#"<variable select="true()">Hello</variable>"#).unwrap();
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser(&names)
            .parse_with_state(stream, &mut state)
            .into_result());
    }

    #[test]
    fn test_simple_parse_variable_broken_xpath() {
        let mut xot = Xot::new();

        let names = Names::new(&mut xot);

        let stream = token_stream(
            &mut xot,
            r#"<variable name="foo" select="let $x := 1">Hello</variable>"#,
        )
        .unwrap();
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser(&names)
            .parse_with_state(stream, &mut state)
            .into_result());
    }
}
