pub mod ast;
mod context;
mod error;
mod lexer;
mod namespaces;
mod operator;
mod parser;
pub mod pattern;
pub mod span;

pub use context::{VariableNames, XPathParserContext};
pub use error::ParserError;
pub use namespaces::{NamespaceLookup, Namespaces, FN_NAMESPACE, XS_NAMESPACE};
pub use span::WithSpan;
