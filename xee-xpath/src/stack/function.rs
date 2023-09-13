use miette::SourceSpan;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::ir;
use crate::sequence;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct InlineFunctionId(pub(crate) usize);

impl InlineFunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StaticFunctionId(pub(crate) usize);

impl StaticFunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CastType {
    pub(crate) xs: Xs,
    pub(crate) empty_sequence_allowed: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct InlineFunction {
    pub(crate) name: String,
    pub(crate) params: Vec<ir::Param>,
    // things referenced by instructions (by index)
    pub(crate) constants: Vec<stack::Value>,
    pub(crate) steps: Vec<xml::Step>,
    pub(crate) cast_types: Vec<CastType>,
    pub(crate) sequence_types: Vec<ast::SequenceType>,
    pub(crate) closure_names: Vec<ir::Name>,
    // the compiled code, and the spans of each instruction
    pub(crate) chunk: Vec<u8>,
    pub(crate) spans: Vec<SourceSpan>,
}

pub(crate) struct Coercion {
    pub(crate) parameter_types: Vec<ast::SequenceType>,
    pub(crate) return_type: Vec<ast::SequenceType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Closure {
    Static {
        static_function_id: StaticFunctionId,
        sequences: Vec<sequence::Sequence>,
    },
    Inline {
        inline_function_id: InlineFunctionId,
        sequences: Vec<sequence::Sequence>,
    },
}

impl Closure {
    pub(crate) fn sequences(&self) -> &[sequence::Sequence] {
        match self {
            Self::Static { sequences, .. } => sequences,
            Self::Inline { sequences, .. } => sequences,
        }
    }
}
