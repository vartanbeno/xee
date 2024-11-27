use crate::{context, sequence, stack};

use super::array::Array;
use super::map::Map;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct InlineFunctionId(pub(crate) usize);

impl InlineFunctionId {
    pub fn new(id: usize) -> Self {
        InlineFunctionId(id)
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StaticFunctionId(pub(crate) usize);

impl StaticFunctionId {
    pub fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Static {
        static_function_id: StaticFunctionId,
        closure_vars: Vec<stack::Value>,
    },
    Inline {
        inline_function_id: InlineFunctionId,
        closure_vars: Vec<stack::Value>,
    },
    Map(Map),
    Array(Array),
}

impl Function {
    pub(crate) fn closure_vars(&self) -> &[stack::Value] {
        match self {
            Self::Static { closure_vars, .. } => closure_vars,
            Self::Inline { closure_vars, .. } => closure_vars,
            _ => unreachable!(),
        }
    }

    pub fn display_representation(
        &self,
        xot: &xot::Xot,
        context: &context::DynamicContext,
    ) -> String {
        match self {
            Self::Static {
                static_function_id, ..
            } => {
                let function = context.static_function_by_id(*static_function_id);
                function.display_representation()
            }
            Self::Inline {
                inline_function_id, ..
            } => {
                let function = context.inline_function_by_id(*inline_function_id);
                function.display_representation()
            }
            Self::Map(map) => map.display_representation(xot, context),
            Self::Array(array) => array.display_representation(xot, context),
        }
    }
}
