use ahash::{HashMap, HashMapExt};

use crate::ast;
use crate::interpret::Operation;
use crate::interpret::{Interpreter, Result, StackEntry};
use crate::parse_ast::parse_xpath;

fn compile_xpath(xpath: &ast::XPath, scope: &mut Scope, operations: &mut Vec<Operation>) {
    compile_expr(&xpath.exprs, scope, operations);
}

fn compile_expr(exprs: &[ast::ExprSingle], scope: &mut Scope, operations: &mut Vec<Operation>) {
    let mut iter = exprs.iter();
    let first_expr = iter.next().unwrap();
    compile_expr_single(first_expr, scope, operations);

    for expr in iter {
        compile_expr_single(expr, scope, operations);
        operations.push(Operation::Comma);
    }
}

#[derive(Debug)]
struct Scope {
    name_stacks: HashMap<ast::Name, Vec<usize>>,
}

impl Scope {
    fn new() -> Self {
        Self {
            name_stacks: HashMap::new(),
        }
    }

    fn get(&self, name: &ast::Name) -> Option<usize> {
        let stack = self.name_stacks.get(name)?;
        stack.last().copied()
    }

    fn push_name(&mut self, name: ast::Name, index: usize) {
        let stack = self.name_stacks.entry(name).or_insert_with(Vec::new);
        stack.push(index);
    }

    fn pop_name(&mut self, name: &ast::Name) {
        let stack = self.name_stacks.get_mut(&name).unwrap();
        stack.pop();
    }
}

fn compile_expr_single(
    expr_single: &ast::ExprSingle,
    scope: &mut Scope,
    operations: &mut Vec<Operation>,
) {
    match expr_single {
        ast::ExprSingle::Path(path_expr) => {
            compile_path_expr(path_expr, scope, operations);
        }
        ast::ExprSingle::Binary(binary_expr) => {
            compile_path_expr(&binary_expr.left, scope, operations);
            compile_path_expr(&binary_expr.right, scope, operations);
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
                ast::Operator::Range => {
                    // left and right of range are on the stack

                    // if they are the same, push left on the stack

                    // if left is greater than right, push empty sequence on stack

                    // otherwise, we need to create a new sequence
                    // operations.push(Operation::NewSequence);

                    // start with left of range

                    // add to range
                }
                _ => {
                    panic!("not supported yet");
                }
            }
        }
        ast::ExprSingle::Let(let_expr) => {
            // we know the result of the next expression is going to be placed
            // here on the stack
            let index = operations.len();
            // XXX ugh clone
            scope.push_name(let_expr.var_name.clone(), index);
            compile_expr_single(&let_expr.var_expr, scope, operations);
            compile_expr_single(&let_expr.return_expr, scope, operations);
            operations.push(Operation::LetDone);
            scope.pop_name(&let_expr.var_name);
        }
        ast::ExprSingle::If(if_expr) => {
            compile_expr(&if_expr.condition, scope, operations);
            // temporary index, we can fill it in later once we've emitted
            // then
            let jump_else_index = operations.len();
            operations.push(Operation::JumpIfFalse(0));
            compile_expr_single(&if_expr.then, scope, operations);
            // temporary index, we fill in it later once we've emitted else
            let jump_end_index = operations.len();
            operations.push(Operation::Jump(0));
            // now we know the index of the else branch
            let else_index = operations.len();
            operations[jump_else_index] = Operation::JumpIfFalse(else_index);
            compile_expr_single(&if_expr.else_, scope, operations);
            // record the end of the whole if expression
            let end_index = operations.len();
            // go back and fill in the jump end target
            operations[jump_end_index] = Operation::Jump(end_index);
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

fn compile_path_expr(
    path_expr: &ast::PathExpr,
    scope: &mut Scope,
    operations: &mut Vec<Operation>,
) {
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
            ast::PrimaryExpr::Expr(exprs) => {
                compile_expr(exprs, scope, operations);
            }
            ast::PrimaryExpr::VarRef(name) => {
                let index = scope.get(name).unwrap();
                operations.push(Operation::VarRef(index));
            }
            _ => {
                panic!("not supported yet");
            }
        }
    } else {
        panic!("not a primary expression");
    }
}

pub(crate) struct CompiledXPath {
    operations: Vec<Operation>,
}

impl<'a> CompiledXPath {
    pub(crate) fn new(xpath: &str) -> Self {
        let ast = parse_xpath(xpath);
        let mut operations = Vec::new();
        let mut scope = Scope::new();
        compile_xpath(&ast, &mut scope, &mut operations);
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
    use crate::interpret::{Atomic, Item, Result, Sequence};

    #[test]
    fn test_compile_expr_single() -> Result<()> {
        let xpath = CompiledXPath::new("1 + 2");
        let operations = &xpath.operations;
        assert_eq!(operations.len(), 3);
        assert_eq!(operations[0], Operation::IntegerLiteral(1));
        assert_eq!(operations[1], Operation::IntegerLiteral(2));
        assert_eq!(operations[2], Operation::Add);

        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 3);
        Ok(())
    }

    #[test]
    fn test_string_concat() -> Result<()> {
        let xpath = CompiledXPath::new("'a' || 'b'");
        let result = xpath.interpret()?;
        assert_eq!(result.as_string()?, "ab");
        Ok(())
    }

    #[test]
    fn test_nested() -> Result<()> {
        let xpath = CompiledXPath::new("1 + (8 - 2)");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 7);
        Ok(())
    }

    #[test]
    fn test_comma() -> Result<()> {
        let xpath = CompiledXPath::new("1, 2");
        let result = xpath.interpret()?;
        assert_eq!(
            result.as_sequence()?,
            Sequence(vec![
                Item::AtomicValue(Atomic::Integer(1)),
                Item::AtomicValue(Atomic::Integer(2))
            ])
        );
        Ok(())
    }

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
}
