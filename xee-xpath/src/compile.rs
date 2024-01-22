use xee_interpreter::{context, error, interpreter::Program};
use xee_ir::{FunctionBuilder, InterpreterCompiler, Scopes};
use xee_xpath_ast::ast;

use crate::ast_ir::IrConverter;

/// Construct a program from an XPath AST.
pub fn compile(
    static_context: &context::StaticContext,
    xpath: ast::XPath,
) -> error::SpannedResult<Program> {
    let mut ir_converter = IrConverter::new(static_context);
    let expr = ir_converter.convert_xpath(&xpath)?;
    // this expression contains a function definition, we're getting it
    // in the end
    let mut program = Program::new(xpath.0.span);
    let mut scopes = Scopes::new();
    let builder = FunctionBuilder::new(&mut program);
    let mut compiler = InterpreterCompiler::new(builder, &mut scopes, static_context);
    compiler.compile_expr(&expr)?;

    Ok(program)
}

// /// Construct a function within an existing program
// pub fn compile_function(
//     program: &mut Program,
//     static_context: &context::StaticContext,
//     xpath: ast::XPath,
// ) -> error::SpannedResult<function::InlineFunctionId> {
//     let mut ir_converter = IrConverter::new(static_context);
//     let expr = ir_converter.convert_xpath(&xpath)?;
//     let mut scopes = Scopes::new();
//     let builder = FunctionBuilder::new(program);
//     let mut compiler = InterpreterCompiler::new(builder, &mut scopes, static_context);
//     compiler.compile_function_id(&expr.value, expr.span)?;
//     Ok(())
// }

/// Parse an XPath string into a program.
pub fn parse(
    static_context: &context::StaticContext,
    xpath: &str,
) -> error::SpannedResult<Program> {
    let xpath = static_context.parse_xpath(xpath)?;
    compile(static_context, xpath)
}
