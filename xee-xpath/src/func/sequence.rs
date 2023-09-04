// https://www.w3.org/TR/xpath-functions-31/#sequence-functions

use ibig::IBig;
use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::sequence;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:empty($arg as item()*) as xs:boolean")]
fn empty(arg: &[sequence::Item]) -> bool {
    arg.is_empty()
}

#[xpath_fn("fn:exists($arg as item()*) as xs:boolean")]
fn exists(arg: &[sequence::Item]) -> bool {
    !arg.is_empty()
}

#[xpath_fn("fn:head($arg as item()*) as item()?")]
fn head(arg: &[sequence::Item]) -> Option<sequence::Item> {
    arg.first().cloned()
}

#[xpath_fn("fn:tail($arg as item()*) as item()*")]
fn tail(arg: &[sequence::Item]) -> sequence::Sequence {
    if arg.is_empty() {
        return sequence::Sequence::empty();
    }
    arg[1..].to_vec().into()
}

#[xpath_fn(
    "fn:insert-before($target as item()*, $position as xs:integer, $inserts as item()*) as item()*"
)]
fn insert_before(
    target: &[sequence::Item],
    position: IBig,
    inserts: &[sequence::Item],
) -> error::Result<sequence::Sequence> {
    if target.is_empty() {
        return Ok(inserts.to_vec().into());
    }
    let position = if position < IBig::from(0) {
        IBig::from(0)
    } else {
        position
    };
    let position: usize = position.try_into().map_err(|_| error::Error::Overflow)?;
    let position = position.saturating_sub(1);
    let position = if position > target.len() {
        target.len()
    } else {
        position
    };

    let items = target[0..position]
        .iter()
        .chain(inserts)
        .chain(target[position..].iter())
        .cloned()
        .collect::<Vec<_>>();
    Ok(items.into())
}

#[xpath_fn("fn:remove($target as item()*, $position as xs:integer) as item()*")]
fn remove(target: &[sequence::Item], position: IBig) -> error::Result<sequence::Sequence> {
    let position = if position < IBig::from(0) {
        IBig::from(0)
    } else {
        position
    };
    let position: usize = position.try_into().map_err(|_| error::Error::Overflow)?;
    if position == 0 || position > target.len() {
        // TODO: unfortunate we can't just copy sequence
        return Ok(target.to_vec().into());
    }
    let mut target = target.to_vec();
    let position = position.saturating_sub(1);
    target.remove(position);
    Ok(target.into())
}

#[xpath_fn("fn:exactly-one($arg as item()*) as item()")]
fn exactly_one(arg: &[sequence::Item]) -> error::Result<sequence::Item> {
    if arg.len() == 1 {
        Ok(arg[0].clone())
    } else {
        Err(error::Error::FORG0005)
    }
}

#[xpath_fn("fn:count($arg as item()*) as xs:integer")]
fn count(arg: &[sequence::Item]) -> IBig {
    arg.len().into()
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(empty),
        wrap_xpath_fn!(exists),
        wrap_xpath_fn!(head),
        wrap_xpath_fn!(tail),
        wrap_xpath_fn!(insert_before),
        wrap_xpath_fn!(remove),
        wrap_xpath_fn!(exactly_one),
        wrap_xpath_fn!(count),
    ]
}
