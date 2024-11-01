// https://www.w3.org/TR/xpath-functions-31/#parsing-and-serializing

use xee_xpath_macros::xpath_fn;

use crate::{context, error, interpreter::Interpreter, wrap_xpath_fn};

use super::StaticFunctionDescription;

#[xpath_fn("fn:parse-xml($arg as xs:string?) as document-node(element(*))?")]
fn parse_xml(
    context: &context::DynamicContext,
    interpreter: &mut Interpreter,
    arg: Option<&str>,
) -> error::Result<Option<xot::Node>> {
    if let Some(arg) = arg {
        let documents = context.documents();
        let handle = documents
            .borrow_mut()
            .add_string(interpreter.xot_mut(), None, arg)
            .map_err(|_| error::Error::FODC0006)?;
        let doc = documents
            .borrow()
            .get_node_by_handle(handle)
            .ok_or(error::Error::FODC0006)?;
        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(parse_xml)]
}
