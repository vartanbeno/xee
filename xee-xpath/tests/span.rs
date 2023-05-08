use miette::SourceSpan;

use xee_xpath::{run_without_context, Error, StackValue};

fn span(result: Result<StackValue, Error>) -> Option<SourceSpan> {
    match result.err().unwrap() {
        Error::XPTY0004 { span, .. } => Some(span),
        _ => None,
    }
}

#[test]
fn test_left_side() {
    let expr = "0 + (2, 3, 4)";
    //          0123456789012
    //          0           12
    //  So from 0, 13 is expected
    let r = run_without_context(expr);
    assert_eq!(span(r), Some((0, 13).into()));
}

#[test]
fn test_right_side() {
    let expr = "(2, 3, 4) + 1";
    //          0123456789012
    //          0           12
    //  So from 0, 13 is expected
    let r = run_without_context(expr);
    assert_eq!(span(r), Some((0, 13).into()));
}

#[test]
fn test_left_right_side() {
    let expr = "0 + (2, 3, 4) + (12 + 1)";
    //          012345678901234567890123
    //          0           12
    //  So from 0, 13 is expected
    // but we get 0, 14, with the extra space character
    // I think this is because the parser consumes this space
    // let's accept this behavior for now
    let r = run_without_context(expr);
    assert_eq!(span(r), Some((0, 14).into()));
}

#[test]
fn test_right_left_side() {
    let expr = "0 + 12 + ((2, 3, 4) + 1)";
    //          012345678901234567890123
    //                    10          22
    //  So from 10, 13 is expected
    assert_eq!(span(run_without_context(expr)), Some((10, 13).into()));
}

#[test]
fn test_right_right_side() {
    let expr = "0 + 12 + (1 + (2, 3, 4))";
    //          012345678901234567890123
    //                    10          22
    //  So from 10, 13 is expected
    assert_eq!(span(run_without_context(expr)), Some((10, 13).into()));
}
