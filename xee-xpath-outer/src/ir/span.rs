#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use xee_xpath::{context, error, sequence, span::SourceSpan};

    use crate::evaluate_without_focus;
    use crate::interpreter::convert_ir;

    fn span(result: error::SpannedResult<sequence::Sequence>) -> SourceSpan {
        result.err().unwrap().span
    }

    #[test]
    fn test_span_sequence_ir() {
        let expr = "(0, 1, 2)";
        //          012345678
        //          0       8
        //  So from 0, 9 is expected
        // lets examine the IR
        let static_context = context::StaticContext::default();
        assert_debug_snapshot!(convert_ir(&static_context, expr));
    }

    #[test]
    fn test_left_side() {
        let expr = "0 + (2, 3, 4)";
        //          0123456789012
        //          0           12
        //  So from 0, 13 is expected
        let r = evaluate_without_focus(expr);
        assert_eq!(span(r), (0..13).into());
    }

    #[test]
    fn test_right_side() {
        let expr = "(2, 3, 4) + 1";
        //          0123456789012
        //          0           12
        //  So from 0, 13 is expected
        let r = evaluate_without_focus(expr);
        assert_eq!(span(r), (0..13).into());
    }

    #[test]
    fn test_left_right_side() {
        let expr = "0 + (2, 3, 4) + (12 + 1)";
        //          012345678901234567890123
        //          0           12
        //  So from 0, 13 is expected
        let r = evaluate_without_focus(expr);
        assert_eq!(span(r), (0..13).into());
    }

    #[test]
    fn test_right_left_side() {
        let expr = "0 + 12 + ((2, 3, 4) + 1)";
        //          012345678901234567890123
        //                    10          22
        assert_eq!(span(evaluate_without_focus(expr)), (10..23).into());
    }

    #[test]
    fn test_right_right_side() {
        let expr = "0 + 12 + (1 + (2, 3, 4))";
        //          012345678901234567890123
        //                    10          22
        assert_eq!(span(evaluate_without_focus(expr)), (10..23).into());
    }
}
