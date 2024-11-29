use insta::assert_debug_snapshot;
use xee_xpath::{error, Documents, Queries, Query, Sequence};

fn run(s: &str) -> error::Result<Sequence> {
    let mut documents = Documents::new();
    let queries = Queries::default();
    let q = queries.sequence(s)?;
    q.execute_build_context(&mut documents, |_builder| ())
}

#[test]
fn test_add_int_to_double() {
    assert_debug_snapshot!(run("12 + 15.4e0"));
}

#[test]
fn test_add_int_to_decimal() {
    assert_debug_snapshot!(run("12 + 15.4"));
}

#[test]
fn test_mul_int() {
    assert_debug_snapshot!(run("12 * 15"));
}

#[test]
fn test_div_decimal() {
    assert_debug_snapshot!(run("12 div 3.0"));
}

#[test]
fn test_div_double() {
    assert_debug_snapshot!(run("12 div 3.0e0"));
}

#[test]
fn test_div_both_integers() {
    // return type is decimal
    assert_debug_snapshot!(run("12 div 3"));
}

#[test]
fn test_integer_div() {
    assert_debug_snapshot!(run("12 idiv 5"));
}

#[test]
fn test_mod() {
    assert_debug_snapshot!(run("12 mod 5"));
}
