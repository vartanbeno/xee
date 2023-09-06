mod accessor;
mod boolean;
mod datetime;
mod duration;
mod fn_;
mod math;
mod node;
mod numeric;
mod qname;
mod sequence;
mod string;
mod xpath_fn;
mod xs;

use crate::context::StaticFunctionDescription;

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    let mut descriptions = Vec::new();
    descriptions.extend(fn_::static_function_descriptions());
    descriptions.extend(string::static_function_descriptions());
    descriptions.extend(xs::static_function_descriptions());
    descriptions.extend(numeric::static_function_descriptions());
    descriptions.extend(math::static_function_descriptions());
    descriptions.extend(boolean::static_function_descriptions());
    descriptions.extend(accessor::static_function_descriptions());
    descriptions.extend(duration::static_function_descriptions());
    descriptions.extend(datetime::static_function_descriptions());
    descriptions.extend(sequence::static_function_descriptions());
    descriptions.extend(node::static_function_descriptions());
    descriptions.extend(qname::static_function_descriptions());
    descriptions
}
