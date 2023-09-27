use crate::context::{DynamicContext, StaticContext};
use crate::error;
use crate::interpreter;

#[derive(Debug)]
pub struct XPath {
    pub(crate) program: interpreter::Program,
}

impl XPath {
    pub fn new(static_context: &StaticContext, xpath: &str) -> error::Result<Self> {
        let program = interpreter::Program::new(static_context, xpath)?;
        Ok(Self { program })
    }

    pub fn runnable<'a>(
        &'a self,
        dynamic_context: &'a DynamicContext,
    ) -> interpreter::Runnable<'a> {
        interpreter::Runnable::new(&self.program, dynamic_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xee_xpath_ast::{Namespaces, FN_NAMESPACE};
    use xot::Xot;

    use crate::context::StaticContext;
    use crate::xml;

    #[test]
    fn test_parse_error() {
        let mut xot = Xot::new();
        let uri = xml::Uri("http://example.com".to_string());
        let mut documents = xml::Documents::new();
        documents.add(&mut xot, &uri, "<doc/>").unwrap();
        let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
        let static_context = StaticContext::new(&namespaces);
        let xpath = "1 + 2 +";
        let r = XPath::new(&static_context, xpath);
        assert!(r.is_err())
    }
}
