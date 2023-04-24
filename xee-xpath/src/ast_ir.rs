use std::iter;

use crate::ast;
use crate::ir;

struct Converter {
    counter: usize,
}

struct Binding {
    name: ir::Name,
    expr: ir::Expr,
}

impl Converter {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn new_name(&mut self) -> ir::Name {
        let name = format!("v{}", self.counter);
        self.counter += 1;
        ir::Name(name)
    }

    fn new_binding(&mut self, expr: ir::Expr) -> (ir::Atom, Binding) {
        let name = self.new_name();
        let atom = ir::Atom::Variable(name.clone());
        let binding = Binding { name, expr };
        (atom, binding)
    }

    fn bind(&mut self, bindings: &[Binding]) -> ir::Expr {
        let expr = &bindings.last().unwrap().expr;
        let bindings = &bindings[..bindings.len() - 1];
        bindings.iter().rev().fold(expr.clone(), |expr, binding| {
            ir::Expr::Let(ir::Let {
                name: binding.name.clone(),
                var_expr: Box::new(binding.expr.clone()),
                return_expr: Box::new(expr),
            })
        })
    }

    fn convert(&mut self, ast: &ast::ExprSingle) -> ir::Expr {
        let (_, bindings) = self.expr_single(ast);
        self.bind(&bindings)
    }

    fn expr_single(&mut self, ast: &ast::ExprSingle) -> (ir::Atom, Vec<Binding>) {
        match ast {
            ast::ExprSingle::Binary(ast) => self.binary_expr(ast),
            _ => todo!(),
        }
    }

    fn path_expr(&mut self, ast: &ast::PathExpr) -> (ir::Atom, Vec<Binding>) {
        let step = &ast.steps[0];
        self.step_expr(step)
    }

    fn step_expr(&mut self, ast: &ast::StepExpr) -> (ir::Atom, Vec<Binding>) {
        match ast {
            ast::StepExpr::PrimaryExpr(ast) => self.primary_expr(ast),
            _ => todo!(),
        }
    }

    fn primary_expr(&mut self, ast: &ast::PrimaryExpr) -> (ir::Atom, Vec<Binding>) {
        match ast {
            ast::PrimaryExpr::Literal(ast) => self.literal(ast),
            ast::PrimaryExpr::VarRef(ast) => self.var_ref(ast),
            ast::PrimaryExpr::Expr(exprs) => self.primary_exprs(exprs),
            _ => todo!("primary_expr: {:?}", ast),
        }
    }

    fn literal(&mut self, ast: &ast::Literal) -> (ir::Atom, Vec<Binding>) {
        match ast {
            ast::Literal::Integer(i) => (ir::Atom::Const(ir::Const::Integer(*i)), vec![]),
            _ => todo!(),
        }
    }

    fn var_ref(&mut self, ast: &ast::Name) -> (ir::Atom, Vec<Binding>) {
        todo!();
    }

    fn primary_exprs(&mut self, exprs: &[ast::ExprSingle]) -> (ir::Atom, Vec<Binding>) {
        // XXX just take the first, which is wrong
        self.expr_single(&exprs[0])
    }

    fn binary_expr(&mut self, ast: &ast::BinaryExpr) -> (ir::Atom, Vec<Binding>) {
        let (left_atom, left_bindings) = self.path_expr(&ast.left);
        let (right_atom, right_bindings) = self.path_expr(&ast.right);
        let op = self.binary_op(ast.operator);
        let expr = ir::Expr::Binary(ir::Binary {
            left: left_atom,
            binary_op: op,
            right: right_atom,
        });
        let (atom, binding) = self.new_binding(expr);

        let bindings = left_bindings
            .into_iter()
            .chain(right_bindings.into_iter())
            .chain(iter::once(binding))
            .collect();
        (atom, bindings)
    }

    fn binary_op(&mut self, operator: ast::Operator) -> ir::BinaryOp {
        match operator {
            ast::Operator::Add => ir::BinaryOp::Add,
            _ => todo!(),
        }
    }

    fn if_expr(&mut self, ast: &ast::IfExpr) -> (ir::Atom, Vec<Binding>) {
        // XXX taking the first expr out of the vec is wrong
        let (condition, condition_bindings) = self.expr_single(&ast.condition[0]);
        let (_, then) = self.expr_single(&ast.then);
        let (_, else_) = self.expr_single(&ast.else_);
        let expr = ir::Expr::If(ir::If {
            condition,
            then: Box::new(self.bind(&then)),
            else_: Box::new(self.bind(&else_)),
        });
        let (atom, binding) = self.new_binding(expr);
        let bindings = condition_bindings
            .into_iter()
            .chain(iter::once(binding))
            .collect();
        (atom, bindings)
    }
}

fn convert_expr_single(s: &str) -> ir::Expr {
    let ast = crate::parse_ast::parse_expr_single(s);
    let mut converter = Converter::new();
    converter.convert(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_add() {
        assert_debug_snapshot!(convert_expr_single("1 + 2"));
    }

    #[test]
    fn test_add2() {
        assert_debug_snapshot!(convert_expr_single("1 + 2 + 3"));
    }
}
