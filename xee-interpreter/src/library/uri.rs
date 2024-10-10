use std::cell::RefCell;
use std::rc::Rc;

use url::Url;
use xee_xpath_macros::xpath_fn;

use crate::{atomic, function::StaticFunctionDescription, wrap_xpath_fn};
use crate::{context, error};

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

#[xpath_fn("fn:resolve-uri($relative as xs:string?) as xs:anyURI?")]
fn resolve_uri1(
    context: &context::DynamicContext,
    relative: Option<&str>,
) -> error::Result<Option<atomic::Atomic>> {
    let base = context.static_context().static_base_uri();
    if let Some(base) = base {
        Ok(resolve_uri(relative, base)?
            .map(|resolved| atomic::Atomic::String(atomic::StringType::AnyURI, Rc::from(resolved))))
    } else {
        Err(error::Error::FONS0005)
    }
}

#[xpath_fn("fn:resolve-uri($relative as xs:string?, $base as xs:string) as xs:anyURI?")]
fn resolve_uri2(relative: Option<&str>, base: &str) -> error::Result<Option<atomic::Atomic>> {
    Ok(resolve_uri(relative, base)?
        .map(|resolved| atomic::Atomic::String(atomic::StringType::AnyURI, Rc::from(resolved))))
}

pub(crate) fn resolve_uri(relative: Option<&str>, base: &str) -> error::Result<Option<String>> {
    Ok(if let Some(relative) = relative {
        let url = strict_url_parse(relative);
        if let Ok(_url) = url {
            // it's an absolute URL, so is returned unchanged
            Some(relative.to_string())
        } else {
            // fn-resolve-uri-3 doesn't allow relative URIs starting with a colon
            if relative.starts_with(':') {
                return Err(error::Error::FORG0002);
            }
            // this URL is not absolute, so is assumed to be relative
            // now it needs to be combined with the base
            let base = strict_url_parse(base)?;
            // fn-resolve-url-26 doesn't allow fragments in the base URI
            if base.fragment().is_some() {
                return Err(error::Error::FORG0002);
            }
            let joined = base.join(relative).map_err(|_e| error::Error::FORG0002)?;
            // check whether the joined URL is valid
            let _ = strict_url_parse(joined.as_str())?;
            let s: String = joined.into();
            Some(s)
        }
    } else {
        None
    })
}

// a strict URL parse that fails on any syntax violations
fn strict_url_parse(url: &str) -> error::Result<Url> {
    let violations = RefCell::new(Vec::new());
    let c = |v| {
        let mut violations = violations.borrow_mut();
        violations.push(v)
    };
    let options = Url::options().syntax_violation_callback(Some(&c));

    let url = options.parse(url).map_err(|_e| error::Error::FORG0002)?;
    if !violations.borrow().is_empty() {
        Err(error::Error::FORG0002)
    } else {
        Ok(url)
    }
}

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
        wrap_xpath_fn!(resolve_uri1),
        wrap_xpath_fn!(resolve_uri2),
        wrap_xpath_fn!(encode_for_uri),
        wrap_xpath_fn!(iri_to_uri),
        wrap_xpath_fn!(escape_html_uri),
    ]
}
