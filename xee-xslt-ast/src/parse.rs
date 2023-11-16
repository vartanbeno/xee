use std::collections::BTreeMap;

use xee_xpath_ast::{ast as xpath_ast, Namespaces};
use xot::{NameId, NamespaceId, Node, SpanInfo, SpanInfoKey, Value, Xot};

use crate::ast_core as ast;
use crate::ast_core::Span;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
enum Error {
    Unexpected,
    // ns, name, span of the element on which the attribute is expected
    AttributeExpected {
        namespace: String,
        local: String,
        span: Span,
    },
    // ns, name, span of the attribute that is unexpected
    AttributeUnexpected {
        namespace: String,
        local: String,
        span: Span,
        message: String,
    },
    UnexpectedSequenceConstructor,
    Invalid {
        value: String,
        span: Span,
    },
    InvalidInstruction {
        span: Span,
    },
    MissingSpan,
    XPath(xee_xpath_ast::ParserError),
}

impl From<xee_xpath_ast::ParserError> for Error {
    fn from(error: xee_xpath_ast::ParserError) -> Self {
        Self::XPath(error)
    }
}

impl Error {
    fn is_unexpected(&self) -> bool {
        matches!(self, Self::Unexpected)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SequenceConstructorName {
    If,
    Variable,
    Copy,
}

struct Names {
    xsl_ns: NamespaceId,

    sequence_constructor_names: BTreeMap<NameId, SequenceConstructorName>,

    copy: xot::NameId,
    if_: xot::NameId,
    variable: xot::NameId,

    test: xot::NameId,
    select: xot::NameId,
    name: xot::NameId,
    as_: xot::NameId,
    static_: xot::NameId,
    visibility: xot::NameId,
    copy_namespaces: xot::NameId,
    inherit_namespaces: xot::NameId,
    use_attribute_sets: xot::NameId,
    validation: xot::NameId,

    // standard attributes on XSLT elements
    standard: StandardNames,
    // standard attributes on literal result elements
    xsl_standard: StandardNames,
}

struct StandardNames {
    default_collation: xot::NameId,
    default_mode: xot::NameId,
    default_validation: xot::NameId,
    exclude_result_prefixes: xot::NameId,
    expand_text: xot::NameId,
    extension_element_prefixes: xot::NameId,
    use_when: xot::NameId,
    version: xot::NameId,
    xpath_default_namespace: xot::NameId,
}

impl StandardNames {
    fn no_ns(xot: &mut Xot) -> Self {
        Self {
            default_collation: xot.add_name("default-collation"),
            default_mode: xot.add_name("default-mode"),
            default_validation: xot.add_name("default-validation"),
            exclude_result_prefixes: xot.add_name("exclude-result-prefixes"),
            expand_text: xot.add_name("expand-text"),
            extension_element_prefixes: xot.add_name("extension-element-prefixes"),
            use_when: xot.add_name("use-when"),
            version: xot.add_name("version"),
            xpath_default_namespace: xot.add_name("xpath-default-namespace"),
        }
    }

    fn xsl(xot: &mut Xot, xsl_ns: NamespaceId) -> Self {
        Self {
            default_collation: xot.add_name_ns("default-collation", xsl_ns),
            default_mode: xot.add_name_ns("default-mode", xsl_ns),
            default_validation: xot.add_name_ns("default-validation", xsl_ns),
            exclude_result_prefixes: xot.add_name_ns("exclude-result-prefixes", xsl_ns),
            expand_text: xot.add_name_ns("expand-text", xsl_ns),
            extension_element_prefixes: xot.add_name_ns("extension-element-prefixes", xsl_ns),
            use_when: xot.add_name_ns("use-when", xsl_ns),
            version: xot.add_name_ns("version", xsl_ns),
            xpath_default_namespace: xot.add_name_ns("xpath-default-namespace", xsl_ns),
        }
    }
}

impl Names {
    fn new(xot: &mut Xot) -> Self {
        let xsl_ns = xot.add_namespace("http://www.w3.org/1999/XSL/Transform");

        let copy = xot.add_name_ns("copy", xsl_ns);
        let if_ = xot.add_name_ns("if", xsl_ns);
        let variable = xot.add_name_ns("variable", xsl_ns);

        let mut sequence_constructor_names = BTreeMap::new();
        sequence_constructor_names.insert(if_, SequenceConstructorName::If);
        sequence_constructor_names.insert(variable, SequenceConstructorName::Variable);
        sequence_constructor_names.insert(copy, SequenceConstructorName::Copy);

        Self {
            xsl_ns,

            sequence_constructor_names,

            copy,
            if_,
            variable,

            test: xot.add_name("test"),
            select: xot.add_name("select"),
            name: xot.add_name("name"),
            as_: xot.add_name("as"),
            static_: xot.add_name("static"),
            visibility: xot.add_name("visibility"),
            copy_namespaces: xot.add_name("copy-namespaces"),
            inherit_namespaces: xot.add_name("inherit-namespaces"),
            use_attribute_sets: xot.add_name("use-attribute-sets"),
            validation: xot.add_name("validation"),

            // standard attributes
            standard: StandardNames::no_ns(xot),
            // standard attributes on literal result elements
            xsl_standard: StandardNames::xsl(xot, xsl_ns),
        }
    }

    fn sequence_constructor_name(&self, name: NameId) -> Option<SequenceConstructorName> {
        self.sequence_constructor_names.get(&name).copied()
    }
}

struct XsltParser<'a> {
    xot: &'a Xot,
    names: &'a Names,
    span_info: &'a SpanInfo,
    namespaces: Namespaces<'a>,
}

impl<'a> XsltParser<'a> {
    fn new(
        xot: &'a Xot,
        names: &'a Names,
        span_info: &'a SpanInfo,
        namespaces: Namespaces<'a>,
    ) -> Self {
        Self {
            xot,
            names,
            span_info,
            namespaces,
        }
    }

    fn element_span(&self, node: Node) -> Result<Span, Error> {
        let span = self
            .span_info
            .get(SpanInfoKey::ElementStart(node))
            .ok_or(Error::MissingSpan)?;

        Ok(span.into())
    }

    fn parse(&self, node: Node) -> Result<ast::SequenceConstructorItem, Error> {
        let element = self.xot.element(node).ok_or(Error::Unexpected)?;
        let element = Element::new(node, element, self)?;
        element.parse(node)
    }
}

struct Element<'a> {
    node: Node,
    element: &'a xot::Element,
    span: Span,

    names: &'a Names,
    span_info: &'a SpanInfo,
    xot: &'a Xot,
    namespaces: &'a Namespaces<'a>,
    xslt_parser: &'a XsltParser<'a>,
}

impl<'a> Element<'a> {
    fn new(
        node: Node,
        element: &'a xot::Element,
        xslt_parser: &'a XsltParser<'a>,
    ) -> Result<Self, Error> {
        Ok(Self {
            node,
            element,
            span: xslt_parser.element_span(node)?,

            names: xslt_parser.names,
            span_info: xslt_parser.span_info,
            xot: xslt_parser.xot,
            namespaces: &xslt_parser.namespaces,
            xslt_parser,
        })
    }

    fn parse(&self, node: Node) -> Result<ast::SequenceConstructorItem, Error> {
        match self.xot.value(node) {
            Value::Text(text) => Ok(ast::SequenceConstructorItem::TextNode(
                text.get().to_string(),
            )),
            Value::Element(element) => {
                let element = Element::new(node, element, self.xslt_parser)?;
                ast::SequenceConstructorItem::parse(&element)
            }
            _ => Err(Error::Unexpected),
        }
    }

    fn standard(&self) -> Result<ast::Standard, Error> {
        self._standard(&self.names.standard)
    }

    fn xsl_standard(&self) -> Result<ast::Standard, Error> {
        self._standard(&self.names.xsl_standard)
    }

    fn _standard(&self, names: &StandardNames) -> Result<ast::Standard, Error> {
        Ok(ast::Standard {
            default_collation: self.optional(names.default_collation, Self::uris)?,
            default_mode: self.optional(names.default_mode, Self::default_mode)?,
            default_validation: self
                .optional(names.default_validation, Self::default_validation)?,
            exclude_result_prefixes: self
                .optional(names.exclude_result_prefixes, Self::exclude_result_prefixes)?,
            expand_text: self.optional(names.expand_text, Self::_boolean)?,
            extension_element_prefixes: self
                .optional(names.extension_element_prefixes, Self::prefixes)?,
            use_when: self.optional(names.use_when, |s, span| self.xpath(s, span))?,
            version: self.optional(names.version, Self::decimal)?,
            xpath_default_namespace: self.optional(names.xpath_default_namespace, Self::uri)?,
        })
    }

    fn sequence_constructor(&self) -> Result<ast::SequenceConstructor, Error> {
        let mut result = Vec::new();
        for node in self.xot.children(self.node) {
            let item = self.parse(node)?;
            result.push(item);
        }
        Ok(result)
    }

    fn optional<T>(
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

    fn required<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.optional(name, parse_value)?.ok_or_else(|| {
            let (local, namespace) = self.xot.name_ns_str(name);
            Error::AttributeExpected {
                namespace: namespace.to_string(),
                local: local.to_string(),
                span: self.span,
            }
        })
    }

    fn boolean(&self, name: NameId, default: bool) -> Result<bool, Error> {
        self.optional(name, Self::_boolean)
            .map(|v| v.unwrap_or(default))
    }

    fn name_span(&self, name: NameId) -> Result<Span, Error> {
        let span = self
            .span_info
            .get(SpanInfoKey::AttributeName(self.node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
    }

    fn value_span(&self, name: NameId) -> Result<Span, Error> {
        let span = self
            .span_info
            .get(SpanInfoKey::AttributeValue(self.node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
    }

    fn eqname(s: &str, _span: Span) -> Result<String, Error> {
        // TODO: should actually parse
        Ok(s.to_string())
    }

    fn uri(s: &str, _span: Span) -> Result<ast::Uri, Error> {
        // TODO: should actually verify URI?
        Ok(s.to_string())
    }

    fn uris(s: &str, span: Span) -> Result<Vec<ast::Uri>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::uri(s, span)?);
        }
        Ok(result)
    }

    fn xpath(&self, s: &str, span: Span) -> Result<ast::Expression, Error> {
        Ok(ast::Expression {
            xpath: xpath_ast::XPath::parse(s, self.namespaces, &[])?,
            span,
        })
    }

    fn eqnames(s: &str, span: Span) -> Result<Vec<String>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::eqname(s, span)?);
        }
        Ok(result)
    }

    fn sequence_type(&self, s: &str, _span: Span) -> Result<xpath_ast::SequenceType, Error> {
        Ok(xpath_ast::SequenceType::parse(s, self.namespaces)?)
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

    fn default_mode(s: &str, span: Span) -> Result<ast::DefaultMode, Error> {
        if s == "#unnamed" {
            Ok(ast::DefaultMode::Unnamed)
        } else {
            Ok(ast::DefaultMode::EqName(Self::eqname(s, span)?))
        }
    }

    fn default_validation(s: &str, span: Span) -> Result<ast::DefaultValidation, Error> {
        match s {
            "preserve" => Ok(ast::DefaultValidation::Preserve),
            "strip" => Ok(ast::DefaultValidation::Strip),
            _ => Err(Error::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    fn prefix(s: &str, _span: Span) -> Result<ast::Prefix, Error> {
        // TODO: check whether it's a valid prefix
        Ok(s.to_string())
    }

    fn prefixes(s: &str, span: Span) -> Result<Vec<ast::Prefix>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::prefix(s, span)?);
        }
        Ok(result)
    }

    fn decimal(s: &str, _span: Span) -> Result<ast::Decimal, Error> {
        // TODO
        Ok(s.to_string())
    }

    fn exclude_result_prefixes(s: &str, span: Span) -> Result<ast::ExcludeResultPrefixes, Error> {
        if s == "#all" {
            Ok(ast::ExcludeResultPrefixes::All)
        } else {
            let mut prefixes = Vec::new();
            for s in s.split_whitespace() {
                prefixes.push(Self::exclude_result_prefix(s, span)?);
            }
            Ok(ast::ExcludeResultPrefixes::Prefixes(prefixes))
        }
    }

    fn exclude_result_prefix(s: &str, span: Span) -> Result<ast::ExcludeResultPrefix, Error> {
        if s == "#default" {
            Ok(ast::ExcludeResultPrefix::Default)
        } else {
            Ok(ast::ExcludeResultPrefix::Prefix(Self::prefix(s, span)?))
        }
    }

    fn visibility_with_abstract(s: &str, span: Span) -> Result<ast::VisibilityWithAbstract, Error> {
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

    fn validation(s: &str, span: Span) -> Result<ast::Validation, Error> {
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

    fn attribute_unexpected(&self, name: NameId, message: &str) -> Error {
        let (local, namespace) = self.xot.name_ns_str(name);
        let span = self.name_span(name);
        match span {
            Ok(span) => Error::AttributeUnexpected {
                namespace: namespace.to_string(),
                local: local.to_string(),
                span,
                message: message.to_string(),
            },
            Err(e) => e,
        }
    }
}

trait InstructionParser: Sized + Into<ast::SequenceConstructorItem> {
    fn parse(element: &Element) -> Result<ast::SequenceConstructorItem, Error> {
        let ast = Self::parse_ast(element)?;
        ast.validate(element)?;
        Ok(ast.into())
    }

    fn validate(&self, _element: &Element) -> Result<(), Error> {
        Ok(())
    }

    fn parse_ast(element: &Element) -> Result<Self, Error>;
}

impl InstructionParser for ast::SequenceConstructorItem {
    fn parse_ast(element: &Element) -> Result<ast::SequenceConstructorItem, Error> {
        let sname = element
            .names
            .sequence_constructor_name(element.element.name());

        if let Some(sname) = sname {
            // parse a known sequence constructor instruction
            match sname {
                SequenceConstructorName::Copy => ast::Copy::parse(element),
                SequenceConstructorName::If => ast::If::parse(element),
                SequenceConstructorName::Variable => ast::Variable::parse(element),
            }
        } else {
            let ns = element.xot.namespace_for_name(element.element.name());
            if ns == element.names.xsl_ns {
                // we have an unknown xsl instruction, fail with error
                Err(Error::InvalidInstruction { span: element.span })
            } else {
                // we parse the literal element
                ast::ElementNode::parse(element)
            }
        }
    }
}

impl InstructionParser for ast::ElementNode {
    fn parse_ast(element: &Element) -> Result<ast::ElementNode, Error> {
        Ok(ast::ElementNode {
            name: to_name(element.xot, element.element.name()),

            standard: element.xsl_standard()?,
            span: element.span,
        })
    }
}

fn to_name(xot: &Xot, name: NameId) -> ast::Name {
    let (local, namespace) = xot.name_ns_str(name);
    ast::Name {
        namespace: namespace.to_string(),
        local: local.to_string(),
    }
}

impl InstructionParser for ast::Copy {
    fn parse_ast(element: &Element) -> Result<Self, Error> {
        let content = element.sequence_constructor()?;
        let names = element.names;
        Ok(ast::Copy {
            select: element.optional(names.select, |s, span| element.xpath(s, span))?,
            copy_namespaces: element.boolean(names.copy_namespaces, true)?,
            inherit_namespaces: element.boolean(names.inherit_namespaces, true)?,
            use_attribute_sets: element.optional(names.use_attribute_sets, Element::eqnames)?,
            type_: element.optional(names.as_, Element::eqname)?,
            validation: element
                .optional(names.validation, Element::validation)?
                // TODO: should depend on global validation attribute
                .unwrap_or(ast::Validation::Strip),
            content,
            standard: element.standard()?,
            span: element.span,
        })
    }
}

// impl InstructionParser for ast::Fallback {
//     fn parse_ast(element: &Element) -> Result<Self, Error> {
//         let parser = element.xslt_parser;
//         let content =
//         Ok(ast::Fallback {
//             content: parser.parse_sequence_constructor(element.node)?;
//             span: element.span,
//         })
//     }
// }

impl InstructionParser for ast::If {
    fn parse_ast(element: &Element) -> Result<Self, Error> {
        let names = element.names;
        Ok(ast::If {
            test: element.required(names.test, |s, span| element.xpath(s, span))?,
            content: element.sequence_constructor()?,
            standard: element.standard()?,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::Variable {
    fn parse_ast(element: &Element) -> Result<Self, Error> {
        let names = element.names;

        // This is a rule somewhere, but not sure whether it's correct;
        // can visibility be absent or is there a default visibility?
        // let visibility = visibility.unwrap_or(if static_ {
        //     ast::VisibilityWithAbstract::Private
        // } else {
        //     ast::VisibilityWithAbstract::Public
        // });

        Ok(ast::Variable {
            name: element.required(names.name, Element::eqname)?,
            select: element.optional(names.select, |s, span| element.xpath(s, span))?,
            as_: element.optional(names.as_, |s, span| element.sequence_type(s, span))?,
            static_: element.boolean(names.static_, false)?,
            visibility: element.optional(names.visibility, Element::visibility_with_abstract)?,
            content: element.sequence_constructor()?,
            standard: element.standard()?,
            span: element.span,
        })
    }

    fn validate(&self, element: &Element) -> Result<(), Error> {
        if self.visibility == Some(ast::VisibilityWithAbstract::Abstract) && self.select.is_some() {
            return Err(element.attribute_unexpected(
                element.names.select,
                "select attribute is not allowed when visibility is abstract",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use insta::assert_ron_snapshot;
    use xee_xpath_ast::Namespaces;

    fn parse(s: &str) -> Result<ast::SequenceConstructorItem, Error> {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let namespaces = Namespaces::default();

        let (node, span_info) = xot.parse_with_span_info(s).unwrap();
        let node = xot.document_element(node).unwrap();
        let parser = XsltParser::new(&xot, &names, &span_info, namespaces);
        parser.parse(node)
    }

    #[test]
    fn test_if() {
        assert_ron_snapshot!(parse(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_variable() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_missing_required() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_broken_xpath() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="let $x := 1">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_sequence_type() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" as="xs:string" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_boolean_default_no_with_explicit_yes() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" static="yes" as="xs:string" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="public">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility_abstract_with_select_is_error() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="abstract" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_copy() {
        assert_ron_snapshot!(parse(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" copy-namespaces="no" inherit-namespaces="no" validation="strict">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_eqnames() {
        assert_ron_snapshot!(parse(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" use-attribute-sets="foo bar baz">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_nested_if() {
        assert_ron_snapshot!(parse(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()"><xsl:if test="true()">Hello</xsl:if></xsl:if>"#
        ));
    }

    #[test]
    fn test_if_with_standard_attribute() {
        assert_ron_snapshot!(parse(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" expand-text="yes">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_literal_result_element() {
        assert_ron_snapshot!(parse(r#"<foo/>"#));
    }

    #[test]
    fn test_literal_result_element_with_standard_attribute() {
        assert_ron_snapshot!(parse(
            r#"<foo xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:expand-text="yes"/>"#
        ));
    }
}
