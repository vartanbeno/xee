use ahash::{HashMap, HashMapExt};

use crate::ast;
use crate::ir;
use crate::static_context::StaticContext;
use crate::value::Step;

#[derive(Debug, Clone)]
struct Binding {
    name: ir::Name,
    expr: ir::Expr,
}

#[derive(Debug, Clone)]
struct Bindings {
    bindings: Vec<Binding>,
}

impl Bindings {
    fn new() -> Self {
        Self { bindings: vec![] }
    }

    fn from_vec(bindings: Vec<Binding>) -> Self {
        Self { bindings }
    }

    fn atom(&mut self) -> ir::Atom {
        let last = self.bindings.last().unwrap();
        let (want_pop, atom) = match &last.expr {
            ir::Expr::Atom(atom) => (true, atom.clone()),
            _ => (false, ir::Atom::Variable(last.name.clone())),
        };
        if want_pop {
            self.bindings.pop();
        }
        atom
    }

    fn expr(&self) -> ir::Expr {
        let last_binding = self.bindings.last().unwrap();
        let bindings = &self.bindings[..self.bindings.len() - 1];
        let expr = last_binding.expr.clone();
        bindings.iter().rev().fold(expr, |expr, binding| {
            ir::Expr::Let(ir::Let {
                name: binding.name.clone(),
                var_expr: Box::new(binding.expr.clone()),
                return_expr: Box::new(expr),
            })
        })
    }

    // for function arguments turn all bindings into atoms; remove those
    // that are atoms already
    fn args(&mut self, arity: usize) -> Vec<ir::Atom> {
        let mut atoms = vec![];
        let prev_bindings = &self.bindings[..self.bindings.len() - arity];
        let arg_bindings = &self.bindings[self.bindings.len() - arity..];
        let mut new_bindings = vec![];
        for binding in arg_bindings {
            match &binding.expr {
                ir::Expr::Atom(atom) => atoms.push(atom.clone()),
                _ => {
                    new_bindings.push(binding.clone());
                    atoms.push(ir::Atom::Variable(binding.name.clone()));
                }
            }
        }
        self.bindings = [prev_bindings, &new_bindings].concat();
        atoms
    }

    fn bind(&self, binding: Binding) -> Self {
        let mut bindings = self.clone();
        bindings.bindings.push(binding);
        bindings
    }

    fn concat(&self, bindings: Bindings) -> Self {
        let mut result = self.clone();
        result.bindings.extend(bindings.bindings);
        result
    }
}

#[derive(Debug)]
pub(crate) struct Converter<'a> {
    counter: usize,
    variables: HashMap<ast::Name, ir::Name>,
    context_name: ast::Name,
    static_context: &'a StaticContext,
}

impl<'a> Converter<'a> {
    pub(crate) fn new(static_context: &'a StaticContext) -> Self {
        Self {
            counter: 0,
            variables: HashMap::new(),
            context_name: ast::Name {
                name: "xee-context".to_string(),
                namespace: None,
            },
            static_context,
        }
    }

    fn new_name(&mut self) -> ir::Name {
        let name = format!("v{}", self.counter);
        self.counter += 1;
        ir::Name(name)
    }

    fn new_context_name(&mut self) -> ir::Name {
        self.new_var_name(&self.context_name.clone())
    }

    fn new_var_name(&mut self, name: &ast::Name) -> ir::Name {
        self.variables.get(name).cloned().unwrap_or_else(|| {
            let new_name = self.new_name();
            self.variables.insert(name.clone(), new_name.clone());
            new_name
        })
    }

    fn var_ref(&mut self, name: &ast::Name) -> Bindings {
        let ir_name = self.variables.get(name).unwrap();
        Bindings::from_vec(vec![Binding {
            name: ir_name.clone(),
            expr: ir::Expr::Atom(ir::Atom::Variable(ir_name.clone())),
        }])
    }

    fn context_item(&mut self) -> Bindings {
        self.var_ref(&self.context_name.clone())
    }

    fn new_binding(&mut self, expr: ir::Expr) -> Binding {
        let name = self.new_name();
        Binding { name, expr }
    }

    fn convert_expr_single(&mut self, ast: &ast::ExprSingle) -> ir::Expr {
        let bindings = self.expr_single(ast);
        bindings.expr()
    }

    pub(crate) fn convert_xpath(&mut self, ast: &ast::XPath) -> ir::Expr {
        let bindings = self.xpath(ast);
        bindings.expr()
    }

    fn xpath(&mut self, ast: &ast::XPath) -> Bindings {
        let context_name = self.new_context_name();
        let exprs_bindings = self.exprs(&ast.exprs);
        // XXX reusing context_name isn't going to work, as each
        // context needs to have its unique name
        let outer_function_expr = ir::Expr::FunctionDefinition(ir::FunctionDefinition {
            params: vec![ir::Param(context_name)],
            body: Box::new(exprs_bindings.expr()),
        });
        let binding = self.new_binding(outer_function_expr);
        Bindings::from_vec(vec![binding])
    }

    fn expr_single(&mut self, ast: &ast::ExprSingle) -> Bindings {
        match ast {
            ast::ExprSingle::Path(ast) => self.path_expr(ast),
            ast::ExprSingle::Apply(ast) => self.apply_expr(ast),
            ast::ExprSingle::Let(ast) => self.let_expr(ast),
            ast::ExprSingle::If(ast) => self.if_expr(ast),
            ast::ExprSingle::Binary(ast) => self.binary_expr(ast),
            ast::ExprSingle::For(ast) => self.for_expr(ast),
            ast::ExprSingle::Quantified(ast) => self.quantified_expr(ast),
        }
    }

    fn path_expr(&mut self, ast: &ast::PathExpr) -> Bindings {
        let first_step = &ast.steps[0];
        let rest_steps = &ast.steps[1..];
        let first_step_bindings = self.step_expr(first_step);
        rest_steps
            .iter()
            .fold(first_step_bindings, |acc, step_expr| {
                let mut step_bindings = acc;
                let step_atom = step_bindings.atom();
                let context_name = self.new_context_name();
                let return_bindings = self.step_expr(step_expr);
                let expr = ir::Expr::Map(ir::Map {
                    var_name: context_name,
                    var_atom: step_atom,
                    return_expr: Box::new(return_bindings.expr()),
                });
                let binding = self.new_binding(expr);
                step_bindings.bind(binding)
            })
    }

    fn step_expr(&mut self, ast: &ast::StepExpr) -> Bindings {
        match ast {
            ast::StepExpr::PrimaryExpr(ast) => self.primary_expr(ast),
            ast::StepExpr::PostfixExpr { primary, postfixes } => self.postfixes(primary, postfixes),
            ast::StepExpr::AxisStep(ast) => self.axis_step(ast),
        }
    }

    fn primary_expr(&mut self, ast: &ast::PrimaryExpr) -> Bindings {
        match ast {
            ast::PrimaryExpr::Literal(ast) => self.literal(ast),
            ast::PrimaryExpr::VarRef(ast) => self.var_ref(ast),
            ast::PrimaryExpr::Expr(exprs) => self.exprs(exprs),
            ast::PrimaryExpr::ContextItem => self.context_item(),
            ast::PrimaryExpr::InlineFunction(ast) => self.inline_function(ast),
            ast::PrimaryExpr::FunctionCall(ast) => self.function_call(ast),
            ast::PrimaryExpr::NamedFunctionRef(ast) => self.named_function_ref(ast),
            _ => todo!("primary_expr: {:?}", ast),
        }
    }

    fn postfixes(&mut self, primary: &ast::PrimaryExpr, postfixes: &[ast::Postfix]) -> Bindings {
        let primary_bindings = self.primary_expr(primary);
        postfixes.iter().fold(primary_bindings, |acc, postfix| {
            let mut bindings = acc;
            match postfix {
                ast::Postfix::Predicate(exprs) => {
                    let atom = bindings.atom();
                    let context_name = self.new_context_name();
                    let return_bindings = self.exprs(exprs);
                    let expr = ir::Expr::Filter(ir::Filter {
                        var_name: context_name,
                        var_atom: atom,
                        return_expr: Box::new(return_bindings.expr()),
                    });
                    let binding = self.new_binding(expr);
                    bindings.bind(binding)
                }
                ast::Postfix::ArgumentList(exprs) => {
                    let atom = bindings.atom();
                    let mut arg_bindings = self.args(exprs);
                    let args = arg_bindings.args(exprs.len());
                    let expr = ir::Expr::FunctionCall(ir::FunctionCall { atom, args });
                    let binding = self.new_binding(expr);
                    bindings.concat(arg_bindings).bind(binding)
                }
                _ => todo!(),
            }
        })
    }

    fn axis_step(&mut self, ast: &ast::AxisStep) -> Bindings {
        // get the current context
        let mut current_context_bindings = self.context_item();

        // create a step atom
        let step = Step {
            axis: ast.axis.clone(),
            node_test: ast.node_test.clone(),
        };
        let atom = ir::Atom::Const(ir::Const::Step(step));

        // given the current context item, apply the step
        let expr = ir::Expr::FunctionCall(ir::FunctionCall {
            atom,
            args: vec![current_context_bindings.atom()],
        });

        // create a new binding for the step
        let binding = self.new_binding(expr);

        // XXX todo predicates
        Bindings::from_vec(vec![binding])
    }

    fn literal(&mut self, ast: &ast::Literal) -> Bindings {
        match ast {
            ast::Literal::Integer(i) => {
                let expr = ir::Expr::Atom(ir::Atom::Const(ir::Const::Integer(*i)));
                let binding = self.new_binding(expr);
                Bindings::from_vec(vec![binding])
            }
            _ => todo!(),
        }
    }

    fn exprs(&mut self, exprs: &[ast::ExprSingle]) -> Bindings {
        if !exprs.is_empty() {
            let first_expr = &exprs[0];
            let rest_exprs = &exprs[1..];
            rest_exprs
                .iter()
                .fold(self.expr_single(first_expr), |acc, expr| {
                    let mut left_bindings = acc;
                    let mut right_bindings = self.expr_single(expr);
                    let expr = ir::Expr::Binary(ir::Binary {
                        left: left_bindings.atom(),
                        op: ir::BinaryOp::Comma,
                        right: right_bindings.atom(),
                    });
                    let binding = self.new_binding(expr);
                    left_bindings.concat(right_bindings).bind(binding)
                })
        } else {
            let expr = ir::Expr::Atom(ir::Atom::Const(ir::Const::EmptySequence));
            let binding = self.new_binding(expr);
            Bindings::from_vec(vec![binding])
        }
    }

    fn binary_expr(&mut self, ast: &ast::BinaryExpr) -> Bindings {
        let mut left_bindings = self.path_expr(&ast.left);
        let mut right_bindings = self.path_expr(&ast.right);
        let op = self.binary_op(ast.operator);
        let expr = ir::Expr::Binary(ir::Binary {
            left: left_bindings.atom(),
            op,
            right: right_bindings.atom(),
        });
        let binding = self.new_binding(expr);

        left_bindings.concat(right_bindings).bind(binding)
    }

    fn binary_op(&mut self, operator: ast::Operator) -> ir::BinaryOp {
        match operator {
            ast::Operator::Add => ir::BinaryOp::Add,
            ast::Operator::Sub => ir::BinaryOp::Sub,
            ast::Operator::ValueEq => ir::BinaryOp::Eq,
            ast::Operator::ValueNe => ir::BinaryOp::Ne,
            ast::Operator::ValueLt => ir::BinaryOp::Lt,
            ast::Operator::ValueLe => ir::BinaryOp::Le,
            ast::Operator::ValueGt => ir::BinaryOp::Gt,
            ast::Operator::ValueGe => ir::BinaryOp::Ge,
            ast::Operator::Union => ir::BinaryOp::Union,
            ast::Operator::Range => ir::BinaryOp::Range,
            _ => todo!("binary_op: {:?}", operator),
        }
    }

    fn apply_expr(&mut self, ast: &ast::ApplyExpr) -> Bindings {
        match &ast.operator {
            ast::ApplyOperator::SimpleMap(path_exprs) => {
                let path_bindings = self.path_expr(&ast.path_expr);
                path_exprs.iter().fold(path_bindings, |acc, path_expr| {
                    let mut path_bindings = acc;
                    let path_atom = path_bindings.atom();
                    let context_name = self.new_context_name();
                    let return_bindings = self.path_expr(path_expr);
                    let expr = ir::Expr::Map(ir::Map {
                        var_name: context_name,
                        var_atom: path_atom,
                        return_expr: Box::new(return_bindings.expr()),
                    });
                    let binding = self.new_binding(expr);
                    path_bindings.bind(binding)
                })
            }
            _ => {
                todo!("ApplyOperator: {:?}", ast.operator)
            }
        }
    }

    fn if_expr(&mut self, ast: &ast::IfExpr) -> Bindings {
        // XXX taking the first expr out of the vec is wrong
        let mut condition_bindings = self.expr_single(&ast.condition[0]);
        let then_bindings = self.expr_single(&ast.then);
        let else_bindings = self.expr_single(&ast.else_);
        let expr = ir::Expr::If(ir::If {
            condition: condition_bindings.atom(),
            then: Box::new(then_bindings.expr()),
            else_: Box::new(else_bindings.expr()),
        });
        let binding = self.new_binding(expr);
        condition_bindings.bind(binding)
    }

    fn let_expr(&mut self, ast: &ast::LetExpr) -> Bindings {
        let name = self.new_var_name(&ast.var_name);
        let var_bindings = self.expr_single(&ast.var_expr);
        let return_bindings = self.expr_single(&ast.return_expr);
        let expr = ir::Expr::Let(ir::Let {
            name,
            var_expr: Box::new(var_bindings.expr()),
            return_expr: Box::new(return_bindings.expr()),
        });
        Bindings::from_vec(vec![self.new_binding(expr)])
    }

    fn for_expr(&mut self, ast: &ast::ForExpr) -> Bindings {
        let name = self.new_var_name(&ast.var_name);
        let mut var_bindings = self.expr_single(&ast.var_expr);
        let var_atom = var_bindings.atom();
        let return_bindings = self.expr_single(&ast.return_expr);
        let expr = ir::Expr::Map(ir::Map {
            var_name: name,
            var_atom,
            return_expr: Box::new(return_bindings.expr()),
        });

        let binding = self.new_binding(expr);
        var_bindings.bind(binding)
    }

    fn quantified_expr(&mut self, ast: &ast::QuantifiedExpr) -> Bindings {
        let name = self.new_var_name(&ast.var_name);
        let mut var_bindings = self.expr_single(&ast.var_expr);
        let var_atom = var_bindings.atom();
        let satisfies_bindings = self.expr_single(&ast.satisfies_expr);
        let expr = ir::Expr::Quantified(ir::Quantified {
            quantifier: self.quantifier(&ast.quantifier),
            var_name: name,
            var_atom,
            satisifies_expr: Box::new(satisfies_bindings.expr()),
        });

        let binding = self.new_binding(expr);
        var_bindings.bind(binding)
    }

    fn quantifier(&mut self, quantifier: &ast::Quantifier) -> ir::Quantifier {
        match quantifier {
            ast::Quantifier::Some => ir::Quantifier::Some,
            ast::Quantifier::Every => ir::Quantifier::Every,
        }
    }

    fn inline_function(&mut self, inline_function: &ast::InlineFunction) -> Bindings {
        let params = inline_function
            .params
            .iter()
            .map(|param| self.param(param))
            .collect();
        let body_bindings = self.exprs(&inline_function.body);
        let expr = ir::Expr::FunctionDefinition(ir::FunctionDefinition {
            params,
            body: Box::new(body_bindings.expr()),
        });
        let binding = self.new_binding(expr);
        Bindings::from_vec(vec![binding])
    }

    fn param(&mut self, param: &ast::Param) -> ir::Param {
        ir::Param(self.new_var_name(&param.name))
    }

    fn function_call(&mut self, ast: &ast::FunctionCall) -> Bindings {
        let arity = ast.arguments.len();
        if arity > u8::MAX as usize {
            panic!("too many arguments");
        }
        let static_function_id = self
            .static_context
            .functions
            .get_by_name(&ast.name, arity as u8)
            .unwrap();
        let constant = ir::Const::StaticFunction(static_function_id);
        let atom = ir::Atom::Const(constant);
        let mut arg_bindings = self.args(&ast.arguments);
        let args = arg_bindings.args(ast.arguments.len());
        let expr = ir::Expr::FunctionCall(ir::FunctionCall { atom, args });
        let binding = self.new_binding(expr);
        arg_bindings.bind(binding)
    }

    fn named_function_ref(&mut self, ast: &ast::NamedFunctionRef) -> Bindings {
        let static_function_id = self
            .static_context
            .functions
            .get_by_name(&ast.name, ast.arity)
            .unwrap();
        let constant = ir::Const::StaticFunction(static_function_id);
        let atom = ir::Atom::Const(constant);
        let expr = ir::Expr::Atom(atom);
        let binding = self.new_binding(expr);
        Bindings::from_vec(vec![binding])
    }

    fn args(&mut self, args: &[ast::ExprSingle]) -> Bindings {
        if args.is_empty() {
            return Bindings::from_vec(vec![]);
        }
        let first = &args[0];
        let rest = &args[1..];
        let bindings = self.expr_single(first);
        rest.iter().fold(bindings, |bindings, arg| {
            let bindings = bindings;
            let arg_bindings = self.expr_single(arg);
            bindings.concat(arg_bindings)
        })
    }
}

fn convert_expr_single(s: &str) -> ir::Expr {
    let ast = crate::parse_ast::parse_expr_single(s);
    let static_context = StaticContext::new();
    let mut converter = Converter::new(&static_context);
    converter.convert_expr_single(&ast)
}

fn convert_xpath(s: &str) -> ir::Expr {
    let ast = crate::parse_ast::parse_xpath(s);
    let static_context = StaticContext::new();
    let mut converter = Converter::new(&static_context);
    converter.convert_xpath(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_integer() {
        assert_debug_snapshot!(convert_expr_single("1"));
    }

    #[test]
    fn test_add() {
        assert_debug_snapshot!(convert_expr_single("1 + 2"));
    }

    #[test]
    fn test_add2() {
        assert_debug_snapshot!(convert_expr_single("1 + 2 + 3"));
    }

    #[test]
    fn test_if() {
        assert_debug_snapshot!(convert_expr_single("if (1 gt 2) then 1 + 2 else 3 + 4"));
    }

    #[test]
    fn test_comma() {
        assert_debug_snapshot!(convert_xpath("1, 2"));
    }

    #[test]
    fn test_comma2() {
        assert_debug_snapshot!(convert_xpath("1, 2, 3"));
    }

    #[test]
    fn test_empty_sequence() {
        assert_debug_snapshot!(convert_xpath("()"));
    }

    #[test]
    fn test_let_expr() {
        assert_debug_snapshot!(convert_expr_single("let $x := 1 return 2"));
    }

    #[test]
    fn test_let_expr_variable() {
        assert_debug_snapshot!(convert_expr_single("let $x := 1 return $x"));
    }

    #[test]
    fn test_let_expr_with_add() {
        assert_debug_snapshot!(convert_expr_single("let $x := (1 + 2) return $x"));
    }

    #[test]
    fn test_for_expr() {
        assert_debug_snapshot!(convert_expr_single("for $x in 1 return 2"));
    }

    #[test]
    fn test_for_expr2() {
        assert_debug_snapshot!(convert_expr_single("for $x in (1, 2) return $x + 1"));
    }

    #[test]
    fn test_simple_map() {
        assert_debug_snapshot!(convert_expr_single("(1, 2) ! 1"));
    }

    #[test]
    fn test_simple_map_with_context() {
        assert_debug_snapshot!(convert_expr_single("(1, 2) ! (. + 1)"));
    }

    #[test]
    fn test_nested_simple_map_with_context() {
        assert_debug_snapshot!(convert_expr_single("(1, 2) ! (. + 1) ! (. + 2)"));
    }

    #[test]
    fn test_quantified() {
        assert_debug_snapshot!(convert_expr_single("some $x in (1, 2) satisfies $x gt 1"));
    }

    #[test]
    fn test_postfix_filter() {
        assert_debug_snapshot!(convert_expr_single("(1, 2)[. gt 2]"));
    }

    #[test]
    fn test_postfix_filter_nested() {
        assert_debug_snapshot!(convert_expr_single("(1, 2)[. gt 2][. lt 3]"));
    }

    #[test]
    fn test_function_definition() {
        assert_debug_snapshot!(convert_expr_single("function($x) { $x + 1 }"));
    }

    #[test]
    fn test_function_call() {
        assert_debug_snapshot!(convert_expr_single("function($x) { $x + 1 }(3)"));
    }

    #[test]
    fn test_function_call2() {
        assert_debug_snapshot!(convert_expr_single("function($x) { $x + 1 }(3 + 5)"));
    }

    #[test]
    fn test_static_function_call() {
        assert_debug_snapshot!(convert_expr_single("my_function(5, 2)"));
    }

    #[test]
    fn test_static_function_call2() {
        assert_debug_snapshot!(convert_expr_single("my_function(1 + 2, 3 + 4)"));
    }

    #[test]
    fn test_static_function_call3() {
        assert_debug_snapshot!(convert_expr_single("my_function(1 + 2 + 3, 4 + 5)"));
    }

    #[test]
    fn test_named_function_ref() {
        assert_debug_snapshot!(convert_expr_single("my_function#2"));
    }

    #[test]
    fn test_path_expr() {
        assert_debug_snapshot!(convert_expr_single("(1, 2) / (. + 1)"));
    }

    #[test]
    fn test_nested_path_expr() {
        assert_debug_snapshot!(convert_expr_single("(1, 2) / (. + 1) / (. + 2)"));
    }

    #[test]
    fn test_single_axis_step() {
        assert_debug_snapshot!(convert_xpath("child::a"));
    }
}
