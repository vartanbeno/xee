use xee_xpath_ast::ast::parse_xpath;

use crate::context::{DynamicContext, StaticContext};
use crate::data::{Atomic, FunctionId, Item, Node, OutputItem, Value};
use crate::error::{Error, Result};
use crate::interpreter::{FunctionBuilder, Interpreter, InterpreterCompiler, Program, Scopes};
use crate::ir::IrConverter;
use crate::{ir, Sequence};

#[derive(Debug)]
pub struct XPath {
    pub(crate) program: Program,
    main: FunctionId,
}

impl XPath {
    pub fn new(static_context: &StaticContext, xpath: &str) -> Result<Self> {
        let ast = parse_xpath(xpath, static_context.namespaces, &static_context.variables)?;
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
            main: inline_id,
        })
    }

    pub fn run(
        &self,
        dynamic_context: &DynamicContext,
        context_item: Option<&Item>,
    ) -> Result<Value> {
        let mut interpreter = Interpreter::new(&self.program, dynamic_context);
        let arguments = dynamic_context.arguments()?;
        interpreter.start(self.main, context_item, &arguments);
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
        let value = interpreter.stack().last().unwrap().clone();
        match value {
            Value::Atomic(Atomic::Absent) => Err(Error::XPDY0002 {
                src: self.program.src.clone(),
                span: (0, self.program.src.len()).into(),
            }),
            Value::Atomic(Atomic::Empty) => Ok(Value::Sequence(Sequence::empty())),
            _ => Ok(value),
        }
    }

    pub fn run_output(
        &self,
        dynamic_context: &DynamicContext,
        context_item: Option<&OutputItem>,
    ) -> Result<Vec<OutputItem>> {
        let context_item: Option<Item> = context_item.map(|item| item.clone().into());
        let value = self.run(dynamic_context, context_item.as_ref())?;
        Ok(value.into_output_items())
    }

    pub fn run_xot_node(&self, dynamic_context: &DynamicContext, node: xot::Node) -> Result<Value> {
        self.run(dynamic_context, Some(&Item::Node(Node::Xot(node))))
    }

    pub fn many(
        &self,
        dynamic_context: &DynamicContext,
        item: &OutputItem,
    ) -> Result<Vec<OutputItem>> {
        self.run_output(dynamic_context, Some(item))
    }

    pub fn one(&self, dynamic_context: &DynamicContext, item: &OutputItem) -> Result<OutputItem> {
        let mut items = self.run_output(dynamic_context, Some(item))?;
        Ok(if items.len() == 1 {
            items.pop().unwrap()
        } else {
            return Err(Error::XPTY0004 {
                src: self.program.src.clone(),
                span: (0, 0).into(),
            });
        })
    }

    pub fn option(
        &self,
        dynamic_context: &DynamicContext,
        item: &OutputItem,
    ) -> Result<Option<OutputItem>> {
        let mut items = self.run_output(dynamic_context, Some(item))?;

        Ok(if items.is_empty() {
            None
        } else if items.len() == 1 {
            Some(items.pop().unwrap())
        } else {
            return Err(Error::XPTY0004 {
                src: self.program.src.clone(),
                span: (0, 0).into(),
            });
        })
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

    use xee_xpath_ast::{Namespaces, FN_NAMESPACE};

    use crate::{
        context::StaticContext,
        document::{Documents, Uri},
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
