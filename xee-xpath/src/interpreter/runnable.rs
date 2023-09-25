use std::rc::Rc;

use xee_xpath_ast::ast;

use crate::context::DynamicContext;
use crate::function;
use crate::xml;
use crate::{error, Collation};

#[derive(Debug, Clone)]
pub(crate) struct Runnable<'a> {
    program: &'a function::Program,
    // TODO: this should be private, but is needed right now
    // to implement call_static without lifetime issues.
    // We could possibly obtain context from the interpreter directly,
    // but this leads to lifetime issues right now.
    pub(crate) dynamic_context: &'a DynamicContext<'a>,
}

impl<'a> Runnable<'a> {
    pub(crate) fn new(
        program: &'a function::Program,
        dynamic_context: &'a DynamicContext<'a>,
    ) -> Self {
        Self {
            program,
            dynamic_context,
        }
    }

    pub(crate) fn program(&self) -> &'a function::Program {
        self.program
    }

    pub(crate) fn dynamic_context(&self) -> &'a DynamicContext {
        self.dynamic_context
    }

    pub(crate) fn annotations(&self) -> &xml::Annotations {
        &self.dynamic_context.documents.annotations
    }

    pub(crate) fn xot(&self) -> &xot::Xot {
        self.dynamic_context.xot
    }

    pub(crate) fn default_collation(&self) -> error::Result<Rc<Collation>> {
        self.dynamic_context.static_context.default_collation()
    }

    pub(crate) fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.dynamic_context.implicit_timezone()
    }

    pub(crate) fn inline_function(
        &self,
        function_id: function::InlineFunctionId,
    ) -> &'a function::InlineFunction {
        &self.program.functions[function_id.0]
    }

    pub(crate) fn static_function(
        &self,
        function_id: function::StaticFunctionId,
    ) -> &'a function::StaticFunction {
        self.dynamic_context
            .static_context
            .functions
            .get_by_index(function_id)
    }

    pub(crate) fn function_info(&'a self, function: &'a function::Function) -> FunctionInfo<'a> {
        FunctionInfo::new(function, self)
    }
}

pub(crate) struct FunctionInfo<'a> {
    function: &'a function::Function,
    runnable: &'a Runnable<'a>,
}

impl<'a> FunctionInfo<'a> {
    pub(crate) fn new(
        function: &'a function::Function,
        runnable: &'a Runnable<'a>,
    ) -> FunctionInfo<'a> {
        FunctionInfo { function, runnable }
    }

    pub(crate) fn arity(&self) -> usize {
        match self.function {
            function::Function::Inline {
                inline_function_id, ..
            } => self
                .runnable
                .inline_function(*inline_function_id)
                .params
                .len(),
            function::Function::Static {
                static_function_id, ..
            } => self.runnable.static_function(*static_function_id).arity(),
            function::Function::Array(_) => 1,
            function::Function::Map(_) => 1,
        }
    }

    pub(crate) fn name(&self) -> Option<ast::Name> {
        match self.function {
            function::Function::Static {
                static_function_id, ..
            } => {
                let static_function = self.runnable.static_function(*static_function_id);
                Some(static_function.name().clone())
            }
            _ => None,
        }
    }

    pub(crate) fn signature(&self) -> Option<ast::Signature> {
        match &self.function {
            function::Function::Static {
                static_function_id, ..
            } => {
                let _static_function = self.runnable.static_function(*static_function_id);
                // todo: modify so that we do have signature
                // Some(static_function.signature().clone())
                todo!()
            }
            function::Function::Inline {
                inline_function_id, ..
            } => {
                let _inline_function = self.runnable.inline_function(*inline_function_id);
                // there is a Signature defined next to inline function,
                // but it's not in use yet
                todo!()
            }
            function::Function::Map(_map) => {
                todo!()
            }
            function::Function::Array(_array) => {
                todo!()
            }
        }
    }

    // pub(crate) fn params(&self) -> &[function::Param] {
    //     match self.function {
    //         function::Function::Inline {
    //             inline_function_id, ..
    //         } => &self.program.functions[inline_function_id.0].params,
    //         function::Function::Static {
    //             static_function_id, ..
    //         } => {
    //             let static_function = self
    //                 .static_context
    //                 .functions
    //                 .get_by_index(static_function_id);
    //             static_function.params()
    //         }
    //         function::Function::Array(_) => &[],
    //         function::Function::Map(_) => &[],
    //     }
    // }
}
