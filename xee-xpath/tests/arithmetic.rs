use insta::assert_debug_snapshot;

use xee_xpath::run_without_context;

#[test]
fn test_add_int_to_double() {
    assert_debug_snapshot!(run_without_context("12 + 15.4e0"));
}

#[test]
fn test_add_int_to_decimal() {
    assert_debug_snapshot!(run_without_context("12 + 15.4"));
}
