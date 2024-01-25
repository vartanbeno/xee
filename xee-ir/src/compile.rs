use xee_interpreter::{context::StaticContext, error::SpannedResult, interpreter::Program};

use crate::{ir, FunctionBuilder, InterpreterCompiler, Scopes};

pub fn compile_xpath(expr: ir::ExprS, static_context: &StaticContext) -> SpannedResult<Program> {
    let mut program = Program::new(expr.span);
    let mut scopes = Scopes::new();
    let builder = FunctionBuilder::new(&mut program);
    let mut compiler = InterpreterCompiler::new(builder, &mut scopes, static_context);
    compiler.compile_expr(&expr)?;
    Ok(program)
}
