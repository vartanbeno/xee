use std::rc::Rc;

use crate::context::DynamicContext;

use crate::function;
use crate::xml;
use crate::{error, Collation};

const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone)]
pub(crate) struct Runnable<'a> {
    program: &'a function::Program,
    // TODO: this should be private, but is needed right now
    // to implement call_static without lifetime issues
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

    pub(crate) fn program(&self) -> &function::Program {
        self.program
    }

    pub(crate) fn dynamic_context(&self) -> &DynamicContext {
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
    ) -> &function::InlineFunction {
        &self.program.functions[function_id.0]
    }

    pub(crate) fn static_function(
        &self,
        function_id: function::StaticFunctionId,
    ) -> &function::StaticFunction {
        self.dynamic_context
            .static_context
            .functions
            .get_by_index(function_id)
    }
}
