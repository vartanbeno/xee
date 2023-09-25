use miette::SourceSpan;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::ir;
use crate::stack;
use crate::xml;

use super::signature::Signature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CastType {
    pub(crate) xs: Xs,
    pub(crate) empty_sequence_allowed: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct InlineFunction {
    pub(crate) name: String,
    pub(crate) signature: Signature,
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

impl InlineFunction {
    pub(crate) fn signature(&self) -> &Signature {
        &self.signature
    }

    pub(crate) fn arity(&self) -> usize {
        self.signature.parameter_types.len()
    }
}
