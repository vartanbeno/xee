use xee_xpath_ast::ast;

use crate::context::{DynamicContext, StaticContext};
use crate::error;
use crate::function;
use crate::interpreter;
use crate::ir;
use crate::ir::IrConverter;

#[derive(Debug)]
pub struct XPath {
    pub(crate) program: function::Program,
}

impl XPath {
    pub fn new(static_context: &StaticContext, xpath: &str) -> error::Result<Self> {
        let ast = ast::XPath::parse(xpath, static_context.namespaces, &static_context.variables)?;
        let mut ir_converter = IrConverter::new(xpath, static_context);
        let expr = ir_converter.convert_xpath(&ast)?;
        // this expression contains a function definition, we're getting it
        // in the end
        let mut program = function::Program::new(xpath.to_string());
        let mut scopes = interpreter::Scopes::new(ir::Name("dummy".to_string()));
        let builder = interpreter::FunctionBuilder::new(&mut program);
        let mut compiler = interpreter::InterpreterCompiler {
            builder,
            scopes: &mut scopes,
            static_context,
        };
        compiler.compile_expr(&expr)?;

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
