use std::collections::BTreeMap;

use xee_xpath_ast::{ast as xpath_ast, Namespaces};
use xot::{NameId, Node, SpanInfo, SpanInfoKey, Value, Xot};

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
}

impl Names {
    fn new(xot: &mut Xot) -> Self {
        let copy = xot.add_name("copy");
        let if_ = xot.add_name("if");
        let variable = xot.add_name("variable");

        let mut sequence_constructor_names = BTreeMap::new();
        sequence_constructor_names.insert(if_, SequenceConstructorName::If);
        sequence_constructor_names.insert(variable, SequenceConstructorName::Variable);
        sequence_constructor_names.insert(copy, SequenceConstructorName::Copy);

        Self {
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

    fn eqname(s: &str, _span: Span) -> Result<String, Error> {
        // TODO: should actually parse
        Ok(s.to_string())
    }

    fn xpath(&self, s: &str, span: Span) -> Result<ast::Expression, Error> {
        Ok(ast::Expression {
            xpath: xpath_ast::XPath::parse(s, &self.namespaces, &[])?,
            span,
        })
    }

    fn sequence_type(&self, s: &str, _span: Span) -> Result<xpath_ast::SequenceType, Error> {
        Ok(xpath_ast::SequenceType::parse(s, &self.namespaces)?)
    }

    fn boolean(s: &str) -> Option<bool> {
        match s {
            "yes" | "true" | "1" => Some(true),
            "no" | "false" | "0" => Some(false),
            _ => None,
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

    fn eqnames(s: &str, span: Span) -> Result<Vec<String>, Error> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::eqname(s, span)?);
        }
        Ok(result)
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

    fn element_span(&self, node: Node) -> Result<Span, Error> {
        let span = self
            .span_info
            .get(SpanInfoKey::ElementStart(node))
            .ok_or(Error::MissingSpan)?;

        Ok(span.into())
    }

    fn element(&self, node: Node) -> Result<Element, Error> {
        let element = self.xot.element(node).ok_or(Error::Unexpected)?;

        Ok(Element {
            node,
            element,
            xslt_parser: self,
            span: self.element_span(node)?,
        })
    }

    fn parse(&self, node: Node) -> Result<ast::SequenceConstructorItem, Error> {
        let element = self.element(node)?;
        let sname = self
            .names
            .sequence_constructor_name(element.element.name())
            .ok_or(Error::Unexpected)?;
        match sname {
            SequenceConstructorName::Copy => ast::Copy::parse(element),
            SequenceConstructorName::If => ast::If::parse(element),
            SequenceConstructorName::Variable => ast::Variable::parse(element),
        }
    }

    fn parse_sequence_constructor(&self, node: Node) -> Result<ast::SequenceConstructor, Error> {
        let mut result = Vec::new();
        for node in self.xot.children(node) {
            match self.xot.value(node) {
                Value::Text(text) => result.push(ast::SequenceConstructorItem::TextNode(
                    text.get().to_string(),
                )),
                _ => return Err(Error::UnexpectedSequenceConstructor),
            }
        }
        Ok(result)
    }

    fn name_span(&self, node: Node, name: NameId) -> Result<Span, Error> {
        let span = self
            .span_info
            .get(SpanInfoKey::AttributeName(node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
    }

    fn attribute_unexpected(&self, node: Node, name: NameId, message: &str) -> Error {
        let (local, namespace) = self.xot.name_ns_str(name);
        let span = self.name_span(node, name);
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

struct Element<'a> {
    node: Node,
    element: &'a xot::Element,
    xslt_parser: &'a XsltParser<'a>,
    span: Span,
}

impl<'a> Element<'a> {
    fn name_span(&self, name: NameId) -> Result<Span, Error> {
        let span = self
            .xslt_parser
            .span_info
            .get(SpanInfoKey::AttributeName(self.node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
    }

    fn value_span(&self, name: NameId) -> Result<Span, Error> {
        let span = self
            .xslt_parser
            .span_info
            .get(SpanInfoKey::AttributeValue(self.node, name))
            .ok_or(Error::MissingSpan)?;
        Ok(span.into())
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
            let (local, namespace) = self.xslt_parser.xot.name_ns_str(name);
            Error::AttributeExpected {
                namespace: namespace.to_string(),
                local: local.to_string(),
                span: self.span,
            }
        })
    }

    fn boolean(&self, name: NameId, default: bool) -> Result<bool, Error> {
        self.optional(name, |s, span| {
            XsltParser::boolean(s).ok_or_else(|| Error::Invalid {
                value: s.to_string(),
                span,
            })
        })
        .map(|v| v.unwrap_or(default))
    }
}

trait InstructionParser: Sized + Into<ast::SequenceConstructorItem> {
    fn parse(element: Element) -> Result<ast::SequenceConstructorItem, Error> {
        let ast = Self::parse_ast(element)?;
        Ok(ast.into())
    }

    fn parse_ast(element: Element) -> Result<Self, Error>;
}

impl InstructionParser for ast::Copy {
    fn parse_ast(element: Element) -> Result<Self, Error> {
        let parser = element.xslt_parser;
        let content = parser.parse_sequence_constructor(element.node)?;
        let names = parser.names;
        Ok(ast::Copy {
            select: element.optional(names.select, |s, span| parser.xpath(s, span))?,
            copy_namespaces: element.boolean(names.copy_namespaces, true)?,
            inherit_namespaces: element.boolean(names.inherit_namespaces, true)?,
            use_attribute_sets: element.optional(names.use_attribute_sets, XsltParser::eqnames)?,
            type_: element.optional(names.as_, XsltParser::eqname)?,
            validation: element
                .optional(names.validation, XsltParser::validation)?
                // TODO: should depend on global validation attribute
                .unwrap_or(ast::Validation::Strip),
            content,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::If {
    fn parse_ast(element: Element) -> Result<Self, Error> {
        let parser = element.xslt_parser;
        Ok(ast::If {
            test: element.required(parser.names.test, |s, span| parser.xpath(s, span))?,
            content: parser.parse_sequence_constructor(element.node)?,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::Variable {
    fn parse_ast(element: Element) -> Result<Self, Error> {
        let parser = element.xslt_parser;
        let names = parser.names;
        let select = element.optional(names.select, |s, span| parser.xpath(s, span))?;
        let static_ = element.boolean(names.static_, false)?;

        let visibility =
            element.optional(names.visibility, XsltParser::visibility_with_abstract)?;
        // This is a rule somewhere, but not sure whether it's correct;
        // can visibility be absent or is there a default visibility?
        // let visibility = visibility.unwrap_or(if static_ {
        //     ast::VisibilityWithAbstract::Private
        // } else {
        //     ast::VisibilityWithAbstract::Public
        // });
        if visibility == Some(ast::VisibilityWithAbstract::Abstract) && select.is_some() {
            return Err(parser.attribute_unexpected(
                element.node,
                names.select,
                "select attribute is not allowed when visibility is abstract",
            ));
        }

        Ok(ast::Variable {
            name: element.required(names.name, XsltParser::eqname)?,
            select,
            as_: element.optional(names.as_, |s, span| parser.sequence_type(s, span))?,
            static_,
            visibility,
            content: parser.parse_sequence_constructor(element.node)?,
            span: element.span,
        })
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
        assert_ron_snapshot!(parse(r#"<if test="true()">Hello</if>"#));
    }

    #[test]
    fn test_variable() {
        assert_ron_snapshot!(parse(
            r#"<variable name="foo" select="true()">Hello</variable>"#
        ));
    }

    #[test]
    fn test_missing_required() {
        assert_ron_snapshot!(parse(r#"<variable select="true()">Hello</variable>"#));
    }

    #[test]
    fn test_broken_xpath() {
        assert_ron_snapshot!(parse(
            r#"<variable name="foo" select="let $x := 1">Hello</variable>"#
        ));
    }

    #[test]
    fn test_sequence_type() {
        assert_ron_snapshot!(parse(
            r#"<variable name="foo" as="xs:string" select="true()">Hello</variable>"#
        ));
    }

    #[test]
    fn test_boolean_default_no_with_explicit_yes() {
        assert_ron_snapshot!(parse(
            r#"<variable name="foo" static="yes" as="xs:string" select="true()">Hello</variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility() {
        assert_ron_snapshot!(parse(
            r#"<variable name="foo" visibility="public">Hello</variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility_abstract_with_select_is_error() {
        assert_ron_snapshot!(parse(
            r#"<variable name="foo" visibility="abstract" select="true()">Hello</variable>"#
        ));
    }

    #[test]
    fn test_copy() {
        assert_ron_snapshot!(parse(
            r#"<copy select="true()" copy-namespaces="no" inherit-namespaces="no" validation="strict">Hello</copy>"#
        ));
    }

    #[test]
    fn test_eqnames() {
        assert_ron_snapshot!(parse(
            r#"<copy use-attribute-sets="foo bar baz">Hello</copy>"#
        ));
    }
}
