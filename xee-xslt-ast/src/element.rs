use xee_xpath_ast::Namespaces;
use xot::{NameId, Node, Value};

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::attributes::Attributes;
use crate::combinator::{end, one, NodeParser, OneParser};
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::state::State;

struct ElementParsers {
    sequence_constructor_sibling_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    sequence_constructor_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    declarations_parser: Box<dyn NodeParser<Vec<ast::Declaration>>>,
}

impl ElementParsers {
    fn new() -> Self {
        let sequence_constructor_sibling_parser = one(|node, state, context| {
            match state.xot.value(node) {
                Value::Text(text) => Ok(ast::SequenceConstructorItem::TextNode(
                    text.get().to_string(),
                )),
                Value::Element(element) => {
                    let attributes = Attributes::new(node, element, state, context.clone())?;
                    let context = context.sub(element.prefixes(), attributes.standard()?);
                    let element = Element::new(node, element, context, state)?;
                    ast::SequenceConstructorItem::parse_sequence_constructor_item(
                        &element,
                        &attributes,
                    )
                }
                _ => Err(ElementError::Unexpected {
                    // TODO: get span right
                    span: Span::new(0, 0),
                }),
            }
        })
        .many();

        let sequence_constructor_parser = sequence_constructor_sibling_parser
            .clone()
            .then_ignore(end())
            .contains();

        let declarations_parser = one(|node, state, context| match state.xot.value(node) {
            Value::Element(element) => {
                let attributes = Attributes::new(node, element, state, context.clone())?;
                let context = context.sub(element.prefixes(), attributes.standard()?);
                let element = Element::new(node, element, context, state)?;
                ast::Declaration::parse_declaration(&element, &attributes)
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
        let attributes = Attributes::new(node, element, state, context.clone())?;
        let context = context.sub(element.prefixes(), attributes.standard()?);
        let element = Element::new(node, element, context, state)?;
        f(&element, &attributes)
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
    pub(crate) context: Context<'a>,
    pub(crate) state: &'a State,
}

impl<'a> Element<'a> {
    pub(crate) fn new(
        node: Node,
        element: &'a xot::Element,
        context: Context<'a>,
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

    fn namespaces(&'a self) -> Namespaces<'a> {
        self.context.namespaces(self.state)
    }
}
