#[cfg(test)]
mod test {
    use xee_xpath_macros::xpath_fn;
    use xot::Xot;

    use crate::atomic;
    use crate::interpreter;
    use crate::sequence;
    use crate::{DynamicContext, StaticContext};

    #[xpath_fn("fn:foo() as xs:string")]
    fn foo() -> String {
        "foo".to_string()
    }

    #[xpath_fn("fn:int_to_string($x as xs:long) as xs:string")]
    fn int_to_string(x: i64) -> String {
        x.to_string()
    }

    #[test]
    fn test_simple() {
        let xot = Xot::new();
        let static_context = StaticContext::default();
        let context = DynamicContext::new(&xot, &static_context);
        let expected =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from("foo"))]);
        let program = interpreter::Program::empty("".to_string());
        let runnable = interpreter::Runnable::new(&program, &context);
        let mut interpreter = interpreter::Interpreter::new(&runnable);
        assert_eq!(foo::WRAPPER(&context, &mut interpreter, &[]), Ok(expected));
    }

    #[test]
    fn test_arg() {
        let xot = Xot::new();
        let static_context = StaticContext::default();
        let context = DynamicContext::new(&xot, &static_context);
        let expected =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from("42"))]);
        let program = interpreter::Program::empty("".to_string());
        let runnable = interpreter::Runnable::new(&program, &context);
        let mut interpreter = interpreter::Interpreter::new(&runnable);
        assert_eq!(
            int_to_string::WRAPPER(
                &context,
                &mut interpreter,
                &[sequence::Sequence::from(vec![sequence::Item::from(
                    atomic::Atomic::from(42i64)
                )])]
            ),
            Ok(expected)
        );
    }
}
