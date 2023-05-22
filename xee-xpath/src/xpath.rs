use std::cell::RefCell;
use std::rc::Rc;

use crate::ast_ir::IrConverter;
use crate::builder::{FunctionBuilder, Program};
use crate::dynamic_context::DynamicContext;
use crate::error::{Error, Result};
use crate::interpret::Interpreter;
use crate::ir_interpret::{InterpreterCompiler, Scopes};
use crate::parse_ast::parse_xpath;
use crate::static_context::StaticContext;
use crate::value::{Atomic, FunctionId, Item, Node, StackValue};
use crate::{ir, Sequence};

#[derive(Debug)]
pub struct XPath {
    pub(crate) program: Program,
    main: FunctionId,
}

impl XPath {
    pub fn new(static_context: &StaticContext, xpath: &str) -> Result<Self> {
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
            main: inline_id,
        })
    }

    pub fn run(
        &self,
        dynamic_context: &DynamicContext,
        context_item: Option<&Item>,
    ) -> Result<StackValue> {
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
            StackValue::Atomic(Atomic::Absent) => Err(Error::XPDY0002 {
                src: self.program.src.clone(),
                span: (0, self.program.src.len()).into(),
            }),
            StackValue::Atomic(Atomic::Empty) => {
                Ok(StackValue::Sequence(Rc::new(RefCell::new(Sequence::new()))))
            }
            _ => Ok(value),
        }
    }

    pub fn run_xot_node(
        &self,
        dynamic_context: &DynamicContext,
        node: xot::Node,
    ) -> Result<StackValue> {
        self.run(dynamic_context, Some(&Item::Node(Node::Xot(node))))
    }

    pub fn many(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<Vec<Item>> {
        let stack_value = self.run(dynamic_context, Some(item))?;
        match stack_value {
            // XXX this clone here is not great
            StackValue::Sequence(seq) => Ok(seq.borrow().items.clone()),
            StackValue::Node(node) => Ok(vec![Item::Node(node)]),
            StackValue::Atomic(atomic) => Ok(vec![Item::Atomic(atomic)]),
            StackValue::Closure(closure) => Ok(vec![Item::Function(closure)]),
            StackValue::Step(..) => panic!("step not expected"),
        }
    }

    pub fn one(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<Item> {
        let stack_value = self.run(dynamic_context, Some(item))?;
        match stack_value {
            StackValue::Sequence(seq) => {
                let borrowed = seq.borrow();
                let value = borrowed.singleton();
                match value {
                    Ok(value) => Ok(value.clone()),
                    Err(_) => Err(Error::XPTY0004 {
                        src: self.program.src.clone(),
                        span: (0, 0).into(),
                    }),
                }
            }
            StackValue::Node(node) => Ok(Item::Node(node)),
            StackValue::Atomic(atomic) => Ok(Item::Atomic(atomic)),
            StackValue::Closure(closure) => Ok(Item::Function(closure)),
            StackValue::Step(..) => panic!("step not expected"),
        }
    }

    pub fn option(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<Option<Item>> {
        let stack_value = self.run(dynamic_context, Some(item))?;
        match stack_value {
            StackValue::Sequence(seq) => {
                let borrowed = seq.borrow();
                let value = borrowed.singleton();
                // XXX not ideal that we turn an error back
                // into an option, perhaps
                match value {
                    Ok(value) => Ok(Some(value.clone())),
                    Err(_) => Ok(None),
                }
            }
            StackValue::Node(node) => Ok(Some(Item::Node(node))),
            StackValue::Atomic(atomic) => Ok(Some(Item::Atomic(atomic))),
            StackValue::Closure(closure) => Ok(Some(Item::Function(closure))),
            StackValue::Step(..) => panic!("step not expected"),
        }
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
