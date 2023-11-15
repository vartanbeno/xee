use xee_xpath_ast::Namespaces;
use xot::{NameId, Node, SpanInfo, SpanInfoKey, Value, Xot};

use crate::ast_core as ast;
use crate::ast_core::Span;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
enum Error {
    Unexpected,
    AttributeExpected {
        namespace: String,
        local: String,
        span: Span,
    },
    UnexpectedSequenceConstructor,
    InvalidBoolean {
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

struct Names {
    copy: xot::NameId,
    if_: xot::NameId,
    test: xot::NameId,
    variable: xot::NameId,
    select: xot::NameId,
    name: xot::NameId,
}

impl Names {
    fn new(xot: &mut Xot) -> Self {
        Self {
            copy: xot.add_name("copy"),
            if_: xot.add_name("if"),
            test: xot.add_name("test"),
            variable: xot.add_name("variable"),
            select: xot.add_name("select"),
            name: xot.add_name("name"),
        }
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
        Ok(s.to_string())
    }

    fn xpath(&self, s: &str, span: Span) -> Result<ast::Expression, Error> {
        Ok(ast::Expression {
            xpath: xee_xpath_ast::ast::XPath::parse(s, &self.namespaces, &[])?,
            span,
        })
    }

    fn boolean(s: &str) -> Option<bool> {
        match s {
            "yes" | "true" | "1" => Some(true),
            "no" | "false" | "0" => Some(false),
            _ => None,
        }
    }

    fn element_span(&self, node: Node) -> Result<Span, Error> {
        let span = self
            .span_info
            .get(SpanInfoKey::ElementStart(node))
            .ok_or(Error::MissingSpan)?;

        Ok(span.into())
    }

    fn element(&self, node: Node, name: NameId) -> Result<Element, Error> {
        let element = self.xot.element(node).ok_or(Error::Unexpected)?;
        if element.name() != name {
            return Err(Error::Unexpected);
        }
        Ok(Element {
            node,
            element,
            xslt_parser: self,
            span: self.element_span(node)?,
        })
    }

    fn parse(&self, node: Node) -> Result<ast::Instruction, Error> {
        match self.parse_if(node).map(ast::Instruction::If) {
            Ok(instruction) => Ok(instruction),
            Err(e) if e.is_unexpected() => {
                match self.parse_variable(node).map(ast::Instruction::Variable) {
                    Ok(instruction) => Ok(instruction),
                    // Err(e) if e.is_unexpected() => Err(Error::Unexpected),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn parse_if(&self, node: Node) -> Result<ast::If, Error> {
        let element = self.element(node, self.names.if_)?;

        let content = self.parse_sequence_constructor(node)?;
        Ok(ast::If {
            test: element.required(self.names.test, |s, span| self.xpath(s, span))?,
            content,
        })
    }

    fn parse_variable(&self, node: Node) -> Result<ast::Variable, Error> {
        let element = self.element(node, self.names.variable)?;

        Ok(ast::Variable {
            name: element.required(self.names.name, Self::eqname)?,
            select: element.optional(self.names.select, |s, span| self.xpath(s, span))?,
            as_: None,
            static_: None,
            visibility: None,
            content: self.parse_sequence_constructor(node)?,
        })
    }

    // fn parse_copy(&self, node: Node) -> Result<ast::Copy, Error> {
    //     let element = self.parse_element(node, self.names.copy)?;
    //     let select = self.get_xpath_attribute(node, element, self.names.select)?;
    //     let copy_namespaces = self.get_boolean_attribute(node, element, self.names.copy_namespaces, true)?
    // }

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
}

struct Element<'a> {
    node: Node,
    element: &'a xot::Element,
    xslt_parser: &'a XsltParser<'a>,
    span: Span,
}

impl<'a> Element<'a> {
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
            XsltParser::boolean(s).ok_or_else(|| Error::InvalidBoolean {
                value: s.to_string(),
                span,
            })
        })
        .map(|v| v.unwrap_or(default))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use insta::assert_ron_snapshot;
    use xee_xpath_ast::Namespaces;

    #[test]
    fn test_simple_parse_if() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let namespaces = Namespaces::default();

        let (node, span_info) = xot
            .parse_with_span_info(r#"<if test="true()">Hello</if>"#)
            .unwrap();
        let node = xot.document_element(node).unwrap();
        let parser = XsltParser::new(&xot, &names, &span_info, namespaces);
        assert_ron_snapshot!(parser.parse(node));
    }

    #[test]
    fn test_simple_parse_variable() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let namespaces = Namespaces::default();

        let (node, span_info) = xot
            .parse_with_span_info(r#"<variable name="foo" select="true()">Hello</variable>"#)
            .unwrap();
        let node = xot.document_element(node).unwrap();
        let parser = XsltParser::new(&xot, &names, &span_info, namespaces);

        assert_ron_snapshot!(parser.parse(node));
    }

    #[test]
    fn test_simple_parse_variable_missing_required_name_attribute() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let namespaces = Namespaces::default();

        let (node, span_info) = xot
            .parse_with_span_info(r#"<variable select="true()">Hello</variable>"#)
            .unwrap();
        let node = xot.document_element(node).unwrap();
        let parser = XsltParser::new(&xot, &names, &span_info, namespaces);

        assert_ron_snapshot!(parser.parse(node));
    }

    #[test]
    fn test_simple_parse_variable_broken_xpath() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let namespaces = Namespaces::default();

        let (node, span_info) = xot
            .parse_with_span_info(r#"<variable name="foo" select="let $x := 1">Hello</variable>"#)
            .unwrap();
        let node = xot.document_element(node).unwrap();
        let parser = XsltParser::new(&xot, &names, &span_info, namespaces);

        assert_ron_snapshot!(parser.parse(node));
    }
}
