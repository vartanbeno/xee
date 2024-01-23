use xot::Xot;

use crate::ast_core as ast;
use crate::error::ElementError as Error;
use crate::{element::XsltParser, names::Names, state::State};

type Result<V> = std::result::Result<V, Error>;

pub fn parse_transform(s: &str) -> Result<ast::Transform> {
    let mut xot = Xot::new();
    let names = Names::new(&mut xot);

    let (node, span_info) = xot.parse_with_span_info(s).unwrap();
    let node = xot.document_element(node).unwrap();
    let context = State::new(xot, span_info, names);
    let parser = XsltParser::new(&context);
    parser.parse_transform(node)
}
