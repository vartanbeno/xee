use ahash::{HashSet, HashSetExt};
use xee_xpath_ast::{ast as xpath_ast, Namespaces};

use crate::ast_core as ast;
use crate::context::Context;
use crate::error::{AttributeError, ElementError};
use crate::name::XmlName;
use crate::names::StandardNames;
use crate::state::State;
use crate::tokenize::split_whitespace_with_spans;
use crate::{ast_core::Span, value_template::ValueTemplateTokenizer};
use xot::{NameId, Node, SpanInfoKey};

pub(crate) struct Attributes<'a> {
    node: Node,
    element: &'a xot::Element,
    state: &'a State,
    context: Context<'a>,
    seen: std::cell::RefCell<HashSet<NameId>>,
    span: Span,
}

impl<'a> Attributes<'a> {
    pub(crate) fn new(
        node: Node,
        element: &'a xot::Element,
        state: &'a State,
        context: Context<'a>,
    ) -> Result<Self, ElementError> {
        let span = state.span(node).ok_or(ElementError::Internal)?;
        Ok(Self {
            node,
            element,
            span,
            state,
            context,
            seen: std::cell::RefCell::new(HashSet::new()),
        })
    }

    pub(crate) fn optional<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError>,
    ) -> Result<Option<T>, AttributeError> {
        self.seen.borrow_mut().insert(name);

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
            AttributeError::NotFound {
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

    pub(crate) fn unseen_attributes(&self) -> Vec<NameId> {
        let mut result = Vec::new();
        let seen = self.seen.borrow();
        for name in self.element.attributes().keys() {
            if !seen.contains(name) {
                result.push(*name);
            }
        }
        result
    }

    fn _boolean(s: &str, span: Span) -> Result<bool, AttributeError> {
        match s {
            "yes" | "true" | "1" => Ok(true),
            "no" | "false" | "0" => Ok(false),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn boolean(&self) -> impl Fn(&'a str, Span) -> Result<bool, AttributeError> + '_ {
        Self::_boolean
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

    fn value_span(&self, name: NameId) -> Result<Span, AttributeError> {
        let span = self
            .state
            .span_info
            .get(SpanInfoKey::AttributeValue(self.node, name))
            .ok_or(AttributeError::Internal)?;
        Ok(span.into())
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
            version: self.optional(names.version, Self::_decimal)?,
            xpath_default_namespace: self.optional(names.xpath_default_namespace, self.uri())?,
        })
    }

    fn namespaces(&self) -> Namespaces {
        self.context.namespaces(self.state)
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

    fn _prefix_or_default(s: &str, span: Span) -> Result<ast::PrefixOrDefault, AttributeError> {
        if s == "#default" {
            Ok(ast::PrefixOrDefault::Default)
        } else {
            Ok(ast::PrefixOrDefault::Prefix(Self::_prefix(s, span)?))
        }
    }

    pub(crate) fn prefix_or_default(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::PrefixOrDefault, AttributeError> + '_ {
        Self::_prefix_or_default
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

    fn _decimal(s: &str, _span: Span) -> Result<ast::Decimal, AttributeError> {
        // TODO
        Ok(s.to_string())
    }

    pub(crate) fn decimal(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Decimal, AttributeError> + '_ {
        Self::_decimal
    }

    fn _method(&self, s: &str, span: Span) -> Result<ast::OutputMethod, AttributeError> {
        use ast::OutputMethod::*;

        match s {
            "xml" => Ok(Xml),
            "html" => Ok(Html),
            "xhtml" => Ok(Xhtml),
            "text" => Ok(Text),
            "json" => Ok(Json),
            "adaptive" => Ok(Adaptive),
            _ => Ok(EqName(self._eqname(s, span)?)),
        }
    }

    pub(crate) fn method(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::OutputMethod, AttributeError> + '_ {
        |s, span| self._method(s, span)
    }

    fn _json_node_output_method(
        &self,
        s: &str,
        span: Span,
    ) -> Result<ast::JsonNodeOutputMethod, AttributeError> {
        use ast::JsonNodeOutputMethod::*;

        match s {
            "xml" => Ok(Xml),
            "html" => Ok(Html),
            "xhtml" => Ok(Xhtml),
            "text" => Ok(Text),
            _ => Ok(EqName(self._eqname(s, span)?)),
        }
    }

    pub(crate) fn json_node_output_method(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::JsonNodeOutputMethod, AttributeError> + '_ {
        |s, span| self._json_node_output_method(s, span)
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

    fn _streamability(&self, s: &str, span: Span) -> Result<ast::Streamability, AttributeError> {
        use ast::Streamability::*;

        match s {
            "unclassified" => Ok(Unclassified),
            "absorbing" => Ok(Absorbing),
            "inspection" => Ok(Inspection),
            "filter" => Ok(Filter),
            "shallow-descent" => Ok(ShallowDescent),
            "deep-descent" => Ok(DeepDescent),
            "ascent" => Ok(Ascent),
            _ => Ok(EqName(self._eqname(s, span)?)),
        }
    }

    pub(crate) fn streamability(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Streamability, AttributeError> + '_ {
        |s, span| self._streamability(s, span)
    }
}
