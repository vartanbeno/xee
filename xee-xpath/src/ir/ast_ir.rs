use std::rc::Rc;

use ahash::{HashMap, HashMapExt};
use miette::SourceSpan;

use crate::ast;
use crate::context::{Namespaces, StaticContext, FN_NAMESPACE};
use crate::data::StaticFunctionId;
use crate::data::Step;
use crate::error::{Error, Result};
use crate::span::Spanned;

use super::ir_core as ir;

#[derive(Debug, Clone)]
struct Binding {
    name: ir::Name,
    expr: ir::Expr,
    span: SourceSpan,
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

    fn atom(&mut self) -> ir::AtomS {
        let last = self.bindings.last().unwrap();
        let (want_pop, atom) = match &last.expr {
            ir::Expr::Atom(atom) => (true, atom.clone()),
            _ => (
                false,
                Spanned::new(ir::Atom::Variable(last.name.clone()), last.span),
            ),
        };
        if want_pop {
            self.bindings.pop();
        }
        atom
    }

    fn expr(&self) -> ir::ExprS {
        let last_binding = self.bindings.last().unwrap();
        let bindings = &self.bindings[..self.bindings.len() - 1];
        let expr = last_binding.expr.clone();
        Spanned::new(
            bindings.iter().rev().fold(expr, |expr, binding| {
                ir::Expr::Let(ir::Let {
                    name: binding.name.clone(),
                    var_expr: Box::new(Spanned::new(binding.expr.clone(), binding.span)),
                    return_expr: Box::new(Spanned::new(expr, last_binding.span)),
                })
            }),
            last_binding.span,
        )
    }

    // for function arguments turn all bindings into atoms; remove those
    // that are atoms already
    fn args(&mut self, arity: usize) -> Vec<ir::AtomS> {
        let mut atoms = vec![];
        let prev_bindings = &self.bindings[..self.bindings.len() - arity];
        let arg_bindings = &self.bindings[self.bindings.len() - arity..];
        let mut new_bindings = vec![];
        for binding in arg_bindings {
            match &binding.expr {
                ir::Expr::Atom(atom) => atoms.push(atom.clone()),
                _ => {
                    new_bindings.push(binding.clone());
                    atoms.push(Spanned::new(
                        ir::Atom::Variable(binding.name.clone()),
                        binding.span,
                    ));
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
enum ContextItem {
    Names(ir::ContextNames),
    Absent,
}

#[derive(Debug)]
pub(crate) struct IrConverter<'a> {
    counter: usize,
    variables: HashMap<ast::Name, ir::Name>,
    context_scope: Vec<ContextItem>,
    src: &'a str,
    static_context: &'a StaticContext<'a>,
    fn_position: ast::Name,
    fn_last: ast::Name,
}

impl<'a> IrConverter<'a> {
    pub(crate) fn new(src: &'a str, static_context: &'a StaticContext) -> Self {
        Self {
            counter: 0,
            variables: HashMap::new(),
            context_scope: Vec::new(),
            src,
            static_context,
            fn_position: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string())),
            fn_last: ast::Name::new("last".to_string(), Some(FN_NAMESPACE.to_string())),
        }
    }

    fn new_name(&mut self) -> ir::Name {
        let name = format!("v{}", self.counter);
        self.counter += 1;
        ir::Name(name)
    }

    fn push_context(&mut self) -> ir::ContextNames {
        let names = ir::ContextNames {
            item: self.new_name(),
            position: self.new_name(),
            last: self.new_name(),
        };
        self.context_scope.push(ContextItem::Names(names.clone()));
        names
    }

    fn push_absent_context(&mut self) {
        self.context_scope.push(ContextItem::Absent);
    }

    fn pop_context(&mut self) {
        self.context_scope.pop();
    }

    fn explicit_context_names(&mut self, name: ir::Name) -> ir::ContextNames {
        ir::ContextNames {
            item: name,
            position: self.new_name(),
            last: self.new_name(),
        }
    }

    fn new_var_name(&mut self, name: &ast::Name) -> ir::Name {
        self.variables.get(name).cloned().unwrap_or_else(|| {
            let new_name = self.new_name();
            self.variables.insert(name.clone(), new_name.clone());
            new_name
        })
    }

    fn var_ref(&mut self, name: &ast::Name, span: SourceSpan) -> Result<Bindings> {
        let ir_name = self.variables.get(name).ok_or_else(|| Error::XPST0008 {
            src: self.src.to_string(),
            span,
        })?;
        Ok(Bindings::from_vec(vec![Binding {
            name: ir_name.clone(),
            expr: ir::Expr::Atom(Spanned::new(ir::Atom::Variable(ir_name.clone()), span)),
            span,
        }]))
    }

    fn current_context_names(&self) -> Option<ir::ContextNames> {
        match self.context_scope.last() {
            Some(ContextItem::Names(names)) => Some(names.clone()),
            Some(ContextItem::Absent) => None,
            None => None,
        }
    }

    fn context_name<F>(&mut self, get_name: F, span: SourceSpan) -> Result<Bindings>
    where
        F: Fn(&ir::ContextNames) -> ir::Name,
    {
        let empty_span: SourceSpan = (0, 0).into();
        if let Some(context_scope) = self.context_scope.last() {
            match context_scope {
                ContextItem::Names(names) => {
                    let ir_name = get_name(names);
                    Ok(Bindings::from_vec(vec![Binding {
                        name: ir_name.clone(),
                        expr: ir::Expr::Atom(Spanned::new(ir::Atom::Variable(ir_name), empty_span)),
                        span: empty_span,
                    }]))
                }
                // we can detect statically that the context is absent if it's in
                // a function definition
                ContextItem::Absent => Err(Error::XPDY0002 {
                    src: self.src.to_string(),
                    span,
                }),
            }
        } else {
            Err(Error::XPDY0002 {
                src: self.src.to_string(),
                span,
            })
        }
    }

    fn context_item(&mut self, span: SourceSpan) -> Result<Bindings> {
        self.context_name(|names| names.item.clone(), span)
    }

    fn fn_position(&mut self, span: SourceSpan) -> Result<Bindings> {
        self.context_name(|names| names.position.clone(), span)
    }

    fn fn_last(&mut self, span: SourceSpan) -> Result<Bindings> {
        self.context_name(|names| names.last.clone(), span)
    }

    fn new_binding(&mut self, expr: ir::Expr, span: SourceSpan) -> Binding {
        let name = self.new_name();
        Binding { name, expr, span }
    }

    fn convert_expr_single(&mut self, ast: &ast::ExprSingleS) -> Result<ir::ExprS> {
        let bindings = self.expr_single(ast)?;
        Ok(bindings.expr())
    }

    pub(crate) fn convert_xpath(&mut self, ast: &ast::XPath) -> Result<ir::ExprS> {
        let bindings = self.xpath(ast)?;
        Ok(bindings.expr())
    }

    fn xpath(&mut self, ast: &ast::XPath) -> Result<Bindings> {
        let context_names = self.push_context();
        // define any external variable names
        let mut ir_names = Vec::new();
        for name in &self.static_context.variables {
            ir_names.push(self.new_var_name(name));
        }
        let exprs_bindings = self.exprs(&ast.exprs)?;
        self.pop_context();
        let mut params = vec![
            ir::Param(context_names.item),
            ir::Param(context_names.position),
            ir::Param(context_names.last),
        ];
        // add any variables defined in static context as parameters
        for ir_name in ir_names {
            params.push(ir::Param(ir_name));
        }
        let outer_function_expr = ir::Expr::FunctionDefinition(ir::FunctionDefinition {
            params,
            body: Box::new(exprs_bindings.expr()),
        });
        let binding = self.new_binding(outer_function_expr, ast.exprs.span);
        Ok(Bindings::from_vec(vec![binding]))
    }

    fn expr_single(&mut self, ast: &ast::ExprSingleS) -> Result<Bindings> {
        let outer_ast = &ast.value;
        let span = ast.span;
        match outer_ast {
            ast::ExprSingle::Path(ast) => self.path_expr(ast),
            ast::ExprSingle::Apply(ast) => self.apply_expr(ast, span),
            ast::ExprSingle::Let(ast) => self.let_expr(ast, span),
            ast::ExprSingle::If(ast) => self.if_expr(ast, span),
            ast::ExprSingle::Binary(ast) => self.binary_expr(ast, span),
            ast::ExprSingle::For(ast) => self.for_expr(ast, span),
            ast::ExprSingle::Quantified(ast) => self.quantified_expr(ast, span),
        }
    }

    fn path_expr(&mut self, ast: &ast::PathExpr) -> Result<Bindings> {
        let first_step = &ast.steps[0];
        let rest_steps = &ast.steps[1..];
        let first_step_bindings = Ok(self.step_expr(first_step)?);
        rest_steps
            .iter()
            .fold(first_step_bindings, |acc, step_expr| {
                let mut step_bindings = acc?;
                let step_atom = step_bindings.atom();
                let context_names = self.push_context();
                let return_bindings = self.step_expr(step_expr)?;
                self.pop_context();
                let expr = ir::Expr::Map(ir::Map {
                    context_names,
                    var_atom: step_atom,
                    return_expr: Box::new(return_bindings.expr()),
                });
                let binding = self.new_binding(expr, step_expr.span);
                Ok(step_bindings.bind(binding))
            })
    }

    fn step_expr(&mut self, ast: &ast::StepExprS) -> Result<Bindings> {
        let outer_ast = &ast.value;
        let span = ast.span;
        match outer_ast {
            ast::StepExpr::PrimaryExpr(ast) => self.primary_expr(ast),
            ast::StepExpr::PostfixExpr { primary, postfixes } => self.postfixes(primary, postfixes),
            ast::StepExpr::AxisStep(ast) => self.axis_step(ast, span),
        }
    }

    fn primary_expr(&mut self, ast: &ast::PrimaryExprS) -> Result<Bindings> {
        let outer_ast = &ast.value;
        let span = ast.span;
        match outer_ast {
            ast::PrimaryExpr::Literal(ast) => Ok(self.literal(ast, span)?),
            ast::PrimaryExpr::VarRef(ast) => self.var_ref(ast, span),
            ast::PrimaryExpr::Expr(exprs) => self.exprs(exprs),
            ast::PrimaryExpr::ContextItem => self.context_item(span),
            ast::PrimaryExpr::InlineFunction(ast) => self.inline_function(ast, span),
            ast::PrimaryExpr::FunctionCall(ast) => self.function_call(ast, span),
            ast::PrimaryExpr::NamedFunctionRef(ast) => self.named_function_ref(ast, span),
            _ => todo!("primary_expr: {:?}", ast),
        }
    }

    fn postfixes(
        &mut self,
        primary: &ast::PrimaryExprS,
        postfixes: &[ast::Postfix],
    ) -> Result<Bindings> {
        let primary_bindings = self.primary_expr(primary);
        postfixes.iter().fold(primary_bindings, |acc, postfix| {
            let mut bindings = acc?;
            match postfix {
                ast::Postfix::Predicate(exprs) => {
                    let atom = bindings.atom();
                    let context_names = self.push_context();
                    let return_bindings = self.exprs(exprs)?;
                    self.pop_context();
                    let expr = ir::Expr::Filter(ir::Filter {
                        context_names,
                        var_atom: atom,
                        return_expr: Box::new(return_bindings.expr()),
                    });
                    // XXX should use postfix span, not exprs span
                    let binding = self.new_binding(expr, exprs.span);
                    Ok(bindings.bind(binding))
                }
                ast::Postfix::ArgumentList(exprs) => {
                    let atom = bindings.atom();
                    let mut arg_bindings = self.args(exprs)?;
                    let args = arg_bindings.args(exprs.len());
                    let expr = ir::Expr::FunctionCall(ir::FunctionCall { atom, args });
                    // XXX should be able to get span for postfix
                    let empty_span = (0, 0).into();
                    let binding = self.new_binding(expr, empty_span);
                    Ok(bindings.concat(arg_bindings).bind(binding))
                }
                _ => todo!(),
            }
        })
    }

    fn axis_step(&mut self, ast: &ast::AxisStep, span: SourceSpan) -> Result<Bindings> {
        // get the current context
        let mut current_context_bindings = self.context_item(span)?;

        // create a step atom
        let step = Rc::new(Step {
            axis: ast.axis.clone(),
            node_test: ast.node_test.clone(),
        });

        let atom = Spanned::new(ir::Atom::Const(ir::Const::Step(step)), span);

        // given the current context item, apply the step
        let expr = ir::Expr::FunctionCall(ir::FunctionCall {
            atom,
            args: vec![current_context_bindings.atom()],
        });

        // create a new binding for the step
        let binding = self.new_binding(expr, span);

        let bindings = Ok(Bindings::from_vec(vec![binding]));

        // now apply predicates
        ast.predicates.iter().fold(bindings, |acc, predicate| {
            let mut bindings = acc?;
            let atom = bindings.atom();
            let context_names = self.push_context();
            let return_bindings = self.exprs(predicate)?;
            self.pop_context();
            let expr = ir::Expr::Filter(ir::Filter {
                context_names,
                var_atom: atom,
                return_expr: Box::new(return_bindings.expr()),
            });
            let binding = self.new_binding(expr, predicate.span);
            Ok(bindings.bind(binding))
        })
    }

    fn literal(&mut self, ast: &ast::Literal, span: SourceSpan) -> Result<Bindings> {
        let atom = match ast {
            ast::Literal::Integer(i) => {
                let i = i.parse::<i64>().map_err(|_e| Error::FOAR0002)?;
                ir::Atom::Const(ir::Const::Integer(i))
            }
            ast::Literal::String(s) => ir::Atom::Const(ir::Const::String(s.clone())),
            ast::Literal::Double(d) => ir::Atom::Const(ir::Const::Double(*d)),
            ast::Literal::Decimal(d) => ir::Atom::Const(ir::Const::Decimal(*d)),
        };
        let expr = ir::Expr::Atom(Spanned::new(atom, span));
        let binding = self.new_binding(expr, span);
        Ok(Bindings::from_vec(vec![binding]))
    }

    fn exprs(&mut self, exprs: &ast::ExprS) -> Result<Bindings> {
        if !exprs.value.is_empty() {
            // XXX could this be reduce?
            let first_expr = &exprs.value[0];
            let span_start = &exprs.value[0].span.offset();
            let rest_exprs = &exprs.value[1..];
            rest_exprs
                .iter()
                .fold(self.expr_single(first_expr), |acc, expr_single| {
                    let mut left_bindings = acc?;
                    let mut right_bindings = self.expr_single(expr_single)?;
                    let expr = ir::Expr::Binary(ir::Binary {
                        left: left_bindings.atom(),
                        op: ir::BinaryOperator::Comma,
                        right: right_bindings.atom(),
                    });
                    let span_end = expr_single.span.offset() + expr_single.span.len();
                    let span = (*span_start, span_end - span_start).into();
                    let binding = self.new_binding(expr, span);
                    Ok(left_bindings.concat(right_bindings).bind(binding))
                })
        } else {
            let expr = ir::Expr::Atom(Spanned::new(
                ir::Atom::Const(ir::Const::EmptySequence),
                exprs.span,
            ));
            let binding = self.new_binding(expr, exprs.span);
            Ok(Bindings::from_vec(vec![binding]))
        }
    }

    fn binary_expr(&mut self, ast: &ast::BinaryExpr, span: SourceSpan) -> Result<Bindings> {
        let mut left_bindings = self.path_expr(&ast.left)?;
        let mut right_bindings = self.path_expr(&ast.right)?;
        let op = self.binary_op(ast.operator);
        let expr = ir::Expr::Binary(ir::Binary {
            left: left_bindings.atom(),
            op,
            right: right_bindings.atom(),
        });
        let binding = self.new_binding(expr, span);

        Ok(left_bindings.concat(right_bindings).bind(binding))
    }

    fn binary_op(&mut self, operator: ast::BinaryOperator) -> ir::BinaryOperator {
        operator
    }

    fn apply_expr(&mut self, ast: &ast::ApplyExpr, span: SourceSpan) -> Result<Bindings> {
        match &ast.operator {
            ast::ApplyOperator::SimpleMap(path_exprs) => {
                let path_bindings = self.path_expr(&ast.path_expr);
                path_exprs.iter().fold(path_bindings, |acc, path_expr| {
                    let mut path_bindings = acc?;
                    let path_atom = path_bindings.atom();
                    let context_names = self.push_context();
                    let return_bindings = self.path_expr(path_expr)?;
                    self.pop_context();
                    let expr = ir::Expr::Map(ir::Map {
                        context_names,
                        var_atom: path_atom,
                        return_expr: Box::new(return_bindings.expr()),
                    });
                    let binding = self.new_binding(expr, span);
                    Ok(path_bindings.bind(binding))
                })
            }
            _ => {
                todo!("ApplyOperator: {:?}", ast.operator)
            }
        }
    }

    fn if_expr(&mut self, ast: &ast::IfExpr, span: SourceSpan) -> Result<Bindings> {
        let mut condition_bindings = self.exprs(&ast.condition)?;
        let then_bindings = self.expr_single(&ast.then)?;
        let else_bindings = self.expr_single(&ast.else_)?;
        let expr = ir::Expr::If(ir::If {
            condition: condition_bindings.atom(),
            then: Box::new(then_bindings.expr()),
            else_: Box::new(else_bindings.expr()),
        });
        let binding = self.new_binding(expr, span);
        Ok(condition_bindings.bind(binding))
    }

    fn let_expr(&mut self, ast: &ast::LetExpr, span: SourceSpan) -> Result<Bindings> {
        let name = self.new_var_name(&ast.var_name);
        let var_bindings = self.expr_single(&ast.var_expr)?;
        let return_bindings = self.expr_single(&ast.return_expr)?;
        let expr = ir::Expr::Let(ir::Let {
            name,
            var_expr: Box::new(var_bindings.expr()),
            return_expr: Box::new(return_bindings.expr()),
        });
        Ok(Bindings::from_vec(vec![self.new_binding(expr, span)]))
    }

    fn for_expr(&mut self, ast: &ast::ForExpr, span: SourceSpan) -> Result<Bindings> {
        let name = self.new_var_name(&ast.var_name);
        let mut var_bindings = self.expr_single(&ast.var_expr)?;
        let var_atom = var_bindings.atom();
        let context_names = self.explicit_context_names(name);
        let return_bindings = self.expr_single(&ast.return_expr)?;
        let expr = ir::Expr::Map(ir::Map {
            context_names,
            var_atom,
            return_expr: Box::new(return_bindings.expr()),
        });

        let binding = self.new_binding(expr, span);
        Ok(var_bindings.bind(binding))
    }

    fn quantified_expr(&mut self, ast: &ast::QuantifiedExpr, span: SourceSpan) -> Result<Bindings> {
        let name = self.new_var_name(&ast.var_name);
        let mut var_bindings = self.expr_single(&ast.var_expr)?;
        let var_atom = var_bindings.atom();

        let context_names = self.explicit_context_names(name);
        let satisfies_bindings = self.expr_single(&ast.satisfies_expr)?;
        let expr = ir::Expr::Quantified(ir::Quantified {
            quantifier: self.quantifier(&ast.quantifier),
            context_names,
            var_atom,
            satisifies_expr: Box::new(satisfies_bindings.expr()),
        });

        let binding = self.new_binding(expr, span);
        Ok(var_bindings.bind(binding))
    }

    fn quantifier(&mut self, quantifier: &ast::Quantifier) -> ir::Quantifier {
        match quantifier {
            ast::Quantifier::Some => ir::Quantifier::Some,
            ast::Quantifier::Every => ir::Quantifier::Every,
        }
    }

    fn inline_function(
        &mut self,
        inline_function: &ast::InlineFunction,
        span: SourceSpan,
    ) -> Result<Bindings> {
        let params = inline_function
            .params
            .iter()
            .map(|param| self.param(param))
            .collect();
        self.push_absent_context();
        let body_bindings = self.exprs(&inline_function.body)?;
        self.pop_context();
        let expr = ir::Expr::FunctionDefinition(ir::FunctionDefinition {
            params,
            body: Box::new(body_bindings.expr()),
        });
        let binding = self.new_binding(expr, span);
        Ok(Bindings::from_vec(vec![binding]))
    }

    fn param(&mut self, param: &ast::Param) -> ir::Param {
        ir::Param(self.new_var_name(&param.name))
    }

    fn function_call(&mut self, ast: &ast::FunctionCall, span: SourceSpan) -> Result<Bindings> {
        let arity = ast.arguments.len();
        if arity > u8::MAX as usize {
            return Err(Error::XPDY0130);
        }
        // hardcoded fn:position and fn:last
        // These should work without hardcoding them, but this is faster
        // (until some advanced compiler optimization is implemented)
        // unfortunately this can generate a type error instead of a 'context absent'
        // error in some circumstances, but we can live with that for now as it's
        // much more efficient
        if ast.name == self.fn_position {
            assert!(arity == 0);
            return self.fn_position(span);
        } else if ast.name == self.fn_last {
            assert!(arity == 0);
            return self.fn_last(span);
        }

        let static_function_id = self
            .static_context
            .functions
            .get_by_name(&ast.name, arity as u8)
            .ok_or_else(|| Error::XPST0017 {
                advice: format!("Either the function name {:?} does not exist, or you are calling it with the wrong number of arguments ({})", ast.name, arity),
                src: self.src.to_string(),
                span
            })?;
        // XXX we don't know yet how to get the proper span here
        let empty_span = (0, 0).into();
        let mut static_function_ref_bindings =
            self.static_function_ref(static_function_id, empty_span);
        let atom = static_function_ref_bindings.atom();
        let mut arg_bindings = self.args(&ast.arguments)?;
        let args = arg_bindings.args(ast.arguments.len());
        let expr = ir::Expr::FunctionCall(ir::FunctionCall { atom, args });
        let binding = self.new_binding(expr, span);
        Ok(static_function_ref_bindings
            .concat(arg_bindings)
            .bind(binding))
    }

    fn named_function_ref(
        &mut self,
        ast: &ast::NamedFunctionRef,
        span: SourceSpan,
    ) -> Result<Bindings> {
        let static_function_id = self
            .static_context
            .functions
            .get_by_name(&ast.name, ast.arity)
            .ok_or_else(|| Error::XPST0017 {
                advice: format!("Either the function name {:?} does not exist, or you are calling it with the wrong number of arguments ({})", ast.name, ast.arity),
                src: self.src.to_string(),
                span
            })?;
        Ok(self.static_function_ref(static_function_id, span))
    }

    fn static_function_ref(
        &mut self,
        static_function_id: StaticFunctionId,
        span: SourceSpan,
    ) -> Bindings {
        let expr =
            ir::Expr::StaticFunctionReference(static_function_id, self.current_context_names());
        let binding = self.new_binding(expr, span);
        Bindings::from_vec(vec![binding])
    }

    fn args(&mut self, args: &[ast::ExprSingleS]) -> Result<Bindings> {
        if args.is_empty() {
            return Ok(Bindings::from_vec(vec![]));
        }
        let first = &args[0];
        let rest = &args[1..];
        let bindings = self.expr_single(first);
        rest.iter().fold(bindings, |bindings, arg| {
            let bindings = bindings?;
            let arg_bindings = self.expr_single(arg)?;
            Ok(bindings.concat(arg_bindings))
        })
    }
}

fn convert_expr_single(s: &str) -> Result<ir::ExprS> {
    let ast = crate::ast::parse_expr_single(s);
    let namespaces = Namespaces::new(None, None);
    let static_context = StaticContext::new(&namespaces);
    let mut converter = IrConverter::new(s, &static_context);
    converter.convert_expr_single(&ast)
}

pub(crate) fn convert_xpath(s: &str) -> Result<ir::ExprS> {
    let namespaces = Namespaces::new(None, None);
    let ast = crate::ast::parse_xpath(s, &namespaces, &[])?;
    let static_context = StaticContext::new(&namespaces);
    let mut converter = IrConverter::new(s, &static_context);
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
    fn test_postfix_index() {
        assert_debug_snapshot!(convert_expr_single("(1, 2)[1]"));
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
    fn test_named_function_ref2() {
        assert_debug_snapshot!(convert_xpath("my_function#2"));
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

    #[test]
    fn test_multiple_axis_steps() {
        assert_debug_snapshot!(convert_xpath("child::a/child::b"));
    }

    #[test]
    fn test_axis_step_with_predicates() {
        assert_debug_snapshot!(convert_xpath("child::a[. gt 1]"));
    }

    #[test]
    fn test_absent_context_in_function() {
        assert_debug_snapshot!(convert_xpath("function() { . }"));
    }

    #[test]
    fn test_unknown_static_function_name() {
        assert_debug_snapshot!(convert_expr_single("unknown_function()"));
    }

    #[test]
    fn test_wrong_amount_of_arguments() {
        assert_debug_snapshot!(convert_expr_single("fn:string(1, 2, 3)"));
    }

    #[test]
    fn test_unknown_variable_name() {
        assert_debug_snapshot!(convert_expr_single("$unknown"));
    }

    #[test]
    fn test_unknown_named_function_ref() {
        assert_debug_snapshot!(convert_expr_single("unknown_function#2"));
    }
}
