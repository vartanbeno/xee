pub mod ast;
mod error;
mod lexer;
mod namespaces;
mod operator;
mod parser;
pub mod span;

pub use error::Error;
pub use namespaces::{Namespaces, FN_NAMESPACE, XS_NAMESPACE};
pub use parser::{
    parse_expr_single, parse_kind_test, parse_sequence_type, parse_signature, parse_xpath,
};
pub use span::WithSpan;
