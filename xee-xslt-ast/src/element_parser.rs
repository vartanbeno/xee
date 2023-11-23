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
    pub(crate) fn many_elements<T>(
        &self,
        node: Node,
        parse: impl Fn(Node) -> Result<T>,
    ) -> Result<(Vec<T>, Option<Node>)>
    where
        T: InstructionParser,
    {
        let mut result = Vec::new();
        let mut current_node = node;
        loop {
            let (item, next) = self.optional_element(current_node, &parse)?;
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

    pub(crate) fn one_or_more_elements<T>(
        &self,
        node: Node,
        parse: impl Fn(Node) -> Result<T>,
    ) -> Result<(Vec<T>, Option<Node>)>
    where
        T: InstructionParser,
    {
        let (items, node) = self.many_elements(node, parse)?;
        if items.is_empty() {
            if let Some(node) = node {
                let span = self
                    .span_info
                    .get(SpanInfoKey::ElementStart(node))
                    .ok_or(Error::MissingSpan)?;
                if let Some(element) = self.xot.element(node) {
                    let (local, namespace) = self.xot.name_ns_str(element.name());
                    return Err(Error::UnexpectedElement {
                        name: XmlName {
                            local: local.to_string(),
                            namespace: namespace.to_string(),
                        },
                        span: span.into(),
                    });
                } else {
                    // how to deal with text nodes and other types of nodes
                    todo!()
                }
            } else {
                todo!()
                // let span = self.span_info.get(SpanInfoKey::ElementEnd(node));
                // return Err(Error::ExpectedElementNotFound {
                //     expected: Name,
                //     span,
                // });
            }
        }
        Ok((items, node))
    }

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
        node: Node,
        parse: impl Fn(Node) -> Result<T>,
    ) -> Result<(Option<T>, Option<Node>)>
    where
        T: InstructionParser,
    {
        let item = parse(node);
        match item {
            Ok(item) => Ok((Some(item), self.xot.next_sibling(node))),
            Err(Error::Unexpected) => Ok((None, Some(node))),
            Err(e) => Err(e),
        }
    }
}
