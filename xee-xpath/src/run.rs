use xot::Xot;

use crate::context::Context;
use crate::document::{Documents, Uri};
use crate::value::StackValue;
use crate::xpath::CompiledXPath;

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate(xml: &str, xpath: &str) -> StackValue {
    let mut xot = Xot::new();
    let uri = Uri("http://example.com".to_string());
    let mut documents = Documents::new();
    documents.add(&mut xot, &uri, xml).unwrap();
    let context = Context::with_documents(&xot, &documents);
    let document = documents.get(&uri).unwrap();

    let xpath = CompiledXPath::new(&context, xpath);
    xpath.run_xot_node(document.root).unwrap()
}
