use xot::Node;

use crate::error::Error as InvalidError;

#[derive(Debug, PartialEq)]
enum Error {
    Unexpected,
    Invalid(InvalidError),
}

impl From<InvalidError> for Error {
    fn from(error: InvalidError) -> Self {
        Self::Invalid(error)
    }
}

type Result<T> = std::result::Result<T, Error>;

trait State {
    fn next(&self, node: Node) -> Option<Node>;
}

trait ChildrenParser<T, S: State> {
    fn parse(&self, node: Option<Node>, state: &S) -> Result<(T, Option<Node>)>;

    fn then<B, O: ChildrenParser<B, S>>(self, other: O) -> CombinedParser<T, B, S, Self, O>
    where
        Self: Sized,
    {
        CombinedParser {
            first: self,
            second: other,
            ta: std::marker::PhantomData,
            tb: std::marker::PhantomData,
            s: std::marker::PhantomData,
        }
    }
}

struct OptionalChildParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    parse_value: P,
}

impl<V, P> OptionalChildParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn new(parse_value: P) -> Self {
        Self { parse_value }
    }
}

impl<V, P, S: State> ChildrenParser<Option<V>, S> for OptionalChildParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn parse(&self, node: Option<Node>, state: &S) -> Result<(Option<V>, Option<Node>)> {
        if let Some(node) = node {
            let item = (self.parse_value)(node);
            match item {
                Ok(item) => Ok((Some(item), state.next(node))),
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

impl<S: State> ChildrenParser<(), S> for EndParser {
    fn parse(&self, node: Option<Node>, _state: &S) -> Result<((), Option<Node>)> {
        if let Some(_node) = node {
            Err(Error::Unexpected)
        } else {
            Ok(((), None))
        }
    }
}

struct ManyChildrenParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    parse_value: P,
}

impl<V, P> ManyChildrenParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn new(parse_value: P) -> Self {
        Self { parse_value }
    }
}

impl<V, P, S: State> ChildrenParser<Vec<V>, S> for ManyChildrenParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn parse(&self, node: Option<Node>, state: &S) -> Result<(Vec<V>, Option<Node>)> {
        let optional_parser = OptionalChildParser {
            parse_value: &self.parse_value,
        };
        let mut result = Vec::new();
        let mut current_node = node;
        loop {
            let (item, next) = optional_parser.parse(current_node, state)?;
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

struct AtLeastOneParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    parse_value: P,
}

impl<V, P> AtLeastOneParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn new(parse_value: P) -> Self {
        Self { parse_value }
    }
}

impl<V, P, S: State> ChildrenParser<Vec<V>, S> for AtLeastOneParser<V, P>
where
    P: Fn(Node) -> Result<V>,
{
    fn parse(&self, node: Option<Node>, state: &S) -> Result<(Vec<V>, Option<Node>)> {
        let many_parser = ManyChildrenParser {
            parse_value: &self.parse_value,
        };
        let (items, next) = many_parser.parse(node, state)?;
        if !items.is_empty() {
            Ok((items, next))
        } else {
            Err(Error::Unexpected)
        }
    }
}

struct CombinedParser<TA, TB, S: State, PA: ChildrenParser<TA, S>, PB: ChildrenParser<TB, S>> {
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<TA>,
    tb: std::marker::PhantomData<TB>,
    s: std::marker::PhantomData<S>,
}

impl<TA, TB, S: State, PA: ChildrenParser<TA, S>, PB: ChildrenParser<TB, S>>
    ChildrenParser<(TA, TB), S> for CombinedParser<TA, TB, S, PA, PB>
{
    fn parse(&self, node: Option<Node>, state: &S) -> Result<((TA, TB), Option<Node>)> {
        let (a, node) = self.first.parse(node, state)?;
        let (b, node) = self.second.parse(node, state)?;
        Ok(((a, b), node))
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

    struct NextState<'a> {
        xot: &'a Xot,
    }

    impl<'a> NextState<'a> {
        fn new(xot: &'a Xot) -> Self {
            Self { xot }
        }
    }

    impl<'a> State for NextState<'a> {
        fn next(&self, node: Node) -> Option<Node> {
            self.xot.next_sibling(node)
        }
    }

    #[test]
    fn test_optional_present() {
        let (xot, _span_info, next) = parse("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node| Ok(Value));
        let state = NextState::new(&xot);

        let (item, next) = optional_parser.parse(next, &state).unwrap();

        assert_eq!(item, Some(Value));
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_present_but_parse_error() {
        let (xot, _span_info, next) = parse("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node| {
            Err(InvalidError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0),
            }
            .into())
        });
        let state = NextState::new(&xot);

        let r: Result<(Option<Value>, Option<Node>)> = optional_parser.parse(next, &state);

        assert_eq!(
            r,
            Err(InvalidError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0)
            }
            .into())
        );
    }

    #[test]
    fn test_optional_unexpected_node() {
        let (xot, _span_info, node) = parse("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node| Err(Error::Unexpected));
        let state = NextState::new(&xot);

        let (item, next): (Option<Value>, Option<Node>) =
            optional_parser.parse(node, &state).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, node);
    }

    #[test]
    fn test_optional_not_present() {
        let (xot, _span_info, next) = parse("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|_node| Ok(Value));
        let state = NextState::new(&xot);

        let (item, next) = optional_parser.parse(next, &state).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, None);
    }

    #[test]
    fn test_end_found() {
        let (_xot, _span_info, next) = parse("<outer></outer>");

        let end_parser = EndParser::new();
        let state = NextState::new(&_xot);
        let r = end_parser.parse(next, &state);

        assert!(r.is_ok());
    }

    #[test]
    fn test_end_not_found() {
        let (_xot, _span_info, next) = parse("<outer><a /></outer>");

        let end_parser = EndParser::new();
        let state = NextState::new(&_xot);
        let r = end_parser.parse(next, &state);

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
        _span_info: &SpanInfo,
        next: Option<Node>,
    ) -> Result<(Option<ValueA>, Option<ValueB>)> {
        let state = NextState::new(xot);

        let optional_parser_a = OptionalChildParser::new(|node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(Error::Unexpected)
        });
        let (item_a, next) = optional_parser_a.parse(next, &state).unwrap();

        let optional_parser_b = OptionalChildParser::new(|node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(Error::Unexpected)
        });
        let (item_b, next) = optional_parser_b.parse(next, &state).unwrap();

        let end_parser = EndParser::new();
        end_parser.parse(next, &state)?;
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
        assert_eq!(r, Err(Error::Unexpected));
    }

    #[test]
    fn test_many() {
        let (xot, _span_info, next) = parse("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(|_node| Ok(Value));
        let state = NextState::new(&xot);

        let (items, next) = many_parser.parse(next, &state).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_many_empty() {
        let (xot, _span_info, next) = parse("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = ManyChildrenParser::new(|_node| Ok(Value));
        let state = NextState::new(&xot);

        let (items, next) = many_parser.parse(next, &state).unwrap();

        assert_eq!(items, vec![]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_optional_then_many() {
        let (mut xot, _span_info, next) = parse("<outer><a /><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(Error::Unexpected)
        });

        let many_parser = ManyChildrenParser::new(|_node| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(Error::Unexpected)
        });
        let state = NextState::new(&xot);

        let (optional_item, next) = optional_parser.parse(next, &state).unwrap();
        let (many_items, next) = many_parser.parse(next, &state).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine() {
        let (mut xot, _span_info, next) = parse("<outer><a /><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(Error::Unexpected)
        });

        let many_parser = ManyChildrenParser::new(|_node| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(Error::Unexpected)
        });

        let combined = optional_parser.then(many_parser);

        let state = NextState::new(&xot);

        let ((optional_item, many_items), next) = combined.parse(next, &state).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_3_values() {
        let (mut xot, _span_info, next) = parse("<outer><a /><b /><b /></outer>");

        let names = Names::new(&mut xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = OptionalChildParser::new(|node| {
            if let Some(element) = xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(Error::Unexpected)
        });

        let many_parser = ManyChildrenParser::new(|_node| {
            if let Some(element) = xot.element(_node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(Error::Unexpected)
        });

        let end_parser = EndParser::new();

        let combined = optional_parser.then(many_parser).then(end_parser);
        let state = NextState::new(&xot);

        let (((optional_item, many_items), _), next) = combined.parse(next, &state).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }
}
