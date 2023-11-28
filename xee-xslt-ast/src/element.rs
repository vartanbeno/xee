use xee_xpath_ast::{ast as xpath_ast, Namespaces};
use xot::{NameId, Node, SpanInfoKey, Value};

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::combinator::{children, end, many, top, ElementError, NodeParser};
use crate::context::Context;
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::name::XmlName;
use crate::names::StandardNames;
use crate::state::State;
use crate::tokenize::split_whitespace_with_spans;
use crate::value_template::{self, ValueTemplateTokenizer};

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AttributeError {
    // Expected attribute of name, not found (element span)
    NotFound { name: XmlName, span: Span },
    // Did not expect attribute of name (attribute span)
    Unexpected { name: XmlName, span: Span },
    // The value of an attribute was invalid
    Invalid { value: String, span: Span },
    // An eqname was invalid
    InvalidEqName { value: String, span: Span },
    // XPath parser error
    XPath(xee_xpath_ast::ParserError),
    // A value templatecould not be parsed
    ValueTemplateError(value_template::Error),
    // Internal error; should not happen
    Internal,
}

impl From<xee_xpath_ast::ParserError> for AttributeError {
    fn from(e: xee_xpath_ast::ParserError) -> Self {
        AttributeError::XPath(e)
    }
}

impl From<value_template::Error> for AttributeError {
    fn from(e: value_template::Error) -> Self {
        AttributeError::ValueTemplateError(e)
    }
}

struct ElementParsers {
    sequence_constructor_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    declarations_parser: Box<dyn NodeParser<Vec<ast::Declaration>>>,
}

impl ElementParsers {
    fn new() -> Self {
        let sequence_constructor_parser = children(
            many(|node, state, context| match state.xot.value(node) {
                Value::Text(text) => Ok(ast::SequenceConstructorItem::TextNode(
                    text.get().to_string(),
                )),
                Value::Element(element) => {
                    let new_context = context.element(element);
                    let element = Element::new(node, element, new_context, state)?;
                    ast::SequenceConstructorItem::parse_sequence_constructor_item(&element)
                }
                _ => Err(ElementError::Unexpected {
                    // TODO: get span right
                    span: Span::new(0, 0),
                }),
            })
            .then_ignore(end()),
        );

        let declarations_parser = children(
            many(|node, state, context| match state.xot.value(node) {
                Value::Element(element) => {
                    let new_context = context.element(element);
                    let element = Element::new(node, element, new_context, state)?;
                    ast::Declaration::parse_declaration(&element)
                }
                _ => Err(ElementError::Unexpected {
                    // TODO: get span right
                    span: Span::new(0, 0),
                }),
            })
            .then_ignore(end()),
        );

        Self {
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
        let parser = top(instruction(self.state.names.xsl_transform));
        parser.parse(Some(node), self.state, &Context::empty())
    }
}

pub(crate) fn by_element<V>(
    f: impl Fn(Element) -> Result<V, ElementError>,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    move |node, state, context| {
        let element = state.xot.element(node).ok_or(ElementError::Unexpected {
            span: state.span(node).ok_or(ElementError::Internal)?,
        })?;
        let element_context = context.element(element);
        let element = Element::new(node, element, element_context, state)?;
        f(element)
    }
}

pub(crate) fn by_element_name<V>(
    name: NameId,
    f: impl Fn(Element) -> Result<V, ElementError>,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    by_element(move |element| {
        if element.element.name() == name {
            f(element)
        } else {
            Err(ElementError::Unexpected { span: element.span })
        }
    })
}

pub(crate) fn instruction<T: InstructionParser>(
    name: NameId,
) -> impl Fn(Node, &State, &Context) -> Result<T, ElementError> {
    by_element_name(name, move |element| T::parse_and_validate(&element))
}

// pub(crate) fn instructions() where T: Into<T> {

// }

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

    fn sub_element(&'a self, node: Node, element: &'a xot::Element) -> Result<Self, ElementError> {
        let context = self.context.element(element);
        Self::new(node, element, context, self.state)
    }

    pub(crate) fn standard(&self) -> Result<ast::Standard, AttributeError> {
        self._standard(&self.state.names.standard)
    }

    pub(crate) fn xsl_standard(&self) -> Result<ast::Standard, AttributeError> {
        self._standard(&self.state.names.xsl_standard)
    }

    fn _standard(&self, names: &StandardNames) -> Result<ast::Standard, AttributeError> {
        Ok(ast::Standard {
            default_collation: self.optional(names.default_collation, self.uris())?,
            default_mode: self.optional(names.default_mode, self.default_mode())?,
            default_validation: self
                .optional(names.default_validation, self.default_validation())?,
            exclude_result_prefixes: self.optional(
                names.exclude_result_prefixes,
                self.exclude_result_prefixes(),
            )?,
            expand_text: self.optional(names.expand_text, self.boolean())?,
            extension_element_prefixes: self
                .optional(names.extension_element_prefixes, self.prefixes())?,
            use_when: self.optional(names.use_when, self.xpath())?,
            version: self.optional(names.version, Self::decimal)?,
            xpath_default_namespace: self.optional(names.xpath_default_namespace, self.uri())?,
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

    pub(crate) fn optional<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError>,
    ) -> Result<Option<T>, AttributeError> {
        if let Some(value) = self.element.get_attribute(name) {
            let span = self.value_span(name)?;
            let value = parse_value(value, span).map_err(|e| {
                if let AttributeError::XPath(e) = e {
                    AttributeError::XPath(e.adjust(span.start))
                } else {
                    e
                }
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn required<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError>,
    ) -> Result<T, AttributeError> {
        self.optional(name, parse_value)?.ok_or_else(|| {
            let (local, namespace) = self.state.xot.name_ns_str(name);
            AttributeError::Unexpected {
                name: XmlName {
                    namespace: namespace.to_string(),
                    local: local.to_string(),
                },
                span: self.span,
            }
        })
    }

    pub(crate) fn boolean_with_default(
        &self,
        name: NameId,
        default: bool,
    ) -> Result<bool, AttributeError> {
        self.optional(name, Self::_boolean)
            .map(|v| v.unwrap_or(default))
    }

    fn namespaces(&'a self) -> Namespaces<'a> {
        self.context.namespaces(self.state)
    }

    fn name_span(&self, name: NameId) -> Result<Span, AttributeError> {
        let span = self
            .state
            .span_info
            .get(SpanInfoKey::AttributeName(self.node, name))
            .ok_or(AttributeError::Internal)?;
        Ok(span.into())
    }

    fn value_span(&self, name: NameId) -> Result<Span, AttributeError> {
        let span = self
            .state
            .span_info
            .get(SpanInfoKey::AttributeValue(self.node, name))
            .ok_or(AttributeError::Internal)?;
        Ok(span.into())
    }

    pub(crate) fn value_template<T>(
        &self,
        _parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError> + 'a,
    ) -> impl Fn(&'a str, Span) -> Result<ast::ValueTemplate<T>, AttributeError> + '_
    where
        T: Clone + PartialEq + Eq,
    {
        let namespaces = self.namespaces();
        move |s, span| {
            let iter = ValueTemplateTokenizer::new(s, span, &namespaces, &[]);
            let mut tokens = Vec::new();
            for t in iter {
                let t = t?;
                tokens.push(t.into());
            }

            Ok(ast::ValueTemplate {
                template: tokens,
                phantom: std::marker::PhantomData,
            })
        }
    }

    fn _eqname(&self, s: &str, span: Span) -> Result<xpath_ast::Name, AttributeError> {
        if let Ok(name) = xpath_ast::Name::parse(s, &self.namespaces()).map(|n| n.value) {
            Ok(name)
        } else {
            Err(AttributeError::InvalidEqName {
                value: s.to_string(),
                span,
            })
        }
    }

    pub(crate) fn eqname(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<xpath_ast::Name, AttributeError> + '_ {
        |s, span| self._eqname(s, span)
    }

    fn _qname(s: &str, _span: Span) -> Result<ast::QName, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn qname(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::QName, AttributeError> + '_ {
        Self::_qname
    }

    fn _id(s: &str, _span: Span) -> Result<ast::Id, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn id(&self) -> impl Fn(&'a str, Span) -> Result<ast::Id, AttributeError> + '_ {
        Self::_id
    }

    fn _string(s: &str, _span: Span) -> Result<String, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn string(&self) -> impl Fn(&'a str, Span) -> Result<String, AttributeError> + '_ {
        Self::_string
    }

    fn _input_type_annotations(
        s: &str,
        span: Span,
    ) -> Result<ast::InputTypeAnnotations, AttributeError> {
        match s {
            "strip" => Ok(ast::InputTypeAnnotations::Strip),
            "preserve" => Ok(ast::InputTypeAnnotations::Preserve),
            "unspecified" => Ok(ast::InputTypeAnnotations::Unspecified),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn input_type_annotations(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::InputTypeAnnotations, AttributeError> + '_ {
        Self::_input_type_annotations
    }

    fn _token(s: &str, _span: Span) -> Result<ast::Token, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn token(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Token, AttributeError> + '_ {
        Self::_token
    }

    fn _tokens(s: &str, span: Span) -> Result<Vec<ast::Token>, AttributeError> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_token(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn tokens(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Token>, AttributeError> + '_ {
        Self::_tokens
    }

    fn _uri(s: &str, _span: Span) -> Result<ast::Uri, AttributeError> {
        // TODO: should actually verify URI?
        Ok(s.to_string())
    }

    pub(crate) fn uri(&self) -> impl Fn(&'a str, Span) -> Result<ast::Uri, AttributeError> + '_ {
        Self::_uri
    }

    fn _uris(s: &str, span: Span) -> Result<Vec<ast::Uri>, AttributeError> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_uri(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn uris(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Uri>, AttributeError> + '_ {
        Self::_uris
    }

    fn _xpath(&self, s: &str, span: Span) -> Result<ast::Expression, AttributeError> {
        Ok(ast::Expression {
            xpath: xpath_ast::XPath::parse(s, &self.namespaces(), &[])?,
            span,
        })
    }

    pub(crate) fn xpath(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Expression, AttributeError> + '_ {
        |s, span| self._xpath(s, span)
    }

    fn _pattern(s: &str, _span: Span) -> Result<ast::Pattern, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn pattern(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Pattern, AttributeError> + '_ {
        Self::_pattern
    }

    fn _phase(s: &str, span: Span) -> Result<ast::AccumulatorPhase, AttributeError> {
        match s {
            "start" => Ok(ast::AccumulatorPhase::Start),
            "end" => Ok(ast::AccumulatorPhase::End),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn phase(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::AccumulatorPhase, AttributeError> + '_ {
        Self::_phase
    }

    fn _eqnames(&self, s: &str, span: Span) -> Result<Vec<xpath_ast::Name>, AttributeError> {
        let mut result = Vec::new();
        for (s, span) in split_whitespace_with_spans(s, span) {
            result.push(self._eqname(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn eqnames(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<xpath_ast::Name>, AttributeError> + '_ {
        |s, span| self._eqnames(s, span)
    }

    fn _sequence_type(
        &self,
        s: &str,
        _span: Span,
    ) -> Result<xpath_ast::SequenceType, AttributeError> {
        Ok(xpath_ast::SequenceType::parse(s, &self.namespaces())?)
    }

    pub(crate) fn sequence_type(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<xpath_ast::SequenceType, AttributeError> + '_ {
        |s, span| self._sequence_type(s, span)
    }

    fn _boolean(s: &str, _span: Span) -> Result<bool, AttributeError> {
        match s {
            "yes" | "true" | "1" => Ok(true),
            "no" | "false" | "0" => Ok(false),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span: _span,
            }),
        }
    }

    pub(crate) fn boolean(&self) -> impl Fn(&'a str, Span) -> Result<bool, AttributeError> + '_ {
        Self::_boolean
    }

    fn _default_mode(&self, s: &str, span: Span) -> Result<ast::DefaultMode, AttributeError> {
        if s == "#unnamed" {
            Ok(ast::DefaultMode::Unnamed)
        } else {
            Ok(ast::DefaultMode::EqName(self._eqname(s, span)?))
        }
    }

    pub(crate) fn default_mode(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DefaultMode, AttributeError> + '_ {
        |s, span| self._default_mode(s, span)
    }

    fn _default_validation(s: &str, span: Span) -> Result<ast::DefaultValidation, AttributeError> {
        match s {
            "preserve" => Ok(ast::DefaultValidation::Preserve),
            "strip" => Ok(ast::DefaultValidation::Strip),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn default_validation(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DefaultValidation, AttributeError> + '_ {
        Self::_default_validation
    }

    fn _prefix(s: &str, _span: Span) -> Result<ast::Prefix, AttributeError> {
        // TODO: check whether it's a valid prefix
        Ok(s.to_string())
    }

    pub(crate) fn prefix(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Prefix, AttributeError> + '_ {
        Self::_prefix
    }

    fn _prefixes(s: &str, span: Span) -> Result<Vec<ast::Prefix>, AttributeError> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_prefix(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn prefixes(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Prefix>, AttributeError> + '_ {
        Self::_prefixes
    }

    fn decimal(s: &str, _span: Span) -> Result<ast::Decimal, AttributeError> {
        // TODO
        Ok(s.to_string())
    }

    fn _exclude_result_prefixes(
        s: &str,
        span: Span,
    ) -> Result<ast::ExcludeResultPrefixes, AttributeError> {
        if s == "#all" {
            Ok(ast::ExcludeResultPrefixes::All)
        } else {
            let mut prefixes = Vec::new();
            for s in s.split_whitespace() {
                prefixes.push(Self::_exclude_result_prefix(s, span)?);
            }
            Ok(ast::ExcludeResultPrefixes::Prefixes(prefixes))
        }
    }

    pub(crate) fn exclude_result_prefixes(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::ExcludeResultPrefixes, AttributeError> {
        Self::_exclude_result_prefixes
    }

    fn _exclude_result_prefix(
        s: &str,
        span: Span,
    ) -> Result<ast::ExcludeResultPrefix, AttributeError> {
        if s == "#default" {
            Ok(ast::ExcludeResultPrefix::Default)
        } else {
            Ok(ast::ExcludeResultPrefix::Prefix(Self::_prefix(s, span)?))
        }
    }

    fn _component(s: &str, span: Span) -> Result<ast::Component, AttributeError> {
        use ast::Component::*;

        match s {
            "template" => Ok(Template),
            "function" => Ok(Function),
            "attribute-set" => Ok(AttributeSet),
            "variable" => Ok(Variable),
            "mode" => Ok(Mode),
            "*" => Ok(Star),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn component(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Component, AttributeError> + '_ {
        Self::_component
    }

    fn _visibility_with_abstract(
        s: &str,
        span: Span,
    ) -> Result<ast::VisibilityWithAbstract, AttributeError> {
        use ast::VisibilityWithAbstract::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            "abstract" => Ok(Abstract),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility_with_abstract(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::VisibilityWithAbstract, AttributeError> {
        Self::_visibility_with_abstract
    }

    fn _visibility_with_hidden(
        s: &str,
        span: Span,
    ) -> Result<ast::VisibilityWithHidden, AttributeError> {
        use ast::VisibilityWithHidden::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            "hidden" => Ok(Hidden),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility_with_hidden(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::VisibilityWithHidden, AttributeError> {
        Self::_visibility_with_hidden
    }

    fn _validation(s: &str, span: Span) -> Result<ast::Validation, AttributeError> {
        use ast::Validation::*;

        match s {
            "strict" => Ok(Strict),
            "lax" => Ok(Lax),
            "preserve" => Ok(Preserve),
            "strip" => Ok(Strip),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn validation(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Validation, AttributeError> + '_ {
        Self::_validation
    }

    fn _language(s: &str, _span: Span) -> Result<ast::Language, AttributeError> {
        // TODO
        Ok(s.to_string())
    }

    pub(crate) fn language(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Language, AttributeError> + '_ {
        Self::_language
    }

    fn _order(s: &str, span: Span) -> Result<ast::Order, AttributeError> {
        use ast::Order::*;

        match s {
            "ascending" => Ok(Ascending),
            "descending" => Ok(Descending),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn order(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Order, AttributeError> + '_ {
        Self::_order
    }

    fn _case_order(s: &str, span: Span) -> Result<ast::CaseOrder, AttributeError> {
        use ast::CaseOrder::*;

        match s {
            "upper-first" => Ok(UpperFirst),
            "lower-first" => Ok(LowerFirst),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn case_order(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::CaseOrder, AttributeError> + '_ {
        Self::_case_order
    }

    fn _data_type(&self, s: &str, span: Span) -> Result<ast::DataType, AttributeError> {
        use ast::DataType::*;

        match s {
            "text" => Ok(Text),
            "number" => Ok(Number),
            _ => Ok(EQName(self._eqname(s, span)?)),
        }
    }

    pub(crate) fn data_type(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DataType, AttributeError> + '_ {
        |s, span| self._data_type(s, span)
    }

    // TODO: message ignored
    pub(crate) fn attribute_unexpected(&self, name: NameId, _message: &str) -> AttributeError {
        let (local, namespace) = self.state.xot.name_ns_str(name);
        let span = self.name_span(name);
        match span {
            Ok(span) => AttributeError::Unexpected {
                name: XmlName {
                    namespace: namespace.to_string(),
                    local: local.to_string(),
                },
                span,
            },
            Err(e) => e,
        }
    }
}
