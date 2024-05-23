mod delimination;
mod explicit_whitespace;
mod lexer;
mod symbol_type;

pub use lexer::{
    BracedURILiteralWildcard, LocalNameWildcard, PrefixWildcard, PrefixedQName, URIQualifiedName,
};
pub use {delimination::lexer, lexer::Token};
