use xee_xpath_macros::xpath_fn;

use crate::{
    context::DynamicContext, error, function::StaticFunctionDescription, library::uri::resolve_uri,
    wrap_xpath_fn, xml::Uri,
};

use super::uri::strict_url_parse;

#[xpath_fn("fn:doc($uri as xs:string?) as document-node()?")]
fn doc(context: &DynamicContext, uri: Option<&str>) -> error::Result<Option<xot::Node>> {
    Ok(if let Some(uri) = uri {
        let uri = if let Some(base) = context.static_context().static_base_uri() {
            // we can unwrap here, as we know we passed in a Some
            resolve_uri(Some(uri), base)?.unwrap()
        } else {
            // TODO: should recognize this is a relative URI and
            // if so, this is a FODC002 error as no static base uri is set
            uri.to_string()
        };
        let _ = strict_url_parse(&uri).map_err(|_| error::Error::FODC0002)?;
        let uri = Uri::new(&uri);

        // first check whether a document is there at all, if so, return it
        let documents = context.documents();
        let documents = documents.borrow();
        let document = documents.get_by_uri(&uri);
        document.map(|document| document.root())

        // TODO: as a fallback, and configurable, we can do an actual
        // request to fetch an external resource, parse it, and return it
    } else {
        None
    })
}

// https://www.w3.org/TR/xpath-functions-31/#fns-on-docs
pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(doc)]
}
