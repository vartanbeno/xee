use crate::ast;
use crate::builder::{Comparison, FunctionBuilder, Program};
use crate::error::Result;
use crate::instruction::Instruction;
use crate::interpret2::Interpreter;
use crate::parse_ast::parse_xpath;
use crate::value::{Closure, FunctionId, Value};

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

    fn known_name(&self, name: &ast::Name) -> bool {
        self.names.iter().any(|n| n == name)
    }
}

struct Scopes {
    scopes: Vec<Scope>,
}

impl Scopes {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn push_name(&mut self, name: &ast::Name) {
        self.scopes.last_mut().unwrap().names.push(name.clone());
    }

    fn pop_name(&mut self) {
        self.scopes.last_mut().unwrap().names.pop();
    }

    fn get(&self, name: &ast::Name) -> Option<usize> {
        self.scopes.last().unwrap().get(name)
    }

    fn is_closed_over_name(&self, name: &ast::Name) -> bool {
        let mut scopes = self.scopes.iter();
        scopes.next();
        scopes.any(|s| s.known_name(name))
    }
}

struct InterpreterCompiler<'a> {
    scopes: &'a mut Scopes,
    builder: FunctionBuilder<'a>,
}

impl<'a> InterpreterCompiler<'a> {
    fn compile_xpath(&mut self, xpath: &ast::XPath) {
        self.compile_expr(&xpath.exprs);
    }

    fn compile_expr(&mut self, exprs: &[ast::ExprSingle]) {
        let mut iter = exprs.iter();
        let first_expr = iter.next().unwrap();
        self.compile_expr_single(first_expr);

        for expr in iter {
            self.compile_expr_single(expr);
            // operations.push(Operation::Comma);
        }
    }

    fn compile_expr_single(&mut self, expr_single: &ast::ExprSingle) {
        match expr_single {
            ast::ExprSingle::Path(path_expr) => {
                self.compile_path_expr(path_expr);
            }
            ast::ExprSingle::Binary(binary_expr) => {
                self.compile_path_expr(&binary_expr.left);
                self.compile_path_expr(&binary_expr.right);
                match binary_expr.operator {
                    ast::Operator::Add => {
                        self.builder.emit(Instruction::Add);
                    }
                    ast::Operator::Sub => {
                        self.builder.emit(Instruction::Sub);
                    }
                    ast::Operator::ValueEq => self.builder.emit_compare_value(Comparison::Eq),
                    ast::Operator::ValueNe => self.builder.emit_compare_value(Comparison::Ne),
                    ast::Operator::ValueLt => self.builder.emit_compare_value(Comparison::Lt),
                    ast::Operator::ValueLe => self.builder.emit_compare_value(Comparison::Le),
                    ast::Operator::ValueGt => self.builder.emit_compare_value(Comparison::Gt),
                    ast::Operator::ValueGe => self.builder.emit_compare_value(Comparison::Ge),
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
                self.scopes.push_name(&let_expr.var_name);
                self.compile_expr_single(&let_expr.var_expr);
                self.compile_expr_single(&let_expr.return_expr);
                self.builder.emit(Instruction::LetDone);
                self.scopes.pop_name();
            }
            ast::ExprSingle::If(if_expr) => {
                self.compile_expr(&if_expr.condition);
                let jump_else = self.builder.emit_test_forward();
                self.compile_expr_single(&if_expr.then);
                let jump_end = self.builder.emit_jump_forward();
                self.builder.patch_jump(jump_else);
                self.compile_expr_single(&if_expr.else_);
                self.builder.patch_jump(jump_end);
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

    fn compile_path_expr(&mut self, path_expr: &ast::PathExpr) {
        let first_step = &path_expr.steps[0];
        self.compile_step_expr(first_step);
    }

    fn compile_step_expr(&mut self, step_expr: &ast::StepExpr) {
        match step_expr {
            ast::StepExpr::PrimaryExpr(primary_expr) => {
                self.compile_primary_expr(primary_expr);
            }
            ast::StepExpr::PostfixExpr { primary, postfixes } => {
                self.compile_primary_expr(primary);
                self.compile_postfixes(postfixes);
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    fn compile_primary_expr(&mut self, primary_expr: &ast::PrimaryExpr) {
        match primary_expr {
            ast::PrimaryExpr::Literal(literal) => match literal {
                ast::Literal::Integer(i) => {
                    self.builder.emit_constant(Value::Integer(*i));
                }
                // ast::Literal::String(s) => {
                //     operations.push(Operation::StringLiteral(s.to_string()));
                // }
                _ => {
                    panic!("literal type not supported yet");
                }
            },
            ast::PrimaryExpr::Expr(exprs) => {
                self.compile_expr(exprs);
            }
            ast::PrimaryExpr::VarRef(name) => {
                self.compile_var_ref(name);
            }
            ast::PrimaryExpr::InlineFunction(inline_function) => {
                let nested_builder = self.builder.builder();
                self.scopes.push_scope();

                let mut compiler = InterpreterCompiler {
                    builder: nested_builder,
                    scopes: self.scopes,
                };
                compiler.compile_function(inline_function);

                compiler.scopes.pop_scope();

                let function = compiler
                    .builder
                    .finish("inline".to_string(), inline_function.params.len());
                let amount = function.closure_names.len();
                if amount > u8::MAX as usize {
                    panic!("too many closure variables");
                }
                // now place all captured names on stack, to ensure we have the
                // closure
                // in reverse order so we can pop them off in the right order
                for name in function.closure_names.iter().rev() {
                    self.compile_var_ref(name);
                }
                let function_id = self.builder.add_function(function);
                self.builder
                    .emit(Instruction::Closure(function_id.as_u16(), amount as u8));
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    fn compile_postfixes(&mut self, postfixes: &[ast::Postfix]) {
        for postfix in postfixes {
            match postfix {
                ast::Postfix::ArgumentList(arguments) => {
                    for argument in arguments {
                        self.compile_argument(argument);
                    }
                    self.builder.emit(Instruction::Call(arguments.len() as u8));
                }
                _ => {
                    panic!("not supported yet");
                }
            }
        }
    }

    fn compile_argument(&mut self, argument: &ast::Argument) {
        match argument {
            ast::Argument::Expr(expr_single) => {
                self.compile_expr_single(expr_single);
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    fn compile_function(&mut self, function: &ast::InlineFunction) {
        for param in &function.params {
            self.scopes.push_name(&param.name);
        }
        self.compile_expr(&function.body);
        for _ in &function.params {
            self.scopes.pop_name();
        }
    }

    fn compile_var_ref(&mut self, name: &ast::Name) {
        if let Some(index) = self.scopes.get(name) {
            if index > u16::MAX as usize {
                panic!("too many variables");
            }
            self.builder.emit(Instruction::Var(index as u16));
        } else {
            // if value is in any outer scopes
            if self.scopes.is_closed_over_name(name) {
                let index = self.builder.add_closure_name(name);
                if index > u16::MAX as usize {
                    panic!("too many closure variables");
                }
                self.builder.emit(Instruction::ClosureVar(index as u16));
            } else {
                // XXX this should become an actual compile error
                panic!("unknown variable {:?}", name);
            }
        }
    }
}

pub(crate) struct CompiledXPath {
    program: Program,
    main: FunctionId,
}

impl CompiledXPath {
    pub(crate) fn new(xpath: &str) -> Self {
        let ast = parse_xpath(xpath);
        let mut program = Program::new();
        let mut scopes = Scopes::new();
        let builder = FunctionBuilder::new(&mut program);
        let mut compiler = InterpreterCompiler {
            builder,
            scopes: &mut scopes,
        };
        compiler.compile_xpath(&ast);
        let main = compiler.builder.finish("main".to_string(), 0);
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

    #[test]
    fn test_function_closure() -> Result<()> {
        let xpath =
            CompiledXPath::new("function() { let $x := 3 return function() { $x + 2 } }()()");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 5);
        Ok(())
    }

    #[test]
    fn test_function_closure_multiple_variables() -> Result<()> {
        let xpath = CompiledXPath::new(
            "function() { let $x := 3, $y := 1 return function() { $x - $y } }()()",
        );
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 2);
        Ok(())
    }

    #[test]
    fn test_function_closure_and_arguments() -> Result<()> {
        let xpath =
            CompiledXPath::new("function() { let $x := 3 return function($y) { $x - $y } }()(1)");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 2);
        Ok(())
    }

    #[test]
    fn test_function_closure_nested() -> Result<()> {
        let xpath =
            CompiledXPath::new("function() { let $x := 3 return function() { let $y := 4 return function() { $x + $y }} }()()()");
        let result = xpath.interpret()?;
        assert_eq!(result.as_integer()?, 7);
        Ok(())
    }
}
