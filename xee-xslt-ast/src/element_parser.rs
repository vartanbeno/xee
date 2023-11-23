use xot::{Node, SpanInfo, SpanInfoKey, Xot};

use crate::{
    error::{Error, Result, XmlName},
    instruction::InstructionParser,
};

struct ElementParser<'a> {
    xot: &'a Xot,
    span_info: &'a SpanInfo,
}

//
// <content><foo/></content>
// <content><foo/></content> or <content></content>
// <content></content> <content><foo/></content> or <content><foo/></foo></content>

impl<'a> ElementParser<'a> {
    // pub(crate) fn many_elements<T>(
    //     &self,
    //     node: Option<Node>,
    //     parse: impl Fn(Node) -> Result<T>,
    // ) -> Result<(Vec<T>, Option<Node>)> {
    //     let mut result = Vec::new();
    //     let mut current_node = node;
    //     loop {
    //         let (item, next) = self.optional_element(current_node, &parse)?;
    //         if let Some(item) = item {
    //             result.push(item);
    //         } else {
    //             // we couldn't match with another parseable item, so continue
    //             return Ok((result, next));
    //         }
    //         if let Some(next) = next {
    //             current_node = next;
    //         } else {
    //             // there are no more siblings
    //             return Ok((result, None));
    //         }
    //     }
    // }

    // pub(crate) fn one_or_more_elements<T>(
    //     &self,
    //     node: Node,
    //     parse: impl Fn(Node) -> Result<T>,
    // ) -> Result<(Vec<T>, Option<Node>)> {
    //     let (items, node) = self.many_elements(node, parse)?;
    //     if items.is_empty() {
    //         if let Some(node) = node {
    //             let span = self
    //                 .span_info
    //                 .get(SpanInfoKey::ElementStart(node))
    //                 .ok_or(Error::MissingSpan)?;
    //             if let Some(element) = self.xot.element(node) {
    //                 let (local, namespace) = self.xot.name_ns_str(element.name());
    //                 return Err(Error::UnexpectedElement {
    //                     name: XmlName {
    //                         local: local.to_string(),
    //                         namespace: namespace.to_string(),
    //                     },
    //                     span: span.into(),
    //                 });
    //             } else {
    //                 // how to deal with text nodes and other types of nodes
    //                 todo!()
    //             }
    //         } else {
    //             todo!()
    //             // let span = self.span_info.get(SpanInfoKey::ElementEnd(node));
    //             // return Err(Error::ExpectedElementNotFound {
    //             //     expected: Name,
    //             //     span,
    //             // });
    //         }
    //     }
    //     Ok((items, node))
    // }

    // pub(crate) fn many_elements_by_name<T>(
    //     &self,
    //     node: Node,
    //     name: NameId,
    // ) -> Result<(Vec<T>, Option<Node>)>
    // where
    //     T: InstructionParser,
    // {
    //     self.many_elements2(node, |node| self.xslt_parser.parse_element(node, name))
    // }

    pub(crate) fn optional_element<T>(
        &self,
        node: Option<Node>,
        parse: impl Fn(Node) -> Result<T>,
    ) -> Result<(Option<T>, Option<Node>)> {
        if let Some(node) = node {
            let item = parse(node);
            match item {
                Ok(item) => Ok((Some(item), self.xot.next_sibling(node))),
                Err(Error::Unexpected) => Ok((None, Some(node))),
                Err(e) => Err(e),
            }
        } else {
            Ok((None, None))
        }
    }

    pub(crate) fn end(&self, node: Option<Node>) -> Result<()> {
        if let Some(_node) = node {
            Err(Error::Unexpected)
        } else {
            Ok(())
        }
    }
}

// * we need to be able to declare that an instruction has absolutely no content, so
// if it finds content that's an error. Example xsl:accept

// * we need to be able to declare that an instruction contains contains only elements,
// so any text node is an error

// * for sequence constructors, text nodes are allowed it's a mixture of zero
// or more instruction elements, mixed with literal elements and literal text
// nodes

// * we need to have some kind of 'we reached the end' check in the parser,
// any nodes following can't be parsed, and are thus an error

#[cfg(test)]
mod tests {
    use crate::ast_core::Span;

    use super::*;

    #[test]
    fn test_optional_element() {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info("<outer><a /></outer>").unwrap();
        let outer = xot.document_element(doc).unwrap();
        let a = xot.first_child(outer);
        let element_parser = ElementParser {
            xot: &xot,
            span_info: &span_info,
        };
        #[derive(Debug, PartialEq)]
        struct Value;
        let (item, next) = element_parser
            .optional_element(a, |_node| Ok(Value))
            .unwrap();
        assert_eq!(item, Some(Value));
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_element_parse_error() {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info("<outer><a /></outer>").unwrap();
        let outer = xot.document_element(doc).unwrap();
        let a = xot.first_child(outer);
        let element_parser = ElementParser {
            xot: &xot,
            span_info: &span_info,
        };
        #[derive(Debug, PartialEq)]
        struct Value;
        let r: Result<(Option<Value>, Option<Node>)> =
            element_parser.optional_element(a, |_node| {
                Err(Error::Invalid {
                    value: "".to_string(),
                    span: Span::new(0, 0),
                })
            });
        assert_eq!(
            r,
            Err(Error::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0)
            })
        );
    }

    #[test]
    fn test_optional_element_parse_unexpected_element() {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info("<outer><a /></outer>").unwrap();
        let outer = xot.document_element(doc).unwrap();
        let a = xot.first_child(outer);
        let element_parser = ElementParser {
            xot: &xot,
            span_info: &span_info,
        };
        #[derive(Debug, PartialEq)]
        struct Value;
        let (item, next): (Option<Value>, Option<Node>) = element_parser
            .optional_element(a, |_node| Err(Error::Unexpected))
            .unwrap();
        assert_eq!(item, None);
        assert_eq!(next, a);
    }

    #[test]
    fn test_optional_element_parse_node_none() {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info("<outer></outer>").unwrap();
        let outer = xot.document_element(doc).unwrap();
        let a = xot.first_child(outer);
        let element_parser = ElementParser {
            xot: &xot,
            span_info: &span_info,
        };
        #[derive(Debug, PartialEq)]
        struct Value;
        let (item, next): (Option<Value>, Option<Node>) = element_parser
            .optional_element(a, |_node| Err(Error::Unexpected))
            .unwrap();
        assert_eq!(item, None);
        assert_eq!(next, None);
    }

    #[test]
    fn test_end_found() {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info("<outer></outer>").unwrap();
        let outer = xot.document_element(doc).unwrap();
        let a = xot.first_child(outer);
        let element_parser = ElementParser {
            xot: &xot,
            span_info: &span_info,
        };
        let r = element_parser.end(a);

        assert!(r.is_ok());
    }

    #[test]
    fn test_end_not_found() {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info("<outer><a /></outer>").unwrap();
        let outer = xot.document_element(doc).unwrap();
        let a = xot.first_child(outer);
        let element_parser = ElementParser {
            xot: &xot,
            span_info: &span_info,
        };
        let r = element_parser.end(a);

        assert_eq!(r, Err(Error::Unexpected));
    }
}
