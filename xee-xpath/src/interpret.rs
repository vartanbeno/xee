use miette::NamedSource;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::builder::Program;
use crate::context::Context;
use crate::error::Error;
use crate::instruction::{read_i16, read_instruction, read_u16, read_u8, EncodedInstruction};

use crate::step::resolve_step;
use crate::value::{
    Atomic, Closure, ClosureFunctionId, Function, FunctionId, Item, Sequence, StackValue,
    StaticFunctionId, Step, ValueError,
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

    pub(crate) fn run(&mut self) -> Result<(), Error> {
        // annotate run with detailed error information
        self.run_actual().map_err(|e| self.err(e))
    }

    pub(crate) fn run_actual(&mut self) -> Result<(), ValueError> {
        let frame = self.frames.last().unwrap();

        let context = self.context;
        let mut function = &self.program.functions[frame.function.0];
        let mut base = frame.base;
        let mut ip = frame.ip;
        loop {
            let instruction = read_instruction(&function.chunk, &mut ip);
            // let (instruction, instruction_size) = decode_instruction(&function.chunk[ip..]);
            // ip += instruction_size;
            match instruction {
                EncodedInstruction::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
                    let result = a.checked_add(b).ok_or(ValueError::OverflowError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Integer(result)));
                }
                EncodedInstruction::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
                    let result = a.checked_sub(b).ok_or(ValueError::OverflowError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Integer(result)));
                }
                EncodedInstruction::Concat => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_string().ok_or(ValueError::TypeError)?;
                    let b = b.as_string().ok_or(ValueError::TypeError)?;
                    let result = a + &b;
                    self.stack
                        .push(StackValue::Atomic(Atomic::String(Rc::new(result))));
                }
                EncodedInstruction::Const => {
                    let index = read_u16(&function.chunk, &mut ip);
                    self.stack.push(function.constants[index as usize].clone());
                }
                EncodedInstruction::Closure => {
                    let function_id = read_u16(&function.chunk, &mut ip);
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
                EncodedInstruction::StaticClosure => {
                    let static_function_id = read_u16(&function.chunk, &mut ip);
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
                EncodedInstruction::Var => {
                    let index = read_u16(&function.chunk, &mut ip);
                    self.stack.push(self.stack[base + index as usize].clone());
                }
                EncodedInstruction::Set => {
                    let index = read_u16(&function.chunk, &mut ip);
                    self.stack[base + index as usize] = self.stack.pop().unwrap();
                }
                EncodedInstruction::ClosureVar => {
                    let index = read_u16(&function.chunk, &mut ip);
                    // the closure is always just below the base
                    let closure = self.stack[base - 1]
                        .as_closure()
                        .ok_or(ValueError::TypeError)?;
                    // and we push the value we need onto the stack
                    self.stack.push(closure.values[index as usize].clone());
                }
                EncodedInstruction::Comma => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_sequence().ok_or(ValueError::TypeError)?;
                    let b = b.as_sequence().ok_or(ValueError::TypeError)?;
                    self.stack.push(StackValue::Sequence(Rc::new(RefCell::new(
                        a.borrow().concat(&b.borrow()),
                    ))));
                }
                EncodedInstruction::Jump => {
                    let displacement = read_i16(&function.chunk, &mut ip);
                    ip = (ip as i32 + displacement as i32) as usize;
                }
                EncodedInstruction::JumpIfTrue => {
                    let displacement = read_i16(&function.chunk, &mut ip);
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let a = a.as_bool().ok_or(ValueError::TypeError)?;
                    if a {
                        ip = (ip as i32 + displacement as i32) as usize;
                    }
                }
                EncodedInstruction::JumpIfFalse => {
                    let displacement = read_i16(&function.chunk, &mut ip);
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let a = a.as_bool().ok_or(ValueError::TypeError)?;
                    if !a {
                        ip = (ip as i32 + displacement as i32) as usize;
                    }
                }
                EncodedInstruction::Eq => {
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
                EncodedInstruction::Ne => {
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
                EncodedInstruction::Lt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a < b)));
                }
                EncodedInstruction::Le => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a <= b)));
                }
                EncodedInstruction::Gt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a > b)));
                }
                EncodedInstruction::Ge => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
                    self.stack.push(StackValue::Atomic(Atomic::Boolean(a >= b)));
                }
                EncodedInstruction::Union => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_sequence().ok_or(ValueError::TypeError)?;
                    let b = b.as_sequence().ok_or(ValueError::TypeError)?;
                    let combined = a
                        .borrow()
                        .union(&b.borrow(), &self.context.documents.annotations)?;
                    self.stack
                        .push(StackValue::Sequence(Rc::new(RefCell::new(combined))));
                }
                EncodedInstruction::Pop => {
                    self.stack.pop();
                }
                EncodedInstruction::Call => {
                    let arity = read_u8(&function.chunk, &mut ip);
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
                        return Err(ValueError::TypeError);
                    }
                }
                EncodedInstruction::Return => {
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
                EncodedInstruction::LetDone => {
                    let return_value = self.stack.pop().unwrap();
                    // pop the variable assignment
                    let _ = self.stack.pop();
                    self.stack.push(return_value);
                }
                EncodedInstruction::Range => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_atomic(context)?;
                    let b = b.as_atomic(context)?;
                    let a = a.as_integer().ok_or(ValueError::TypeError)?;
                    let b = b.as_integer().ok_or(ValueError::TypeError)?;
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
                EncodedInstruction::SequenceNew => {
                    let sequence = Sequence::new();
                    self.stack
                        .push(StackValue::Sequence(Rc::new(RefCell::new(sequence))));
                }
                EncodedInstruction::SequenceLen => {
                    let sequence = self.stack.pop().unwrap();
                    let sequence = sequence.as_sequence().ok_or(ValueError::TypeError)?;
                    let len = sequence.borrow().items.len();
                    self.stack
                        .push(StackValue::Atomic(Atomic::Integer(len as i64)));
                }
                EncodedInstruction::SequenceGet => {
                    let sequence = self.stack.pop().unwrap();
                    let index = self.stack.pop().unwrap();

                    let sequence = sequence.as_sequence().ok_or(ValueError::TypeError)?;
                    let index = index.as_atomic(context)?;
                    let index = index.as_integer().ok_or(ValueError::TypeError)?;
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
                EncodedInstruction::SequencePush => {
                    let sequence = self.stack.pop().unwrap();
                    let stack_value = self.stack.pop().unwrap();

                    let sequence = sequence.as_sequence().ok_or(ValueError::TypeError)?;
                    sequence.borrow_mut().push_stack_value(stack_value);
                }
                EncodedInstruction::PrintTop => {
                    let top = self.stack.last().unwrap();
                    println!("{:#?}", top);
                }
                EncodedInstruction::PrintStack => {
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
    ) -> Result<(), ValueError> {
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
    ) -> Result<(), ValueError> {
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

    fn call_step(&mut self, step: Rc<Step>) -> Result<(), ValueError> {
        // take one argument from the stack
        let node = self
            .stack
            .pop()
            .unwrap()
            .as_node()
            .ok_or(ValueError::TypeError)?;
        // pop off the callable too
        self.stack.pop();
        let sequence = resolve_step(step.as_ref(), node, self.context.xot);
        self.stack
            .push(StackValue::Sequence(Rc::new(RefCell::new(sequence))));
        Ok(())
    }

    fn err(&self, value_error: ValueError) -> Error {
        match value_error {
            ValueError::XPTY0004 => Error::XPTY0004 {
                src: NamedSource::new("input", self.context.src.to_string()),
                span: (0, 0).into(),
            },
            ValueError::TypeError => Error::XPTY0004 {
                src: NamedSource::new("input", self.context.src.to_string()),
                span: (0, 0).into(),
            },
            ValueError::OverflowError => Error::FOAR0002,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xot::Xot;

    use crate::builder::{FunctionBuilder, JumpCondition};
    use crate::instruction::{decode_instructions, Instruction};
    use crate::name::Namespaces;
    use crate::static_context::StaticContext;

    #[test]
    fn test_interpreter() -> Result<(), ValueError> {
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
        interpreter.run_actual()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(3))]
        );
        Ok(())
    }

    #[test]
    fn test_emit_jump_forward() -> Result<(), Error> {
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
    fn test_condition_true() -> Result<(), ValueError> {
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
        interpreter.run_actual()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(3))]
        );
        Ok(())
    }

    #[test]
    fn test_condition_false() -> Result<(), ValueError> {
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
        interpreter.run_actual()?;
        assert_eq!(
            interpreter.stack,
            vec![StackValue::Atomic(Atomic::Integer(4))]
        );
        Ok(())
    }
}
