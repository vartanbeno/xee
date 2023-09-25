use std::cmp::Ordering;
use std::rc::Rc;

use ibig::IBig;
use miette::SourceSpan;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::atomic::{self, AtomicCompare};
use crate::atomic::{
    op_add, op_div, op_idiv, op_mod, op_multiply, op_subtract, OpEq, OpGe, OpGt, OpLe, OpLt, OpNe,
};
use crate::context::{self, DynamicContext};
use crate::error::Error;
use crate::function;
use crate::occurrence::Occurrence;
use crate::sequence;
use crate::stack;
use crate::xml;
use crate::{error, Collation};

use super::instruction::{read_i16, read_instruction, read_u16, read_u8, EncodedInstruction};
use super::state::State;

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
