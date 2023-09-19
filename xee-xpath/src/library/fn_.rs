use ibig::IBig;
use xee_xpath_ast::{ast, FN_NAMESPACE};
use xee_xpath_macros::xpath_fn;

use crate::context::DynamicContext;
use crate::error;
use crate::function::StaticFunctionDescription;
use crate::interpreter;
use crate::sequence;
use crate::wrap_xpath_fn;
use crate::xml;

#[xpath_fn("fn:my_function($a as xs:integer, $b as xs:integer) as xs:integer")]
fn my_function(a: IBig, b: IBig) -> IBig {
    a + b
}

#[xpath_fn("fn:generate-id($arg as node()?) as xs:string", context_first)]
fn generate_id(context: &DynamicContext, arg: Option<xml::Node>) -> String {
    if let Some(arg) = arg {
        let annotations = &context.documents.annotations;
        let annotation = annotations.get(arg).unwrap();
        annotation.generate_id()
    } else {
        "".to_string()
    }
}

fn error(
    _context: &DynamicContext,
    _interpreter: &mut interpreter::Interpreter,
    _arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    Err(error::Error::FOER0000)
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(my_function),
        wrap_xpath_fn!(generate_id),
        StaticFunctionDescription {
            name: ast::Name::new("error".to_string(), Some(FN_NAMESPACE.to_string()), None),
            arity: 0,
            function_kind: None,
            func: error,
        },
    ]
}
