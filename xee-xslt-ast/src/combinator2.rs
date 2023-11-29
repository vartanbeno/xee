use xot::Node;

use crate::ast_core::Span;
use crate::context::Context;
// use crate::error::Error as AttributeError;
use crate::element::AttributeError;
use crate::state::State;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

pub(crate) trait NodeParser<V> {
    fn parse(&self, node: Option<Node>, state: &State, context: &Context) -> Result<V> {
        let (item, next) = self.parse_next(node, state, context)?;
        if let Some(next) = next {
            // we shouldn't have any next item at this point
            Err(ElementError::Unexpected {
                span: state.span(next).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(item)
        }
    }

    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(V, Option<Node>)>;

    fn option(self) -> OptionParser<V, Self>
    where
        Self: Sized,
    {
        OptionParser {
            parser: self,
            v: std::marker::PhantomData,
        }
    }

    fn many(self) -> ManyParser<V, Self>
    where
        Self: Sized,
    {
        ManyParser {
            parser: self,
            v: std::marker::PhantomData,
        }
    }

    fn one_or_more(self) -> OneOrMoreParser<V, Self>
    where
        Self: Sized,
    {
        OneOrMoreParser {
            parser: self,
            v: std::marker::PhantomData,
        }
    }

    fn contains(self) -> ContainsParser<V, Self>
    where
        Self: Sized,
    {
        ContainsParser {
            parser: self,
            t: std::marker::PhantomData,
        }
    }

    fn then<B, O: NodeParser<B>>(self, other: O) -> CombinedParser<V, B, Self, O>
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

    fn then_ignore<B, O: NodeParser<B>>(self, other: O) -> IgnoreRightCombinedParser<V, B, Self, O>
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

    fn or<O: NodeParser<V>>(self, other: O) -> OrParser<V, Self, O>
    where
        Self: Sized,
    {
        OrParser {
            first: self,
            second: other,
            t: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OneParser<V, P>
where
    P: Fn(Node, &State, &Context) -> Result<V>,
{
    parse_value: P,
}

pub(crate) fn one<V, P>(parse_value: P) -> OneParser<V, P>
where
    P: Fn(Node, &State, &Context) -> Result<V>,
{
    OneParser { parse_value }
}

impl<V, P> NodeParser<V> for OneParser<V, P>
where
    P: Fn(Node, &State, &Context) -> Result<V>,
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(V, Option<Node>)> {
        if let Some(node) = node {
            let item = (self.parse_value)(node, state, context)?;
            Ok((item, state.next(node)))
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OptionParser<V, P>
where
    P: NodeParser<V>,
{
    parser: P,
    v: std::marker::PhantomData<V>,
}

impl<V, P> NodeParser<Option<V>> for OptionParser<V, P>
where
    P: NodeParser<V>,
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(Option<V>, Option<Node>)> {
        match self.parser.parse_next(node, state, context) {
            Ok((item, next)) => Ok((Some(item), next)),
            Err(ElementError::Unexpected { .. }) => Ok((None, node)),
            Err(ElementError::UnexpectedEnd) => Ok((None, None)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ManyParser<V, P>
where
    P: NodeParser<V>,
{
    parser: P,
    v: std::marker::PhantomData<V>,
}

impl<V, P> NodeParser<Vec<V>> for ManyParser<V, P>
where
    P: NodeParser<V>,
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(Vec<V>, Option<Node>)> {
        let mut result = Vec::new();
        let mut current_node = node;
        loop {
            match self.parser.parse_next(current_node, state, context) {
                Ok((item, next)) => {
                    result.push(item);
                    current_node = next;
                }
                Err(ElementError::UnexpectedEnd) => {
                    return Ok((result, None));
                }
                Err(ElementError::Unexpected { .. }) => {
                    return Ok((result, current_node));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OneOrMoreParser<V, P>
where
    P: NodeParser<V>,
{
    parser: P,
    v: std::marker::PhantomData<V>,
}

impl<V, P> NodeParser<Vec<V>> for OneOrMoreParser<V, P>
where
    P: NodeParser<V> + Copy,
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(Vec<V>, Option<Node>)> {
        let many_parser = ManyParser {
            parser: self.parser,
            v: std::marker::PhantomData,
        };
        let (items, next) = many_parser.parse_next(node, state, context)?;
        if !items.is_empty() {
            Ok((items, next))
        } else if let Some(node) = node {
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

pub(crate) struct EndParser;

impl EndParser {
    pub(crate) fn new() -> Self {
        Self
    }
}

pub(crate) fn end() -> EndParser {
    EndParser::new()
}

impl NodeParser<()> for EndParser {
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        _context: &Context,
    ) -> Result<((), Option<Node>)> {
        if let Some(node) = node {
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(((), None))
        }
    }
}

pub(crate) struct CombinedParser<VA, VB, PA: NodeParser<VA>, PB: NodeParser<VB>> {
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<VA>,
    tb: std::marker::PhantomData<VB>,
}

impl<VA, VB, PA: NodeParser<VA>, PB: NodeParser<VB>> NodeParser<(VA, VB)>
    for CombinedParser<VA, VB, PA, PB>
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<((VA, VB), Option<Node>)> {
        let (a, node) = self.first.parse_next(node, state, context)?;
        let (b, node) = self.second.parse_next(node, state, context)?;
        Ok(((a, b), node))
    }
}

pub(crate) struct IgnoreRightCombinedParser<VA, VB, PA: NodeParser<VA>, PB: NodeParser<VB>> {
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<VA>,
    tb: std::marker::PhantomData<VB>,
}

impl<VA, VB, PA: NodeParser<VA>, PB: NodeParser<VB>> NodeParser<VA>
    for IgnoreRightCombinedParser<VA, VB, PA, PB>
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(VA, Option<Node>)> {
        let (a, node) = self.first.parse_next(node, state, context)?;
        let (_b, node) = self.second.parse_next(node, state, context)?;
        Ok((a, node))
    }
}

pub(crate) struct OrParser<V, PA: NodeParser<V>, PB: NodeParser<V>> {
    first: PA,
    second: PB,
    t: std::marker::PhantomData<V>,
}

impl<V, PA: NodeParser<V>, PB: NodeParser<V>> NodeParser<V> for OrParser<V, PA, PB> {
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(V, Option<Node>)> {
        // try the first parser, if that works, return result
        // if it isn't working, try the other parser
        let r = self.first.parse_next(node, state, context);
        if r.is_ok() {
            r
        } else {
            self.second.parse_next(node, state, context)
        }
    }
}

pub(crate) struct ContainsParser<V, P>
where
    P: NodeParser<V>,
{
    parser: P,
    t: std::marker::PhantomData<V>,
}

impl<V, P> NodeParser<V> for ContainsParser<V, P>
where
    P: NodeParser<V>,
{
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(V, Option<Node>)> {
        if let Some(node) = node {
            self.parser
                .parse_next(state.xot.first_child(node), state, context)
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

// pub(crate) struct TopParser<V, P>
// where
//     P: Fn(Node, &State, &Context) -> Result<V>,
// {
//     parse_value: P,
// }

// impl<T, P> NodeParser<T> for TopParser<T, P>
// where
//     P: Fn(Node, &State, &Context) -> Result<T>,
// {
//     fn parse_next(
//         &self,
//         node: Option<Node>,
//         state: &State,
//         context: &Context,
//     ) -> Result<(T, Option<Node>)> {
//         let node = node.ok_or(ElementError::UnexpectedEnd)?;

//         (self.parse_value)(node, state, context).map(|item| (item, None))
//     }
// }

// pub(crate) fn top<V, P>(parse_value: P) -> TopParser<V, P>
// where
//     P: Fn(Node, &State, &Context) -> Result<V>,
// {
//     TopParser { parse_value }
// }

#[cfg(test)]
mod tests {
    use xot::NameId;
    use xot::Xot;

    use crate::ast_core::Span;
    use crate::names::Names;

    use super::*;

    fn parse_base(s: &str) -> (State, Context, Node) {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let (doc, span_info) = xot.parse_with_span_info(s).unwrap();
        let outer = xot.document_element(doc).unwrap();
        let state = State::new(xot, span_info, names);
        let element = state.xot.element(outer).unwrap();
        let context = Context::new(element);
        (state, context, outer)
    }

    fn parse_next(s: &str) -> (State, Context, Option<Node>) {
        let (state, context, outer) = parse_base(s);
        let next = state.xot.first_child(outer);
        (state, context, next)
    }

    #[test]
    fn test_one() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|_node, _, _| Ok(Value));

        let (item, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(item, Value);
        assert_eq!(next, None);
    }

    #[test]
    fn test_one_no_node() {
        let (state, context, next) = parse_next("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|_node, _, _| Ok(Value));

        let r = parser.parse_next(next, &state, &context);
        assert_eq!(r, Err(ElementError::UnexpectedEnd));
    }

    #[test]
    fn test_one_wrong_node() {
        let (mut state, context, next) = parse_next("<outer><b/></outer>");

        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|node, _, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(Value);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        });

        let r = parser.parse_next(next, &state, &context);
        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_option_present() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|_node, _, _| Ok(Value)).option();

        let (item, next) = option_parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(item, Some(Value));
        assert_eq!(next, None);
    }

    #[test]
    fn test_option_present_but_parse_error() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|_node, _, _| {
            Err(AttributeError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0),
            }
            .into())
        })
        .option();

        let r: Result<(Option<Value>, Option<Node>)> =
            option_parser.parse_next(next, &state, &context);

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
    fn test_option_unexpected_node() {
        let (state, context, node) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = one(|node, state, _| {
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item, next): (Option<Value>, Option<Node>) =
            optional_parser.parse_next(node, &state, &context).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, node);
    }

    #[test]
    fn test_option_not_present() {
        let (state, context, next) = parse_next("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = one(|_node, _, _| Ok(Value)).option();

        let (item, next) = optional_parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, None);
    }

    #[derive(Debug, PartialEq)]
    struct ValueA;
    #[derive(Debug, PartialEq)]
    struct ValueB;

    struct TestNames {
        name_a: NameId,
        name_b: NameId,
        foo: NameId,
    }

    impl TestNames {
        fn new(xot: &mut Xot) -> Self {
            Self {
                name_a: xot.add_name("a"),
                name_b: xot.add_name("b"),
                foo: xot.add_name("foo"),
            }
        }
    }

    fn parse_two_option(
        state: &State,
        context: &Context,
        names: &TestNames,
        next: Option<Node>,
    ) -> Result<(Option<ValueA>, Option<ValueB>)> {
        let optional_parser_a = one(|node, _, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item_a, next) = optional_parser_a.parse_next(next, state, context).unwrap();

        let optional_parser_b = one(|node, _, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item_b, next) = optional_parser_b.parse_next(next, state, context).unwrap();

        let end_parser = EndParser::new();
        end_parser.parse_next(next, state, context)?;
        Ok((item_a, item_b))
    }

    #[test]
    fn test_two_option_both_present() {
        let (mut state, context, next) = parse_next("<outer><a /><b /></outer>");
        let names = TestNames::new(&mut state.xot);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_option_only_a_present() {
        let (mut state, context, next) = parse_next("<outer><a /></outer>");
        let names = TestNames::new(&mut state.xot);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_option_only_b_present() {
        let (mut state, context, next) = parse_next("<outer><b /></outer>");
        let names = TestNames::new(&mut state.xot);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_option_neither_present() {
        let (mut state, context, next) = parse_next("<outer></outer>");

        let names = TestNames::new(&mut state.xot);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_option_unexpected() {
        let (mut state, context, next) = parse_next("<outer><c /></outer>");
        let names = TestNames::new(&mut state.xot);

        let r = parse_two_option(&state, &context, &names, next);
        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_end_found() {
        let (state, context, next) = parse_next("<outer></outer>");

        let end_parser = end();

        let r = end_parser.parse_next(next, &state, &context);

        assert!(r.is_ok());
    }

    #[test]
    fn test_end_not_found() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        let end_parser = end();

        let r = end_parser.parse_next(next, &state, &context);

        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_many() {
        let (state, context, next) = parse_next("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = one(|_node, _, _| Ok(Value)).many();

        let (items, next) = many_parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_many_empty() {
        let (state, context, next) = parse_next("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = one(|_node, _, _| Ok(Value)).many();

        let (items, next) = many_parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(items, vec![]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_option_then_many() {
        let (mut state, context, next) = parse_next("<outer><a /><b /><b /></outer>");

        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let many_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let (optional_item, next) = option_parser.parse_next(next, &state, &context).unwrap();
        let (many_items, next) = many_parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine() {
        let (mut state, context, next) = parse_next("<outer><a /><b /><b /></outer>");

        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let many_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let combined = option_parser.then(many_parser);

        let ((optional_item, many_items), next) =
            combined.parse_next(next, &state, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_3_values() {
        let (mut state, context, next) = parse_next("<outer><a /><b /><b /></outer>");

        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let many_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let end_parser = end();

        let combined = option_parser.then(many_parser).then(end_parser);

        let (((option_item, many_items), _), next) =
            combined.parse_next(next, &state, &context).unwrap();

        assert_eq!(option_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_then_ignore() {
        let (mut state, context, next) = parse_next("<outer><b /><b /></outer>");

        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let end_parser = end();

        let combined = many_parser.then_ignore(end_parser);

        let (many_items, next) = combined.parse_next(next, &state, &context).unwrap();

        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_attribute() {
        let (mut state, context, next) = parse_next(r#"<outer><b foo="FOO"/></outer>"#);

        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        struct Value {
            foo: String,
        }

        let parser = one(|node, state, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    let value = element.get_attribute(names.foo).unwrap();
                    return Ok(Value {
                        foo: value.to_string(),
                    });
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item, _next) = parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(
            item,
            Some(Value {
                foo: "FOO".to_string()
            })
        );
    }

    #[test]
    fn test_contains() {
        let (state, context, outer) = parse_base("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|_node, _, _| Ok(Value)).many().contains();

        let (items, next) = parser.parse_next(Some(outer), &state, &context).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_or() {
        let (mut state, context, next) = parse_next("<outer><a/><b/></outer>");
        let names = TestNames::new(&mut state.xot);

        #[derive(Debug, PartialEq)]
        enum AnyValue {
            A(ValueA),
            B(ValueB),
        }

        let parser_a = one(|node, _, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_a {
                    return Ok(AnyValue::A(ValueA));
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        });

        let parser_b = one(|node, _, _| {
            if let Some(element) = state.xot.element(node) {
                if element.name() == names.name_b {
                    return Ok(AnyValue::B(ValueB));
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        });

        let parser = parser_a.or(parser_b).many();

        let (items, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], AnyValue::A(ValueA));
        assert_eq!(items[1], AnyValue::B(ValueB));
        assert_eq!(next, None);
    }
}
