use crate::ast;
use crate::builder::{Comparison, FunctionBuilder, Program};
use crate::error::Result;
use crate::instruction::Instruction;
use crate::interpret::Interpreter;
use crate::parse_ast::parse_xpath;
use crate::scope::Scopes;
use crate::static_context::StaticContext;
use crate::value::{Atomic, FunctionId, StackValue};

struct InterpreterCompiler<'a> {
    scopes: &'a mut Scopes,
    static_context: &'a StaticContext,
    builder: FunctionBuilder<'a>,
    context_item_name: &'a ast::Name,
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
            self.builder.emit(Instruction::Comma);
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
                        self.builder.emit(Instruction::Range);
                    }
                    _ => {
                        panic!("operator not supported yet {:?}", binary_expr.operator);
                    }
                }
            }
            ast::ExprSingle::Let(let_expr) => {
                self.scopes.push_name(&let_expr.var_name);
                self.compile_expr_single(&let_expr.var_expr);
                self.compile_expr_single(&let_expr.return_expr);
                self.builder.emit(Instruction::LetDone);
                self.scopes.pop_name();
            }
            ast::ExprSingle::If(if_expr) => {
                self.compile_expr(&if_expr.condition);
                let jump_else = self.builder.emit_test_true_forward();
                self.compile_expr_single(&if_expr.then);
                let jump_end = self.builder.emit_jump_forward();
                self.builder.patch_jump(jump_else);
                self.compile_expr_single(&if_expr.else_);
                self.builder.patch_jump(jump_end);
            }
            ast::ExprSingle::For(for_expr) => {
                self.compile_map_expr(
                    |s| {
                        s.compile_expr_single(&for_expr.var_expr);
                    },
                    |s| {
                        // ensure it's named the loop item
                        s.scopes.push_name(&for_expr.var_name);
                        // execute expression over it
                        s.compile_expr_single(&for_expr.return_expr);
                        // named loop item
                        s.scopes.pop_name();
                    },
                    |s| {
                        // get rid of named loop item
                        s.builder.emit(Instruction::Pop);
                    },
                );
            }
            ast::ExprSingle::Apply(apply_expr) => match &apply_expr.operator {
                ast::ApplyOperator::SimpleMap(path_exprs) => {
                    self.compile_simple_map(&apply_expr.path_expr, path_exprs);
                }
                _ => {
                    panic!("apply operator not supported yet {:?}", apply_expr.operator);
                }
            },
            ast::ExprSingle::Quantified(quantified_expr) => {
                self.compile_quantified_expr(
                    &quantified_expr.quantifier,
                    |s| {
                        s.compile_expr_single(&quantified_expr.var_expr);
                    },
                    |s| {
                        // ensure it's named the loop item
                        s.scopes.push_name(&quantified_expr.var_name);
                        s.compile_expr_single(&quantified_expr.satisfies_expr);
                        // named loop item
                        s.scopes.pop_name();
                    },
                    |s| {
                        // get rid of named loop item
                        s.builder.emit(Instruction::Pop);
                    },
                );
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
                    self.builder
                        .emit_constant(StackValue::Atomic(Atomic::Integer(*i)));
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
                    static_context: self.static_context,
                    context_item_name: self.context_item_name,
                };
                compiler.compile_function(inline_function);

                compiler.scopes.pop_scope();

                let function = compiler
                    .builder
                    .finish("inline".to_string(), inline_function.params.len());

                // now place all captured names on stack, to ensure we have the
                // closure
                // in reverse order so we can pop them off in the right order
                for name in function.closure_names.iter().rev() {
                    self.compile_var_ref(name);
                }
                let function_id = self.builder.add_function(function);
                self.builder
                    .emit(Instruction::Closure(function_id.as_u16()));
            }
            ast::PrimaryExpr::FunctionCall(function_call) => {
                let arity = function_call.arguments.len();
                if arity > u8::MAX as usize {
                    panic!("too many arguments");
                }
                let function_id = self
                    .static_context
                    .functions
                    .get_by_name(&function_call.name, arity as u8)
                    .expect("static function not found");
                self.builder.emit_static_function(function_id);
                self.compile_call(&function_call.arguments);
            }
            ast::PrimaryExpr::NamedFunctionRef(named_function_ref) => {
                let function_id = self
                    .static_context
                    .functions
                    .get_by_name(&named_function_ref.name, named_function_ref.arity)
                    .expect("static function not found");
                self.builder.emit_static_function(function_id);
            }
            ast::PrimaryExpr::ContextItem => {
                self.compile_var_ref(self.context_item_name);
            }
            _ => {
                panic!("not supported yet {:?}", primary_expr);
            }
        }
    }

    fn compile_postfixes(&mut self, postfixes: &[ast::Postfix]) {
        for postfix in postfixes {
            match postfix {
                ast::Postfix::ArgumentList(arguments) => {
                    self.compile_call(arguments);
                }
                _ => {
                    panic!("not supported yet");
                }
            }
        }
    }

    fn compile_call(&mut self, arguments: &[ast::ExprSingle]) {
        for argument in arguments {
            self.compile_expr_single(argument);
        }
        self.builder.emit(Instruction::Call(arguments.len() as u8));
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

    fn compile_var_set(&mut self, name: &ast::Name) {
        if let Some(index) = self.scopes.get(name) {
            if index > u16::MAX as usize {
                panic!("too many variables");
            }
            self.builder.emit(Instruction::Set(index as u16));
        } else {
            panic!("can only set locals: {:?}", name);
        }
    }

    fn compile_map_expr<S, M, C>(
        &mut self,
        mut compile_sequence_expr: S,
        mut compile_map_expr: M,
        mut compile_map_cleanup: C,
    ) where
        S: FnMut(&mut Self),
        M: FnMut(&mut Self),
        C: FnMut(&mut Self),
    {
        // place the resulting sequence on the stack
        let new_sequence = ast::Name {
            name: "xee_new_sequence".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&new_sequence);
        self.builder.emit(Instruction::SequenceNew);

        // execute the sequence expression, placing sequence on stack
        let sequence = ast::Name {
            name: "xee_sequence".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&sequence);

        compile_sequence_expr(self);

        // and sequence length
        self.compile_var_ref(&sequence);
        let sequence_length = ast::Name {
            name: "xee_sequence_length".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&sequence_length);
        self.builder.emit(Instruction::SequenceLen);

        // place index on stack
        self.builder
            .emit_constant(StackValue::Atomic(Atomic::Integer(0)));
        let index = ast::Name {
            name: "xee_index".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&index);
        let loop_start = self.builder.loop_start();

        // get item at the index
        self.compile_var_ref(&index);
        self.compile_var_ref(&sequence);
        self.builder.emit(Instruction::SequenceGet);

        // execute the map expression, placing result on stack
        compile_map_expr(self);

        // push result to new sequence
        self.compile_var_ref(&new_sequence);
        self.builder.emit(Instruction::SequencePush);

        // we may need to clean up the stack after this
        compile_map_cleanup(self);

        // update the index with 1
        self.compile_var_ref(&index);
        self.builder
            .emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        self.builder.emit(Instruction::Add);
        self.compile_var_set(&index);
        // compare with sequence length
        self.compile_var_ref(&index);
        self.compile_var_ref(&sequence_length);
        // unless we reached the end, we jump back to the start
        self.builder
            .emit_compare_backward(Comparison::Ge, loop_start);
        // pop old sequence, length and index; new sequence is on top
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);

        // pop new sequence name & sequence name & sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
        self.scopes.pop_name();
        self.scopes.pop_name();
    }

    fn compile_quantified_expr<S, M, C>(
        &mut self,
        quantifier: &ast::Quantifier,
        mut compile_sequence_expr: S,
        mut compile_satisfies_expr: M,
        mut compile_satisfies_cleanup: C,
    ) where
        S: FnMut(&mut Self),
        M: FnMut(&mut Self),
        C: FnMut(&mut Self),
    {
        // execute the sequence expression, placing sequence on stack
        let sequence = ast::Name {
            name: "xee_sequence".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&sequence);
        compile_sequence_expr(self);

        // and sequence length
        self.compile_var_ref(&sequence);
        let sequence_length = ast::Name {
            name: "xee_sequence_length".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&sequence_length);
        self.builder.emit(Instruction::SequenceLen);

        // place index on stack
        self.builder
            .emit_constant(StackValue::Atomic(Atomic::Integer(0)));
        let index = ast::Name {
            name: "xee_index".to_string(),
            namespace: None,
        };
        self.scopes.push_name(&index);
        let loop_start = self.builder.loop_start();

        // get item at the index
        self.compile_var_ref(&index);
        self.compile_var_ref(&sequence);
        self.builder.emit(Instruction::SequenceGet);

        // execute the satisfies expression, placing result in on stack
        compile_satisfies_expr(self);

        let jump_out_end = match quantifier {
            ast::Quantifier::Some => self.builder.emit_test_false_forward(),
            ast::Quantifier::Every => self.builder.emit_test_true_forward(),
        };
        // we didn't jump out, clean up quantifier variable
        compile_satisfies_cleanup(self);

        // update the index with 1
        self.compile_var_ref(&index);
        self.builder
            .emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        self.builder.emit(Instruction::Add);
        self.compile_var_set(&index);
        // compare with sequence length
        self.compile_var_ref(&index);
        self.compile_var_ref(&sequence_length);
        // unless we reached the end, we jump back to the start
        self.builder
            .emit_compare_backward(Comparison::Ge, loop_start);
        // if we reached the end, without jumping out
        // pop old sequence, length and index
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);

        let reached_end_value = match quantifier {
            ast::Quantifier::Some => StackValue::Atomic(Atomic::Boolean(false)),
            ast::Quantifier::Every => StackValue::Atomic(Atomic::Boolean(true)),
        };
        self.builder.emit_constant(reached_end_value);
        let end = self.builder.emit_jump_forward();

        // we jumped out
        self.builder.patch_jump(jump_out_end);
        // clean up quantifier variable
        compile_satisfies_cleanup(self);
        // pop old sequence, length and index
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);

        let jumped_out_value = match quantifier {
            ast::Quantifier::Some => StackValue::Atomic(Atomic::Boolean(true)),
            ast::Quantifier::Every => StackValue::Atomic(Atomic::Boolean(false)),
        };
        // if we jumped out, we set satisfies to true
        self.builder.emit_constant(jumped_out_value);

        self.builder.patch_jump(end);
        // pop sequence name & sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
        self.scopes.pop_name();
    }

    fn compile_simple_map(&mut self, main_path_expr: &ast::PathExpr, path_exprs: &[ast::PathExpr]) {
        let path_expr = &path_exprs[0];
        let rest_path_expr = &path_exprs[1..];
        self.compile_map_expr(
            |s| {
                s.compile_path_expr(main_path_expr);
            },
            |s| {
                // ensure it's named the loop item
                s.scopes.push_name(s.context_item_name);
                s.compile_path_expr(path_expr);
                s.scopes.pop_name();
            },
            |s| {
                // get rid of context item
                s.builder.emit(Instruction::Pop);
            },
        );
        for path_expr in rest_path_expr {
            let map_result = ast::Name {
                name: "xee_map_result".to_string(),
                namespace: None,
            };
            self.scopes.push_name(&map_result);
            self.compile_map_expr(
                |s| s.compile_var_ref(&map_result),
                |s| {
                    // ensure it's named the loop item
                    s.scopes.push_name(s.context_item_name);
                    s.compile_path_expr(path_expr);
                    s.scopes.pop_name();
                },
                |s| {
                    // get rid of context item
                    s.builder.emit(Instruction::Pop);
                },
            );
            // the top of the stack contains the result of the map, but also the variable
            // under it, get rid of the variable
            self.builder.emit(Instruction::LetDone);
            self.scopes.pop_name();
        }
    }
}

pub(crate) struct CompiledXPath {
    program: Program,
    static_context: StaticContext,
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
            static_context: &StaticContext::new(),
            context_item_name: &ast::Name {
                name: "xee_context_item".to_string(),
                namespace: None,
            },
        };
        compiler.compile_xpath(&ast);
        let main = compiler.builder.finish("main".to_string(), 0);
        let main = program.add_function(main);
        let static_context = StaticContext::new();
        Self {
            program,
            static_context,
            main,
        }
    }

    pub(crate) fn interpret(&self) -> Result<StackValue> {
        let mut interpreter = Interpreter::new(&self.program, &self.static_context);
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
    use crate::value::{Item, Sequence};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn as_integer(value: &StackValue) -> i64 {
        value.as_atomic().unwrap().as_integer().unwrap()
    }

    fn as_bool(value: &StackValue) -> bool {
        value.as_atomic().unwrap().as_bool().unwrap()
    }

    fn as_sequence(value: &StackValue) -> Rc<RefCell<Sequence>> {
        value.as_sequence().unwrap()
    }

    #[test]
    fn test_compile_expr_single() -> Result<()> {
        let xpath = CompiledXPath::new("1 + 2");

        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 3);
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
        assert_eq!(as_integer(&result), 7);
        Ok(())
    }

    #[test]
    fn test_comma() -> Result<()> {
        let xpath = CompiledXPath::new("1, 2");
        let result = xpath.interpret()?;

        let sequence = result.as_sequence().unwrap();
        let expected_sequence = Sequence::from_vec(vec![
            Item::Atomic(Atomic::Integer(1)),
            Item::Atomic(Atomic::Integer(2)),
        ]);
        assert_eq!(sequence, Rc::new(RefCell::new(expected_sequence)));
        Ok(())
    }

    #[test]
    fn test_comma_sequences() -> Result<()> {
        let xpath = CompiledXPath::new("(1, 2), (3, 4)");
        let result = xpath.interpret()?;

        let sequence = result.as_sequence().unwrap();
        let expected_sequence = Sequence::from_vec(vec![
            Item::Atomic(Atomic::Integer(1)),
            Item::Atomic(Atomic::Integer(2)),
            Item::Atomic(Atomic::Integer(3)),
            Item::Atomic(Atomic::Integer(4)),
        ]);
        assert_eq!(sequence, Rc::new(RefCell::new(expected_sequence)));
        Ok(())
    }

    #[test]
    fn test_let() -> Result<()> {
        let xpath = CompiledXPath::new("let $x := 1 return $x + 2");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 3);
        Ok(())
    }

    #[test]
    fn test_let_nested() -> Result<()> {
        let xpath = CompiledXPath::new("let $x := 1, $y := $x + 3 return $y + 5");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 9);
        Ok(())
    }

    #[test]
    fn test_if() -> Result<()> {
        let xpath = CompiledXPath::new("if (1) then 2 else 3");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 2);
        Ok(())
    }

    #[test]
    fn test_if_false() -> Result<()> {
        let xpath = CompiledXPath::new("if (0) then 2 else 3");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 3);
        Ok(())
    }

    #[test]
    fn test_value_eq_true() -> Result<()> {
        let xpath = CompiledXPath::new("1 eq 1");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_value_eq_false() -> Result<()> {
        let xpath = CompiledXPath::new("1 eq 2");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_value_ne_true() -> Result<()> {
        let xpath = CompiledXPath::new("1 ne 2");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_value_ne_false() -> Result<()> {
        let xpath = CompiledXPath::new("1 ne 1");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_value_lt_true() -> Result<()> {
        let xpath = CompiledXPath::new("1 lt 2");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_value_lt_false() -> Result<()> {
        let xpath = CompiledXPath::new("2 lt 1");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_function_without_args() -> Result<()> {
        let xpath = CompiledXPath::new("function() { 5 } ()");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 5);
        Ok(())
    }

    #[test]
    fn test_function_with_args() -> Result<()> {
        let xpath = CompiledXPath::new("function($x) { $x + 5 } ( 3 )");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 8);
        Ok(())
    }

    #[test]
    fn test_function_with_args2() -> Result<()> {
        let xpath = CompiledXPath::new("function($x, $y) { $x + $y } ( 3, 5 )");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 8);
        Ok(())
    }

    #[test]
    fn test_function_nested() -> Result<()> {
        let xpath = CompiledXPath::new("function($x) { function($y) { $y + 2 }($x + 1) } (5)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 8);
        Ok(())
    }

    #[test]
    fn test_function_closure() -> Result<()> {
        let xpath =
            CompiledXPath::new("function() { let $x := 3 return function() { $x + 2 } }()()");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 5);
        Ok(())
    }

    #[test]
    fn test_function_closure_multiple_variables() -> Result<()> {
        let xpath = CompiledXPath::new(
            "function() { let $x := 3, $y := 1 return function() { $x - $y } }()()",
        );
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 2);
        Ok(())
    }

    #[test]
    fn test_function_closure_and_arguments() -> Result<()> {
        let xpath =
            CompiledXPath::new("function() { let $x := 3 return function($y) { $x - $y } }()(1)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 2);
        Ok(())
    }

    #[test]
    fn test_function_closure_nested() -> Result<()> {
        let xpath =
            CompiledXPath::new("function() { let $x := 3 return function() { let $y := 4 return function() { $x + $y }} }()()()");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 7);
        Ok(())
    }

    #[test]
    fn test_static_function_call() -> Result<()> {
        let xpath = CompiledXPath::new("my_function(5, 2)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 7);
        Ok(())
    }

    #[test]
    fn test_named_function_ref_call() -> Result<()> {
        let xpath = CompiledXPath::new("my_function#2(5, 2)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 7);
        Ok(())
    }

    #[test]
    fn test_static_call_with_placeholders() -> Result<()> {
        let xpath = CompiledXPath::new("my_function(?, 2)(5)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 7);
        Ok(())
    }

    #[test]
    fn test_function_with_args_placeholdered() -> Result<()> {
        let xpath = CompiledXPath::new("function($x, $y) { $x - $y } ( ?, 3 ) (5)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 2);
        Ok(())
    }

    #[test]
    fn test_function_with_args_placeholdered2() -> Result<()> {
        let xpath = CompiledXPath::new("function($x, $y) { $x - $y } ( ?, 3 ) (?) (5)");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 2);
        Ok(())
    }

    #[test]
    fn test_range() -> Result<()> {
        let xpath = CompiledXPath::new("1 to 5");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(1)),
                Item::Atomic(Atomic::Integer(2)),
                Item::Atomic(Atomic::Integer(3)),
                Item::Atomic(Atomic::Integer(4)),
                Item::Atomic(Atomic::Integer(5))
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_range_greater() -> Result<()> {
        let xpath = CompiledXPath::new("5 to 1");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![])))
        );
        Ok(())
    }

    #[test]
    fn test_range_equal() -> Result<()> {
        let xpath = CompiledXPath::new("1 to 1");
        let result = xpath.interpret()?;
        assert_eq!(as_integer(&result), 1);
        Ok(())
    }

    #[test]
    fn test_for_loop() -> Result<()> {
        let xpath = CompiledXPath::new("for $x in 1 to 5 return $x + 2");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(3)),
                Item::Atomic(Atomic::Integer(4)),
                Item::Atomic(Atomic::Integer(5)),
                Item::Atomic(Atomic::Integer(6)),
                Item::Atomic(Atomic::Integer(7))
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_nested_for_loop() -> Result<()> {
        let xpath = CompiledXPath::new("for $i in (10, 20), $j in (1, 2) return $i + $j");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(11)),
                Item::Atomic(Atomic::Integer(12)),
                Item::Atomic(Atomic::Integer(21)),
                Item::Atomic(Atomic::Integer(22)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_nested_for_loop_variable_scope() -> Result<()> {
        let xpath = CompiledXPath::new("for $i in (10, 20), $j in ($i + 1, $i + 2) return $i + $j");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(21)),
                Item::Atomic(Atomic::Integer(22)),
                Item::Atomic(Atomic::Integer(41)),
                Item::Atomic(Atomic::Integer(42)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_simple_map() -> Result<()> {
        let xpath = CompiledXPath::new("(1, 2) ! (. + 1)");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(2)),
                Item::Atomic(Atomic::Integer(3)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_simple_map_sequence() -> Result<()> {
        let xpath = CompiledXPath::new("(1, 2) ! (., 0)");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(1)),
                Item::Atomic(Atomic::Integer(0)),
                Item::Atomic(Atomic::Integer(2)),
                Item::Atomic(Atomic::Integer(0)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_simple_map_single() -> Result<()> {
        let xpath = CompiledXPath::new("1 ! (., 0)");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(1)),
                Item::Atomic(Atomic::Integer(0)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_simple_multiple_steps() -> Result<()> {
        let xpath = CompiledXPath::new("(1, 2) ! (. + 1) ! (. + 2)");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(4)),
                Item::Atomic(Atomic::Integer(5)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_simple_multiple_steps2() -> Result<()> {
        let xpath = CompiledXPath::new("(1, 2) ! (. + 1) ! (. + 2) ! (. + 3)");
        let result = xpath.interpret()?;
        assert_eq!(
            as_sequence(&result),
            Rc::new(RefCell::new(Sequence::from_vec(vec![
                Item::Atomic(Atomic::Integer(7)),
                Item::Atomic(Atomic::Integer(8)),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_some_quantifier_expr_true() -> Result<()> {
        let xpath = CompiledXPath::new("some $x in (1, 2, 3) satisfies $x eq 2");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_some_quantifier_expr_false() -> Result<()> {
        let xpath = CompiledXPath::new("some $x in (1, 2, 3) satisfies $x eq 5");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_nested_some_quantifier_expr_true() -> Result<()> {
        let xpath = CompiledXPath::new("some $x in (1, 2, 3), $y in (2, 3) satisfies $x gt $y");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_nested_some_quantifier_expr_false() -> Result<()> {
        let xpath = CompiledXPath::new("some $x in (1, 2, 3), $y in (5, 6) satisfies $x gt $y");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_every_quantifier_expr_true() -> Result<()> {
        let xpath = CompiledXPath::new("every $x in (1, 2, 3) satisfies $x gt 0");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_every_quantifier_expr_false() -> Result<()> {
        let xpath = CompiledXPath::new("every $x in (1, 2, 3) satisfies $x gt 2");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_every_quantifier_expr_nested_true() -> Result<()> {
        let xpath = CompiledXPath::new("every $x in (2, 3, 4), $y in (0, 1) satisfies $x gt $y");
        let result = xpath.interpret()?;
        assert!(as_bool(&result));
        Ok(())
    }

    #[test]
    fn test_every_quantifier_expr_nested_false() -> Result<()> {
        let xpath = CompiledXPath::new("every $x in (2, 3, 4), $y in (1, 2) satisfies $x gt $y");
        let result = xpath.interpret()?;
        assert!(!as_bool(&result));
        Ok(())
    }
}
