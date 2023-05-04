use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::builder::Program;
use crate::context::Context;
use crate::error::{Error, Result};
use crate::instruction::{decode_instruction, Instruction};

use crate::step::resolve_step;
use crate::value::{
    Atomic, Closure, ClosureFunctionId, Function, FunctionId, Item, Sequence, StackValue,
    StaticFunctionId, Step,
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
    context: &'a Context<'a>,
    stack: Vec<StackValue>,
    frames: Vec<Frame>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(program: &'a Program, context: &'a Context) -> Self {
        Interpreter {
            program,
            context,
            stack: vec![],
            frames: Vec::new(),
        }
    }

    pub(crate) fn stack(&self) -> &[StackValue] {
        &self.stack
    }

    pub(crate) fn start(&mut self, function_id: FunctionId, context_item: Item) {
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: 0,
        });
        // the context item
        self.stack.push(StackValue::from_item(context_item));
        // position & size
        self.stack.push(StackValue::Atomic(Atomic::Integer(1)));
        self.stack.push(StackValue::Atomic(Atomic::Integer(1)));
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        let frame = self.frames.last().unwrap();

        let context = self.context;
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
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a.checked_add(b).ok_or(Error::FOAR0002)?;
                    self.stack.push(StackValue::Atomic(Atomic::Integer(result)));
                }
                Instruction::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    let result = a.checked_sub(b).ok_or(Error::FOAR0002)?;
                    self.stack.push(StackValue::Atomic(Atomic::Integer(result)));
                }
                Instruction::Concat => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_string().ok_or(Error::TypeError)?;
                    let b = b.as_string().ok_or(Error::TypeError)?;
                    let result = a + &b;
                    self.stack
                        .push(StackValue::Atomic(Atomic::String(Rc::new(result))));
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
                        function_id: ClosureFunctionId::Dynamic(FunctionId(function_id as usize)),
                        values,
                    })));
                }
                Instruction::StaticClosure(static_function_id) => {
                    let static_function = &self
                        .context
                        .static_context
                        .functions
                        .get_by_index(StaticFunctionId(static_function_id as usize));
                    // get any context value from the stack if needed
                    let values = match static_function.context_rule {
                        Some(_) => {
                            vec![self.stack.pop().unwrap()]
                        }
                        None => {
                            vec![]
                        }
                    };
                    self.stack.push(StackValue::Closure(Rc::new(Closure {
                        function_id: ClosureFunctionId::Static(StaticFunctionId(
                            static_function_id as usize,
                        )),
                        values,
                    })));
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
                Instruction::JumpIfTrue(displacement) => {
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let a = a.as_bool().ok_or(Error::TypeError)?;
                    if a {
                        ip = (ip as i32 + displacement as i32) as usize;
                    }
                }
                Instruction::JumpIfFalse(displacement) => {
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let a = a.as_bool().ok_or(Error::TypeError)?;
                    if !a {
                        ip = (ip as i32 + displacement as i32) as usize;
                    }
                }
                Instruction::Eq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    if a == Atomic::Empty {
                        self.stack.push(StackValue::Atomic(Atomic::Boolean(false)));
                        continue;
                    }
                    let b = b.as_atomic(context)?;
                    if b == Atomic::Empty {
                        self.stack.push(StackValue::Atomic(Atomic::Boolean(false)));
                        continue;
                    }
                    // XXX can functions be value compared?
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a == b)));
                }
                Instruction::Ne => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    if a == Atomic::Empty {
                        self.stack.push(StackValue::Atomic(Atomic::Boolean(false)));
                        continue;
                    }
                    let b = b.as_atomic(context)?;
                    if b == Atomic::Empty {
                        self.stack.push(StackValue::Atomic(Atomic::Boolean(false)));
                        continue;
                    }
                    // XXX can functions be value compared?
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a != b)));
                }
                Instruction::Lt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a < b)));
                }
                Instruction::Le => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a <= b)));
                }
                Instruction::Gt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a > b)));
                }
                Instruction::Ge => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(Error::TypeError)?;
                    let b = b.as_integer().ok_or(Error::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a >= b)));
                }
                Instruction::Union => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_sequence().ok_or(Error::TypeError)?;
                    let b = b.as_sequence().ok_or(Error::TypeError)?;
                    let combined = a
                        .borrow()
                        .union(&b.borrow(), &self.context.documents.annotations)?;
                    self.stack
                        .push(StackValue::Sequence(Rc::new(RefCell::new(combined))));
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Call(arity) => {
                    // XXX check that arity of function matches arity of call

                    // get callable from stack, by peeking back
                    let callable = &self.stack[self.stack.len() - (arity as usize + 1)];

                    if let Some(closure) = callable.as_closure() {
                        match closure.function_id {
                            ClosureFunctionId::Dynamic(function_id) => {
                                self.call_closure(
                                    function_id,
                                    arity,
                                    &mut ip,
                                    &mut base,
                                    &mut function,
                                )?;
                            }
                            ClosureFunctionId::Static(static_function_id) => {
                                // XXX wish I didn't need to clone
                                let closure_values = &closure.values.clone();
                                self.call_static(static_function_id, arity, closure_values)?;
                            }
                        }
                    } else if let Some(step) = callable.as_step() {
                        self.call_step(step)?;
                    } else {
                        return Err(Error::TypeError);
                    }
                }
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap();

                    // truncate the stack to the base
                    self.stack.truncate(base);

                    // pop off the function id we just called
                    // for the outer main function this is the context item
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
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
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
                    let index = index.as_atomic(context)?;
                    let index = index.as_integer().ok_or(Error::TypeError)?;
                    // substract 1 as Xpath is 1-indexed
                    let item = sequence.borrow().items[index as usize - 1].clone();
                    match item {
                        Item::Atomic(atomic) => {
                            self.stack.push(StackValue::Atomic(atomic));
                        }
                        Item::Function(closure) => {
                            self.stack.push(StackValue::Closure(closure));
                        }
                        Item::Node(node) => {
                            self.stack.push(StackValue::Node(node));
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

    fn call_static(
        &mut self,
        static_function_id: StaticFunctionId,
        arity: u8,
        closure_values: &[StackValue],
    ) -> Result<()> {
        let static_function = &self
            .context
            .static_context
            .functions
            .get_by_index(static_function_id);
        let arguments = &self.stack[self.stack.len() - (arity as usize)..];
        let result = static_function.invoke(self.context, arguments, closure_values)?;
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

    fn call_step(&mut self, step: Rc<Step>) -> Result<()> {
        // take one argument from the stack
        let node = self
            .stack
            .pop()
            .unwrap()
            .as_node()
            .ok_or(Error::TypeError)?;
        // pop off the callable too
        self.stack.pop();
        let sequence = resolve_step(step.as_ref(), node, self.context.xot);
        self.stack
            .push(StackValue::Sequence(Rc::new(RefCell::new(sequence))));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xot::Xot;

    use crate::builder::{FunctionBuilder, JumpCondition};
    use crate::instruction::decode_instructions;
    use crate::name::Namespaces;
    use crate::static_context::StaticContext;

    #[test]
    fn test_interpreter() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        let empty_span = (0, 0).into();
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)), empty_span);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(2)), empty_span);
        builder.emit(Instruction::Add, empty_span);
        let function = builder.finish("main".to_string(), 0, empty_span);

        let main_id = program.add_function(function);
        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = Context::new(&xot, "", static_context);

        let mut interpreter = Interpreter::new(&program, &context);
        interpreter.start(main_id, Item::Atomic(Atomic::Integer(0)));
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
        let empty_span = (0, 0).into();
        let jump = builder.emit_jump_forward(JumpCondition::Always, empty_span);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(3)), empty_span);
        builder.patch_jump(jump);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(4)), empty_span);
        let function = builder.finish("main".to_string(), 0, empty_span);

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
        let empty_span = (0, 0).into();
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)), empty_span);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(2)), empty_span);
        builder.emit(Instruction::Lt, empty_span);
        let lt_false = builder.emit_jump_forward(JumpCondition::False, empty_span);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(3)), empty_span);
        let end = builder.emit_jump_forward(JumpCondition::Always, empty_span);
        builder.patch_jump(lt_false);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(4)), empty_span);
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0, empty_span);

        let main_id = program.add_function(function);

        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = Context::new(&xot, "", static_context);

        let mut interpreter = Interpreter::new(&program, &context);
        interpreter.start(main_id, Item::Atomic(Atomic::Integer(0)));
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
        let empty_span = (0, 0).into();
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(2)), empty_span);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(1)), empty_span);
        builder.emit(Instruction::Lt, empty_span);
        let lt_false = builder.emit_jump_forward(JumpCondition::False, empty_span);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(3)), empty_span);
        let end = builder.emit_jump_forward(JumpCondition::Always, empty_span);
        builder.patch_jump(lt_false);
        builder.emit_constant(StackValue::Atomic(Atomic::Integer(4)), empty_span);
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0, empty_span);

        let main_id = program.add_function(function);

        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = Context::new(&xot, "", static_context);
        let mut interpreter = Interpreter::new(&program, &context);
        interpreter.start(main_id, Item::Atomic(Atomic::Integer(0)));
        interpreter.run()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(4))]
        );
        Ok(())
    }
}
