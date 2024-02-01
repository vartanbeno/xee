use crate::context;
use crate::declaration::Declarations;
use crate::function;
use xee_xpath_ast::ast::Span;

use super::Runnable;

#[derive(Debug)]
pub struct Program {
    span: Span,
    pub functions: Vec<function::InlineFunction>,
    pub declarations: Declarations,
}

impl Program {
    pub fn new(span: Span) -> Self {
        Program {
            span,
            functions: Vec::new(),
            declarations: Declarations::new(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    /// Obtain a runnable version of this program, with a particular dynamic context.
    pub fn runnable<'a>(&'a self, dynamic_context: &'a context::DynamicContext) -> Runnable<'a> {
        Runnable::new(self, dynamic_context)
    }

    pub fn add_function(
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
