use std::rc::Rc;
use xot::Xot;

use xee_xpath::{Atomic, DynamicContext, Namespaces, StaticContext, Value};

use xee_xpath_macros::xpath_fn;

#[xpath_fn("fn:foo() as xs:string")]
fn foo() -> String {
    "foo".to_string()
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
