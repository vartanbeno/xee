use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Spanned<T> {
    pub(crate) value: T,
    pub(crate) span: SourceSpan,
}

impl<T> Spanned<T> {
    pub(crate) fn new(value: T, span: SourceSpan) -> Self {
        Self { value, span }
    }

    pub(crate) fn map(&self, f: impl FnOnce(&T) -> T) -> Self {
        Self {
            value: f(&self.value),
            span: self.span,
        }
    }
}

// pub(crate) type Spanned<T> = (T, SourceSpan);

pub(crate) fn not_spanned<T>(value: T) -> Spanned<T> {
    Spanned {
        value,
        span: (0, 0).into(),
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::ast::parse_xpath;
    use crate::ir::convert_xpath;
    use crate::name::Namespaces;
    use crate::static_context::StaticContext;

    #[test]
    fn test_span_sequence_ast() {
        let expr = "(0, 1, 2)";
        //          012345678
        //          0       8
        //  So from 0, 9 is expected
        // let's examine the AST
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        assert_debug_snapshot!(parse_xpath(expr, &static_context));
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

    // #[test]
    // fn span_too_long_problem_ast() {
    //     let expr = "0 + (2, 3) + 3";
    //     //          01234567890123
    //     //          0           12
    //     //  So from 0, 13 is expected
    //     let namespaces = Namespaces::new(None, None);
    //     assert_debug_snapshot!(parse_xpath(expr, &namespaces));
    // }

    // #[test]
    // fn span_too_long_problem_ir() {
    //     let expr = "0 + (2, 3) + 3";
    //     //          01234567890123
    //     //          0           12
    //     //  So from 0, 13 is expected
    //     // let's examine the IR
    //     assert_debug_snapshot!(convert_xpath(expr));
    // }
}
