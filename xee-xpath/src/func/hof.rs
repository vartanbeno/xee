// https://www.w3.org/TR/xpath-functions-31/#higher-order-functions

// use ordered_float::OrderedFloat;
// use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
// use crate::error;
// use crate::wrap_xpath_fn;
// use crate::Atomic;

// #[xpath_fn("fn:for-each($seq as item()*, $action as function(item()) as item()*) as item()*")]
// fn for_each(
//     seq: &sequence::Sequence,
//     action: &sequence::Sequence,
// ) -> error::Result<sequence::Sequence> {
//     todo!()
// }

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![]
}
