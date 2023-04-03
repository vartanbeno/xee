use crate::ast;
use crate::interpret::Operation;
use crate::interpret::{Interpreter, Result, StackEntry};
use crate::parse_ast::parse_expr_single;

fn compile_expr_single(expr_single: &ast::ExprSingle, operations: &mut Vec<Operation>) {
    match expr_single {
        ast::ExprSingle::Path(path_expr) => {
            compile_path_expr(path_expr, operations);
        }
        ast::ExprSingle::Binary(binary_expr) => {
            compile_path_expr(&binary_expr.left, operations);
            compile_path_expr(&binary_expr.right, operations);
            match binary_expr.operator {
                ast::Operator::Add => {
                    operations.push(Operation::Add);
                }
                ast::Operator::Sub => {
                    operations.push(Operation::Sub);
                }
                ast::Operator::Concat => {
                    operations.push(Operation::Concat);
                }
                _ => {
                    panic!("not supported yet");
                }
            }
        }
        _ => {
            panic!("not supported yet");
        }
    }
}

fn compile_path_expr(path_expr: &ast::PathExpr, operations: &mut Vec<Operation>) {
    let first_step = &path_expr.steps[0];
    if let ast::StepExpr::PrimaryExpr(primary_expr) = first_step {
        match primary_expr {
            ast::PrimaryExpr::Literal(literal) => match literal {
                ast::Literal::Integer(i) => {
                    operations.push(Operation::IntegerLiteral(*i));
                }
                ast::Literal::String(s) => {
                    operations.push(Operation::StringLiteral(s.to_string()));
                }
                _ => {
                    panic!("literal type not supported yet");
                }
            },
            ast::PrimaryExpr::Expr(expressions) => {
                // XXX doesn't really handle multiple expressions properly yet
                for expr in expressions {
                    compile_expr_single(expr, operations);
                }
            }
            _ => {
                panic!("not supported yet");
            }
        }
    } else {
        panic!("not a primary expression");
    }
}

pub(crate) struct CompiledExprSingle {
    operations: Vec<Operation>,
}

impl<'a> CompiledExprSingle {
    pub(crate) fn new(expr_single: &str) -> Self {
        let ast = parse_expr_single(expr_single);
        let mut operations = Vec::new();
        compile_expr_single(&ast, &mut operations);
        Self { operations }
    }

    pub(crate) fn interpret(&self) -> Result<StackEntry> {
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&self.operations)?;
        Ok(interpreter.stack.pop().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::Result;

    // fn execute(input: &str) -> StackEntry {
    //     let expr_single = parse_expr_single(input);
    //     let mut operations = Vec::new();
    //     compile_expr_single(&expr_single, &mut operations);
    //     let mut interpreter = Interpreter::new();
    //     interpreter.interpret(&operations);
    //     interpreter.stack.pop().unwrap()
    // }

    #[test]
    fn test_compile_expr_single() -> Result<()> {
        let expr_single = CompiledExprSingle::new("1 + 2");
        let operations = &expr_single.operations;
        assert_eq!(operations.len(), 3);
        assert_eq!(operations[0], Operation::IntegerLiteral(1));
        assert_eq!(operations[1], Operation::IntegerLiteral(2));
        assert_eq!(operations[2], Operation::Add);

        let result = expr_single.interpret()?;
        assert_eq!(result.as_integer()?, 3);
        Ok(())
    }

    #[test]
    fn test_string_concat() -> Result<()> {
        let expr_single = CompiledExprSingle::new("'a' || 'b'");
        let result = expr_single.interpret()?;
        assert_eq!(result.as_string()?, "ab");
        Ok(())
    }

    #[test]
    fn test_nested() -> Result<()> {
        let expr_single = CompiledExprSingle::new("1 + (8 - 2)");
        let result = expr_single.interpret()?;
        assert_eq!(result.as_integer()?, 7);
        Ok(())
    }
}
