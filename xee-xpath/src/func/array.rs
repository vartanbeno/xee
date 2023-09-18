// https://www.w3.org/TR/xpath-functions-31/#array-functions

use ibig::IBig;

use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::sequence;
use crate::stack;
use crate::wrap_xpath_fn;

#[xpath_fn("array:get($array as array(*), $position as xs:integer) as item()*")]
fn get(array: stack::Array, position: IBig) -> error::Result<sequence::Sequence> {
    let position: i64 = position.try_into()?;
    let position = position - 1;
    if position < 0 {
        return Err(error::Error::FOAY0001);
    }
    let item = array
        .index(position as usize)
        .ok_or(error::Error::FOAY0001)?;
    Ok(item.clone())
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(get)]
}
