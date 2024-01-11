use xee_interpreter::{context, error, interpreter::Program};
use xee_xpath_ast::ast;

use crate::ir;

use super::builder::FunctionBuilder;
use super::ir_interpret::InterpreterCompiler;
use super::scope::Scopes;

/// Construct a program from an XPath AST.
pub fn compile(
    static_context: &context::StaticContext,
    xpath: ast::XPath,
) -> error::SpannedResult<Program> {
    let mut ir_converter = ir::IrConverter::new(static_context);
    let expr = ir_converter.convert_xpath(&xpath)?;
    // this expression contains a function definition, we're getting it
    // in the end
    let mut program = Program {
        xpath,
        functions: Vec::new(),
    };
    let mut scopes = Scopes::new(ir::Name::new("dummy".to_string()));
    let builder = FunctionBuilder::new(&mut program);
    let mut compiler = InterpreterCompiler {
        builder,
        scopes: &mut scopes,
        static_context,
    };
    compiler.compile_expr(&expr)?;

    Ok(program)
}

/// Parse an XPath string into a program.
pub fn parse(
    static_context: &context::StaticContext,
    xpath: &str,
) -> error::SpannedResult<Program> {
    let xpath = static_context.parse_xpath(xpath)?;
    compile(static_context, xpath)
}

pub fn convert_ir(
    static_context: &context::StaticContext,
    xpath: &str,
) -> error::SpannedResult<ir::ExprS> {
    let ast = static_context.parse_xpath(xpath)?;
    let mut converter = ir::IrConverter::new(static_context);
    converter.convert_xpath(&ast)
}
