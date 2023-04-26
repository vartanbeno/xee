use std::cell::RefCell;
use std::rc::Rc;

use crate::ast_ir::Converter;
use crate::builder::{BackwardJumpRef, Comparison, FunctionBuilder, JumpCondition, Program};
use crate::context::Context;
use crate::error::Result;
use crate::instruction::Instruction;
use crate::interpret::{Interpreter, Mode};
use crate::ir;
use crate::parse_ast::parse_xpath;
use crate::value::{Atomic, FunctionId, Item, Node, Sequence, StackValue, StaticFunctionId};

type Scopes = crate::scope::Scopes<ir::Name>;

struct InterpreterCompiler<'a> {
    scopes: &'a mut Scopes,
    context: &'a Context<'a>,
    builder: FunctionBuilder<'a>,
    sequence_length_name: &'a ir::Name,
    sequence_index_name: &'a ir::Name,
}

impl<'a> InterpreterCompiler<'a> {
    fn compile_expr(&mut self, expr: &ir::Expr) {
        match expr {
            ir::Expr::Atom(atom) => {
                self.compile_atom(atom);
            }
            ir::Expr::Let(let_) => {
                self.compile_let(let_);
            }
            ir::Expr::Binary(binary) => {
                self.compile_binary(binary);
            }
            ir::Expr::FunctionDefinition(function_definition) => {
                self.compile_function_definition(function_definition);
            }
            ir::Expr::FunctionCall(function_call) => {
                self.compile_function_call(function_call);
            }
            ir::Expr::If(if_) => {
                self.compile_if(if_);
            }
            ir::Expr::Map(map) => {
                self.compile_map(map);
            }
            ir::Expr::Quantified(quantified) => {
                self.compile_quantified(quantified);
            }
            _ => {
                todo!()
            }
        }
    }

    fn compile_atom(&mut self, atom: &ir::Atom) {
        match atom {
            ir::Atom::Const(c) => {
                let stack_value = match c {
                    ir::Const::Integer(i) => StackValue::Atomic(Atomic::Integer(*i)),
                    ir::Const::EmptySequence => {
                        StackValue::Sequence(Rc::new(RefCell::new(Sequence::new())))
                    }
                    ir::Const::StaticFunction(id) => StackValue::StaticFunction(*id),
                    _ => {
                        todo!()
                    }
                };
                self.builder.emit_constant(stack_value);
            }
            ir::Atom::Variable(name) => {
                self.compile_variable(name);
            }
        }
    }

    fn compile_variable(&mut self, name: &ir::Name) {
        if let Some(index) = self.scopes.get(name) {
            if index > u16::MAX as usize {
                panic!("too many variables");
            }
            self.builder.emit(Instruction::Var(index as u16));
        } else {
            // if value is in any outer scopes
            if self.scopes.is_closed_over_name(name) {
                let index = self.builder.add_ir_closure_name(name);
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

    fn compile_variable_set(&mut self, name: &ir::Name) {
        if let Some(index) = self.scopes.get(name) {
            if index > u16::MAX as usize {
                panic!("too many variables");
            }
            self.builder.emit(Instruction::Set(index as u16));
        } else {
            panic!("can only set locals: {:?}", name);
        }
    }

    fn compile_let(&mut self, let_: &ir::Let) {
        self.compile_expr(&let_.var_expr);
        self.scopes.push_name(&let_.name);
        self.compile_expr(&let_.return_expr);
        self.builder.emit(Instruction::LetDone);
        self.scopes.pop_name();
    }

    fn compile_if(&mut self, if_: &ir::If) {
        self.compile_atom(&if_.condition);
        let jump_else = self.builder.emit_jump_forward(JumpCondition::False);
        self.compile_expr(&if_.then);
        let jump_end = self.builder.emit_jump_forward(JumpCondition::Always);
        self.builder.patch_jump(jump_else);
        self.compile_expr(&if_.else_);
        self.builder.patch_jump(jump_end);
    }

    fn compile_binary(&mut self, binary: &ir::Binary) {
        self.compile_atom(&binary.left);
        self.compile_atom(&binary.right);
        match &binary.op {
            ir::BinaryOp::Add => {
                self.builder.emit(Instruction::Add);
            }
            ir::BinaryOp::Sub => {
                self.builder.emit(Instruction::Sub);
            }
            ir::BinaryOp::Eq => {
                self.builder.emit(Instruction::Eq);
            }
            ir::BinaryOp::Ne => {
                self.builder.emit(Instruction::Ne);
            }
            ir::BinaryOp::Lt => {
                self.builder.emit(Instruction::Lt);
            }
            ir::BinaryOp::Le => {
                self.builder.emit(Instruction::Le);
            }
            ir::BinaryOp::Gt => {
                self.builder.emit(Instruction::Gt);
            }
            ir::BinaryOp::Ge => {
                self.builder.emit(Instruction::Ge);
            }
            ir::BinaryOp::Comma => {
                self.builder.emit(Instruction::Comma);
            }
            ir::BinaryOp::Union => {
                self.builder.emit(Instruction::Union);
            }
            ir::BinaryOp::Range => {
                self.builder.emit(Instruction::Range);
            }
        }
    }

    fn compile_function_definition(&mut self, function_definition: &ir::FunctionDefinition) {
        let nested_builder = self.builder.builder();
        self.scopes.push_scope();

        let mut compiler = InterpreterCompiler {
            builder: nested_builder,
            scopes: self.scopes,
            context: self.context,
            sequence_length_name: self.sequence_length_name,
            sequence_index_name: self.sequence_index_name,
        };

        for param in &function_definition.params {
            compiler.scopes.push_name(&param.0);
        }
        compiler.compile_expr(&function_definition.body);
        for _ in &function_definition.params {
            compiler.scopes.pop_name();
        }

        compiler.scopes.pop_scope();

        let function = compiler
            .builder
            .finish("inline".to_string(), function_definition.params.len());
        // now place all captured names on stack, to ensure we have the
        // closure
        // in reverse order so we can pop them off in the right order
        for name in function.ir_closure_names.iter().rev() {
            self.compile_variable(name);
        }
        let function_id = self.builder.add_function(function);
        self.builder
            .emit(Instruction::Closure(function_id.as_u16()));
    }

    fn compile_function_call(&mut self, function_call: &ir::FunctionCall) {
        self.compile_atom(&function_call.atom);
        for arg in &function_call.args {
            self.compile_atom(arg);
        }
        self.builder
            .emit(Instruction::Call(function_call.args.len() as u8));
    }

    fn compile_map(&mut self, map: &ir::Map) {
        // place the resulting sequence on the stack
        let new_sequence = ir::Name("xee_new_sequence".to_string());
        self.scopes.push_name(&new_sequence);
        self.builder.emit(Instruction::SequenceNew);

        let loop_start = self.compile_sequence_loop_init(&map.var_atom);

        self.compile_sequence_get_item(&map.var_atom);
        // name it
        self.scopes.push_name(&map.var_name);
        // execute the map expression, placing result on stack
        self.compile_expr(&map.return_expr);
        self.scopes.pop_name();

        // push result to new sequence
        self.compile_variable(&new_sequence);
        self.builder.emit(Instruction::SequencePush);

        // clean up the var_name item
        self.builder.emit(Instruction::Pop);

        self.compile_sequence_loop_iterate(loop_start);

        self.compile_sequence_loop_end();

        // pop new sequence name & sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
        self.scopes.pop_name();
    }

    fn compile_quantified(&mut self, quantified: &ir::Quantified) {
        let loop_start = self.compile_sequence_loop_init(&quantified.var_atom);

        self.compile_sequence_get_item(&quantified.var_atom);
        // name it
        self.scopes.push_name(&quantified.var_name);
        // execute the satisfies expression, placing result in on stack
        self.compile_expr(&quantified.satisifies_expr);
        self.scopes.pop_name();

        let jump_out_end = match quantified.quantifier {
            ir::Quantifier::Some => self.builder.emit_jump_forward(JumpCondition::True),
            ir::Quantifier::Every => self.builder.emit_jump_forward(JumpCondition::False),
        };
        // we didn't jump out, clean up quantifier variable
        self.builder.emit(Instruction::Pop);

        self.compile_sequence_loop_iterate(loop_start);

        // if we reached the end, without jumping out
        self.compile_sequence_loop_end();

        let reached_end_value = match quantified.quantifier {
            ir::Quantifier::Some => StackValue::Atomic(Atomic::Boolean(false)),
            ir::Quantifier::Every => StackValue::Atomic(Atomic::Boolean(true)),
        };
        self.builder.emit_constant(reached_end_value);
        let end = self.builder.emit_jump_forward(JumpCondition::Always);

        // we jumped out
        self.builder.patch_jump(jump_out_end);
        // clean up quantifier variable
        self.builder.emit(Instruction::Pop);
        self.compile_sequence_loop_end();

        let jumped_out_value = match quantified.quantifier {
            ir::Quantifier::Some => StackValue::Atomic(Atomic::Boolean(true)),
            ir::Quantifier::Every => StackValue::Atomic(Atomic::Boolean(false)),
        };
        // if we jumped out, we set satisfies to true
        self.builder.emit_constant(jumped_out_value);

        self.builder.patch_jump(end);
        // pop sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
    }

    fn compile_sequence_loop_init(&mut self, atom: &ir::Atom) -> BackwardJumpRef {
        //  sequence length
        self.compile_atom(atom);
        self.scopes.push_name(self.sequence_length_name);
        self.builder.emit(Instruction::SequenceLen);

        // place index on stack
        self.builder
            .emit_constant(StackValue::Atomic(Atomic::Integer(0)));
        self.scopes.push_name(self.sequence_index_name);
        self.builder.loop_start()
    }

    fn compile_sequence_get_item(&mut self, atom: &ir::Atom) {
        // get item at the index
        self.compile_variable(self.sequence_index_name);
        self.compile_atom(atom);
        self.builder.emit(Instruction::SequenceGet);
    }

    fn compile_sequence_loop_iterate(&mut self, loop_start: BackwardJumpRef) {
        // update index with 1
        self.compile_variable(self.sequence_index_name);
        self.builder
            .emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        self.builder.emit(Instruction::Add);
        self.compile_variable_set(self.sequence_index_name);
        // compare with sequence length
        self.compile_variable(self.sequence_index_name);
        self.compile_variable(self.sequence_length_name);
        // unless we reached the end, we jump back to the start
        self.builder.emit(Instruction::Lt);
        self.builder
            .emit_jump_backward(loop_start, JumpCondition::True);
    }

    fn compile_sequence_loop_end(&mut self) {
        // pop length and index
        self.builder.emit(Instruction::Pop);
        self.builder.emit(Instruction::Pop);
    }
}

pub(crate) struct CompiledXPath<'a> {
    program: Program,
    context: &'a Context<'a>,
    main: FunctionId,
}

impl<'a> CompiledXPath<'a> {
    pub(crate) fn new(context: &'a Context, xpath: &str) -> Self {
        let ast = parse_xpath(xpath);
        let mut converter = Converter::new(&context.static_context);
        let expr = converter.convert_xpath(&ast);
        // we get an inline function, unwrap it for now
        let (arg_name, expr) = unwrap_inline_function(expr);
        let mut program = Program::new();
        let mut scopes = Scopes::new(ir::Name("dummy".to_string()));
        let builder = FunctionBuilder::new(&mut program);
        let mut compiler = InterpreterCompiler {
            builder,
            scopes: &mut scopes,
            context,
            sequence_length_name: &ir::Name("xee_sequence_length".to_string()),
            sequence_index_name: &ir::Name("xee_sequence_index".to_string()),
        };
        compiler.scopes.push_name(&arg_name);
        compiler.compile_expr(&expr);

        let main = compiler.builder.finish("main".to_string(), 0);
        let main = program.add_function(main);
        Self {
            program,
            context,
            main,
        }
    }

    pub(crate) fn interpret(&self) -> Result<StackValue> {
        // a fake context value
        self.interpret_with_context(Item::Atomic(Atomic::Integer(0)))
    }

    pub(crate) fn interpret_with_context(&self, context_item: Item) -> Result<StackValue> {
        let mut interpreter = Interpreter::new(&self.program, self.context, context_item, Mode::Ir);
        interpreter.start(self.main);
        interpreter.run()?;
        // the stack has to be 1 values and return the result of the expression
        // why 1 value if the context item is on the top of the stack? This is because
        // the outer main function will pop the context item; this code is there to
        // remove the function id from the stack but the main function has no function id
        assert_eq!(
            interpreter.stack().len(),
            1,
            "stack must only have 1 value but found {:?}",
            interpreter.stack()
        );
        Ok(interpreter.stack().last().unwrap().clone())
    }

    pub(crate) fn interpret_with_xot_node(&self, node: xot::Node) -> Result<StackValue> {
        self.interpret_with_context(Item::Node(Node::Xot(node)))
    }
}

fn unwrap_inline_function(expr: ir::Expr) -> (ir::Name, ir::Expr) {
    match expr {
        ir::Expr::FunctionDefinition(ir::FunctionDefinition { params, body, .. }) => {
            assert_eq!(params.len(), 1);
            (params[0].0.clone(), *body)
        }
        _ => panic!("expected inline function"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_debug_snapshot;
    use std::cell::RefCell;
    use std::rc::Rc;
    use xot::Xot;

    use crate::{
        document::{Documents, Uri},
        value::{Item, Node, Sequence},
    };

    fn as_integer(value: &StackValue) -> i64 {
        value.as_atomic().unwrap().as_integer().unwrap()
    }

    fn as_bool(value: &StackValue) -> bool {
        value.as_atomic().unwrap().as_bool().unwrap()
    }

    fn as_sequence(value: &StackValue) -> Rc<RefCell<Sequence>> {
        value.as_sequence().unwrap()
    }

    fn xot_nodes_to_sequence(node: &[xot::Node]) -> Sequence {
        Sequence {
            items: node
                .iter()
                .map(|&node| Item::Node(Node::Xot(node)))
                .collect(),
        }
    }

    fn run(s: &str) -> StackValue {
        let xot = Xot::new();
        let context = Context::new(&xot);
        let xpath = CompiledXPath::new(&context, s);
        xpath.interpret().unwrap()
    }

    fn run_debug(s: &str) -> StackValue {
        let xot = Xot::new();
        let context = Context::new(&xot);
        let xpath = CompiledXPath::new(&context, s);
        dbg!(&xpath.program.get_function(0).decoded());
        xpath.interpret().unwrap()
    }

    #[test]
    fn test_compile_add() {
        assert_debug_snapshot!(run("1 + 2"));
    }

    #[test]
    fn test_nested() {
        assert_debug_snapshot!(run("1 + (8 - 2)"));
    }

    #[test]
    fn test_comma() {
        assert_debug_snapshot!(run("1, 2"));
    }

    #[test]
    fn test_empty_sequence() {
        assert_debug_snapshot!(run("()"));
    }

    #[test]
    fn test_comma_squences() {
        assert_debug_snapshot!(run("(1, 2), (3, 4)"));
    }

    #[test]
    fn test_let() {
        assert_debug_snapshot!(run("let $x := 1 return $x + 2"));
    }

    #[test]
    fn test_let_nested() {
        assert_debug_snapshot!(run("let $x := 1, $y := $x + 3 return $y + 5"));
    }

    #[test]
    fn test_let_on_right_side() {
        assert_debug_snapshot!(run("1 + (let $x := 2 return $x + 10)"));
    }

    #[test]
    fn test_if() {
        assert_debug_snapshot!(run("if (1) then 2 else 3"));
    }

    #[test]
    fn test_if_false() {
        assert_debug_snapshot!(run("if (0) then 2 else 3"));
    }

    #[test]
    fn test_if_with_let_true() {
        assert_debug_snapshot!(run(
            "if (1) then (let $x := 2 return $x) else (let $x := 3 return $x)"
        ));
    }

    #[test]
    fn test_if_with_let_false() {
        assert_debug_snapshot!(run(
            "if (0) then (let $x := 2 return $x) else (let $x := 3 return $x)"
        ));
    }

    #[test]
    fn test_value_eq_true() {
        assert_debug_snapshot!(run("1 eq 1"));
    }

    #[test]
    fn test_value_eq_false() {
        assert_debug_snapshot!(run("1 eq 2"));
    }

    #[test]
    fn test_value_ne_true() {
        assert_debug_snapshot!(run("1 ne 2"));
    }

    #[test]
    fn test_value_ne_false() {
        assert_debug_snapshot!(run("1 ne 1"));
    }

    #[test]
    fn test_value_lt_true() {
        assert_debug_snapshot!(run("1 lt 2"));
    }

    #[test]
    fn test_value_lt_false() {
        assert_debug_snapshot!(run("2 lt 1"));
    }

    #[test]
    fn test_inline_function_without_args() {
        assert_debug_snapshot!(run("function() { 5 } ()"));
    }

    #[test]
    fn test_inline_function_with_single_arg() {
        assert_debug_snapshot!(run("function($x) { $x + 5 } (3)"));
    }

    #[test]
    fn test_inline_function_with_multiple_args() {
        assert_debug_snapshot!(run("function($x, $y) { $x + $y } (3, 5)"));
    }

    #[test]
    fn test_function_nested() {
        assert_debug_snapshot!(run("function($x) { function($y) { $y + 2 }($x + 1) } (5)"));
    }

    #[test]
    fn test_function_closure() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3 return function() { $x + 2 } }()()"
        ));
    }

    #[test]
    fn test_function_closure_with_multiple_variables() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3, $y := 1 return function() { $x - $y } }()()"
        ));
    }

    #[test]
    fn test_function_closure_with_multiple_variables_arguments() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3 return function($y) { $x - $y } }()(1)"
        ));
    }

    #[test]
    fn test_function_closure_nested() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3 return function() { let $y := 4 return function() { $x + $y }} }()()()"
        ));
    }

    #[test]
    fn test_static_function_call() {
        assert_debug_snapshot!(run("my_function(5, 2)"));
    }

    #[test]
    fn test_named_function_ref_call() {
        assert_debug_snapshot!(run("my_function#2(5, 2)"));
    }

    #[test]
    fn test_static_call_with_placeholders() {
        assert_debug_snapshot!(run("my_function(?, 2)(5)"));
    }

    #[test]
    fn test_inline_function_with_args_placeholdered() {
        assert_debug_snapshot!(run("function($x, $y) { $x - $y } ( ?, 3 ) (5)"));
    }

    #[test]
    fn test_inline_function_with_args_placeholdered2() {
        assert_debug_snapshot!(run("function($x, $y) { $x - $y } ( ?, 3 ) (?) (5)"));
    }

    #[test]
    fn test_range() {
        assert_debug_snapshot!(run("1 to 5"));
    }

    #[test]
    fn test_range_greater() {
        assert_debug_snapshot!(run("5 to 1"));
    }

    #[test]
    fn test_range_equal() {
        assert_debug_snapshot!(run("1 to 1"));
    }

    #[test]
    fn test_for_loop() {
        assert_debug_snapshot!(run("for $x in 1 to 5 return $x + 2"));
    }

    #[test]
    fn test_nested_for_loop() {
        assert_debug_snapshot!(run("for $i in (10, 20, 30), $j in (1, 2) return $i + $j"));
    }

    #[test]
    fn test_nested_for_loop_variable_scope() {
        assert_debug_snapshot!(run(
            "for $i in (10, 20), $j in ($i + 1, $i + 2) return $i + $j"
        ));
    }

    #[test]
    fn test_simple_map() {
        assert_debug_snapshot!(run("(1, 2) ! (. + 1)"));
    }

    #[test]
    fn test_simple_map_sequence() {
        assert_debug_snapshot!(run("(1, 2) ! (., 0)"));
    }

    #[test]
    fn test_simple_map_single() {
        assert_debug_snapshot!(run("1 ! (. , 0)"));
    }

    #[test]
    fn test_simple_map_multiple_steps() {
        assert_debug_snapshot!(run("(1, 2) ! (. + 1) ! (. + 2)"));
    }

    #[test]
    fn test_simple_map_multiple_steps2() {
        assert_debug_snapshot!(run("(1, 2) ! (. + 1) ! (. + 2) ! (. + 3)"));
    }

    #[test]
    fn test_some_quantifier_expr_true() {
        assert_debug_snapshot!(run("some $x in (1, 2, 3) satisfies $x eq 2"));
    }
}
