use xee_xpath_ast::ast;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SourceSpan(usize, usize);

impl SourceSpan {
    pub(crate) fn entire(src: &str) -> Self {
        Self(0, src.len())
    }

    pub(crate) fn empty() -> Self {
        Self(0, 0)
    }
}

impl From<ast::Span> for SourceSpan {
    fn from(span: ast::Span) -> Self {
        Self(span.start, span.end)
    }
}

impl From<std::ops::Range<usize>> for SourceSpan {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self(range.start, range.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_debug_snapshot;

    use crate::evaluate_without_focus;
    use crate::ir::convert_xpath;
    use crate::{sequence, SpannedResult};

    fn span(result: SpannedResult<sequence::Sequence>) -> SourceSpan {
        result.err().unwrap().span
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
