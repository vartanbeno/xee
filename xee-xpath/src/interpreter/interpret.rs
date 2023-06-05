use arrayvec::ArrayVec;
use miette::SourceSpan;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::comparison;
use crate::context::DynamicContext;
use crate::data::{
    Atomic, Closure, ClosureFunctionId, ContextInto, ContextTryInto, Function, FunctionId, Item,
    Sequence, StaticFunctionId, Step, Value, ValueError,
};
use crate::error::Error;
use crate::interpreter::builder::Program;
use crate::interpreter::instruction::{
    read_i16, read_instruction, read_u16, read_u8, EncodedInstruction,
};
use crate::op;
use crate::step::resolve_step;

type Seq = Rc<RefCell<Sequence>>;

const FRAMES_MAX: usize = 64;

#[derive(Debug, Clone)]
struct Frame {
    function: FunctionId,
    ip: usize,
    base: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Interpreter<'a> {
    program: &'a Program,
    dynamic_context: &'a DynamicContext<'a>,
    stack: Vec<Value>,
    frames: ArrayVec<Frame, FRAMES_MAX>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(program: &'a Program, dynamic_context: &'a DynamicContext) -> Self {
        Interpreter {
            program,
            dynamic_context,
            stack: vec![],
            frames: ArrayVec::new(),
        }
    }

    pub(crate) fn stack(&self) -> &[Value] {
        &self.stack
    }

    pub(crate) fn start(
        &mut self,
        function_id: FunctionId,
        context_item: Option<&Item>,
        arguments: &[Value],
    ) {
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: 0,
        });
        if let Some(context_item) = context_item {
            // the context item
            self.stack.push(Value::from_item(context_item.clone()));
            // position & size
            self.stack.push(Value::Atomic(Atomic::Integer(1)));
            self.stack.push(Value::Atomic(Atomic::Integer(1)));
        } else {
            // absent context, position and size
            self.stack.push(Value::Atomic(Atomic::Absent));
            self.stack.push(Value::Atomic(Atomic::Absent));
            self.stack.push(Value::Atomic(Atomic::Absent));
        }
        // and any arguments
        for arg in arguments {
            self.stack.push(arg.clone());
        }
    }

    pub(crate) fn run(&mut self) -> Result<(), Error> {
        // annotate run with detailed error information
        self.run_actual().map_err(|e| self.err(e))
    }

    fn frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    pub(crate) fn function(&self) -> &Function {
        &self.program.functions[self.frame().function.0]
    }

    pub(crate) fn run_actual(&mut self) -> Result<(), ValueError> {
        let context = self.dynamic_context;
        // we can make this an infinite loop as all functions end
        // with the return instruction
        loop {
            let instruction = self.read_instruction();
            match instruction {
                EncodedInstruction::Add => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack.push(Value::Atomic(op::numeric_add(&a, &b)?));
                }
                EncodedInstruction::Sub => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(op::numeric_substract(&a, &b)?));
                }
                EncodedInstruction::Mul => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(op::numeric_multiply(&a, &b)?));
                }
                EncodedInstruction::Div => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack.push(Value::Atomic(op::numeric_divide(&a, &b)?));
                }
                EncodedInstruction::IntDiv => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(op::numeric_integer_divide(&a, &b)?));
                }
                EncodedInstruction::Mod => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack.push(Value::Atomic(op::numeric_mod(&a, &b)?));
                }
                EncodedInstruction::Concat => {
                    let (a, b) = self.pop_atomic2()?;
                    let a = a.to_str()?;
                    let b = b.to_str()?;
                    let result = a.to_owned() + b;
                    self.stack
                        .push(Value::Atomic(Atomic::String(Rc::new(result))));
                }
                EncodedInstruction::Const => {
                    let index = self.read_u16();
                    self.stack
                        .push(self.function().constants[index as usize].clone());
                }
                EncodedInstruction::Closure => {
                    let function_id = self.read_u16();
                    let mut values = Vec::new();
                    let closure_function = &self.program.functions[function_id as usize];
                    for _ in 0..closure_function.closure_names.len() {
                        values.push(self.stack.pop().unwrap());
                    }
                    self.stack.push(Value::Closure(Rc::new(Closure {
                        function_id: ClosureFunctionId::Dynamic(FunctionId(function_id as usize)),
                        values,
                    })));
                }
                EncodedInstruction::StaticClosure => {
                    let static_function_id = self.read_u16();
                    let static_function = &self
                        .dynamic_context
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
                    self.stack.push(Value::Closure(Rc::new(Closure {
                        function_id: ClosureFunctionId::Static(StaticFunctionId(
                            static_function_id as usize,
                        )),
                        values,
                    })));
                }
                EncodedInstruction::Var => {
                    let index = self.read_u16();
                    self.stack
                        .push(self.stack[self.frame().base + index as usize].clone());
                }
                EncodedInstruction::Set => {
                    let index = self.read_u16();
                    let base = self.frame().base;
                    self.stack[base + index as usize] = self.stack.pop().unwrap();
                }
                EncodedInstruction::ClosureVar => {
                    let index = self.read_u16();
                    // the closure is always just below the base
                    let closure = self.stack[self.frame().base - 1].to_closure()?;
                    // and we push the value we need onto the stack
                    self.stack.push(closure.values[index as usize].clone());
                }
                EncodedInstruction::Comma => {
                    let (a, b) = self.pop_seq2()?;
                    self.stack.push(Value::Sequence(Rc::new(RefCell::new(
                        a.borrow().concat(&b.borrow()),
                    ))));
                }
                EncodedInstruction::Jump => {
                    let displacement = self.read_i16();
                    self.frame_mut().ip = (self.frame().ip as i32 + displacement as i32) as usize;
                }
                EncodedInstruction::JumpIfTrue => {
                    let displacement = self.read_i16();
                    let a = self.pop_effective_boolean()?;
                    if a {
                        self.frame_mut().ip =
                            (self.frame().ip as i32 + displacement as i32) as usize;
                    }
                }
                EncodedInstruction::JumpIfFalse => {
                    let displacement = self.read_i16();
                    let a = self.pop_effective_boolean()?;
                    if !a {
                        self.frame_mut().ip =
                            (self.frame().ip as i32 + displacement as i32) as usize;
                    }
                }
                EncodedInstruction::Eq => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(comparison::value_eq(&a, &b)?));
                }
                EncodedInstruction::Ne => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(comparison::value_ne(&a, &b)?));
                }
                EncodedInstruction::Lt => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(comparison::value_lt(&a, &b)?));
                }
                EncodedInstruction::Le => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(comparison::value_le(&a, &b)?));
                }
                EncodedInstruction::Gt => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(comparison::value_gt(&a, &b)?));
                }
                EncodedInstruction::Ge => {
                    let (a, b) = self.pop_atomic2()?;
                    self.stack
                        .push(Value::Atomic(comparison::value_ge(&a, &b)?));
                }
                EncodedInstruction::GenEq => {
                    let (atomized_a, atomized_b) = self.pop_atomized2()?;
                    self.stack.push(Value::Atomic(comparison::general_eq(
                        &atomized_a,
                        &atomized_b,
                    )?));
                }
                EncodedInstruction::GenNe => {
                    let (atomized_a, atomized_b) = self.pop_atomized2()?;
                    self.stack.push(Value::Atomic(comparison::general_ne(
                        &atomized_a,
                        &atomized_b,
                    )?));
                }
                EncodedInstruction::GenLt => {
                    let (atomized_a, atomized_b) = self.pop_atomized2()?;
                    self.stack.push(Value::Atomic(comparison::general_lt(
                        &atomized_a,
                        &atomized_b,
                    )?));
                }
                EncodedInstruction::GenLe => {
                    let (atomized_a, atomized_b) = self.pop_atomized2()?;
                    self.stack.push(Value::Atomic(comparison::general_le(
                        &atomized_a,
                        &atomized_b,
                    )?));
                }
                EncodedInstruction::GenGt => {
                    let (atomized_a, atomized_b) = self.pop_atomized2()?;
                    self.stack.push(Value::Atomic(comparison::general_gt(
                        &atomized_a,
                        &atomized_b,
                    )?));
                }
                EncodedInstruction::GenGe => {
                    let (atomized_a, atomized_b) = self.pop_atomized2()?;
                    self.stack.push(Value::Atomic(comparison::general_ge(
                        &atomized_a,
                        &atomized_b,
                    )?));
                }
                EncodedInstruction::Union => {
                    let (a, b) = self.pop_seq2()?;
                    let combined = a
                        .borrow()
                        .union(&b.borrow(), &self.dynamic_context.documents.annotations)?;
                    self.stack
                        .push(Value::Sequence(Rc::new(RefCell::new(combined))));
                }
                EncodedInstruction::Dup => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(value.clone());
                    self.stack.push(value);
                }
                EncodedInstruction::Pop => {
                    self.stack.pop();
                }
                EncodedInstruction::Call => {
                    let arity = self.read_u8();
                    // XXX check that arity of function matches arity of call

                    // get callable from stack, by peeking back
                    let callable = &self.stack[self.stack.len() - (arity as usize + 1)];

                    if let Ok(closure) = callable.to_closure() {
                        match closure.function_id {
                            ClosureFunctionId::Dynamic(function_id) => {
                                self.call_closure(function_id, arity)?;
                            }
                            ClosureFunctionId::Static(static_function_id) => {
                                // XXX wish I didn't need to clone
                                let closure_values = &closure.values.clone();
                                self.call_static(static_function_id, arity, closure_values)?;
                            }
                        }
                    } else if let Ok(step) = callable.to_step() {
                        self.call_step(step)?;
                    } else {
                        return Err(ValueError::Type);
                    }
                }
                EncodedInstruction::Return => {
                    let return_value = self.stack.pop().unwrap();

                    // truncate the stack to the base
                    self.stack.truncate(self.frame().base);

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
                }
                EncodedInstruction::LetDone => {
                    let return_value = self.stack.pop().unwrap();
                    // pop the variable assignment
                    let _ = self.stack.pop();
                    self.stack.push(return_value);
                }
                EncodedInstruction::Range => {
                    let (a, b) = self.pop_atomic2()?;
                    let a: i64 = a.try_into()?;
                    let b: i64 = b.try_into()?;
                    match a.cmp(&b) {
                        Ordering::Greater => self
                            .stack
                            .push(Value::Sequence(Rc::new(RefCell::new(Sequence::new())))),
                        Ordering::Equal => self.stack.push(Value::Atomic(Atomic::Integer(a))),
                        Ordering::Less => {
                            let sequence = Sequence::from_vec(
                                (a..=b)
                                    .map(|i| Item::Atomic(Atomic::Integer(i)))
                                    .collect::<Vec<Item>>(),
                            );
                            self.stack
                                .push(Value::Sequence(Rc::new(RefCell::new(sequence))));
                        }
                    }
                }
                EncodedInstruction::SequenceNew => {
                    let sequence = Sequence::new();
                    self.stack
                        .push(Value::Sequence(Rc::new(RefCell::new(sequence))));
                }
                EncodedInstruction::SequenceLen => {
                    let sequence = self.pop_seq()?;
                    let len = sequence.borrow().items.len();
                    self.stack.push(Value::Atomic(Atomic::Integer(len as i64)));
                }
                EncodedInstruction::SequenceGet => {
                    let sequence = self.pop_seq()?;
                    let index = self.pop_atomic()?;
                    let index: i64 = index.try_into()?;
                    // substract 1 as Xpath is 1-indexed
                    let item = sequence.borrow().items[index as usize - 1].clone();
                    self.stack.push(item.to_stack_value())
                }
                EncodedInstruction::SequencePush => {
                    let sequence = self.pop_seq()?;
                    let stack_value = self.stack.pop().unwrap();
                    sequence.borrow_mut().push_value(stack_value);
                }
                EncodedInstruction::IsNumeric => {
                    // This may fail. This is fine, as the only check later on
                    // in Filter is to check for effective boolean value, which
                    // uses effectively the same check
                    let atomic = self.pop_atomic()?;
                    let is_numeric = atomic.is_numeric();
                    self.stack.push(Value::Atomic(Atomic::Boolean(is_numeric)));
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
        closure_values: &[Value],
    ) -> Result<(), ValueError> {
        let static_function = &self
            .dynamic_context
            .static_context
            .functions
            .get_by_index(static_function_id);
        let arguments = &self.stack[self.stack.len() - (arity as usize)..];
        let result = static_function.invoke(self.dynamic_context, arguments, closure_values)?;
        // truncate the stack to the base
        self.stack.truncate(self.stack.len() - (arity as usize + 1));
        self.stack.push(result);
        Ok(())
    }

    fn call_closure(&mut self, function_id: FunctionId, arity: u8) -> Result<(), ValueError> {
        if self.frames.len() >= self.frames.capacity() {
            return Err(ValueError::StackOverflow);
        }
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: self.stack.len() - (arity as usize),
        });
        Ok(())
    }

    fn call_step(&mut self, step: Rc<Step>) -> Result<(), ValueError> {
        // take one argument from the stack
        let node = self.stack.pop().unwrap().to_node()?;
        // pop off the callable too
        self.stack.pop();
        let sequence = resolve_step(step.as_ref(), node, self.dynamic_context.xot);
        self.stack
            .push(Value::Sequence(Rc::new(RefCell::new(sequence))));
        Ok(())
    }

    fn pop_atomic(&mut self) -> Result<Atomic, ValueError> {
        let atomic = self.stack.pop().unwrap();
        atomic.context_try_into(self.dynamic_context)
    }

    fn pop_atomic2(&mut self) -> Result<(Atomic, Atomic), ValueError> {
        let b = self.pop_atomic()?;
        let a = self.pop_atomic()?;
        Ok((a, b))
    }

    fn pop_seq(&mut self) -> Result<Seq, ValueError> {
        let sequence = self.stack.pop().unwrap();
        sequence.try_into()
    }

    fn pop_seq2(&mut self) -> Result<(Seq, Seq), ValueError> {
        let b = self.pop_seq()?;
        let a = self.pop_seq()?;
        Ok((a, b))
    }

    fn pop_atomized2(&mut self) -> Result<(Vec<Atomic>, Vec<Atomic>), ValueError> {
        let (sequence_a, sequence_b) = self.pop_seq2()?;
        let atomized_a = sequence_a.context_into(self.dynamic_context);
        let atomized_b = sequence_b.context_into(self.dynamic_context);
        Ok((atomized_a, atomized_b))
    }

    fn pop_effective_boolean(&mut self) -> Result<bool, ValueError> {
        let a = self.stack.pop().unwrap();
        a.effective_boolean_value()
    }

    fn err(&self, value_error: ValueError) -> Error {
        Error::from_value_error(self.program, self.current_span(), value_error)
    }

    fn current_span(&self) -> SourceSpan {
        let frame = self.frame();
        let function = &self.program.functions[frame.function.0];
        // we substract 1 to end up in the current instruction - this
        // because the ip is already on the next instruction
        function.spans[frame.ip - 1]
    }

    fn read_instruction(&mut self) -> EncodedInstruction {
        let frame = &mut self.frames.last_mut().unwrap();
        let function = &self.program.functions[frame.function.0];
        let chunk = &function.chunk;
        read_instruction(chunk, &mut frame.ip)
    }

    fn read_u16(&mut self) -> u16 {
        let frame = &mut self.frames.last_mut().unwrap();
        let function = &self.program.functions[frame.function.0];
        let chunk = &function.chunk;
        read_u16(chunk, &mut frame.ip)
    }

    fn read_i16(&mut self) -> i16 {
        let frame = &mut self.frames.last_mut().unwrap();
        let function = &self.program.functions[frame.function.0];
        let chunk = &function.chunk;
        read_i16(chunk, &mut frame.ip)
    }

    fn read_u8(&mut self) -> u8 {
        let frame = &mut self.frames.last_mut().unwrap();
        let function = &self.program.functions[frame.function.0];
        let chunk = &function.chunk;
        read_u8(chunk, &mut frame.ip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xot::Xot;

    use crate::context::{Namespaces, StaticContext};
    use crate::interpreter::builder::{FunctionBuilder, JumpCondition};
    use crate::interpreter::instruction::{decode_instructions, Instruction};

    #[test]
    fn test_interpreter() -> Result<(), ValueError> {
        let mut program = Program::new("".to_string());

        let mut builder = FunctionBuilder::new(&mut program);
        let empty_span = (0, 0).into();
        builder.emit_constant(Value::Atomic(Atomic::Integer(1)), empty_span);
        builder.emit_constant(Value::Atomic(Atomic::Integer(2)), empty_span);
        builder.emit(Instruction::Add, empty_span);
        let function = builder.finish("main".to_string(), 0, empty_span);

        let main_id = program.add_function(function);
        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);

        let mut interpreter = Interpreter::new(&program, &context);
        interpreter.start(main_id, Some(&Item::Atomic(Atomic::Integer(0))), &[]);
        interpreter.run_actual()?;
        assert_eq!(interpreter.stack, vec![Value::Atomic(Atomic::Integer(3))]);
        Ok(())
    }

    #[test]
    fn test_emit_jump_forward() -> Result<(), Error> {
        let mut program = Program::new("".to_string());

        let mut builder = FunctionBuilder::new(&mut program);
        let empty_span = (0, 0).into();
        let jump = builder.emit_jump_forward(JumpCondition::Always, empty_span);
        builder.emit_constant(Value::Atomic(Atomic::Integer(3)), empty_span);
        builder.patch_jump(jump);
        builder.emit_constant(Value::Atomic(Atomic::Integer(4)), empty_span);
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
        let mut program = Program::new("".to_string());

        let mut builder = FunctionBuilder::new(&mut program);
        let empty_span = (0, 0).into();
        builder.emit_constant(Value::Atomic(Atomic::Integer(1)), empty_span);
        builder.emit_constant(Value::Atomic(Atomic::Integer(2)), empty_span);
        builder.emit(Instruction::Lt, empty_span);
        let lt_false = builder.emit_jump_forward(JumpCondition::False, empty_span);
        builder.emit_constant(Value::Atomic(Atomic::Integer(3)), empty_span);
        let end = builder.emit_jump_forward(JumpCondition::Always, empty_span);
        builder.patch_jump(lt_false);
        builder.emit_constant(Value::Atomic(Atomic::Integer(4)), empty_span);
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0, empty_span);

        let main_id = program.add_function(function);

        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);

        let mut interpreter = Interpreter::new(&program, &context);
        interpreter.start(main_id, Some(&Item::Atomic(Atomic::Integer(0))), &[]);
        interpreter.run_actual()?;
        assert_eq!(interpreter.stack, vec![Value::Atomic(Atomic::Integer(3))]);
        Ok(())
    }

    #[test]
    fn test_condition_false() -> Result<(), ValueError> {
        let mut program = Program::new("".to_string());

        let mut builder = FunctionBuilder::new(&mut program);
        let empty_span = (0, 0).into();
        builder.emit_constant(Value::Atomic(Atomic::Integer(2)), empty_span);
        builder.emit_constant(Value::Atomic(Atomic::Integer(1)), empty_span);
        builder.emit(Instruction::Lt, empty_span);
        let lt_false = builder.emit_jump_forward(JumpCondition::False, empty_span);
        builder.emit_constant(Value::Atomic(Atomic::Integer(3)), empty_span);
        let end = builder.emit_jump_forward(JumpCondition::Always, empty_span);
        builder.patch_jump(lt_false);
        builder.emit_constant(Value::Atomic(Atomic::Integer(4)), empty_span);
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0, empty_span);

        let main_id = program.add_function(function);

        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        let mut interpreter = Interpreter::new(&program, &context);
        interpreter.start(main_id, Some(&Item::Atomic(Atomic::Integer(0))), &[]);
        interpreter.run_actual()?;
        assert_eq!(interpreter.stack, vec![Value::Atomic(Atomic::Integer(4))]);
        Ok(())
    }
}
