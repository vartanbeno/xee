use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub value: T,
    pub span: SourceSpan,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: SourceSpan) -> Self {
        Self { value, span }
    }

    pub fn map(&self, f: impl FnOnce(&T) -> T) -> Self {
        Self {
            value: f(&self.value),
            span: self.span,
        }
    }
}

pub fn not_spanned<T>(value: T) -> Spanned<T> {
    Spanned {
        value,
        span: (0, 0).into(),
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::ast::parse_xpath;
    use crate::namespaces::Namespaces;

    #[test]
    fn test_span_sequence_ast() {
        let expr = "(0, 1, 2)";
        //          012345678
        //          0       8
        //  So from 0, 9 is expected
        // let's examine the AST
        let namespaces = Namespaces::new(None, None);
        assert_debug_snapshot!(parse_xpath(expr, &namespaces, &[]));
    }
}
