use insta::assert_debug_snapshot;

use xee_xpath::run_without_context;

#[test]
fn test_right_left_side() {
    let expr = "0 + 12 + ((2, 3, 4) + 1)";
    //          012345678901234567890123
    //                    10           23
    //  So from 10, 13 is expected
    // we can't atomize the sequence which happens because of addition
    assert_debug_snapshot!(run_without_context(expr));
}

#[test]
fn test_right_right_side() {
    let expr = "0 + 12 + (1 + (2, 3, 4))";
    //          012345678901234567890123
    //                    10           23
    //  So from 10, 13 is expected
    // we can't atomize the sequence which happens because of addition
    assert_debug_snapshot!(run_without_context(expr));
}
