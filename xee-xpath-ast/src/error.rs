use chumsky::prelude::{Rich, SimpleSpan as Span};
// use miette::{Diagnostic, SourceSpan};
// use thiserror::Error;

use crate::lexer::Token;

#[derive(Debug)]
pub struct Error<'a> {
    pub src: &'a str,
    pub errors: Vec<Rich<'a, Token<'a>>>,
}

impl<'a> Error<'a> {
    pub(crate) fn new(src: &'a str, errors: Vec<Rich<'a, Token<'a>>>) -> Self {
        Self { src, errors }
    }
    pub fn span(&self) -> Span {
        *self.errors[0].span()
    }
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[cfg(test)]
impl serde::Serialize for Error<'_> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let formatted = format!("{:?}", self.errors);
        serializer.serialize_str(&formatted)

        // let mut errors = serializer.serialize_struct("ParseError", 1)?;
        // now output formatted as serialized
        // use serde::ser::SerializeStruct;
        // // errors.serialize_field("errors", &formatted)?;
        // errors.end()
    }
}

// #[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
// pub enum Error {
//     /// Parse error.
//     ///
//     /// It is a static error if an expression is not a valid instance of the
//     /// grammar defined in A.1 EBNF.
//     #[error("Parse error")]
//     #[diagnostic(code(XPST0003), help("Invalid XPath expression"))]
//     ParseError {
//         #[source_code]
//         src: String,
//         #[label("Could not parse beyond this")]
//         span: SourceSpan,
//     },
// }
