use iri_string::types::{IriReferenceStr, IriString};
use xee_xpath_macros::xpath_fn;

use crate::{
    context::DynamicContext, error, function::StaticFunctionDescription, wrap_xpath_fn, xml::Uri,
};

#[xpath_fn("fn:doc($uri as xs:string?) as document-node()?")]
fn doc(context: &DynamicContext, uri: Option<&str>) -> error::Result<Option<xot::Node>> {
    if let Some(uri) = uri {
        let iri_reference: &IriReferenceStr = uri.try_into().map_err(|_| error::Error::FODC0005)?;
        let iri: IriString = match iri_reference.to_iri() {
            Ok(iri) => iri.into(),
            Err(relative_iri) => {
                let base = context.static_context().static_base_uri();
                if let Some(base) = base {
                    relative_iri.resolve_against(base).into()
                } else {
                    return Err(error::Error::FODC0002);
                }
            }
        };
        let uri = Uri::new(iri.as_str());
        // first check whether a document is there at all, if so, return it
        let documents = context.documents();
        let documents = documents.borrow();
        let document = documents.get_by_uri(&uri);

        if let Some(document) = document {
            Ok(Some(document.root()))
        } else {
            // The document doesn't exist, so return an error
            Err(error::Error::FODC0002)
        }
    } else {
        Ok(None)
    }
}

// https://www.w3.org/TR/xpath-functions-31/#fns-on-docs
pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(doc)]
}
