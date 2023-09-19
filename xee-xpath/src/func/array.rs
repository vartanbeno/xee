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
    let position = convert_position(position)?;
    let item = array
        .index(position as usize)
        .ok_or(error::Error::FOAY0001)?;
    Ok(item.clone())
}

#[xpath_fn("array:size($array as array(*)) as xs:integer")]
fn size(array: stack::Array) -> IBig {
    array.len().into()
}

#[xpath_fn(
    "array:put($array as array(*), $position as xs:integer, $member as item()*) as array(*)"
)]
fn put(
    array: stack::Array,
    position: IBig,
    member: &sequence::Sequence,
) -> error::Result<stack::Array> {
    let position = convert_position(position)?;
    array.put(position, member).ok_or(error::Error::FOAY0001)
}

#[xpath_fn("array:append($array as array(*), $appendage as item()*) as array(*)")]
fn append(array: stack::Array, appendage: &sequence::Sequence) -> stack::Array {
    array.append(appendage)
}

#[xpath_fn("array:subarray($array as array(*), $start as xs:integer) as array(*)")]
fn subarray2(array: stack::Array, start: IBig) -> error::Result<stack::Array> {
    let start = convert_position(start)?;
    let length = array.len() - start;
    array.subarray(start, length).ok_or(error::Error::FOAY0001)
}

#[xpath_fn(
    "array:subarray($array as array(*), $start as xs:integer, $length as xs:integer) as array(*)"
)]
fn subarray3(array: stack::Array, start: IBig, length: IBig) -> error::Result<stack::Array> {
    let start = convert_position(start)?;
    let length = convert_length(length)?;
    array.subarray(start, length).ok_or(error::Error::FOAY0001)
}

#[xpath_fn("array:remove($array as array(*), $positions as xs:integer*) as array(*)")]
fn remove(array: stack::Array, positions: &[IBig]) -> error::Result<stack::Array> {
    let positions = positions
        .iter()
        .map(|position| convert_position(position.clone()))
        .collect::<error::Result<Vec<usize>>>()?;
    array
        .remove_positions(&positions)
        .ok_or(error::Error::FOAY0001)
}

#[xpath_fn("array:insert-before($array as array(*), $position as xs:integer, $member as item()*) as array(*)")]
fn insert_before(
    array: stack::Array,
    position: IBig,
    member: &sequence::Sequence,
) -> error::Result<stack::Array> {
    let position = convert_position(position)?;
    array
        .insert_before(position, member)
        .ok_or(error::Error::FOAY0001)
}

#[xpath_fn("array:head($array as array(*)) as item()*")]
fn head(array: stack::Array) -> error::Result<sequence::Sequence> {
    let item = array.index(0).ok_or(error::Error::FOAY0001)?;
    Ok(item.clone())
}

#[xpath_fn("array:tail($array as array(*)) as item()*")]
fn tail(array: stack::Array) -> error::Result<stack::Array> {
    array
        .subarray(1, array.len() - 1)
        .ok_or(error::Error::FOAY0001)
}

fn convert_position(position: IBig) -> error::Result<usize> {
    let position: i64 = position.try_into()?;
    let position = position - 1;
    if position < 0 {
        return Err(error::Error::FOAY0001);
    }
    Ok(position as usize)
}

fn convert_length(length: IBig) -> error::Result<usize> {
    let length: i64 = length.try_into()?;
    if length < 0 {
        return Err(error::Error::FOAY0002);
    }
    let length = length as usize;
    Ok(length)
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(get),
        wrap_xpath_fn!(size),
        wrap_xpath_fn!(put),
        wrap_xpath_fn!(append),
        wrap_xpath_fn!(subarray2),
        wrap_xpath_fn!(subarray3),
        wrap_xpath_fn!(remove),
        wrap_xpath_fn!(insert_before),
        wrap_xpath_fn!(head),
        wrap_xpath_fn!(tail),
    ]
}
