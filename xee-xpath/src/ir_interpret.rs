use std::cell::RefCell;
use std::rc::Rc;

use crate::ast;
use crate::ast_ir::Converter;
use crate::builder::{BackwardJumpRef, Comparison, FunctionBuilder, JumpCondition, Program};
use crate::context::Context;
use crate::error::Result;
use crate::instruction::Instruction;
use crate::interpret::Interpreter;
use crate::ir;
use crate::parse_ast::parse_xpath;
use crate::value::{Atomic, FunctionId, Item, Node, Sequence, StackValue};

type Scopes = crate::scope::Scopes<ir::Name>;

struct InterpreterCompiler<'a> {
    scopes: &'a mut Scopes,
    context: &'a Context<'a>,
    builder: FunctionBuilder<'a>,
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
                    _ => {
                        todo!()
                    }
                };
                self.builder.emit_constant(stack_value);
            }
            ir::Atom::Variable(name) => {
                if let Some(index) = self.scopes.get(name) {
                    if index > u16::MAX as usize {
                        panic!("too many variables");
                    }
                    self.builder.emit(Instruction::Var(index as u16));
                } else {
                    // if value is in any outer scopes
                    todo!();
                    // if self.scopes.is_closed_over_name(name) {
                    //     let index = self.builder.add_closure_name(name);
                    //     if index > u16::MAX as usize {
                    //         panic!("too many closure variables");
                    //     }
                    //     self.builder.emit(Instruction::ClosureVar(index as u16));
                    // } else {
                    //     // XXX this should become an actual compile error
                    //     panic!("unknown variable {:?}", name);
                    // }
                }
            }
        }
    }

    fn compile_let(&mut self, let_: &ir::Let) {
        self.scopes.push_name(&let_.name);
        self.compile_expr(&let_.var_expr);
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
        // for name in function.closure_names.iter().rev() {
        //     self.compile_var_ref(name);
        // }
        let function_id = self.builder.add_function(function);
        self.builder
            .emit(Instruction::Closure(function_id.as_u16()));
    }

    fn compile_function_call(&mut self, function_call: &ir::FunctionCall) {
        self.builder.emit(Instruction::PrintStack);
        self.compile_atom(&function_call.atom);
        for arg in &function_call.args {
            self.compile_atom(arg);
        }
        self.builder
            .emit(Instruction::Call(function_call.args.len() as u8));
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
        let mut interpreter = Interpreter::new(&self.program, self.context, context_item);
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

    #[test]
    fn test_compile_add() {
        assert_debug_snapshot!(run("1 + 2"));
    }

    #[test]
    fn test_nested() {
        assert_debug_snapshot!(&run("1 + (8 - 2)"));
    }

    #[test]
    fn test_comma() {
        assert_debug_snapshot!(&run("1, 2"));
    }

    #[test]
    fn test_empty_sequence() {
        assert_debug_snapshot!(&run("()"));
    }

    #[test]
    fn test_comma_squences() {
        assert_debug_snapshot!(&run("(1, 2), (3, 4)"));
    }

    #[test]
    fn test_let() {
        assert_debug_snapshot!(&run("let $x := 1 return $x + 2"));
    }

    #[test]
    fn test_let_nested() {
        assert_debug_snapshot!(&run("let $x := 1, $y := $x + 3 return $y + 5"));
    }

    #[test]
    fn test_if() {
        assert_debug_snapshot!(&run("if (1) then 2 else 3"));
    }

    #[test]
    fn test_if_false() {
        assert_debug_snapshot!(&run("if (0) then 2 else 3"));
    }
}
