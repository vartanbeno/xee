// https://www.w3.org/TR/xpath-functions-31/#higher-order-functions

use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::interpreter::Interpreter;
use crate::sequence;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:for-each($seq as item()*, $action as function(item()) as item()*) as item()*")]
fn for_each(
    interpreter: &mut Interpreter,
    seq: &sequence::Sequence,
    action: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let mut result: Vec<sequence::Item> = Vec::with_capacity(seq.len());
    let closure = action.to_function()?;

    for item in seq.items() {
        let item = item?;
        let value = interpreter.call_closure_with_arguments(closure.clone(), &[item.into()])?;
        for item in value.items() {
            result.push(item?);
        }
    }
    Ok(result.into())
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(for_each)]
}
