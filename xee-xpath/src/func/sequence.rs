// https://www.w3.org/TR/xpath-functions-31/#sequence-functions

use ibig::IBig;
use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::sequence;
use crate::wrap_xpath_fn;
use crate::Atomic;
use crate::DynamicContext;

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

#[xpath_fn("fn:reverse($arg as item()*) as item()*")]
fn reverse(arg: &[sequence::Item]) -> sequence::Sequence {
    let mut items = arg.to_vec();
    items.reverse();
    items.into()
}

#[xpath_fn("fn:subsequence($sourceSeq as item()*, $startingLoc as xs:double) as item()*")]
fn subsequence2(source_seq: &[sequence::Item], starting_loc: f64) -> Vec<sequence::Item> {
    if starting_loc.is_nan() {
        return Vec::new();
    }
    let starting_loc = starting_loc - 1.0;
    let starting_loc = starting_loc.clamp(0.0, (source_seq.len()) as f64);
    let starting_loc = starting_loc as usize;
    source_seq[starting_loc..].to_vec()
}

#[xpath_fn(
    "fn:subsequence($sourceSeq as item()*, $startingLoc as xs:double, $length as xs:double) as item()*"
)]
fn subsequence3(
    source_seq: &[sequence::Item],
    starting_loc: f64,
    length: f64,
) -> Vec<sequence::Item> {
    let starting_loc = starting_loc.round();
    let starting_loc = starting_loc - 1.0;
    let length = length.round();
    let end = starting_loc + length;
    if end.is_nan() {
        return Vec::new();
    }
    let starting_loc = starting_loc.clamp(0.0, (source_seq.len()) as f64);
    let end = end.clamp(starting_loc, (source_seq.len()) as f64);
    let starting_loc = starting_loc as usize;
    let end = end as usize;

    source_seq[starting_loc..end].to_vec()
}

#[xpath_fn("fn:unordered($sourceSeq as item()*) as item()*")]
fn unordered(source_seq: &sequence::Sequence) -> sequence::Sequence {
    // TODO: annoying that a clone is needed there.
    // would be better if we could get an Rc of sequence so the clone is
    // much more cheap
    source_seq.clone()
}

// #[xpath_fn(
//     "fn:distinct-values($arg as xs:anyAtomicType*, $collation as xs:string) as xs:anyAtomicType*",
//     collation
// )]
// fn distinct_values(arg: &[Atomic], collation: &str) -> Atomic {
//     todo!();
// }

#[xpath_fn("fn:index-of($seq as xs:anyAtomicType*, $search as xs:anyAtomicType, $collation as xs:string) as xs:integer*", collation)]
fn index_of(
    context: &DynamicContext,
    seq: &[Atomic],
    search: Atomic,
    collation: &str,
) -> error::Result<Vec<IBig>> {
    let collation = context.static_context.collation(collation)?;
    let default_offset = context.implicit_timezone();
    // TODO: annoying that we have to clone both atoms here
    let indices = seq.iter().enumerate().filter_map(|(i, atom)| {
        if atom.equal(&search, &collation, default_offset) {
            Some((i + 1).into())
        } else {
            None
        }
    });
    Ok(indices.collect::<Vec<_>>())
}

#[xpath_fn("fn:deep-equal($parameter1 as item()*, $parameter2 as item()*, $collation as xs:string) as xs:boolean", collation)]
fn deep_equal(
    context: &DynamicContext,
    parameter1: &sequence::Sequence,
    parameter2: &sequence::Sequence,
    collation: &str,
) -> error::Result<bool> {
    let collation = context.static_context.collation(collation)?;
    let default_offset = context.implicit_timezone();
    parameter1.deep_equal(parameter2, &collation, default_offset)
}

#[xpath_fn("fn:zero-or-one($arg as item()*) as item()?")]
fn zero_or_one(arg: &[sequence::Item]) -> error::Result<Option<sequence::Item>> {
    match arg.len() {
        0 => Ok(None),
        1 => Ok(Some(arg[0].clone())),
        _ => Err(error::Error::FORG0003),
    }
}

#[xpath_fn("fn:one-or-more($arg as item()*) as item()+")]
fn one_or_more(arg: &[sequence::Item]) -> error::Result<sequence::Sequence> {
    if arg.is_empty() {
        Err(error::Error::FORG0004)
    } else {
        Ok(arg.to_vec().into())
    }
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
        wrap_xpath_fn!(reverse),
        wrap_xpath_fn!(subsequence2),
        wrap_xpath_fn!(subsequence3),
        wrap_xpath_fn!(unordered),
        wrap_xpath_fn!(index_of),
        wrap_xpath_fn!(deep_equal),
        wrap_xpath_fn!(zero_or_one),
        wrap_xpath_fn!(one_or_more),
        wrap_xpath_fn!(exactly_one),
        wrap_xpath_fn!(count),
    ]
}
