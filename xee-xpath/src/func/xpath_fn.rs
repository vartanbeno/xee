#[cfg(test)]
mod test {
    use crate::data::{Atomic, StackValue};
    use crate::{DynamicContext, Namespaces, StaticContext};
    use std::rc::Rc;
    use xee_xpath_macros::xpath_fn;
    use xot::Xot;

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
        assert_eq!(
            foo::WRAPPER(&context, &[]),
            Ok(StackValue::Atomic(Atomic::String(Rc::new(
                "foo".to_string()
            ))))
        );
    }

    #[test]
    fn test_arg() {
        let xot = Xot::new();
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        assert_eq!(
            int_to_string::WRAPPER(&context, &[StackValue::Atomic(Atomic::Integer(42))]),
            Ok(StackValue::Atomic(Atomic::String(Rc::new(
                "42".to_string()
            ))))
        );
    }
}
