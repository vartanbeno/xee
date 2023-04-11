use ahash::{HashMap, HashMapExt};

use crate::ast;
// use crate::interpret::Operation;
// use crate::interpret::{Interpreter, Result, StackEntry};
use crate::error::Result;
use crate::interpret2::{
    Comparison, FunctionBuilder, FunctionId, Instruction, Interpreter, Program, Value,
};
use crate::parse_ast::parse_xpath;

fn compile_xpath(xpath: &ast::XPath, scope: &mut Scope, builder: &mut FunctionBuilder) {
    compile_expr(&xpath.exprs, scope, builder);
}

fn compile_expr(exprs: &[ast::ExprSingle], scope: &mut Scope, builder: &mut FunctionBuilder) {
    let mut iter = exprs.iter();
    let first_expr = iter.next().unwrap();
    compile_expr_single(first_expr, scope, builder);

    for expr in iter {
        compile_expr_single(expr, scope, builder);
        // operations.push(Operation::Comma);
    }
}

struct Scope {
    names: Vec<ast::Name>,
}

impl Scope {
    fn new() -> Self {
        Self { names: Vec::new() }
    }

    fn get(&self, name: &ast::Name) -> Option<usize> {
        for (i, n) in self.names.iter().enumerate().rev() {
            if n == name {
                return Some(i);
            }
        }
        None
    }
}

fn compile_expr_single(
    expr_single: &ast::ExprSingle,
    scope: &mut Scope,
    builder: &mut FunctionBuilder,
) {
    match expr_single {
        ast::ExprSingle::Path(path_expr) => {
            compile_path_expr(path_expr, scope, builder);
        }
        ast::ExprSingle::Binary(binary_expr) => {
            compile_path_expr(&binary_expr.left, scope, builder);
            compile_path_expr(&binary_expr.right, scope, builder);
            match binary_expr.operator {
                ast::Operator::Add => {
                    builder.emit(Instruction::Add);
                }
                ast::Operator::Sub => {
                    builder.emit(Instruction::Sub);
                }
                ast::Operator::ValueEq => builder.emit_compare_value(Comparison::Eq),
                ast::Operator::ValueNe => builder.emit_compare_value(Comparison::Ne),
                ast::Operator::ValueLt => builder.emit_compare_value(Comparison::Lt),
                ast::Operator::ValueLe => builder.emit_compare_value(Comparison::Le),
                ast::Operator::ValueGt => builder.emit_compare_value(Comparison::Gt),
                ast::Operator::ValueGe => builder.emit_compare_value(Comparison::Ge),
                // ast::Operator::ValueNe => {
                //     operations.push(Operation::ValueNe);
                // }
                // ast::Operator::ValueLt => {
                //     operations.push(Operation::ValueLt);
                // }
                // ast::Operator::ValueLe => {
                //     operations.push(Operation::ValueLe);
                // }
                // ast::Operator::ValueGt => {
                //     operations.push(Operation::ValueGt);
                // }
                // ast::Operator::ValueGe => {
                //     operations.push(Operation::ValueGe);
                // }
                // ast::Operator::Concat => {
                //     operations.push(Operation::Concat);
                // }
                ast::Operator::Range => {
                    // // left and right of range are on the stack
                    // operations.push(Operation::Peek(2));
                    // operations.push(Operation::Peek(1));
                    // operations.push(Operation::ValueEq);

                    // let jump_if_equal_index = operations.len();
                    // operations.push(Operation::JumpIfFalse(0));

                    // operations.push(Operation::Peek(2));
                    // operations.push(Operation::Peek(1));
                    // operations.push(Operation::ValueGt);

                    // let jump_if_greater_index = operations.len();
                    // operations.push(Operation::JumpIfFalse(0));

                    // // left and right are equal: we can just return left
                    // operations.push(Operation::Pop);
                    // let equal_jump_to_end = operations.len();
                    // operations.push(Operation::Jump(0));

                    // // if left is greater than right, push empty sequence on stack
                    // operations.push(Operation::SequenceNew);
                    // let greater_jump_to_end = operations.len();
                    // operations.push(Operation::Jump(0));

                    // // left is less than right: we need to create a sequence
                    // let sequence_index = operations.len();
                    // operations.push(Operation::SequenceNew);
                    // // start index
                    // operations.push(Operation::Peek(3));
                    // operations.push(Operation::Dup);
                    // operations.push(Operation::SequencePush(sequence_index));
                    // // if start is at end, we're done

                    // let end = operations.len();
                    // otherwise, we need to create a new sequence
                    // operations.push(Operation::NewSequence);

                    // start with left of range

                    // add to range
                }
                _ => {
                    panic!("operator supported yet {:?}", binary_expr.operator);
                }
            }
        }
        ast::ExprSingle::Let(let_expr) => {
            // XXX ugh clone
            scope.names.push(let_expr.var_name.clone());
            compile_expr_single(&let_expr.var_expr, scope, builder);
            compile_expr_single(&let_expr.return_expr, scope, builder);
            builder.emit(Instruction::LetDone);
            scope.names.pop();
        }
        ast::ExprSingle::If(if_expr) => {
            compile_expr(&if_expr.condition, scope, builder);
            let jump_else = builder.emit_test_forward();
            compile_expr_single(&if_expr.then, scope, builder);
            let jump_end = builder.emit_jump_forward();
            builder.patch_jump(jump_else);
            compile_expr_single(&if_expr.else_, scope, builder);
            builder.patch_jump(jump_end);
        }
        ast::ExprSingle::For(for_expr) => {
            // operations.push(Operation::NewSequence);
            // // execute the sequence expression, placing sequence on stack
            // compile_expr_single(&for_expr.var_expr, scope, operations);
            // // we get the total length of the sequence
            // operations.push(Operation::LenSequence);
            // // the index in the sequence, start at 0
            // operations.push(Operation::IntegerLiteral(0));

            // // we know the result of the next expression is going to be placed
            // // here on the stack
            // let name_index = operations.len();
            // // XXX ugh clone
            // scope.push_name(for_expr.var_name.clone(), name_index);

            // // now we take the first entry in the sequence, place it on the stack
            // operations.push(Operation::IndexSequence);

            // compile_expr_single(&for_expr.return_expr, scope, operations);
            // scope.pop_name(&for_expr.var_name);
        }
        _ => {
            panic!("not supported yet");
        }
    }
}

fn compile_path_expr(path_expr: &ast::PathExpr, scope: &mut Scope, builder: &mut FunctionBuilder) {
    let first_step = &path_expr.steps[0];
    compile_step_expr(first_step, scope, builder);
}

fn compile_step_expr(step_expr: &ast::StepExpr, scope: &mut Scope, builder: &mut FunctionBuilder) {
    match step_expr {
        ast::StepExpr::PrimaryExpr(primary_expr) => {
            compile_primary_expr(primary_expr, scope, builder);
        }
        ast::StepExpr::PostfixExpr { primary, postfixes } => {
            compile_primary_expr(primary, scope, builder);
            compile_postfixes(postfixes, scope, builder);
        }
        _ => {
            panic!("not supported yet");
        }
    }
}

fn compile_primary_expr(
    primary_expr: &ast::PrimaryExpr,
    scope: &mut Scope,
    builder: &mut FunctionBuilder,
) {
    match primary_expr {
        ast::PrimaryExpr::Literal(literal) => match literal {
            ast::Literal::Integer(i) => {
                builder.emit_constant(Value::Integer(*i));
            }
            // ast::Literal::String(s) => {
            //     operations.push(Operation::StringLiteral(s.to_string()));
            // }
            _ => {
                panic!("literal type not supported yet");
            }
        },
        ast::PrimaryExpr::Expr(exprs) => {
            compile_expr(exprs, scope, builder);
        }
        ast::PrimaryExpr::VarRef(name) => {
            let index = scope.get(name).unwrap();
            // XXX check for max
            builder.emit(Instruction::Var(index as u16));
        }
        ast::PrimaryExpr::InlineFunction(inline_function) => {
            let mut nested_builder = builder.builder();
            let mut nested_scope = Scope::new();
            compile_function(inline_function, &mut nested_scope, &mut nested_builder);
            let function =
                nested_builder.finish("inline".to_string(), inline_function.params.len());
            let function_id = builder.add_function(function);
            builder.emit(Instruction::Function(function_id.as_u16()));
        }
        _ => {
            panic!("not supported yet");
        }
    }
}

fn compile_postfixes(postfixes: &[ast::Postfix], scope: &mut Scope, builder: &mut FunctionBuilder) {
    for postfix in postfixes {
        match postfix {
            ast::Postfix::ArgumentList(arguments) => {
                for argument in arguments {
                    compile_argument(argument, scope, builder);
                }
                builder.emit(Instruction::Call(arguments.len() as u8));
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }
}

fn compile_argument(argument: &ast::Argument, scope: &mut Scope, builder: &mut FunctionBuilder) {
    match argument {
        ast::Argument::Expr(expr_single) => {
            compile_expr_single(expr_single, scope, builder);
        }
        _ => {
            panic!("not supported yet");
        }
    }
}

fn compile_function(
    function: &ast::InlineFunction,
    scope: &mut Scope,
    builder: &mut FunctionBuilder,
) {
    for param in &function.params {
        scope.names.push(param.name.clone());
    }
    compile_expr(&function.body, scope, builder);
    for _ in &function.params {
        scope.names.pop();
    }
}

pub(crate) struct CompiledXPath {
    program: Program,
    main: FunctionId,
}

impl<'a> CompiledXPath {
    pub(crate) fn new(xpath: &str) -> Self {
        let ast = parse_xpath(xpath);
        let mut program = Program::new();
        let mut scope = Scope::new();
        let mut builder = FunctionBuilder::new(&mut program);
        compile_xpath(&ast, &mut scope, &mut builder);
        let main = builder.finish("main".to_string(), 0);
        let main = program.add_function(main);
        Self { program, main }
    }

    pub(crate) fn interpret(&self) -> Result<Value> {
        let mut interpreter = Interpreter::new(&self.program);
        interpreter.start(self.main);
        interpreter.run()?;
        // the stack has to be 1 value, as we return the result of the expression
        assert_eq!(
            interpreter.stack().len(),
            1,
            "stack must only have 1 value but found {:?}",
            interpreter.stack()
        );
        Ok(interpreter.stack().last().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::interpret::{Atomic, Item, Result, Sequence};

    #[test]
    fn test_compile_expr_single() -> Result<()> {
        let xpath = CompiledXPath::new("1 + 2");

        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 3);
        Ok(())
    }

    // #[test]
    // fn test_string_concat() -> Result<()> {
    //     let xpath = CompiledXPath::new("'a' || 'b'");
    //     let result = xpath.interpret()?;
    //     assert_eq!(result.as_string()?, "ab");
    //     Ok(())
    // }

    #[test]
    fn test_nested() -> Result<()> {
        let xpath = CompiledXPath::new("1 + (8 - 2)");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 7);
        Ok(())
    }

    // #[test]
    // fn test_comma() -> Result<()> {
    //     let xpath = CompiledXPath::new("1, 2");
    //     let result = xpath.interpret()?;
    //     assert_eq!(
    //         result.as_sequence()?,
    //         Sequence(vec![
    //             Item::AtomicValue(Atomic::Integer(1)),
    //             Item::AtomicValue(Atomic::Integer(2))
    //         ])
    //     );
    //     Ok(())
    // }

    #[test]
    fn test_let() -> Result<()> {
        let xpath = CompiledXPath::new("let $x := 1 return $x + 2");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 3);
        Ok(())
    }

    #[test]
    fn test_let_nested() -> Result<()> {
        let xpath = CompiledXPath::new("let $x := 1, $y := $x + 3 return $y + 5");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 9);
        Ok(())
    }

    #[test]
    fn test_if() -> Result<()> {
        let xpath = CompiledXPath::new("if (1) then 2 else 3");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 2);
        Ok(())
    }

    #[test]
    fn test_if_false() -> Result<()> {
        let xpath = CompiledXPath::new("if (0) then 2 else 3");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 3);
        Ok(())
    }

    #[test]
    fn test_value_eq_true() -> Result<()> {
        let xpath = CompiledXPath::new("1 eq 1");
        let result = xpath.interpret()?;
        assert!(result.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_value_eq_false() -> Result<()> {
        let xpath = CompiledXPath::new("1 eq 2");
        let result = xpath.interpret()?;
        assert!(!result.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_value_ne_true() -> Result<()> {
        let xpath = CompiledXPath::new("1 ne 2");
        let result = xpath.interpret()?;
        assert!(result.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_value_ne_false() -> Result<()> {
        let xpath = CompiledXPath::new("1 ne 1");
        let result = xpath.interpret()?;
        assert!(!result.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_value_lt_true() -> Result<()> {
        let xpath = CompiledXPath::new("1 lt 2");
        let result = xpath.interpret()?;
        assert!(result.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_value_lt_false() -> Result<()> {
        let xpath = CompiledXPath::new("2 lt 1");
        let result = xpath.interpret()?;
        assert!(!result.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_function_without_args() -> Result<()> {
        let xpath = CompiledXPath::new("function() { 5 } ()");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 5);
        Ok(())
    }

    #[test]
    fn test_function_with_args() -> Result<()> {
        let xpath = CompiledXPath::new("function($x) { $x + 5 } ( 3 )");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 8);
        Ok(())
    }

    #[test]
    fn test_function_with_args2() -> Result<()> {
        let xpath = CompiledXPath::new("function($x, $y) { $x + $y } ( 3, 5 )");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 8);
        Ok(())
    }

    #[test]
    fn test_function_nested() -> Result<()> {
        let xpath = CompiledXPath::new("function($x) { function($y) { $y + 2 }($x + 1) } (5)");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 8);
        Ok(())
    }
}
