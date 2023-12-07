use crate::context;
use crate::error;
use crate::function;
use crate::ir;

use super::builder::FunctionBuilder;
use super::ir_interpret::{InterpreterCompiler, Scopes};
use super::Runnable;

#[derive(Debug, Clone)]
pub struct Program {
    pub(crate) src: String,
    pub(crate) functions: Vec<function::InlineFunction>,
}

impl Program {
    pub fn new(static_context: &context::StaticContext, xpath: &str) -> error::SpannedResult<Self> {
        let ast = static_context.parse_xpath(xpath)?;
        let mut ir_converter = ir::IrConverter::new(xpath, static_context);
        let expr = ir_converter.convert_xpath(&ast)?;
        // this expression contains a function definition, we're getting it
        // in the end
        let mut program = Program::empty(xpath.to_string());
        let mut scopes = Scopes::new(ir::Name("dummy".to_string()));
        let builder = FunctionBuilder::new(&mut program);
        let mut compiler = InterpreterCompiler {
            builder,
            scopes: &mut scopes,
            static_context,
        };
        compiler.compile_expr(&expr)?;

        Ok(program)
    }

    pub(crate) fn empty(src: String) -> Self {
        Program {
            src,
            functions: Vec::new(),
        }
    }

    pub fn runnable<'a>(&'a self, dynamic_context: &'a context::DynamicContext) -> Runnable<'a> {
        Runnable::new(self, dynamic_context)
    }

    pub(crate) fn add_function(
        &mut self,
        function: function::InlineFunction,
    ) -> function::InlineFunctionId {
        let id = self.functions.len();
        if id > u16::MAX as usize {
            panic!("too many functions");
        }
        self.functions.push(function);

        function::InlineFunctionId(id)
    }

    pub(crate) fn get_function(&self, index: usize) -> &function::InlineFunction {
        &self.functions[index]
    }

    pub(crate) fn get_function_by_id(
        &self,
        id: function::InlineFunctionId,
    ) -> &function::InlineFunction {
        self.get_function(id.0)
    }

    pub(crate) fn main_id(&self) -> function::InlineFunctionId {
        function::InlineFunctionId(self.functions.len() - 1)
    }
}
