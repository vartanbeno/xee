#[cfg(test)]
mod test {

    use xee_xpath_macros::xpath_fn;
    use xot::Xot;

    use crate::atomic;
    use crate::sequence;
    use crate::{DynamicContext, Namespaces, StaticContext};

    #[xpath_fn("fn:foo() as xs:string")]
    fn foo() -> String {
        "foo".to_string()
    }

    #[xpath_fn("fn:int_to_string($x as xs:integer) as xs:string")]
    fn int_to_string(x: i64) -> String {
        x.to_string()
    }

    #[test]
    fn test_simple() {
        let xot = Xot::new();
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        let expected =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from("foo"))]);
        assert_eq!(foo::WRAPPER(&context, &[]), Ok(expected));
    }

    #[test]
    fn test_arg() {
        let xot = Xot::new();
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        let expected =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from("42"))]);
        assert_eq!(
            int_to_string::WRAPPER(
                &context,
                &[sequence::Sequence::from(vec![sequence::Item::from(
                    atomic::Atomic::from(42i64)
                )])]
            ),
            Ok(expected)
        );
    }
}
