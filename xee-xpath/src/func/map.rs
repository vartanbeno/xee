// https://www.w3.org/TR/xpath-functions-31/#array-functions

use ibig::IBig;

use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::function;
use crate::function::StaticFunctionDescription;
use crate::sequence;
use crate::wrap_xpath_fn;

#[xpath_fn("map:get($map as map(*), $key as xs:anyAtomicType) as item()*")]
fn get(map: function::Map, key: atomic::Atomic) -> sequence::Sequence {
    map.get(&key).unwrap_or(sequence::Sequence::empty())
}

#[xpath_fn("map:size($map as map(*)) as xs:integer")]
fn size(map: function::Map) -> IBig {
    map.len().into()
}

#[xpath_fn("map:keys($map as map(*)) as xs:anyAtomicType*")]
fn keys(map: function::Map) -> sequence::Sequence {
    map.keys().collect::<Vec<_>>().into()
}

#[xpath_fn("map:contains($map as map(*), $key as xs:anyAtomicType) as xs:boolean")]
fn contains(map: function::Map, key: atomic::Atomic) -> bool {
    map.get(&key).is_some()
}

#[xpath_fn("map:put($map as map(*), $key as xs:anyAtomicType, $value as item()*) as map(*)")]
fn put(map: function::Map, key: atomic::Atomic, value: &sequence::Sequence) -> function::Map {
    map.put(key, value)
}

#[xpath_fn("map:entry($key as xs:anyAtomicType, $value as item()*) as map(*)")]
fn entry(key: atomic::Atomic, value: &sequence::Sequence) -> function::Map {
    function::Map::new(vec![(key, value.clone())]).unwrap()
}

#[xpath_fn("map:remove($map as map(*), $keys as xs:anyAtomicType*) as map(*)")]
fn remove(map: function::Map, keys: &[atomic::Atomic]) -> function::Map {
    map.remove_keys(keys)
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(get),
        wrap_xpath_fn!(size),
        wrap_xpath_fn!(keys),
        wrap_xpath_fn!(contains),
        wrap_xpath_fn!(put),
        wrap_xpath_fn!(entry),
        wrap_xpath_fn!(remove),
    ]
}
