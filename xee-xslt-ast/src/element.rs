use xee_xpath_ast::Namespaces;
use xot::{NameId, Node, Value};

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::attributes::Attributes;
use crate::combinator::{end, multi, one, NodeParser, OneParser};
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::state::State;
use crate::value_template::{ValueTemplateItem, ValueTemplateTokenizer};

pub(crate) fn parse_element_attributes<'a, V>(
    node: Node,
    element: &'a xot::Element,
    state: &'a State,
    context: &Context,
    f: impl FnOnce(&Element<'a>, &Attributes<'a>) -> Result<V, ElementError>,
) -> Result<V, ElementError> {
    let attributes = Attributes::new(node, element, state, context.clone())?;
    let context = context.sub(element.prefixes(), attributes.standard()?);
    let element = Element::new(node, element, context, state)?;
    f(&element, &attributes)
}

struct ElementParsers {
    sequence_constructor_sibling_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    sequence_constructor_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    declarations_parser: Box<dyn NodeParser<Vec<ast::Declaration>>>,
}

impl ElementParsers {
    fn new() -> Self {
        let sequence_constructor_sibling_parser = multi(|node, state, context| {
            match state.xot.value(node) {
                Value::Text(text) => {
                    let span = state.span(node).ok_or(ElementError::Internal)?;
                    let namespaces = context.namespaces(state);
                    if context.expand_text {
                        text_value_template(text.get(), span, &namespaces)
                    } else {
                        Ok(vec![ast::SequenceConstructorItem::Content(
                            ast::Content::Text(text.get().to_string()),
                        )])
                    }
                }
                Value::Element(element) => parse_element_attributes(
                    node,
                    element,
                    state,
                    context,
                    |element, attributes| {
                        Ok(vec![
                            ast::SequenceConstructorItem::parse_sequence_constructor_item(
                                element, attributes,
                            )?,
                        ])
                    },
                ),
                _ => Err(ElementError::Unexpected {
                    // TODO: get span right
                    span: Span::new(0, 0),
                }),
            }
        })
        .flatten();

        let sequence_constructor_parser = sequence_constructor_sibling_parser
            .clone()
            .then_ignore(end())
            .contains();

        let declarations_parser = one(|node, state, context| match state.xot.value(node) {
            Value::Element(element) => {
                parse_element_attributes(node, element, state, context, |element, attributes| {
                    ast::Declaration::parse_declaration(element, attributes)
                })
            }
            _ => Err(ElementError::Unexpected {
                // TODO: get span right
                span: Span::new(0, 0),
            }),
        })
        .many()
        .then_ignore(end())
        .contains();

        Self {
            sequence_constructor_sibling_parser: Box::new(sequence_constructor_sibling_parser),
            sequence_constructor_parser: Box::new(sequence_constructor_parser),
            declarations_parser: Box::new(declarations_parser),
        }
    }
}

fn text_value_template(
    s: &str,
    span: Span,
    namespaces: &Namespaces,
) -> Result<Vec<ast::SequenceConstructorItem>, ElementError> {
    let mut items = Vec::new();
    for token in ValueTemplateTokenizer::new(s, span, namespaces, &[]) {
        let token = token?;
        let content = match token {
            ValueTemplateItem::String { text, span: _ } => ast::Content::Text(text.to_string()),
            ValueTemplateItem::Curly { c } => ast::Content::Text(c.to_string()),
            ValueTemplateItem::Value { xpath, span: _ } => ast::Content::Value(Box::new(xpath)),
        };
        let item = ast::SequenceConstructorItem::Content(content);
        items.push(item);
    }
    Ok(items)
}

pub(crate) struct XsltParser<'a> {
    state: &'a State,
}

impl<'a> XsltParser<'a> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self { state }
    }

    pub(crate) fn parse_transform(&self, node: Node) -> Result<ast::Transform, ElementError> {
        let parser = instruction(self.state.names.xsl_transform);
        parser.parse(Some(node), self.state, &Context::empty())
    }
}

pub(crate) fn by_element<V>(
    f: impl Fn(&Element, &Attributes) -> Result<V, ElementError>,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    move |node, state, context| {
        let element = state.xot.element(node).ok_or(ElementError::Unexpected {
            span: state.span(node).ok_or(ElementError::Internal)?,
        })?;
        parse_element_attributes(node, element, state, context, &f)
    }
}

pub(crate) fn by_element_name<V>(
    name: NameId,
    f: impl Fn(&Element, &Attributes) -> Result<V, ElementError>,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    by_element(move |element, attributes| {
        if element.element.name() == name {
            f(element, attributes)
        } else {
            Err(ElementError::Unexpected { span: element.span })
        }
    })
}

pub(crate) fn by_instruction<V: InstructionParser>(
    name: NameId,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    by_element_name(name, move |element, attributes| {
        V::parse_and_validate(element, attributes)
    })
}

pub(crate) fn instruction<V: InstructionParser>(
    name: NameId,
) -> OneParser<V, impl Fn(Node, &State, &Context) -> Result<V, ElementError>> {
    one(by_instruction(name))
}

pub(crate) struct SequenceConstructorNodeParser;

impl NodeParser<ast::SequenceConstructor> for SequenceConstructorNodeParser {
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(ast::SequenceConstructor, Option<Node>), ElementError> {
        let element_parsers = ElementParsers::new();
        element_parsers
            .sequence_constructor_sibling_parser
            .parse_next(node, state, context)
    }
}

pub(crate) fn sequence_constructor() -> SequenceConstructorNodeParser {
    SequenceConstructorNodeParser
}

pub(crate) fn content_parse<V, P>(parser: P) -> impl Fn(&Element) -> Result<V, ElementError>
where
    P: NodeParser<V>,
{
    move |element| {
        let (item, next) = parser.parse_next(
            element.state.xot.first_child(element.node),
            element.state,
            &element.context,
        )?;
        // handle end of content check here
        if let Some(next) = next {
            Err(ElementError::Unexpected {
                span: element.state.span(next).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(item)
        }
    }
}

pub(crate) struct Element<'a> {
    pub(crate) node: Node,
    pub(crate) element: &'a xot::Element,
    pub(crate) span: Span,
    pub(crate) context: Context,
    pub(crate) state: &'a State,
}

impl<'a> Element<'a> {
    pub(crate) fn new(
        node: Node,
        element: &'a xot::Element,
        context: Context,
        state: &'a State,
    ) -> Result<Self, ElementError> {
        let span = state.span(node).ok_or(ElementError::Internal)?;

        Ok(Self {
            node,
            element,
            span,
            context,
            state,
        })
    }

    pub(crate) fn sequence_constructor(&self) -> Result<ast::SequenceConstructor, ElementError> {
        let element_parsers = ElementParsers::new();
        element_parsers.sequence_constructor_parser.parse(
            Some(self.node),
            self.state,
            &self.context,
        )
    }

    pub(crate) fn declarations(&self) -> Result<ast::Declarations, ElementError> {
        let element_parsers = ElementParsers::new();
        element_parsers
            .declarations_parser
            .parse(Some(self.node), self.state, &self.context)
    }
}
