use xee_xpath_ast::Namespaces;
use xot::{Element, NameId, Node, SpanInfo, SpanInfoKey, Value, Xot};

use crate::ast_core as ast;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
struct Span {
    start: usize,
    end: usize,
}

impl From<&xot::Span> for Span {
    fn from(span: &xot::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

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

    fn eqname(&self, s: &str) -> Result<String, Error> {
        Ok(s.to_string())
    }

    fn xpath(&self, s: &str) -> Result<xee_xpath_ast::ast::XPath, Error> {
        Ok(xee_xpath_ast::ast::XPath::parse(s, &self.namespaces, &[])?)
    }

    fn attribute_missing_error_with_span(&self, node: Node, f: impl Fn(Span) -> Error) -> Error {
        let span = self.attribute_missing_span(node);
        match span {
            Ok(span) => f(span),
            Err(e) => e,
        }
    }

    fn attribute_value_error_with_span(
        &self,
        node: Node,
        name: NameId,
        f: impl Fn(Span) -> Error,
    ) -> Error {
        let span = self.attribute_value_span(node, name);
        match span {
            Ok(span) => f(span),
            Err(e) => e,
        }
    }

    fn attribute_missing_span(&self, node: Node) -> Result<Span, Error> {
        let span = self.span_info.get(SpanInfoKey::ElementStart(node));
        if let Some(span) = span {
            Ok(span.into())
        } else {
            Err(Error::MissingSpan)
        }
    }

    fn attribute_value_span(&self, node: Node, name: NameId) -> Result<Span, Error> {
        let span = self.span_info.get(SpanInfoKey::AttributeValue(node, name));
        if let Some(span) = span {
            Ok(span.into())
        } else {
            Err(Error::MissingSpan)
        }
    }

    fn parse_element(&self, node: Node, name: NameId) -> Result<&'a Element, Error> {
        let element = self.xot.element(node).ok_or(Error::Unexpected)?;
        if element.name() != name {
            return Err(Error::Unexpected);
        }
        Ok(element)
    }

    fn parse_attributes(&self, node: Node, name: NameId) -> Result<Attributes, Error> {
        let element = self.xot.element(node).ok_or(Error::Unexpected)?;
        if element.name() != name {
            return Err(Error::Unexpected);
        }
        Ok(Attributes {
            node,
            element,
            xslt_parser: self,
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

    fn parse_boolean(&self, s: &str) -> Option<bool> {
        match s {
            "yes" | "true" | "1" => Some(true),
            "no" | "false" | "0" => Some(false),
            _ => None,
        }
    }

    fn parse_if(&self, node: Node) -> Result<ast::If, Error> {
        let attributes = self.parse_attributes(node, self.names.if_)?;

        let test = attributes.required_attribute(self.names.test, |s| self.xpath(s))?;

        let content = self.parse_sequence_constructor(node)?;
        Ok(ast::If { test, content })
    }

    fn parse_variable(&self, node: Node) -> Result<ast::Variable, Error> {
        let attributes = self.parse_attributes(node, self.names.variable)?;

        let name = attributes.required_attribute(self.names.name, |s| self.eqname(s))?;
        let select = attributes.attribute(self.names.select, |s| self.xpath(s))?;

        Ok(ast::Variable {
            name,
            select,
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

struct Attributes<'a> {
    node: Node,
    element: &'a Element,
    xslt_parser: &'a XsltParser<'a>,
}

impl<'a> Attributes<'a> {
    fn attribute<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str) -> Result<T, Error>,
    ) -> Result<Option<T>, Error> {
        if let Some(value) = self.element.get_attribute(name) {
            let value = parse_value(value).map_err(|e| {
                if let Error::XPath(e) = e {
                    Error::XPath(
                        e.adjust(
                            self.xslt_parser
                                .span_info
                                .get(SpanInfoKey::AttributeValue(self.node, name))
                                .unwrap()
                                .start,
                        ),
                    )
                } else {
                    e
                }
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn required_attribute<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.attribute(name, parse_value)?.ok_or_else(|| {
            self.xslt_parser
                .attribute_missing_error_with_span(self.node, |span| {
                    let (local, namespace) = self.xslt_parser.xot.name_ns_str(name);
                    Error::AttributeExpected {
                        namespace: namespace.to_string(),
                        local: local.to_string(),
                        span,
                    }
                })
        })
    }

    fn boolean(&self, name: NameId, default: bool) -> Result<bool, Error> {
        self.attribute(name, |s| {
            self.xslt_parser.parse_boolean(s).ok_or_else(|| {
                self.xslt_parser
                    .attribute_value_error_with_span(self.node, name, |span| {
                        Error::InvalidBoolean {
                            value: s.to_string(),
                            span,
                        }
                    })
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
