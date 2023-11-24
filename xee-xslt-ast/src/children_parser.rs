use xot::Node;
use xot::SpanInfo;
use xot::SpanInfoKey;
use xot::Xot;

use crate::ast_core::Span;
use crate::error::Error as AttributeError;

#[derive(Debug, PartialEq)]
pub(crate) enum ElementError {
    // Did not expect this node
    Unexpected { span: Span },
    // Did not expect end TODO: how to get span info?
    UnexpectedEnd,
    // An attribute of the element was invalid
    Attribute(AttributeError),

    // internal error, should not happen
    Internal,
}

impl From<AttributeError> for ElementError {
    fn from(error: AttributeError) -> Self {
        Self::Attribute(error)
    }
}

type Result<T> = std::result::Result<T, ElementError>;

pub(crate) struct Data {}

pub(crate) struct Context<'a> {
    xot: &'a Xot,
    span_info: &'a SpanInfo,
    data: Data,
}

impl<'a> Context<'a> {
    fn new(xot: &'a Xot, span_info: &'a SpanInfo) -> Self {
        Self {
            xot,
            span_info,
            data: Data {},
        }
    }

    fn next(&self, node: Node) -> Option<Node> {
        self.xot.next_sibling(node)
    }

    fn span(&self, node: Node) -> Option<Span> {
        span_for_node(self.xot, self.span_info, node)
    }
}

fn span_for_node(xot: &Xot, span_info: &SpanInfo, node: Node) -> Option<Span> {
    use xot::Value::*;

    match xot.value(node) {
        Element(_element) => span_info.get(SpanInfoKey::ElementStart(node)),
        Text(_text) => span_info.get(SpanInfoKey::Text(node)),
        Comment(_comment) => span_info.get(SpanInfoKey::Comment(node)),
        ProcessingInstruction(_pi) => span_info.get(SpanInfoKey::PiTarget(node)),
        Root => unreachable!(),
    }
    .map(|span| span.into())
}

pub(crate) trait ChildrenParser<T> {
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<(T, Option<Node>)>;

    fn then<B, O: ChildrenParser<B>>(self, other: O) -> CombinedParser<T, B, Self, O>
    where
        Self: Sized,
    {
        CombinedParser {
            first: self,
            second: other,
            ta: std::marker::PhantomData,
            tb: std::marker::PhantomData,
        }
    }

    fn then_ignore<B, O: ChildrenParser<B>>(
        self,
        other: O,
    ) -> IgnoreRightCombinedParser<T, B, Self, O>
    where
        Self: Sized,
    {
        IgnoreRightCombinedParser {
            first: self,
            second: other,
            ta: std::marker::PhantomData,
            tb: std::marker::PhantomData,
        }
    }
}

pub(crate) struct OptionalChildParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    parse_value: P,
}

impl<V, P> OptionalChildParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    pub(crate) fn new(parse_value: P) -> Self {
        Self { parse_value }
    }
}

impl<V, P> ChildrenParser<Option<V>> for OptionalChildParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<(Option<V>, Option<Node>)> {
        if let Some(node) = node {
            let item = (self.parse_value)(node, &context.data);
            match item {
                Ok(item) => Ok((Some(item), context.next(node))),
                Err(ElementError::Unexpected { .. }) => Ok((None, Some(node))),
                Err(e) => Err(e),
            }
        } else {
            Ok((None, None))
        }
    }
}

pub(crate) struct EndParser;

impl EndParser {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl ChildrenParser<()> for EndParser {
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<((), Option<Node>)> {
        if let Some(node) = node {
            Err(ElementError::Unexpected {
                span: context.span(node).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(((), None))
        }
    }
}

pub(crate) struct ManyChildrenParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    parse_value: P,
}

impl<V, P> ManyChildrenParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    pub(crate) fn new(parse_value: P) -> Self {
        Self { parse_value }
    }
}

impl<V, P> ChildrenParser<Vec<V>> for ManyChildrenParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<(Vec<V>, Option<Node>)> {
        let optional_parser = OptionalChildParser {
            parse_value: &self.parse_value,
        };
        let mut result = Vec::new();
        let mut current_node = node;
        loop {
            let (item, next) = optional_parser.parse(current_node, context)?;
            if let Some(item) = item {
                result.push(item);
                if let Some(next) = next {
                    current_node = Some(next);
                } else {
                    // there are no more siblings
                    return Ok((result, None));
                }
            } else {
                // we couldn't match with another parseable item, so we're done
                return Ok((result, next));
            }
        }
    }
}

pub(crate) struct AtLeastOneParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    parse_value: P,
}

impl<V, P> AtLeastOneParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    pub(crate) fn new(parse_value: P) -> Self {
        Self { parse_value }
    }
}

impl<V, P> ChildrenParser<Vec<V>> for AtLeastOneParser<V, P>
where
    P: Fn(Node, &Data) -> Result<V>,
{
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<(Vec<V>, Option<Node>)> {
        let many_parser = ManyChildrenParser {
            parse_value: &self.parse_value,
        };
        let (items, next) = many_parser.parse(node, context)?;
        if !items.is_empty() {
            Ok((items, next))
        } else if let Some(node) = node {
            Err(ElementError::Unexpected {
                span: context.span(node).ok_or(ElementError::Internal)?,
            })
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

pub(crate) struct CombinedParser<TA, TB, PA: ChildrenParser<TA>, PB: ChildrenParser<TB>> {
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<TA>,
    tb: std::marker::PhantomData<TB>,
}

impl<TA, TB, PA: ChildrenParser<TA>, PB: ChildrenParser<TB>> ChildrenParser<(TA, TB)>
    for CombinedParser<TA, TB, PA, PB>
{
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<((TA, TB), Option<Node>)> {
        let (a, node) = self.first.parse(node, context)?;
        let (b, node) = self.second.parse(node, context)?;
        Ok(((a, b), node))
    }
}

pub(crate) struct IgnoreRightCombinedParser<TA, TB, PA: ChildrenParser<TA>, PB: ChildrenParser<TB>>
{
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<TA>,
    tb: std::marker::PhantomData<TB>,
}

impl<TA, TB, PA: ChildrenParser<TA>, PB: ChildrenParser<TB>> ChildrenParser<TA>
    for IgnoreRightCombinedParser<TA, TB, PA, PB>
{
    fn parse(&self, node: Option<Node>, context: &Context) -> Result<(TA, Option<Node>)> {
        let (a, node) = self.first.parse(node, context)?;
        let (_b, node) = self.second.parse(node, context)?;
        Ok((a, node))
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
    use xot::SpanInfo;
    use xot::Xot;

    use crate::ast_core::Span;

    use super::*;

    fn parse(s: &str) -> (Xot, SpanInfo, Option<Node>) {
        let mut xot = Xot::new();
        let (doc, span_info) = xot.parse_with_span_info(s).unwrap();
        let outer = xot.document_element(doc).unwrap();
        let next = xot.first_child(outer);
        (xot, span_info, next)
    }

    #[test]
    fn test_optional_present() {
        let (xot, span_info, next) = parse("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node, _| Ok(Value));
        let context = Context::new(&xot, &span_info);

        let (item, next) = optional_parser.parse(next, &context).unwrap();

        assert_eq!(item, Some(Value));
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_present_but_parse_error() {
        let (xot, span_info, next) = parse("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node, _| {
            Err(AttributeError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0),
            }
            .into())
        });
        let context = Context::new(&xot, &span_info);

        let r: Result<(Option<Value>, Option<Node>)> = optional_parser.parse(next, &context);

        assert_eq!(
            r,
            Err(AttributeError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0)
            }
            .into())
        );
    }

    #[test]
    fn test_optional_unexpected_node() {
        let (xot, span_info, node) = parse("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node, _| {
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, node).ok_or(ElementError::Internal)?,
            })
        });
        let context = Context::new(&xot, &span_info);

        let (item, next): (Option<Value>, Option<Node>) =
            optional_parser.parse(node, &context).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, node);
    }

    #[test]
    fn test_optional_not_present() {
        let (xot, span_info, next) = parse("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node, _| Ok(Value));
        let context = Context::new(&xot, &span_info);

        let (item, next) = optional_parser.parse(next, &context).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, None);
    }

    #[test]
    fn test_end_found() {
        let (xot, span_info, next) = parse("<outer></outer>");

        let end_parser = EndParser::new();
        let context = Context::new(&xot, &span_info);
        let r = end_parser.parse(next, &context);

        assert!(r.is_ok());
    }

    #[test]
    fn test_end_not_found() {
        let (xot, span_info, next) = parse("<outer><a /></outer>");

        let end_parser = EndParser::new();
        let context = Context::new(&xot, &span_info);
        let r = end_parser.parse(next, &context);

        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
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
        let context = Context::new(xot, span_info);

        let optional_parser_a = OptionalChildParser::new(|node, _| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: context.span(node).ok_or(ElementError::Internal)?,
            })
        });
        let (item_a, next) = optional_parser_a.parse(next, &context).unwrap();

        let optional_parser_b = OptionalChildParser::new(|node, _| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: context.span(node).ok_or(ElementError::Internal)?,
            })
        });
        let (item_b, next) = optional_parser_b.parse(next, &context).unwrap();

        let end_parser = EndParser::new();
        end_parser.parse(next, &context)?;
        Ok((item_a, item_b))
    }

    #[test]
    fn test_two_optional_both_present() {
        let (mut xot, span_info, next) = parse("<outer><a /><b /></outer>");
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_optional_only_a_present() {
        let (mut xot, span_info, next) = parse("<outer><a /></outer>");
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_optional_only_b_present() {
        let (mut xot, span_info, next) = parse("<outer><b /></outer>");
        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_optional_neither_present() {
        let (mut xot, span_info, next) = parse("<outer></outer>");

        let names = Names::new(&mut xot);

        let (item_a, item_b) = parse_two_optional_elements(&names, &xot, &span_info, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_optional_unexpected() {
        let (mut xot, span_info, next) = parse("<outer><c /></outer>");
        let names = Names::new(&mut xot);

        let r = parse_two_optional_elements(&names, &xot, &span_info, next);
        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_many() {
        let (xot, span_info, next) = parse("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(|_node, _| Ok(Value));
        let context = Context::new(&xot, &span_info);

        let (items, next) = many_parser.parse(next, &context).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_many_empty() {
        let (xot, span_info, next) = parse("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(|_node, _| Ok(Value));
        let context = Context::new(&xot, &span_info);

        let (items, next) = many_parser.parse(next, &context).unwrap();

        assert_eq!(items, vec![]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_then_many() {
        let (mut xot, span_info, next) = parse("<outer><a /><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node, _| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, node).ok_or(ElementError::Internal)?,
            })
        });

        let many_parser = ManyChildrenParser::new(|_node, _| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, _node).ok_or(ElementError::Internal)?,
            })
        });
        let context = Context::new(&xot, &span_info);

        let (optional_item, next) = optional_parser.parse(next, &context).unwrap();
        let (many_items, next) = many_parser.parse(next, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine() {
        let (mut xot, span_info, next) = parse("<outer><a /><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node, _| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, node).ok_or(ElementError::Internal)?,
            })
        });

        let many_parser = ManyChildrenParser::new(|_node, _| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, _node).ok_or(ElementError::Internal)?,
            })
        });

        let combined = optional_parser.then(many_parser);

        let context = Context::new(&xot, &span_info);

        let ((optional_item, many_items), next) = combined.parse(next, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_3_values() {
        let (mut xot, span_info, next) = parse("<outer><a /><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node, _| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, node).ok_or(ElementError::Internal)?,
            })
        });

        let many_parser = ManyChildrenParser::new(|_node, _| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, _node).ok_or(ElementError::Internal)?,
            })
        });

        let end_parser = EndParser::new();

        let combined = optional_parser.then(many_parser).then(end_parser);
        let context = Context::new(&xot, &span_info);

        let (((optional_item, many_items), _), next) = combined.parse(next, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_then_ignore() {
        let (mut xot, span_info, next) = parse("<outer><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(|_node, _| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: span_for_node(&xot, &span_info, _node).ok_or(ElementError::Internal)?,
            })
        });

        let end_parser = EndParser::new();

        let combined = many_parser.then_ignore(end_parser);
        let context = Context::new(&xot, &span_info);

        let (many_items, next) = combined.parse(next, &context).unwrap();

        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }
}
