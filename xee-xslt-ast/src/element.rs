use std::sync::OnceLock;

use xee_xpath_ast::Namespaces;
use xot::{NameId, Node, Value};

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::attributes::Attributes;
use crate::combinator::{multi, one, Content, NodeParser, OneParser};
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::state::State;
use crate::value_template::{ValueTemplateItem, ValueTemplateTokenizer};

pub(crate) type NodeParserLock<V> =
    OnceLock<Box<dyn NodeParser<V> + std::marker::Sync + std::marker::Send>>;

// We use OnceLock to declare content parser once, and then reuse them
pub(crate) type ContentParseLock<V> = OnceLock<
    Box<dyn Fn(&Content) -> Result<V, ElementError> + std::marker::Sync + std::marker::Send>,
>;

static SEQUENCE_CONSTRUCTOR: NodeParserLock<ast::SequenceConstructor> = OnceLock::new();
static SEQUENCE_CONSTRUCTOR_CONTENT: ContentParseLock<ast::SequenceConstructor> = OnceLock::new();
static DECLARATIONS_CONTENT: ContentParseLock<ast::Declarations> = OnceLock::new();

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

impl<'a> Content<'a> {
    pub(crate) fn parse_element<V>(
        self,
        element: &'a xot::Element,
        f: impl FnOnce(&Attributes<'a>) -> Result<V, ElementError>,
    ) -> Result<V, ElementError> {
        // first we want to be aware of the ns prefixes of the new element
        let content = self.with_context(self.context.with_prefixes(element.prefixes()));
        // we create an attributes object to obtain the standard attributes
        let attributes = Attributes::new(content.clone(), element);
        // after this, we construct a new content based on the standard attributes
        let content = content.with_context(content.context.with_standard(attributes.standard()?));
        // we create a new attributes object with the new content
        let attributes = attributes.with_content(content);
        f(&attributes)
    }

    pub(crate) fn span(&self) -> Result<Span, ElementError> {
        self.state.span(self.node).ok_or(ElementError::Internal)
    }

    pub(crate) fn sequence_constructor(&self) -> Result<ast::SequenceConstructor, ElementError> {
        SEQUENCE_CONSTRUCTOR_CONTENT.get_or_init(|| children(sequence_constructor()))(self)
    }

    pub(crate) fn declarations(&self) -> Result<ast::Declarations, ElementError> {
        DECLARATIONS_CONTENT.get_or_init(|| children(declarations()))(self)
    }
}

pub(crate) fn sequence_constructor() -> impl NodeParser<ast::SequenceConstructor> {
    multi(|content| {
        let node = content.node;
        let state = &content.state;
        let context = &content.context;
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
            Value::Element(element) => content.parse_element(element, |attributes| {
                Ok(vec![
                    ast::SequenceConstructorItem::parse_sequence_constructor_item(attributes)?,
                ])
            }),
            _ => Err(ElementError::Unexpected {
                // TODO: get span right
                span: Span::new(0, 0),
            }),
        }
    })
    .flatten()
}

fn declarations() -> impl NodeParser<ast::Declarations> {
    one(|content| {
        match content.state.xot.value(content.node) {
            Value::Element(element) => content.parse_element(element, |attributes| {
                ast::Declaration::parse_declaration(attributes)
            }),
            _ => Err(ElementError::Unexpected {
                // TODO: get span right
                span: Span::new(0, 0),
            }),
        }
    })
    .many()
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

type ContentParse<V> =
    Box<dyn Fn(&Content) -> Result<V, ElementError> + std::marker::Sync + std::marker::Send>;

pub(crate) fn children<V, P>(parser: P) -> ContentParse<V>
where
    P: NodeParser<V> + std::marker::Sync + std::marker::Send + 'static,
    V: std::marker::Sync + std::marker::Send + 'static,
{
    Box::new(move |content| {
        let (item, next) = parser.parse_next(
            content.state.xot.first_child(content.node),
            content.state,
            &content.context,
        )?;
        // handle end of content check here
        if let Some(next) = next {
            Err(ElementError::Unexpected {
                span: content.state.span(next).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(item)
        }
    })
}

pub(crate) fn by_element<V>(
    f: impl Fn(&Attributes) -> Result<V, ElementError>,
) -> impl Fn(Content) -> Result<V, ElementError> {
    move |content| {
        let element = content
            .state
            .xot
            .element(content.node)
            .ok_or(ElementError::Unexpected {
                span: content
                    .state
                    .span(content.node)
                    .ok_or(ElementError::Internal)?,
            })?;
        content.parse_element(element, &f)
    }
}

pub(crate) fn by_element_name<V>(
    name: NameId,
    f: impl Fn(&Attributes) -> Result<V, ElementError>,
) -> impl Fn(Content) -> Result<V, ElementError> {
    by_element(move |attributes| {
        if attributes.element.name() == name {
            f(attributes)
        } else {
            Err(ElementError::Unexpected {
                span: attributes.content.span()?,
            })
        }
    })
}

pub(crate) fn by_instruction<V: InstructionParser>(
    name: NameId,
) -> impl Fn(Content) -> Result<V, ElementError> {
    by_element_name(name, move |attributes| V::parse_and_validate(attributes))
}

pub(crate) fn instruction<V: InstructionParser>(
    name: NameId,
) -> OneParser<V, impl Fn(Content) -> Result<V, ElementError>> {
    one(by_instruction(name))
}
