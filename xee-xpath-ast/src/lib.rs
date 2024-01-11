pub mod ast;
mod context;
mod error;
mod lexer;
mod operator;
mod parser;
pub mod pattern;
pub mod span;

pub use xee_name::{Name, NamespaceLookup, Namespaces, VariableNames, FN_NAMESPACE, XS_NAMESPACE};

pub use context::XPathParserContext;
pub use error::ParserError;
pub use parser::parse_name;
pub use pattern::Pattern;
pub use span::WithSpan;
