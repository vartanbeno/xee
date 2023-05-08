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

#[test]
fn test_mul_int() {
    assert_debug_snapshot!(run_without_context("12 * 15"));
}

#[test]
fn test_div_decimal() {
    assert_debug_snapshot!(run_without_context("12 div 3.0"));
}

#[test]
fn test_div_double() {
    assert_debug_snapshot!(run_without_context("12 div 3.0e0"));
}

#[test]
fn test_div_both_integers() {
    // return type is decimal
    assert_debug_snapshot!(run_without_context("12 div 3"));
}

#[test]
fn test_integer_div() {
    assert_debug_snapshot!(run_without_context("12 idiv 5"));
}

#[test]
fn test_mod() {
    assert_debug_snapshot!(run_without_context("12 mod 5"));
}
