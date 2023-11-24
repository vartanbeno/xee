use xee_xpath_ast::{ast as xpath_ast, Namespaces};
use xot::{NameId, Node, SpanInfoKey, Value};

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::children_parser::{ChildrenParser, ElementError, EndParser, ManyChildrenParser};
use crate::context::Context;
use crate::element_namespaces::ElementNamespaces;
use crate::error::{Error, XmlName};
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::names::StandardNames;
use crate::tokenize::split_whitespace_with_spans;
use crate::value_template::ValueTemplateTokenizer;

pub(crate) struct XsltParser<'a> {
    context: &'a Context,
}

impl<'a> XsltParser<'a> {
    pub(crate) fn new(context: &'a Context) -> Self {
        // let sequence_constructor_parser =
        //     ManyChildrenParser::new(|node, context| match context.xot.value(node) {
        //         Value::Text(text) => Ok(ast::SequenceConstructorItem::TextNode(
        //             text.get().to_string(),
        //         )),
        //         Value::Element(element) => {
        //             let element_namespaces = context.element_namespaces.push(element);
        //             let element = Element::new(node, element, element_namespaces, context)?;
        //             ast::SequenceConstructorItem::parse_sequence_constructor_item(&element)
        //         }
        //         _ => Err(ElementError::Unexpected {
        //             // TODO: get span right
        //             span: Span::new(0, 0),
        //         }),
        //     })
        //     .then_ignore(EndParser::new());

        Self {
            context, // sequence_constructor_parser: Box::new(sequence_constructor_parser),
        }
    }

    pub(crate) fn parse_sequence_constructor_item(
        &self,
        node: Node,
    ) -> Result<ast::SequenceConstructorItem, Error> {
        let element = self.context.xot.element(node).ok_or(Error::Unexpected)?;
        let element_namespaces = ElementNamespaces::new(&self.context.xot, element);
        let element = Element::new(node, element, element_namespaces, self.context)?;
        element.sequence_constructor_item(node)
    }

    pub(crate) fn parse_transform(&self, node: Node) -> Result<ast::Transform, Error> {
        let element = self.context.xot.element(node).ok_or(Error::Unexpected)?;
        let element_namespaces = ElementNamespaces::new(&self.context.xot, element);
        let element = Element::new(node, element, element_namespaces, self.context)?;
        element.parse_transform(node)
    }
}

pub(crate) struct Element<'a> {
    node: Node,
    pub(crate) element: &'a xot::Element,
    pub(crate) span: Span,

    element_namespaces: ElementNamespaces<'a>,
    pub(crate) context: &'a Context,
}

impl<'a> Element<'a> {
    fn new(
        node: Node,
        element: &'a xot::Element,
        element_namespaces: ElementNamespaces<'a>,
        context: &'a Context,
    ) -> Result<Self, Error> {
        let span = context.span(node).ok_or(Error::MissingSpan)?;

        Ok(Self {
            node,
            element,
            span,
            element_namespaces,

            context,
        })
    }

    fn sub_element(&'a self, node: Node, element: &'a xot::Element) -> Result<Self, Error> {
        let element_namespaces = self.element_namespaces.push(element);
        Self::new(node, element, element_namespaces, self.context)
    }

    pub(crate) fn standard(&self) -> Result<ast::Standard, Error> {
        self._standard(&self.context.names.standard)
    }

    pub(crate) fn xsl_standard(&self) -> Result<ast::Standard, Error> {
        self._standard(&self.context.names.xsl_standard)
    }

    fn _standard(&self, names: &StandardNames) -> Result<ast::Standard, Error> {
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

    pub(crate) fn sequence_constructor(&self) -> Result<ast::SequenceConstructor, Error> {
        let mut result = Vec::new();
        for node in self.context.xot.children(self.node) {
            let item = self.sequence_constructor_item(node)?;
            result.push(item);
        }
        Ok(result)
    }

    fn sequence_constructor_item(&self, node: Node) -> Result<ast::SequenceConstructorItem, Error> {
        match self.context.xot.value(node) {
            Value::Text(text) => Ok(ast::SequenceConstructorItem::TextNode(
                text.get().to_string(),
            )),
            Value::Element(element) => {
                let element = self.sub_element(node, element)?;
                ast::SequenceConstructorItem::parse_sequence_constructor_item(&element)
            }
            _ => Err(Error::Unexpected),
        }
    }

    pub(crate) fn declarations(&self) -> Result<ast::Declarations, Error> {
        let mut result = Vec::new();
        for node in self.context.xot.children(self.node) {
            let item = self.declaration_item(node)?;
            result.push(item);
        }
        Ok(result)
    }

    fn declaration_item(&self, node: Node) -> Result<ast::Declaration, Error> {
        match self.context.xot.value(node) {
            Value::Element(element) => {
                let element = self.sub_element(node, element)?;
                ast::Declaration::parse_declaration(&element)
            }
            _ => Err(Error::Unexpected),
        }
    }

    pub(crate) fn many_elements2<T>(
        &self,
        node: Node,
        parse: impl Fn(Node) -> Result<T, Error>,
    ) -> Result<(Vec<T>, Option<Node>), Error>
    where
        T: InstructionParser,
    {
        let mut result = Vec::new();
        let mut current_node = node;
        loop {
            let (item, next) = self.optional_element2(current_node, &parse)?;
            if let Some(item) = item {
                result.push(item);
            } else {
                // we couldn't match with another parseable item, so continue
                return Ok((result, next));
            }
            if let Some(next) = next {
                current_node = next;
            } else {
                // there are no more siblings
                return Ok((result, None));
            }
        }
    }

    pub(crate) fn one_or_more_elements2<T>(
        &self,
        node: Node,
        parse: impl Fn(Node) -> Result<T, Error>,
    ) -> Result<(Vec<T>, Option<Node>), Error>
    where
        T: InstructionParser,
    {
        let (items, node) = self.many_elements2(node, parse)?;
        if items.is_empty() {
            return Err(Error::ElementMissing { span: self.span });
        }
        Ok((items, node))
    }

    pub(crate) fn many_elements_by_name<T>(
        &self,
        node: Node,
        name: NameId,
    ) -> Result<(Vec<T>, Option<Node>), Error>
    where
        T: InstructionParser,
    {
        self.many_elements2(node, |node| self.parse_element(node, name))
    }

    pub(crate) fn optional_element2<T>(
        &self,
        node: Node,
        parse: impl Fn(Node) -> Result<T, Error>,
    ) -> Result<(Option<T>, Option<Node>), Error>
    where
        T: InstructionParser,
    {
        let item = parse(node);
        match item {
            Ok(item) => Ok((Some(item), self.context.xot.next_sibling(node))),
            Err(Error::Unexpected { .. }) => Ok((None, Some(node))),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn many_elements<T>(&self, name: NameId) -> Result<Vec<T>, Error>
    where
        T: InstructionParser,
    {
        let mut result = Vec::new();
        for node in self.context.xot.children(self.node) {
            let item = self.parse_element(node, name)?;
            result.push(item);
        }
        Ok(result)
    }

    pub(crate) fn one_or_more_elements<T>(&self, name: NameId) -> Result<Vec<T>, Error>
    where
        T: InstructionParser,
    {
        let result = self.many_elements(name)?;
        if result.is_empty() {
            return Err(Error::ElementMissing { span: self.span });
        }
        Ok(result)
    }

    pub(crate) fn optional_element<T>(&self, name: NameId) -> Result<Option<T>, Error>
    where
        T: InstructionParser,
    {
        self.parse_optional_element(self.node, name)
    }

    pub(crate) fn parse_sequence_constructor_item(
        &self,
        node: Node,
    ) -> Result<ast::SequenceConstructorItem, Error> {
        let element = self.context.xot.element(node).ok_or(Error::Unexpected)?;
        let element = self.sub_element(node, element)?;
        element.sequence_constructor_item(node)
    }

    pub(crate) fn parse_transform(&self, node: Node) -> Result<ast::Transform, Error> {
        self.parse_element(node, self.context.names.xsl_transform)
    }

    fn parse_element<T: InstructionParser>(&self, node: Node, name: NameId) -> Result<T, Error> {
        let element = self.context.xot.element(node).ok_or(Error::Unexpected)?;
        let element = self.sub_element(node, element)?;
        //     let element_namespaces = ElementNamespaces::new(self.xot, element);
        // let element = Element::new(node, element, self, element_namespaces)?;
        if element.element.name() != name {
            return Err(Error::InvalidInstruction { span: element.span });
        }
        T::parse(&element)
    }

    fn parse_optional_element<T: InstructionParser>(
        &self,
        node: Node,
        name: NameId,
    ) -> Result<Option<T>, Error> {
        let element = self.context.xot.element(node).ok_or(Error::Unexpected)?;
        // let element_namespaces = ElementNamespaces::new(self.xot, element);
        let element = self.sub_element(node, element)?;
        // let element = Element::new(node, element, self, element_namespaces)?;
        if element.element.name() != name {
            return Ok(None);
        }
        T::parse(&element).map(Some)
    }

    pub(crate) fn optional<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, Error>,
    ) -> Result<Option<T>, Error> {
        if let Some(value) = self.element.get_attribute(name) {
            let span = self.value_span(name)?;
            let value = parse_value(value, span).map_err(|e| {
                if let Error::XPath(e) = e {
                    Error::XPath(e.adjust(span.start))
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
        parse_value: impl Fn(&'a str, Span) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.optional(name, parse_value)?.ok_or_else(|| {
            let (local, namespace) = self.context.xot.name_ns_str(name);
            Error::AttributeExpected {
                name: XmlName {
                    namespace: namespace.to_string(),
                    local: local.to_string(),
                },
                span: self.span,
            }
        })
    }

    pub(crate) fn boolean_with_default(&self, name: NameId, default: bool) -> Result<bool, Error> {
        self.optional(name, Self::_boolean)
            .map(|v| v.unwrap_or(default))
    }

    fn namespaces(&'a self) -> Namespaces<'a> {
        self.element_namespaces.namespaces()
    }

    fn name_span(&self, name: NameId) -> Result<Span, Error> {
        let span = self
            .context
            .span_info
            .get(SpanInfoKey::AttributeName(self.node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
    }

    fn value_span(&self, name: NameId) -> Result<Span, Error> {
        let span = self
            .context
            .span_info
            .get(SpanInfoKey::AttributeValue(self.node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
    }

    pub(crate) fn value_template<T>(
        &self,
        _parse_value: impl Fn(&'a str, Span) -> Result<T, Error> + 'a,
    ) -> impl Fn(&'a str, Span) -> Result<ast::ValueTemplate<T>, Error> + '_
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

    fn _eqname(&self, s: &str, span: Span) -> Result<xpath_ast::Name, Error> {
        if let Ok(name) = xpath_ast::Name::parse(s, &self.namespaces()).map(|n| n.value) {
            Ok(name)
        } else {
            Err(Error::InvalidEqName {
                value: s.to_string(),
                span,
            })
        }
    }

    pub(crate) fn eqname(&self) -> impl Fn(&'a str, Span) -> Result<xpath_ast::Name, Error> + '_ {
        |s, span| self._eqname(s, span)
    }

    fn _id(s: &str, _span: Span) -> Result<ast::Id, Error> {
        Ok(s.to_string())
    }

    pub(crate) fn id(&self) -> impl Fn(&'a str, Span) -> Result<ast::Id, Error> + '_ {
        Self::_id
    }

    fn _string(s: &str, _span: Span) -> Result<String, Error> {
        Ok(s.to_string())
    }

    pub(crate) fn string(&self) -> impl Fn(&'a str, Span) -> Result<String, Error> + '_ {
        Self::_string
    }

    fn _input_type_annotations(s: &str, span: Span) -> Result<ast::InputTypeAnnotations, Error> {
        match s {
            "strip" => Ok(ast::InputTypeAnnotations::Strip),
            "preserve" => Ok(ast::InputTypeAnnotations::Preserve),
            "unspecified" => Ok(ast::InputTypeAnnotations::Unspecified),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn input_type_annotations(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::InputTypeAnnotations, Error> + '_ {
        Self::_input_type_annotations
    }

    fn _token(s: &str, _span: Span) -> Result<ast::Token, Error> {
        Ok(s.to_string())
    }

    pub(crate) fn token(&self) -> impl Fn(&'a str, Span) -> Result<ast::Token, Error> + '_ {
        Self::_token
    }

    fn _tokens(s: &str, span: Span) -> Result<Vec<ast::Token>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_token(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn tokens(&self) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Token>, Error> + '_ {
        Self::_tokens
    }

    fn _uri(s: &str, _span: Span) -> Result<ast::Uri, Error> {
        // TODO: should actually verify URI?
        Ok(s.to_string())
    }

    pub(crate) fn uri(&self) -> impl Fn(&'a str, Span) -> Result<ast::Uri, Error> + '_ {
        Self::_uri
    }

    fn _uris(s: &str, span: Span) -> Result<Vec<ast::Uri>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_uri(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn uris(&self) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Uri>, Error> + '_ {
        Self::_uris
    }

    fn _xpath(&self, s: &str, span: Span) -> Result<ast::Expression, Error> {
        Ok(ast::Expression {
            xpath: xpath_ast::XPath::parse(s, &self.namespaces(), &[])?,
            span,
        })
    }

    pub(crate) fn xpath(&self) -> impl Fn(&'a str, Span) -> Result<ast::Expression, Error> + '_ {
        |s, span| self._xpath(s, span)
    }

    fn _pattern(s: &str, _span: Span) -> Result<ast::Pattern, Error> {
        Ok(s.to_string())
    }

    pub(crate) fn pattern(&self) -> impl Fn(&'a str, Span) -> Result<ast::Pattern, Error> + '_ {
        Self::_pattern
    }

    fn _phase(s: &str, span: Span) -> Result<ast::AccumulatorPhase, Error> {
        match s {
            "start" => Ok(ast::AccumulatorPhase::Start),
            "end" => Ok(ast::AccumulatorPhase::End),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn phase(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::AccumulatorPhase, Error> + '_ {
        Self::_phase
    }

    fn _eqnames(&self, s: &str, span: Span) -> Result<Vec<xpath_ast::Name>, Error> {
        let mut result = Vec::new();
        for (s, span) in split_whitespace_with_spans(s, span) {
            result.push(self._eqname(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn eqnames(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<xpath_ast::Name>, Error> + '_ {
        |s, span| self._eqnames(s, span)
    }

    fn _sequence_type(&self, s: &str, _span: Span) -> Result<xpath_ast::SequenceType, Error> {
        Ok(xpath_ast::SequenceType::parse(s, &self.namespaces())?)
    }

    pub(crate) fn sequence_type(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<xpath_ast::SequenceType, Error> + '_ {
        |s, span| self._sequence_type(s, span)
    }

    fn _boolean(s: &str, _span: Span) -> Result<bool, Error> {
        match s {
            "yes" | "true" | "1" => Ok(true),
            "no" | "false" | "0" => Ok(false),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span: _span,
            }),
        }
    }

    pub(crate) fn boolean(&self) -> impl Fn(&'a str, Span) -> Result<bool, Error> + '_ {
        Self::_boolean
    }

    fn _default_mode(&self, s: &str, span: Span) -> Result<ast::DefaultMode, Error> {
        if s == "#unnamed" {
            Ok(ast::DefaultMode::Unnamed)
        } else {
            Ok(ast::DefaultMode::EqName(self._eqname(s, span)?))
        }
    }

    pub(crate) fn default_mode(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DefaultMode, Error> + '_ {
        |s, span| self._default_mode(s, span)
    }

    fn _default_validation(s: &str, span: Span) -> Result<ast::DefaultValidation, Error> {
        match s {
            "preserve" => Ok(ast::DefaultValidation::Preserve),
            "strip" => Ok(ast::DefaultValidation::Strip),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn default_validation(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DefaultValidation, Error> + '_ {
        Self::_default_validation
    }

    fn _prefix(s: &str, _span: Span) -> Result<ast::Prefix, Error> {
        // TODO: check whether it's a valid prefix
        Ok(s.to_string())
    }

    pub(crate) fn prefix(&self) -> impl Fn(&'a str, Span) -> Result<ast::Prefix, Error> + '_ {
        Self::_prefix
    }

    fn _prefixes(s: &str, span: Span) -> Result<Vec<ast::Prefix>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_prefix(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn prefixes(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Prefix>, Error> + '_ {
        Self::_prefixes
    }

    fn decimal(s: &str, _span: Span) -> Result<ast::Decimal, Error> {
        // TODO
        Ok(s.to_string())
    }

    fn _exclude_result_prefixes(s: &str, span: Span) -> Result<ast::ExcludeResultPrefixes, Error> {
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
    ) -> impl Fn(&'a str, Span) -> Result<ast::ExcludeResultPrefixes, Error> {
        Self::_exclude_result_prefixes
    }

    fn _exclude_result_prefix(s: &str, span: Span) -> Result<ast::ExcludeResultPrefix, Error> {
        if s == "#default" {
            Ok(ast::ExcludeResultPrefix::Default)
        } else {
            Ok(ast::ExcludeResultPrefix::Prefix(Self::_prefix(s, span)?))
        }
    }

    fn _component(s: &str, span: Span) -> Result<ast::Component, Error> {
        use ast::Component::*;

        match s {
            "template" => Ok(Template),
            "function" => Ok(Function),
            "attribute-set" => Ok(AttributeSet),
            "variable" => Ok(Variable),
            "mode" => Ok(Mode),
            "*" => Ok(Star),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn component(&self) -> impl Fn(&'a str, Span) -> Result<ast::Component, Error> + '_ {
        Self::_component
    }

    fn _visibility_with_abstract(
        s: &str,
        span: Span,
    ) -> Result<ast::VisibilityWithAbstract, Error> {
        use ast::VisibilityWithAbstract::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            "abstract" => Ok(Abstract),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility_with_abstract(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::VisibilityWithAbstract, Error> {
        Self::_visibility_with_abstract
    }

    fn _visibility_with_hidden(s: &str, span: Span) -> Result<ast::VisibilityWithHidden, Error> {
        use ast::VisibilityWithHidden::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            "hidden" => Ok(Hidden),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility_with_hidden(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::VisibilityWithHidden, Error> {
        Self::_visibility_with_hidden
    }

    fn _validation(s: &str, span: Span) -> Result<ast::Validation, Error> {
        use ast::Validation::*;

        match s {
            "strict" => Ok(Strict),
            "lax" => Ok(Lax),
            "preserve" => Ok(Preserve),
            "strip" => Ok(Strip),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn validation(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Validation, Error> + '_ {
        Self::_validation
    }

    pub(crate) fn attribute_unexpected(&self, name: NameId, message: &str) -> Error {
        let (local, namespace) = self.context.xot.name_ns_str(name);
        let span = self.name_span(name);
        match span {
            Ok(span) => Error::AttributeUnexpected {
                name: XmlName {
                    namespace: namespace.to_string(),
                    local: local.to_string(),
                },
                span,
                message: message.to_string(),
            },
            Err(e) => e,
        }
    }
}
