use xot::{Node, SpanInfo, Xot};

use crate::error::{Error, Result};

trait ChildrenParser<T> {
    fn parse(&self, node: Option<Node>) -> Result<(T, Option<Node>)>;
}

struct OptionalChildParser<'a, V, P>
where
    P: Fn(Node) -> Result<V>,
{
    xot: &'a Xot,
    span_info: &'a SpanInfo,
    parse_value: P,
}

impl<'a, V, P> OptionalChildParser<'a, V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn new(xot: &'a Xot, span_info: &'a SpanInfo, parse_value: P) -> Self {
        Self {
            xot,
            span_info,
            parse_value,
        }
    }
}

impl<'a, V, P> ChildrenParser<Option<V>> for OptionalChildParser<'a, V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn parse(&self, node: Option<Node>) -> Result<(Option<V>, Option<Node>)> {
        if let Some(node) = node {
            let item = (self.parse_value)(node);
            match item {
                Ok(item) => Ok((Some(item), self.xot.next_sibling(node))),
                Err(Error::Unexpected) => Ok((None, Some(node))),
                Err(e) => Err(e),
            }
        } else {
            Ok((None, None))
        }
    }
}

struct EndParser;

impl EndParser {
    fn new() -> Self {
        Self
    }
}

impl ChildrenParser<()> for EndParser {
    fn parse(&self, node: Option<Node>) -> Result<((), Option<Node>)> {
        if let Some(_node) = node {
            Err(Error::Unexpected)
        } else {
            Ok(((), None))
        }
    }
}

struct ManyChildrenParser<'a, V, P>
where
    P: Fn(Node) -> Result<V>,
{
    xot: &'a Xot,
    span_info: &'a SpanInfo,
    parse_value: P,
}

impl<'a, V, P> ManyChildrenParser<'a, V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn new(xot: &'a Xot, span_info: &'a SpanInfo, parse_value: P) -> Self {
        Self {
            xot,
            span_info,
            parse_value,
        }
    }
}

impl<'a, V, P> ChildrenParser<Vec<V>> for ManyChildrenParser<'a, V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn parse(&self, node: Option<Node>) -> Result<(Vec<V>, Option<Node>)> {
        let mut result = Vec::new();
        let mut current_node = node;
        let optional_parser = OptionalChildParser {
            xot: self.xot,
            span_info: self.span_info,
            parse_value: &self.parse_value,
        };
        loop {
            let (item, next) = optional_parser.parse(current_node)?;
            if let Some(item) = item {
                result.push(item);
            } else {
                // we couldn't match with another parseable item, so we're done
                return Ok((result, next));
            }
            if let Some(next) = next {
                current_node = Some(next);
            } else {
                // there are no more siblings
                return Ok((result, None));
            }
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
    use xot::NameId;

    use crate::ast_core::Span;

    use super::*;

    fn parse2(s: &str) -> (Xot, SpanInfo, Option<Node>) {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info(s).unwrap();
        let outer = xot.document_element(doc).unwrap();
        let next = xot.first_child(outer);
        (xot, span_info, next)
    }

    #[test]
    fn test_optional_present() {
        let (xot, span_info, next) = parse2("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(&xot, &span_info, |_node| Ok(Value));

        let (item, next) = optional_parser.parse(next).unwrap();

        assert_eq!(item, Some(Value));
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_present_but_parse_error() {
        let (xot, span_info, next) = parse2("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(&xot, &span_info, |_node| {
            Err(Error::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0),
            })
        });

        let r: Result<(Option<Value>, Option<Node>)> = optional_parser.parse(next);

        assert_eq!(
            r,
            Err(Error::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0)
            })
        );
    }

    #[test]
    fn test_optional_unexpected_node() {
        let (xot, span_info, node) = parse2("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser =
            OptionalChildParser::new(&xot, &span_info, |_node| Err(Error::Unexpected));
        let (item, next): (Option<Value>, Option<Node>) = optional_parser.parse(node).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, node);
    }

    #[test]
    fn test_optional_not_present() {
        let (xot, span_info, next) = parse2("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(&xot, &span_info, |_node| Ok(Value));
        let (item, next) = optional_parser.parse(next).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, None);
    }

    #[test]
    fn test_end_found() {
        let (_xot, _span_info, next) = parse2("<outer></outer>");

        let end_parser = EndParser::new();
        let r = end_parser.parse(next);

        assert!(r.is_ok());
    }

    #[test]
    fn test_end_not_found() {
        let (_xot, _span_info, next) = parse2("<outer><a /></outer>");

        let end_parser = EndParser::new();
        let r = end_parser.parse(next);

        assert_eq!(r, Err(Error::Unexpected));
    }

    #[derive(Debug, PartialEq)]
    struct ValueA;
    #[derive(Debug, PartialEq)]
    struct ValueB;

    struct Names {
        name_a: NameId,
        name_b: NameId,
    }

    impl Names {
        fn new(xot: &mut Xot) -> Self {
            Self {
                name_a: xot.add_name("a"),
                name_b: xot.add_name("b"),
            }
        }
    }

    fn parse_two_optional_elements(
        names: &Names,
        xot: &Xot,
        span_info: &SpanInfo,
        next: Option<Node>,
    ) -> Result<(Option<ValueA>, Option<ValueB>)> {
        let optional_parser_a = OptionalChildParser::new(xot, span_info, |node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(Error::Unexpected)
        });
        let (item_a, next) = optional_parser_a.parse(next).unwrap();

        let optional_parser_b = OptionalChildParser::new(xot, span_info, |node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(Error::Unexpected)
        });
        let (item_b, next) = optional_parser_b.parse(next).unwrap();

        let end_parser = EndParser::new();
        end_parser.parse(next)?;
        Ok((item_a, item_b))
    }

    #[test]
    fn test_two_optional_both_present() {
        let (mut xot, span_info, next) = parse2("<outer><a /><b /></outer>");
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_optional_only_a_present() {
        let (mut xot, span_info, next) = parse2("<outer><a /></outer>");
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_optional_only_b_present() {
        let (mut xot, span_info, next) = parse2("<outer><b /></outer>");
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_optional_neither_present() {
        let (mut xot, span_info, next) = parse2("<outer></outer>");
        // let mut xot = Xot::new();
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_optional_unexpected() {
        let (mut xot, span_info, next) = parse2("<outer><c /></outer>");
        let names = Names::new(&mut xot);

        let r = parse_two_optional_elements(&names, &xot, &span_info, next);
        assert_eq!(r, Err(Error::Unexpected));
    }

    #[test]
    fn test_many() {
        let (xot, span_info, next) = parse2("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(&xot, &span_info, |_node| Ok(Value));

        let (items, next) = many_parser.parse(next).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_many_empty() {
        let (xot, span_info, next) = parse2("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(&xot, &span_info, |_node| Ok(Value));

        let (items, next) = many_parser.parse(next).unwrap();

        assert_eq!(items, vec![]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_then_many() {
        let (mut xot, span_info, next) = parse2("<outer><a /><b /><b /></outer>");
        // let mut xot = Xot::new();

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(&xot, &span_info, |node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(Error::Unexpected)
        });

        let many_parser = ManyChildrenParser::new(&xot, &span_info, |_node| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(Error::Unexpected)
        });

        let (optional_item, next) = optional_parser.parse(next).unwrap();
        let (many_items, next) = many_parser.parse(next).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }
}
