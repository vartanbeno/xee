use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::builder::Program;
use crate::error::{Error, Result};
use crate::instruction::{decode_instruction, Instruction};
use crate::static_context::StaticContext;
use crate::value::{
    Atomic, Closure, Function, FunctionId, Item, Sequence, StackValue, StaticFunctionId,
};

#[derive(Debug, Clone)]
struct Frame {
    function: FunctionId,
    ip: usize,
    base: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Interpreter<'a> {
    program: &'a Program,
    static_context: &'a StaticContext,
    stack: Vec<StackValue>,
    frames: Vec<Frame>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(program: &'a Program, static_context: &'a StaticContext) -> Self {
        Interpreter {
            program,
            static_context,
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub(crate) fn stack(&self) -> &[StackValue] {
        &self.stack
    }

    pub(crate) fn start(&mut self, function_id: FunctionId) {
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: 0,
        });
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        let frame = self.frames.last().unwrap();

        let mut function = &self.program.functions[frame.function.0];
        let mut base = frame.base;
        let mut ip = frame.ip;
        while ip < function.chunk.len() {
            let (instruction, instruction_size) = decode_instruction(&function.chunk[ip..]);
            ip += instruction_size;
            match instruction {
                Instruction::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a.checked_add(b).ok_or(Error::IntegerOverflow)?;
                    self.stack.push(StackValue::Atomic(Atomic::Integer(result)));
                }
                Instruction::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a.checked_sub(b).ok_or(Error::IntegerOverflow)?;
                    self.stack.push(StackValue::Atomic(Atomic::Integer(result)));
                }
                Instruction::Const(index) => {
                    self.stack.push(function.constants[index as usize].clone());
                }
                Instruction::Closure(function_id) => {
                    let mut values = Vec::new();
                    let closure_function = &self.program.functions[function_id as usize];
                    for _ in 0..closure_function.closure_names.len() {
                        values.push(self.stack.pop().unwrap());
                    }
                    self.stack.push(StackValue::Closure(Rc::new(Closure {
                        function_id: FunctionId(function_id as usize),
                        values,
                    })));
                }
                Instruction::StaticFunction(static_function_id) => {
                    self.stack.push(StackValue::StaticFunction(StaticFunctionId(
                        static_function_id as usize,
                    )));
                }
                Instruction::Var(index) => {
                    self.stack.push(self.stack[base + index as usize].clone());
                }
                Instruction::Set(index) => {
                    self.stack[base + index as usize] = self.stack.pop().unwrap();
                }
                Instruction::ClosureVar(index) => {
                    // the closure is always just below the base
                    let closure = self.stack[base - 1].as_closure().ok_or(Error::TypeError)?;
                    // and we push the value we need onto the stack
                    self.stack.push(closure.values[index as usize].clone());
                }
                Instruction::Comma => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_sequence().ok_or(Error::TypeError)?;
                    let b = b.as_sequence().ok_or(Error::TypeError)?;
                    self.stack.push(StackValue::Sequence(Rc::new(RefCell::new(
                        a.borrow().concat(&b.borrow()),
                    ))));
                }
                Instruction::Jump(displacement) => {
                    ip = (ip as i32 + displacement as i32) as usize;
                }
                Instruction::Eq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    // XXX can functions be value compared?
                    let result = a == b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::Ne => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    // XXX can functions be value compared?
                    let result = a != b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::Lt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a < b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::Le => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a <= b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::Gt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a > b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::Ge => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a >= b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::TestTrue => {
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_bool().ok_or(Error::TypeError)?;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if a {
                        ip += 3;
                    }
                }
                Instruction::TestFalse => {
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_bool().ok_or(Error::TypeError)?;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if !a {
                        ip += 3;
                    }
                }
                Instruction::Dup => {
                    let a = self.stack.last().unwrap().clone();
                    self.stack.push(a);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Call(arity) => {
                    // XXX check that arity of function matches arity of call

                    // get callable from stack, by peeking back
                    let callable = &self.stack[self.stack.len() - (arity as usize + 1)];
                    if let Some(static_function_id) = callable.as_static_function() {
                        self.call_static(static_function_id, arity)?;
                    } else {
                        let closure = callable.as_closure().ok_or(Error::TypeError)?;
                        self.call_closure(
                            closure.function_id,
                            arity,
                            &mut ip,
                            &mut base,
                            &mut function,
                        )?;
                    }
                }
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap();

                    // truncate the stack to the base
                    self.stack.truncate(base);

                    // pop off the function id we just called
                    if !self.stack.is_empty() {
                        self.stack.pop();
                    }

                    // push back return value
                    self.stack.push(return_value);

                    // now pop off the frame
                    self.frames.pop();

                    if self.frames.is_empty() {
                        // we can't return any further, so we're done
                        break;
                    }
                    let frame = self.frames.last().unwrap();
                    base = frame.base;
                    ip = frame.ip;
                    function = &self.program.functions[frame.function.0];
                }
                Instruction::LetDone => {
                    let return_value = self.stack.pop().unwrap();
                    // pop the variable assignment
                    let _ = self.stack.pop();
                    self.stack.push(return_value);
                }
                Instruction::Range => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic().ok_or(Error::TypeError)?;
                    let b = b.as_atomic().ok_or(Error::TypeError)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    match a.cmp(&b) {
                        Ordering::Greater => self
                            .stack
                            .push(StackValue::Sequence(Rc::new(RefCell::new(Sequence::new())))),
                        Ordering::Equal => self.stack.push(StackValue::Atomic(Atomic::Integer(a))),
                        Ordering::Less => {
                            let sequence = Sequence::from_vec(
                                (a..=b)
                                    .map(|i| Item::Atomic(Atomic::Integer(i)))
                                    .collect::<Vec<Item>>(),
                            );
                            self.stack
                                .push(StackValue::Sequence(Rc::new(RefCell::new(sequence))));
                        }
                    }
                }
                Instruction::SequenceNew => {
                    let sequence = Sequence::new();
                    self.stack
                        .push(StackValue::Sequence(Rc::new(RefCell::new(sequence))));
                }
                Instruction::SequenceLen => {
                    let sequence = self.stack.pop().unwrap();
                    let sequence = sequence.as_sequence().ok_or(Error::TypeError)?;
                    let len = sequence.borrow().items.len();
                    self.stack
                        .push(StackValue::Atomic(Atomic::Integer(len as i64)));
                }
                Instruction::SequenceGet => {
                    let sequence = self.stack.pop().unwrap();
                    let index = self.stack.pop().unwrap();

                    let sequence = sequence.as_sequence().ok_or(Error::TypeError)?;
                    let index = index.as_atomic().ok_or(Error::TypeError)?;
                    let index = index.as_integer().ok_or(Error::TypeError)?;
                    let item = sequence.borrow().items[index as usize].clone();
                    match item {
                        Item::Atomic(atomic) => {
                            self.stack.push(StackValue::Atomic(atomic));
                        }
                        Item::Function(closure) => {
                            self.stack.push(StackValue::Closure(closure));
                        }
                    }
                }
                Instruction::SequencePush => {
                    let sequence = self.stack.pop().unwrap();
                    let stack_value = self.stack.pop().unwrap();

                    let sequence = sequence.as_sequence().ok_or(Error::TypeError)?;
                    sequence.borrow_mut().push_stack_value(stack_value);
                }
                Instruction::PrintTop => {
                    let top = self.stack.last().unwrap();
                    println!("{:#?}", top);
                }
                Instruction::PrintStack => {
                    println!("{:#?}", self.stack);
                }
            }
        }
        Ok(())
    }

    fn call_static(&mut self, static_function_id: StaticFunctionId, arity: u8) -> Result<()> {
        let static_function = &self
            .static_context
            .functions
            .get_by_index(static_function_id);
        let arguments = &self.stack[self.stack.len() - (arity as usize)..];
        let result = static_function.invoke(arguments)?;
        // truncate the stack to the base
        self.stack.truncate(self.stack.len() - (arity as usize + 1));
        self.stack.push(result);
        Ok(())
    }

    fn call_closure(
        &mut self,
        function_id: FunctionId,
        arity: u8,
        ip: &mut usize,
        base: &mut usize,
        function: &mut &'a Function,
    ) -> Result<()> {
        // store ip of next instruction in current frame
        let frame = self.frames.last_mut().unwrap();
        frame.ip = *ip;
        *function = &self.program.functions[function_id.0];
        let stack_size = self.stack.len();
        *base = stack_size - (arity as usize);
        *ip = 0;
        self.frames.push(Frame {
            function: function_id,
            ip: *ip,
            base: *base,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::builder::{Comparison, FunctionBuilder};
    use crate::instruction::decode_instructions;

    #[test]
    fn test_interpreter() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(2)));
        builder.emit(Instruction::Add);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let static_context = StaticContext::new();
        let mut interpreter = Interpreter::new(&program, &static_context);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(3))]
        );
        Ok(())
    }

    #[test]
    fn test_emit_jump_forward() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        let jump = builder.emit_jump_forward();
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(3)));
        builder.patch_jump(jump);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(4)));
        let function = builder.finish("main".to_string(), 0);

        let instructions = decode_instructions(&function.chunk);
        program.add_function(function);
        assert_eq!(
            instructions,
            vec![
                Instruction::Jump(3),
                Instruction::Const(0),
                Instruction::Const(1),
                Instruction::Return
            ]
        );
        Ok(())
    }

    #[test]
    fn test_condition_true() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(2)));
        let lt_false = builder.emit_compare_forward(Comparison::Lt);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(3)));
        let end = builder.emit_jump_forward();
        builder.patch_jump(lt_false);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(4)));
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let static_context = StaticContext::new();
        let mut interpreter = Interpreter::new(&program, &static_context);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(3))]
        );
        Ok(())
    }

    #[test]
    fn test_condition_false() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(2)));
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        let lt_false = builder.emit_compare_forward(Comparison::Lt);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(3)));
        let end = builder.emit_jump_forward();
        builder.patch_jump(lt_false);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(4)));
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let static_context = StaticContext::new();
        let mut interpreter = Interpreter::new(&program, &static_context);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(4))]
        );
        Ok(())
    }

    #[test]
    fn test_loop() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(10)));
        let loop_start = builder.loop_start();
        builder.emit(Instruction::Dup);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(5)));
        let end = builder.emit_compare_forward(Comparison::Gt);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)));
        builder.emit(Instruction::Sub);
        builder.emit_jump_backward(loop_start);
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let static_context = StaticContext::new();
        let mut interpreter = Interpreter::new(&program, &static_context);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(5))]
        );
        Ok(())
    }
}
