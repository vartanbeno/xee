#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::ir::convert_xpath;

    #[test]
    fn test_span_sequence_ir() {
        let expr = "(0, 1, 2)";
        //          012345678
        //          0       8
        //  So from 0, 9 is expected
        // let's examine the IR
        assert_debug_snapshot!(convert_xpath(expr));
    }
}
