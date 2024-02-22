use ahash::HashMapExt;
use xee_interpreter::{context::StaticContext, error::SpannedResult, interpreter::Program};

use crate::{
    declaration_compiler::{DeclarationCompiler, ModeIds},
    ir, FunctionBuilder, FunctionCompiler, Scopes,
};

pub fn compile_xpath(expr: ir::ExprS, static_context: &StaticContext) -> SpannedResult<Program> {
    let mut program = Program::new(expr.span);
    let mut scopes = Scopes::new();
    let builder = FunctionBuilder::new(&mut program);
    let empty_mode_ids = ModeIds::new();
    let mut compiler = FunctionCompiler::new(builder, &mut scopes, static_context, &empty_mode_ids);
    compiler.compile_expr(&expr)?;
    Ok(program)
}

pub fn compile_xslt(
    declarations: ir::Declarations,
    static_context: &StaticContext,
) -> SpannedResult<Program> {
    let mut program = Program::new((0..0).into());
    let mut compiler = DeclarationCompiler::new(&mut program, static_context);
    compiler.compile_declarations(&declarations)?;
    Ok(program)
}
