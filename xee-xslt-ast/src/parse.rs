use ahash::HashMap;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use std::borrow::Cow;
use xee_xpath_ast::ast as xpath_ast;
use xee_xpath_ast::Namespaces;
use xot::{Node, Xot};

use crate::ast_core as ast;

pub(crate) struct State<'a> {
    pub(crate) namespaces: Cow<'a, Namespaces<'a>>,
}

type Extra<'a, T> = Full<Rich<'a, T>, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a, Token<'a>>>;

pub type Span = SimpleSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) enum Token<'a> {
    ElementStart(Name<'a>, HashMap<Name<'a>, &'a str>),
    ElementEnd(Name<'a>),
    Text(&'a str),
    Comment(&'a str),
    ProcessingInstruction(&'a str, Option<&'a str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) struct Name<'a> {
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
        .map(|text| ast::SequenceConstructor::Text(text.to_string()))
        .boxed();

    let if_instruction = if_
        .then(sequence_constructor)
        .try_map_with_state(|(attributes, content), span, state: &mut State| {
            let name = Name {
                namespace: "",
                localname: "test",
            };
            let test = attributes.get(&name).unwrap();
            // let test = xpath_ast::XPath::parse(test, state.namespaces.as_ref(), &[])?;
            Ok(ast::If {
                test: test.to_string(),
                content: vec![content],
            })
        })
        .then_ignore(just(Token::ElementEnd(Name {
            namespace: "",
            localname: "if",
        })))
        .boxed();
    if_instruction
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
        assert_ron_snapshot!(parser()
            .parse_with_state(stream, &mut state)
            .into_result()
            .unwrap());
    }
}
