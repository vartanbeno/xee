use crate::ast;
use crate::interpret::Operation;

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
        if let ast::PrimaryExpr::Literal(literal) = primary_expr {
            if let ast::Literal::Integer(i) = literal {
                operations.push(Operation::IntegerLiteral(*i));
            } else {
                panic!("literal not an integer literal");
            }
        } else {
            panic!("primary expression not a literal");
        }
    } else {
        panic!("not a primary expression");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::Interpreter;
    use crate::parse_ast::parse_expr_single;

    #[test]
    fn test_compile_expr_single() {
        let expr_single = parse_expr_single("1 + 2");
        let mut operations = Vec::new();
        compile_expr_single(&expr_single, &mut operations);
        assert_eq!(operations.len(), 3);
        assert_eq!(operations[0], Operation::IntegerLiteral(1));
        assert_eq!(operations[1], Operation::IntegerLiteral(2));
        assert_eq!(operations[2], Operation::Add);

        let mut interpreter = Interpreter::new();
        interpreter.interpret(&operations);
        assert_eq!(interpreter.stack.pop().unwrap().as_integer(), 3);
    }
}
