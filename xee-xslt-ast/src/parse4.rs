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
    if_: xot::NameId,
    test: xot::NameId,
    variable: xot::NameId,
    select: xot::NameId,
    name: xot::NameId,
}

impl Names {
    fn new(xot: &mut Xot) -> Self {
        Self {
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

    fn get_attribute<T>(
        &self,
        element: &'a Element,
        name: NameId,
        parse_value: impl Fn(&'a str) -> Result<T, Error>,
    ) -> Result<Option<T>, Error> {
        if let Some(value) = element.get_attribute(name) {
            let value = parse_value(value)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn get_xpath_attribute(
        &self,
        node: Node,
        element: &'a Element,
        name: NameId,
    ) -> Result<Option<xee_xpath_ast::ast::XPath>, Error> {
        self.get_attribute(element, name, |s| {
            Ok(
                xee_xpath_ast::ast::XPath::parse(s, &self.namespaces, &[]).map_err(|e| {
                    e.adjust(
                        self.span_info
                            .get(SpanInfoKey::AttributeValue(node, name))
                            .unwrap()
                            .start,
                    )
                })?,
            )
        })
    }

    fn get_required_attribute<T>(
        &self,
        node: Node,
        element: &'a Element,
        name: NameId,
        parse_value: impl Fn(&'a str) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let value = element.get_attribute(name).ok_or_else(|| {
            let span = self.span_info.get(SpanInfoKey::ElementStart(node));
            if let Some(span) = span {
                let (local, namespace) = self.xot.name_ns_str(name);
                Error::AttributeExpected {
                    namespace: namespace.to_string(),
                    local: local.to_string(),
                    span: span.into(),
                }
            } else {
                Error::MissingSpan
            }
        })?;
        parse_value(value)
    }

    fn parse_element(&self, node: Node, name: NameId) -> Result<&'a Element, Error> {
        let element = self.xot.element(node).ok_or(Error::Unexpected)?;
        if element.name() != name {
            return Err(Error::Unexpected);
        }
        Ok(element)
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
        let element = self.parse_element(node, self.names.if_)?;
        let test = self.get_required_attribute(node, element, self.names.test, Ok)?;
        let test = xee_xpath_ast::ast::XPath::parse(test, &self.namespaces, &[])?;
        let content = self.parse_sequence_constructor(node)?;
        Ok(ast::If { test, content })
    }

    fn parse_variable(&self, node: Node) -> Result<ast::Variable, Error> {
        let element = self.parse_element(node, self.names.variable)?;
        let name = self.get_required_attribute(node, element, self.names.name, Ok)?;
        let select = self.get_xpath_attribute(node, element, self.names.select)?;

        Ok(ast::Variable {
            name: name.to_string(),
            select,
            content: self.parse_sequence_constructor(node)?,
        })
    }

    fn parse_sequence_constructor(&self, node: Node) -> Result<ast::SequenceConstructor, Error> {
        let mut result = Vec::new();
        for node in self.xot.children(node) {
            match self.xot.value(node) {
                Value::Text(text) => {
                    result.push(ast::SequenceConstructorItem::Text(text.get().to_string()))
                }
                _ => return Err(Error::UnexpectedSequenceConstructor),
            }
        }
        Ok(result)
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
