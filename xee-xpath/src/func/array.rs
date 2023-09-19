// https://www.w3.org/TR/xpath-functions-31/#array-functions

use ibig::IBig;

use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context::StaticFunctionDescription;
use crate::error;
use crate::interpreter::Interpreter;
use crate::sequence;
use crate::stack;
use crate::wrap_xpath_fn;
use crate::Occurrence;

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

#[xpath_fn("array:reverse($array as array(*)) as array(*)")]
fn reverse(array: stack::Array) -> stack::Array {
    array.reversed()
}

#[xpath_fn("array:join($arrays as array(*)*) as array(*)")]
fn join(arrays: &[stack::Array]) -> stack::Array {
    stack::Array::join(arrays)
}

#[xpath_fn(
    "array:for-each($array as array(*), $action as function(item()*) as item()*) as array(*)"
)]
fn for_each(
    interpreter: &mut Interpreter,
    array: stack::Array,
    action: sequence::Item,
) -> error::Result<stack::Array> {
    let closure = action.to_function()?;
    let mut result = stack::Array::new(vec![]);
    for sequence in array.iter() {
        let sequence =
            interpreter.call_closure_with_arguments(closure.clone(), &[sequence.clone()])?;
        result.push(sequence);
    }
    Ok(result)
}

#[xpath_fn(
    "array:filter($array as array(*), $function as function(item()*) as xs:boolean) as array(*)"
)]
fn filter(
    interpreter: &mut Interpreter,
    array: stack::Array,
    function: sequence::Item,
) -> error::Result<stack::Array> {
    let closure = function.to_function()?;
    let mut result = stack::Array::new(vec![]);
    for sequence in array.iter() {
        let include =
            interpreter.call_closure_with_arguments(closure.clone(), &[sequence.clone()])?;
        let include: atomic::Atomic = include.items().one()?.to_atomic()?;
        let include: bool = include.try_into()?;
        if include {
            result.push(sequence.clone());
        }
    }
    Ok(result)
}

#[xpath_fn("array:fold-left($array as array(*), $zero as item()*, $function as function(item()*, item()*) as item()*) as item()*")]
fn fold_left(
    interpreter: &mut Interpreter,
    array: stack::Array,
    zero: &sequence::Sequence,
    function: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let closure = function.to_function()?;

    let mut accumulator = zero.clone();
    for sequence in array.iter() {
        accumulator = interpreter
            .call_closure_with_arguments(closure.clone(), &[accumulator, sequence.clone()])?;
    }
    Ok(accumulator)
}

#[xpath_fn("array:fold-right($array as array(*), $zero as item()*, $function as function(item()*, item()*) as item()*) as item()*")]
fn fold_right(
    interpreter: &mut Interpreter,
    array: stack::Array,
    zero: &sequence::Sequence,
    function: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let closure = function.to_function()?;

    let mut accumulator = zero.clone();
    for sequence in array.iter().rev() {
        accumulator = interpreter
            .call_closure_with_arguments(closure.clone(), &[sequence.clone(), accumulator])?;
    }
    Ok(accumulator)
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
        wrap_xpath_fn!(reverse),
        wrap_xpath_fn!(join),
        wrap_xpath_fn!(for_each),
        wrap_xpath_fn!(filter),
        wrap_xpath_fn!(fold_left),
        wrap_xpath_fn!(fold_right),
    ]
}
