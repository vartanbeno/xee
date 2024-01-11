mod ast_ir;
mod ir_core;
mod span;

#[cfg(test)]
pub(crate) use ast_ir::convert_xpath;
pub(crate) use ast_ir::IrConverter;
pub(crate) use ir_core::*;
