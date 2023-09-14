// https://www.w3.org/TR/xpath-functions-31/#higher-order-functions

use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context;
use crate::context::StaticFunctionDescription;
use crate::error;
use crate::interpreter::Interpreter;
use crate::sequence;
use crate::wrap_xpath_fn;
use crate::Occurrence;

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

#[xpath_fn("fn:filter($seq as item()*, $predicate as function(item()) as xs:boolean) as item()*")]
fn filter(
    interpreter: &mut Interpreter,
    seq: &sequence::Sequence,
    predicate: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let mut result: Vec<sequence::Item> = Vec::new();
    let closure = predicate.to_function()?;

    for item in seq.items() {
        let item = item?;
        let value =
            interpreter.call_closure_with_arguments(closure.clone(), &[item.clone().into()])?;
        let atom: atomic::Atomic = value.items().one()?.to_atomic()?;
        let value: bool = atom.try_into()?;
        if value {
            result.push(item);
        }
    }
    Ok(result.into())
}

#[xpath_fn("fn:fold-left($seq as item()*, $zero as item()*, $f as function(item()*, item()) as item()*) as item()*")]
fn fold_left(
    interpreter: &mut Interpreter,
    seq: &sequence::Sequence,
    zero: &sequence::Sequence,
    f: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let closure = f.to_function()?;

    let mut accumulator = zero.clone();
    for item in seq.items() {
        let item = item?;
        accumulator = interpreter
            .call_closure_with_arguments(closure.clone(), &[accumulator, item.into()])?;
    }
    Ok(accumulator)
}

#[xpath_fn("fn:fold-right($seq as item()*, $zero as item()*, $f as function(item(), item()*) as item()*) as item()*")]
fn fold_right(
    interpreter: &mut Interpreter,
    seq: &sequence::Sequence,
    zero: &sequence::Sequence,
    f: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let closure = f.to_function()?;

    let mut accumulator = zero.clone();
    // TODO: do not have reverse iterator, so have to collect first
    let seq = seq.items().collect::<error::Result<Vec<_>>>()?;
    for item in seq.into_iter().rev() {
        accumulator = interpreter
            .call_closure_with_arguments(closure.clone(), &[item.into(), accumulator])?;
    }
    Ok(accumulator)
}

#[xpath_fn("fn:for-each-pair($seq1 as item()*, $seq2 as item()*, $action as function(item(), item()) as item()*) as item()*")]
fn for_each_pair(
    interpreter: &mut Interpreter,
    seq1: &sequence::Sequence,
    seq2: &sequence::Sequence,
    action: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let mut result: Vec<sequence::Item> = Vec::with_capacity(seq1.len());
    let closure = action.to_function()?;

    for (item1, item2) in seq1.items().zip(seq2.items()) {
        let item1 = item1?;
        let item2 = item2?;
        let value = interpreter
            .call_closure_with_arguments(closure.clone(), &[item1.into(), item2.into()])?;
        for item in value.items() {
            result.push(item?);
        }
    }
    Ok(result.into())
}

#[xpath_fn("fn:sort($input as item()*) as item()*")]
fn sort1(
    context: &context::DynamicContext,
    input: &sequence::Sequence,
) -> error::Result<sequence::Sequence> {
    sort_without_key(
        context,
        input,
        context.static_context.default_collation_uri(),
    )
}

#[xpath_fn("fn:sort($input as item()*, $collation as xs:string?) as item()*")]
fn sort2(
    context: &context::DynamicContext,
    input: &sequence::Sequence,
    collation: Option<&str>,
) -> error::Result<sequence::Sequence> {
    let collation = collation.unwrap_or(context.static_context.default_collation_uri());
    sort_without_key(context, input, collation)
}

#[xpath_fn("fn:sort($input as item()*, $collation as xs:string?, $key as function(item()) as xs:anyAtomicType) as item()*")]
fn sort3(
    context: &context::DynamicContext,
    interpreter: &mut Interpreter,
    input: &sequence::Sequence,
    collation: Option<&str>,
    key: sequence::Item,
) -> error::Result<sequence::Sequence> {
    let collation = collation.unwrap_or(context.static_context.default_collation_uri());
    let closure = key.to_function()?;
    sort_by_sequence(context, input, collation, |item| {
        let value =
            interpreter.call_closure_with_arguments(closure.clone(), &[item.clone().into()])?;
        Ok(value)
    })
}

fn sort_without_key(
    context: &context::DynamicContext,
    input: &sequence::Sequence,
    collation: &str,
) -> error::Result<sequence::Sequence> {
    sort_by_sequence(context, input, collation, |item| {
        // the sequivalent of fn:data()
        let seq: sequence::Sequence = item.clone().into();
        let atoms = seq
            .atomized(context.xot)
            .collect::<error::Result<Vec<_>>>()?;
        Ok(atoms.into())
    })
}

fn sort_by_sequence<F>(
    context: &context::DynamicContext,
    input: &sequence::Sequence,
    collation: &str,
    get: F,
) -> error::Result<sequence::Sequence>
where
    F: FnMut(&sequence::Item) -> error::Result<sequence::Sequence>,
{
    let collation = context.static_context.collation(collation)?;
    let items = input.items().collect::<error::Result<Vec<_>>>()?;
    let keys = items.iter().map(get).collect::<error::Result<Vec<_>>>()?;

    let mut keys_and_items = keys.into_iter().zip(items).collect::<Vec<_>>();
    // sort by key. unfortunately sort_by requires the compare function
    // to be infallible. It's not in reality, so we make any failures
    // sort less, so they appear early on in the sequence.
    keys_and_items.sort_by(|(a_key, _), (b_key, _)| {
        a_key.compare(b_key, &collation, context.implicit_timezone())
    });
    // a pass to detect any errors; if sorting between two items is
    // impossible we want to raise a type error
    for ((a_key, _), (b_key, _)) in keys_and_items.iter().zip(keys_and_items.iter().skip(1)) {
        a_key.fallible_compare(b_key, &collation, context.implicit_timezone())?;
    }
    // now pick up items again
    let items = keys_and_items
        .into_iter()
        .map(|(_, item)| item)
        .collect::<Vec<_>>();
    Ok(items.into())
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(for_each),
        wrap_xpath_fn!(filter),
        wrap_xpath_fn!(fold_left),
        wrap_xpath_fn!(fold_right),
        wrap_xpath_fn!(for_each_pair),
        wrap_xpath_fn!(sort1),
        wrap_xpath_fn!(sort2),
        wrap_xpath_fn!(sort3),
    ]
}
