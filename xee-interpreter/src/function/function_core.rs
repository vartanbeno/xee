use crate::sequence;

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
        closure_vars: Vec<sequence::Sequence>,
    },
    Inline {
        inline_function_id: InlineFunctionId,
        closure_vars: Vec<sequence::Sequence>,
    },
    Map(Map),
    Array(Array),
}

impl Function {
    pub(crate) fn closure_vars(&self) -> &[sequence::Sequence] {
        match self {
            Self::Static { closure_vars, .. } => closure_vars,
            Self::Inline { closure_vars, .. } => closure_vars,
            _ => unreachable!(),
        }
    }

    pub fn display_representation(&self, xot: &xot::Xot) -> String {
        match self {
            Self::Static {
                static_function_id, ..
            } => todo!(),
            Self::Inline {
                inline_function_id, ..
            } => todo!(),
            Self::Map(map) => format!("map {}", map.display_representation(xot)),
            Self::Array(array) => format!("array {}", array.display_representation(xot)),
        }
    }
}
