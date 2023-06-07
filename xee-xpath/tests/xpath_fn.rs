use xee_xpath_macros::xpath_fn;

#[xpath_fn("fn:foo() as xs:string")]
fn foo() -> String {
    "foo".to_string()
}

#[test]
fn test_simple() {
    assert_eq!(foo(), "foo");
}
