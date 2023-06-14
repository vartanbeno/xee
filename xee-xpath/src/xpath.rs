use xee_xpath_ast::ast::parse_xpath;

use crate::context::{DynamicContext, StaticContext};
use crate::error::{Error, Result};
use crate::interpreter::{FunctionBuilder, Interpreter, InterpreterCompiler, Program, Scopes};
use crate::ir;
use crate::ir::IrConverter;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug)]
pub struct XPath {
    pub(crate) program: Program,
    main: stack::FunctionId,
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
        let inline_id = stack::FunctionId(program.functions.len() - 1);
        Ok(Self {
            program,
            main: inline_id,
        })
    }

    pub(crate) fn run_value(
        &self,
        dynamic_context: &DynamicContext,
        context_item: Option<&stack::StackItem>,
    ) -> Result<stack::StackValue> {
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
            stack::StackValue::Atomic(stack::Atomic::Absent) => Err(Error::XPDY0002 {
                src: self.program.src.clone(),
                span: (0, self.program.src.len()).into(),
            }),
            stack::StackValue::Atomic(stack::Atomic::Empty) => {
                Ok(stack::StackValue::Sequence(stack::StackSequence::empty()))
            }
            _ => Ok(value),
        }
    }

    pub fn many_xot_node(
        &self,
        dynamic_context: &DynamicContext,
        node: xot::Node,
    ) -> Result<output::OutputSequence> {
        self.many(
            dynamic_context,
            Some(&output::OutputItem::Node(xml::Node::Xot(node))),
        )
    }

    pub fn many(
        &self,
        dynamic_context: &DynamicContext,
        item: Option<&output::OutputItem>,
    ) -> Result<output::OutputSequence> {
        let context_item: Option<stack::StackItem> = item.map(|item| item.clone().into());
        let value = self.run_value(dynamic_context, context_item.as_ref())?;
        Ok(value.into_output_sequence())
    }

    pub fn one(
        &self,
        dynamic_context: &DynamicContext,
        item: Option<&output::OutputItem>,
    ) -> Result<output::OutputItem> {
        let sequence = self.many(dynamic_context, item)?;
        let items = sequence.items();
        Ok(if items.len() == 1 {
            items[0].clone()
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
        item: Option<&output::OutputItem>,
    ) -> Result<Option<output::OutputItem>> {
        let sequence = self.many(dynamic_context, item)?;
        let items = sequence.items();

        Ok(if items.is_empty() {
            None
        } else if items.len() == 1 {
            Some(items[0].clone())
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
    use super::*;

    use xee_xpath_ast::{Namespaces, FN_NAMESPACE};
    use xot::Xot;

    use crate::context::StaticContext;

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
