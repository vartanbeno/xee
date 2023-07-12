use xee_xpath_ast::ast::Span;

pub(crate) fn to_miette(span: Span) -> miette::SourceSpan {
    (span.start, span.end - span.start).into()
}

pub(crate) fn to_ast(span: miette::SourceSpan) -> Span {
    let span_range = span.offset()..(span.offset() + span.len());
    span_range.into()
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use miette::SourceSpan;

    use crate::ir::convert_xpath;
    use crate::sequence;
    use crate::{evaluate_without_focus, Error};

    fn span(result: Result<sequence::Sequence, Error>) -> Option<SourceSpan> {
        match result.err().unwrap() {
            Error::SpannedType { span, .. } => Some(span),
            _ => None,
        }
    }

    #[test]
    fn test_span_sequence_ir() {
        let expr = "(0, 1, 2)";
        //          012345678
        //          0       8
        //  So from 0, 9 is expected
        // let's examine the IR
        assert_debug_snapshot!(convert_xpath(expr));
    }

    #[test]
    fn test_left_side() {
        let expr = "0 + (2, 3, 4)";
        //          0123456789012
        //          0           12
        //  So from 0, 13 is expected
        let r = evaluate_without_focus(expr);
        assert_eq!(span(r), Some((0, 13).into()));
    }

    #[test]
    fn test_right_side() {
        let expr = "(2, 3, 4) + 1";
        //          0123456789012
        //          0           12
        //  So from 0, 13 is expected
        let r = evaluate_without_focus(expr);
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
        let r = evaluate_without_focus(expr);
        assert_eq!(span(r), Some((0, 13).into()));
    }

    #[test]
    fn test_right_left_side() {
        let expr = "0 + 12 + ((2, 3, 4) + 1)";
        //          012345678901234567890123
        //                    10          22
        //  So from 10, 13 is expected
        assert_eq!(span(evaluate_without_focus(expr)), Some((10, 13).into()));
    }

    #[test]
    fn test_right_right_side() {
        let expr = "0 + 12 + (1 + (2, 3, 4))";
        //          012345678901234567890123
        //                    10          22
        //  So from 10, 13 is expected
        assert_eq!(span(evaluate_without_focus(expr)), Some((10, 13).into()));
    }
}
