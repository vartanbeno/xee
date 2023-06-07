#[cfg(test)]
mod test {
    use crate::{Atomic, DynamicContext, Namespaces, StaticContext, Value};
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
            wrapper_foo(&context, &[]),
            Ok(Value::Atomic(Atomic::String(Rc::new("foo".to_string()))))
        );
    }

    #[test]
    fn test_arg() {
        let xot = Xot::new();
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        assert_eq!(
            wrapper_int_to_string(&context, &[Value::Atomic(Atomic::Integer(42))]),
            Ok(Value::Atomic(Atomic::String(Rc::new("42".to_string()))))
        );
    }
}
