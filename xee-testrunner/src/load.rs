use xee_name::Namespaces;
use xee_xpath::{
    context::{DynamicContext, StaticContext},
    sequence::Item,
    Queries, Query, Session,
};
use xot::Xot;

use crate::error::Result;

pub(crate) const XPATH_NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

pub(crate) fn convert_string(_: &mut Session, item: &Item) -> xee_xpath::error::Result<String> {
    item.to_atomic()?.try_into()
}

pub(crate) fn convert_boolean(
    session: &mut Session,
    item: &Item,
) -> xee_xpath::error::Result<bool> {
    Ok(convert_string(session, item)? == "true")
}

pub(crate) trait Loadable: Sized {
    fn query(queries: Queries) -> Result<(Queries, impl Query<Self>)>;

    fn load_from_xml(xot: &mut Xot, xml: &str, ns: &str) -> Result<Self> {
        let root = xot.parse(xml)?;
        let document_element = xot.document_element(root)?;
        Self::load(xot, document_element, ns)
    }

    fn load(xot: &mut Xot, node: xot::Node, ns: &str) -> Result<Self> {
        let namespaces = Namespaces::new(
            Namespaces::default_namespaces(),
            ns,
            Namespaces::FN_NAMESPACE,
        );
        let static_context = StaticContext::from_namespaces(namespaces);
        let queries = Queries::new(&static_context);
        let r = {
            let dynamic_context = DynamicContext::empty(&static_context);
            let (queries, query) = Self::query(queries)?;

            let mut session = queries.session(&dynamic_context, xot);
            // the query has a lifetime for the dynamic context, and a lifetime
            // for the static context
            query.execute(&mut session, &Item::from(node))?
        };
        Ok(r)
    }
}
