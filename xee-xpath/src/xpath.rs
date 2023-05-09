use crate::ast_ir::IrConverter;
use crate::builder::{FunctionBuilder, Program};
use crate::dynamic_context::DynamicContext;
use crate::error::Result;
use crate::interpret::Interpreter;
use crate::ir;
use crate::ir_interpret::{InterpreterCompiler, Scopes};
use crate::parse_ast::parse_xpath;
use crate::static_context::StaticContext;
use crate::value::{Atomic, FunctionId, Item, Node, StackValue};

pub struct XPath<'a> {
    pub(crate) program: Program,
    static_context: &'a StaticContext<'a>,
    main: FunctionId,
}

impl<'a> XPath<'a> {
    pub fn new(static_context: &'a StaticContext, xpath: &str) -> Result<Self> {
        let ast = parse_xpath(xpath, static_context.namespaces)?;
        let mut ir_converter = IrConverter::new(xpath, static_context);
        let expr = ir_converter.convert_xpath(&ast)?;
        // this expression contains a function definition, we're getting it
        // in the end
        let mut program = Program::new(xpath.to_string());
        let mut scopes = Scopes::new(ir::Name("dummy".to_string()));
        let builder = FunctionBuilder::new(&mut program);
        let mut compiler = InterpreterCompiler {
            builder,
            scopes: &mut scopes,
            static_context,
        };
        compiler.compile_expr(&expr)?;

        // the inline function should be the last finished function
        let inline_id = FunctionId(program.functions.len() - 1);
        Ok(Self {
            program,
            static_context,
            main: inline_id,
        })
    }

    pub(crate) fn run_no_focus(&self, dynamic_context: &DynamicContext) -> Result<StackValue> {
        // a fake context value
        self.run(dynamic_context, Item::Atomic(Atomic::Integer(0)))
    }

    pub fn run(&self, dynamic_context: &DynamicContext, context_item: Item) -> Result<StackValue> {
        let mut interpreter = Interpreter::new(&self.program, dynamic_context);
        interpreter.start(self.main, context_item);
        interpreter.run()?;
        // the stack has to be 1 values and return the result of the expression
        // why 1 value if the context item is on the top of the stack? This is because
        // the outer main function will pop the context item; this code is there to
        // remove the function id from the stack but the main function has no function id
        assert_eq!(
            interpreter.stack().len(),
            1,
            "stack must only have 1 value but found {:?}",
            interpreter.stack()
        );
        Ok(interpreter.stack().last().unwrap().clone())
    }

    pub fn run_xot_node(
        &self,
        dynamic_context: &DynamicContext,
        node: xot::Node,
    ) -> Result<StackValue> {
        self.run(dynamic_context, Item::Node(Node::Xot(node)))
    }
}

fn unwrap_inline_function(expr: ir::Expr) -> (ir::Name, ir::Expr) {
    match expr {
        ir::Expr::FunctionDefinition(ir::FunctionDefinition { params, body, .. }) => {
            assert_eq!(params.len(), 3);
            (params[0].0.clone(), body.value)
        }
        _ => panic!("expected inline function"),
    }
}

#[cfg(test)]
mod tests {
    use xot::Xot;

    use crate::{
        document::{Documents, Uri},
        name::{Namespaces, FN_NAMESPACE},
        static_context::StaticContext,
    };

    use super::*;

    #[test]
    fn test_parse_error() {
        let mut xot = Xot::new();
        let uri = Uri("http://example.com".to_string());
        let mut documents = Documents::new();
        documents.add(&mut xot, &uri, "<doc/>").unwrap();
        let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
        let static_context = StaticContext::new(&namespaces);
        let xpath = "1 + 2 +";
        let r = XPath::new(&static_context, xpath);
        assert!(r.is_err())
    }
}
