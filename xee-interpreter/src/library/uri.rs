use xee_xpath_macros::xpath_fn;

use crate::{function::StaticFunctionDescription, wrap_xpath_fn};

#[xpath_fn("fn:escape-html-uri($uri as xs:string?) as xs:string")]
fn escape_html_uri(uri: Option<&str>) -> String {
    if let Some(uri) = uri {
        percent_encoding::utf8_percent_encode(uri, percent_encoding::CONTROLS).to_string()
    } else {
        "".to_string()
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(escape_html_uri)]
}
