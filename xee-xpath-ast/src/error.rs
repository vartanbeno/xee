use chumsky::prelude::{Rich, SimpleSpan as Span};

use crate::lexer::Token;

#[derive(Debug)]
pub struct Error<'a> {
    pub src: &'a str,
    pub errors: Vec<Rich<'a, Token<'a>>>,
}

impl<'a> Error<'a> {
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
    }
}
