use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
pub enum Error {
    /// Parse error.
    ///
    /// It is a static error if an expression is not a valid instance of the
    /// grammar defined in A.1 EBNF.
    #[error("Parse error")]
    #[diagnostic(code(XPST0003), help("Invalid XPath expression"))]
    ParseError {
        #[source_code]
        src: String,
        #[label("Could not parse beyond this")]
        span: SourceSpan,
    },
}
