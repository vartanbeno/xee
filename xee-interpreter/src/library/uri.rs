use xee_xpath_macros::xpath_fn;

use crate::{function::StaticFunctionDescription, wrap_xpath_fn};

const IRI: percent_encoding::AsciiSet = percent_encoding::CONTROLS
    .add(b'<')
    .add(b'>')
    .add(b'"')
    .add(b' ')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'`');

// the only things not encoded are the upper and lower case letters, the
// digits, '-', '_', '.' and '~'.
const ENCODE_FOR_URI: percent_encoding::AsciiSet = percent_encoding::CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[xpath_fn("fn:encode-for-uri($uripart as xs:string?) as xs:string")]
fn encode_for_uri(uripart: Option<&str>) -> String {
    if let Some(uri_part) = uripart {
        percent_encoding::utf8_percent_encode(uri_part, &ENCODE_FOR_URI).to_string()
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:iri-to-uri($iri as xs:string?) as xs:string")]
fn iri_to_uri(iri: Option<&str>) -> String {
    if let Some(iri) = iri {
        percent_encoding::utf8_percent_encode(iri, &IRI).to_string()
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:escape-html-uri($uri as xs:string?) as xs:string")]
fn escape_html_uri(uri: Option<&str>) -> String {
    if let Some(uri) = uri {
        percent_encoding::utf8_percent_encode(uri, percent_encoding::CONTROLS).to_string()
    } else {
        "".to_string()
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(encode_for_uri),
        wrap_xpath_fn!(iri_to_uri),
        wrap_xpath_fn!(escape_html_uri),
    ]
}
