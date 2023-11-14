use ahash::HashMap;
use chumsky::util::MaybeRef;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use std::borrow::Cow;
use xee_xpath_ast::Namespaces;
use xot::{Node, Xot};

use crate::ast_core as ast;

pub(crate) struct State<'a> {
    pub(crate) namespaces: Cow<'a, Namespaces<'a>>,
}

type Extra<'a> = Full<ParserError, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a>>;

pub type Span = SimpleSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Token<'a> {
    ElementStart(Name<'a>, HashMap<Name<'a>, &'a str>),
    ElementEnd(Name<'a>),
    Text(&'a str),
    Comment(&'a str),
    ProcessingInstruction(&'a str, Option<&'a str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Name<'a> {
    namespace: &'a str,
    localname: &'a str,
}

impl<'a> From<(&'a str, &'a str)> for Name<'a> {
    fn from((localname, namespace): (&'a str, &'a str)) -> Self {
        Self {
            namespace,
            localname,
        }
    }
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

struct TokenizedTraverse<'a, T: Iterator<Item = xot::NodeEdge>> {
    xot: &'a Xot,
    traverse: T,
}

impl<'a, T: Iterator<Item = xot::NodeEdge>> Iterator for TokenizedTraverse<'a, T> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let edge = self.traverse.next()?;
        Some(match edge {
            xot::NodeEdge::Start(node) => match self.xot.value(node) {
                xot::Value::Element(e) => {
                    let name: Name = self.xot.name_ns_str(e.name()).into();
                    let attributes = e
                        .attributes()
                        .iter()
                        .map(|(name, value)| {
                            let name: Name = self.xot.name_ns_str(*name).into();
                            (name, value.as_ref())
                        })
                        .collect::<HashMap<_, _>>();
                    Token::ElementStart(name, attributes)
                }
                xot::Value::Text(text) => Token::Text(text.get()),
                xot::Value::Comment(comment) => Token::Comment(comment.get()),
                xot::Value::ProcessingInstruction(pi) => {
                    Token::ProcessingInstruction(pi.target(), pi.data())
                }
                xot::Value::Root => self.next()?,
            },
            xot::NodeEdge::End(node) => {
                if let xot::Value::Element(e) = self.xot.value(node) {
                    let name: Name = self.xot.name_ns_str(e.name()).into();
                    Token::ElementEnd(name)
                } else {
                    self.next()?
                }
            }
        })
    }
}

fn tokenize(xot: &Xot, node: Node) -> TokenizedTraverse<impl Iterator<Item = xot::NodeEdge> + '_> {
    TokenizedTraverse {
        xot,
        traverse: xot.traverse(node),
    }
}

fn parser<'a, I>() -> BoxedParser<'a, I, ast::If>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let if_ = select! {
        Token::ElementStart(Name { namespace: "", localname: "if"}, attrs) => attrs,
    }
    .boxed();

    let text = select! {
        Token::Text(s) => s,
    }
    .boxed();

    let sequence_constructor = text
        .map(|text| ast::SequenceConstructorItem::Text(text.to_string()))
        .boxed();

    if_.then(sequence_constructor)
        .try_map_with_state(|(attributes, content), _span, state: &mut State| {
            let name = Name {
                namespace: "",
                localname: "test",
            };
            let test = attributes.get(&name).unwrap();
            let namespaces = state.namespaces.as_ref();

            let test = xee_xpath_ast::ast::XPath::parse(test, namespaces, &[])?;
            Ok(ast::If {
                test,
                content: vec![content],
            })
        })
        .then_ignore(just(Token::ElementEnd(Name {
            namespace: "",
            localname: "if",
        })))
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::input::Stream;
    use insta::assert_ron_snapshot;

    #[test]
    fn test_tokenize() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<if test="true()">Hello</if>"#).unwrap();
        let tokens = tokenize(&xot, doc).collect::<Vec<_>>();
        assert_ron_snapshot!(tokens);
    }

    #[test]
    fn test_simple_parse_if() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<if test="true()">Hello</if>"#).unwrap();
        let tokens = tokenize(&xot, doc);
        let stream = Stream::from_iter(tokens);
        let namespaces = Namespaces::default();
        let mut state = State {
            namespaces: Cow::Owned(namespaces),
        };
        assert_ron_snapshot!(parser().parse_with_state(stream, &mut state).into_result());
    }
}
