use xee_xpath_macros::xpath_fn;

use crate::context::DynamicContext;
use crate::function::StaticFunctionDescription;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:generate-id($arg as node()?) as xs:string", context_first)]
fn generate_id(context: &DynamicContext, arg: Option<xot::Node>) -> String {
    if let Some(arg) = arg {
        let documents = context.documents();
        let documents = documents.borrow();
        let annotations = documents.annotations();
        let annotation = annotations.get(arg).unwrap();
        annotation.generate_id()
    } else {
        "".to_string()
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(generate_id)]
}
