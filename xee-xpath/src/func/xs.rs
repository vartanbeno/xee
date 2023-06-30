use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context::StaticFunctionDescription;
use crate::error;
use crate::wrap_xpath_fn;

#[xpath_fn("xs:string($arg as xs:anyAtomicType?) as xs:string?")]
fn xs_string(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    Ok(arg.map(|arg| arg.cast_to_string()))
}

#[xpath_fn("xs:untypedAtomic($arg as xs:anyAtomicType?) as xs:untypedAtomic?")]
fn xs_untyped_atomic(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    Ok(arg.map(|arg| arg.cast_to_untyped_atomic()))
}

#[xpath_fn("xs:float($arg as xs:anyAtomicType?) as xs:float?")]
fn xs_float(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_float()).transpose()
}

#[xpath_fn("xs:double($arg as xs:anyAtomicType?) as xs:double?")]
fn xs_double(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_double()).transpose()
}

#[xpath_fn("xs:decimal($arg as xs:anyAtomicType?) as xs:decimal?")]
fn xs_decimal(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_decimal()).transpose()
}

#[xpath_fn("xs:integer($arg as xs:anyAtomicType?) as xs:integer?")]
fn xs_integer(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_integer()).transpose()
}

#[xpath_fn("xs:long($arg as xs:anyAtomicType?) as xs:long?")]
fn xs_long(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_long()).transpose()
}

#[xpath_fn("xs:int($arg as xs:anyAtomicType?) as xs:int?")]
fn xs_int(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_int()).transpose()
}

#[xpath_fn("xs:short($arg as xs:anyAtomicType?) as xs:short?")]
fn xs_short(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_short()).transpose()
}

#[xpath_fn("xs:byte($arg as xs:anyAtomicType?) as xs:byte?")]
fn xs_byte(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_byte()).transpose()
}

#[xpath_fn("xs:unsignedLong($arg as xs:anyAtomicType?) as xs:unsignedLong?")]
fn xs_unsigned_long(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_unsigned_long()).transpose()
}

#[xpath_fn("xs:unsignedInt($arg as xs:anyAtomicType?) as xs:unsignedInt?")]
fn xs_unsigned_int(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_unsigned_int()).transpose()
}

#[xpath_fn("xs:unsignedShort($arg as xs:anyAtomicType?) as xs:unsignedShort?")]
fn xs_unsigned_short(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_unsigned_short()).transpose()
}

#[xpath_fn("xs:unsignedByte($arg as xs:anyAtomicType?) as xs:unsignedByte?")]
fn xs_unsigned_byte(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_unsigned_byte()).transpose()
}

#[xpath_fn("xs:boolean($arg as xs:anyAtomicType?) as xs:boolean?")]
fn xs_boolean(arg: Option<atomic::Atomic>) -> error::Result<Option<atomic::Atomic>> {
    arg.map(|arg| arg.cast_to_boolean()).transpose()
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(xs_string),
        wrap_xpath_fn!(xs_untyped_atomic),
        wrap_xpath_fn!(xs_float),
        wrap_xpath_fn!(xs_double),
        wrap_xpath_fn!(xs_decimal),
        wrap_xpath_fn!(xs_integer),
        wrap_xpath_fn!(xs_long),
        wrap_xpath_fn!(xs_int),
        wrap_xpath_fn!(xs_short),
        wrap_xpath_fn!(xs_byte),
        wrap_xpath_fn!(xs_unsigned_long),
        wrap_xpath_fn!(xs_unsigned_int),
        wrap_xpath_fn!(xs_unsigned_short),
        wrap_xpath_fn!(xs_unsigned_byte),
        wrap_xpath_fn!(xs_boolean),
    ]
}
