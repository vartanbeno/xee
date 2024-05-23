mod delimination2;
mod explicit_whitespace;
mod lexer;
mod symbol_type;

pub use lexer::{
    BracedURILiteralWildcard, LocalNameWildcard, PrefixWildcard, PrefixedQName, URIQualifiedName,
};
pub use {delimination2::lexer, lexer::Token};
