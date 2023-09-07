use std::cmp::Ordering;

use arrayvec::ArrayVec;
use ibig::IBig;
use miette::SourceSpan;
use xee_schema_type::Xs;

use crate::atomic::{self, AtomicCompare};
use crate::atomic::{
    op_add, op_div, op_idiv, op_mod, op_multiply, op_subtract, OpEq, OpGe, OpGt, OpLe, OpLt, OpNe,
};
use crate::context::DynamicContext;
use crate::error;
use crate::error::Error;
use crate::occurrence::Occurrence;
use crate::sequence;
use crate::stack;
use crate::xml;

use super::builder::Program;
use super::instruction::{read_i16, read_instruction, read_u16, read_u8, EncodedInstruction};

const FRAMES_MAX: usize = 64;
const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone)]
struct Frame {
    function: stack::FunctionId,
    ip: usize,
    base: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Interpreter<'a> {
    program: &'a Program,
    dynamic_context: &'a DynamicContext<'a>,
    stack: Vec<stack::Value>,
    build_stack: Vec<Vec<sequence::Item>>,
    frames: ArrayVec<Frame, FRAMES_MAX>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(program: &'a Program, dynamic_context: &'a DynamicContext) -> Self {
        Interpreter {
            program,
            dynamic_context,
            stack: vec![],
            build_stack: vec![],
            frames: ArrayVec::new(),
        }
    }

    pub(crate) fn stack(&self) -> &[stack::Value] {
        &self.stack
    }

    pub(crate) fn start(
        &mut self,
        function_id: stack::FunctionId,
        context_item: Option<&sequence::Item>,
        arguments: Vec<Vec<sequence::Item>>,
    ) {
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: 0,
        });
        if let Some(context_item) = context_item {
            // the context item
            self.stack.push(stack::Value::from(context_item.clone()));
            // position & size
            self.stack.push(1i64.into());
            self.stack.push(1i64.into());
        } else {
            // absent context, position and size
            self.stack.push(stack::Value::Absent);
            self.stack.push(stack::Value::Absent);
            self.stack.push(stack::Value::Absent);
        }
        // and any arguments
        for arg in arguments {
            self.stack.push(stack::Value::from(arg));
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

    pub(crate) fn function(&self) -> &stack::Function {
        &self.program.functions[self.frame().function.0]
    }

    pub(crate) fn run_actual(&mut self) -> error::Result<()> {
        // we can make this an infinite loop as all functions end
        // with the return instruction
        loop {
            let instruction = self.read_instruction();
            match instruction {
                EncodedInstruction::Add => {
                    self.arithmetic_with_offset(op_add)?;
                }
                EncodedInstruction::Sub => {
                    self.arithmetic_with_offset(op_subtract)?;
                }
                EncodedInstruction::Mul => {
                    self.arithmetic(op_multiply)?;
                }
                EncodedInstruction::Div => {
                    self.arithmetic(op_div)?;
                }
                EncodedInstruction::IntDiv => {
                    self.arithmetic(op_idiv)?;
                }
                EncodedInstruction::Mod => {
                    self.arithmetic(op_mod)?;
                }
                EncodedInstruction::Plus => {
                    self.unary_arithmetic(|a| a.plus())?;
                }
                EncodedInstruction::Minus => {
                    self.unary_arithmetic(|a| a.minus())?;
                }
                EncodedInstruction::Concat => {
                    let (a, b) = self.pop_atomic2()?;
                    let a = a.to_str()?;
                    let b = b.to_str()?;
                    let result = a.to_owned() + b;
                    self.stack.push(result.into());
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
                    self.stack.push(
                        stack::Closure {
                            function_id: stack::ClosureFunctionId::Dynamic(stack::FunctionId(
                                function_id as usize,
                            )),
                            values,
                        }
                        .into(),
                    );
                }
                EncodedInstruction::StaticClosure => {
                    let static_function_id = self.read_u16();
                    let static_function = &self
                        .dynamic_context
                        .static_context
                        .functions
                        .get_by_index(stack::StaticFunctionId(static_function_id as usize));
                    // get any context value from the stack if needed
                    let values = if static_function.needs_context() {
                        vec![self.stack.pop().unwrap()]
                    } else {
                        vec![]
                    };

                    self.stack.push(
                        stack::Closure {
                            function_id: stack::ClosureFunctionId::Static(stack::StaticFunctionId(
                                static_function_id as usize,
                            )),
                            values,
                        }
                        .into(),
                    );
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
                    let closure: &stack::Closure =
                        (&self.stack[self.frame().base - 1]).try_into()?;
                    // and we push the value we need onto the stack
                    self.stack.push(closure.values[index as usize].clone());
                }
                EncodedInstruction::Comma => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a.concat(b));
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
                    self.value_compare(OpEq)?;
                }
                EncodedInstruction::Ne => self.value_compare(OpNe)?,
                EncodedInstruction::Lt => {
                    self.value_compare(OpLt)?;
                }
                EncodedInstruction::Le => {
                    self.value_compare(OpLe)?;
                }
                EncodedInstruction::Gt => {
                    self.value_compare(OpGt)?;
                }
                EncodedInstruction::Ge => {
                    self.value_compare(OpGe)?;
                }
                EncodedInstruction::GenEq => {
                    self.general_compare(OpEq)?;
                }
                EncodedInstruction::GenNe => {
                    self.general_compare(OpNe)?;
                }
                EncodedInstruction::GenLt => {
                    self.general_compare(OpLt)?;
                }
                EncodedInstruction::GenLe => {
                    self.general_compare(OpLe)?;
                }
                EncodedInstruction::GenGt => {
                    self.general_compare(OpGt)?;
                }
                EncodedInstruction::GenGe => {
                    self.general_compare(OpGe)?;
                }
                EncodedInstruction::Union => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let combined = a.union(b, &self.dynamic_context.documents.annotations)?;
                    self.stack.push(combined);
                }
                EncodedInstruction::Intersect => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let combined = a.intersect(b, &self.dynamic_context.documents.annotations)?;
                    self.stack.push(combined);
                }
                EncodedInstruction::Except => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let combined = a.except(b, &self.dynamic_context.documents.annotations)?;
                    self.stack.push(combined);
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

                    if let Ok(closure) = callable.try_into() as error::Result<&stack::Closure> {
                        match closure.function_id {
                            stack::ClosureFunctionId::Dynamic(function_id) => {
                                self.call_closure(function_id, arity)?;
                            }
                            stack::ClosureFunctionId::Static(static_function_id) => {
                                // XXX wish I didn't need to clone
                                let closure_values = &closure.values.clone();
                                self.call_static(static_function_id, arity, closure_values)?;
                            }
                        }
                    } else {
                        return Err(error::Error::Type);
                    }
                }
                EncodedInstruction::Step => {
                    let step_id = self.read_u16();
                    let node = self.stack.pop().unwrap().try_into()?;
                    let step = &(self.function().steps[step_id as usize]);
                    let value = xml::resolve_step(step, node, self.dynamic_context.xot);
                    self.stack.push(value);
                }
                EncodedInstruction::Deduplicate => {
                    let value = self.stack.pop().unwrap();
                    let value = value.deduplicate(&self.dynamic_context.documents.annotations)?;
                    self.stack.push(value);
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
                EncodedInstruction::ReturnConvert => {
                    let sequence_type_id = self.read_u16();
                    let value = self.stack.pop().unwrap();
                    let sequence: sequence::Sequence = value.into();
                    let sequence_type =
                        &(self.function().sequence_types[sequence_type_id as usize]);

                    let sequence = sequence.sequence_type_matching_function_conversion(
                        sequence_type,
                        self.dynamic_context,
                    )?;
                    self.stack.push(sequence.into());
                }
                EncodedInstruction::LetDone => {
                    let return_value = self.stack.pop().unwrap();
                    // pop the variable assignment
                    let _ = self.stack.pop();
                    self.stack.push(return_value);
                }
                EncodedInstruction::Cast => {
                    let type_id = self.read_u16();
                    let value = self.pop_atomic_option()?;
                    let cast_type = &(self.function().cast_types[type_id as usize]);
                    if let Some(value) = value {
                        let cast_value =
                            value.cast_to_schema_type(cast_type.xs, self.dynamic_context)?;
                        self.stack.push(cast_value.into());
                    } else if cast_type.empty_sequence_allowed {
                        self.stack.push(stack::Value::Empty);
                    } else {
                        Err(error::Error::Type)?;
                    }
                }
                EncodedInstruction::Castable => {
                    let type_id = self.read_u16();
                    let value = self.pop_atomic_option()?;
                    let cast_type = &(self.function().cast_types[type_id as usize]);
                    if let Some(value) = value {
                        let cast_value =
                            value.cast_to_schema_type(cast_type.xs, self.dynamic_context);
                        self.stack.push(cast_value.is_ok().into());
                    } else if cast_type.empty_sequence_allowed {
                        self.stack.push(true.into())
                    } else {
                        self.stack.push(false.into());
                    }
                }
                EncodedInstruction::InstanceOf => {
                    let sequence_type_id = self.read_u16();
                    let value = self.stack.pop().unwrap();
                    let sequence_type =
                        &(self.function().sequence_types[sequence_type_id as usize]);
                    let sequence: sequence::Sequence = value.into();
                    let matches =
                        sequence.sequence_type_matching(sequence_type, self.dynamic_context.xot);
                    if matches.is_ok() {
                        self.stack.push(true.into());
                    } else {
                        self.stack.push(false.into());
                    }
                }
                EncodedInstruction::Range => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let mut a = a.atomized(self.dynamic_context.xot);
                    let mut b = b.atomized(self.dynamic_context.xot);
                    let a = a.option()?;
                    let b = b.option()?;
                    let (a, b) = match (a, b) {
                        (None, None) | (None, _) | (_, None) => {
                            self.stack.push(stack::Value::Empty);
                            continue;
                        }
                        (Some(a), Some(b)) => (a, b),
                    };
                    // we want to ensure we have integers at this point;
                    // we don't want to be casting strings or anything
                    a.ensure_base_schema_type(Xs::Integer)?;
                    b.ensure_base_schema_type(Xs::Integer)?;

                    let a = a.cast_to_integer_value::<i64>()?;
                    let b = b.cast_to_integer_value::<i64>()?;

                    match a.cmp(&b) {
                        Ordering::Greater => self.stack.push(stack::Value::Empty),
                        Ordering::Equal => self.stack.push(a.into()),
                        Ordering::Less => {
                            if (b - a) > MAXIMUM_RANGE_SIZE {
                                return Err(error::Error::XPDY0130);
                            }
                            let items = (a..=b).map(|i| i.into()).collect::<Vec<sequence::Item>>();
                            self.stack.push(items.into())
                        }
                    }
                }

                EncodedInstruction::SequenceLen => {
                    let value = self.stack.pop().unwrap();
                    let l: IBig = value.len()?.into();
                    self.stack.push(l.into());
                }
                EncodedInstruction::SequenceGet => {
                    let value = self.stack.pop().unwrap();
                    let index = self.pop_atomic()?;
                    let index = index.cast_to_integer_value::<i64>()? as usize;
                    // substract 1 as Xpath is 1-indexed
                    let item = value.index(index - 1)?;
                    self.stack.push(item.into())
                }
                EncodedInstruction::BuildNew => {
                    self.build_stack.push(Vec::new());
                }
                EncodedInstruction::BuildPush => {
                    let build = &mut self.build_stack.last_mut().unwrap();
                    let value = self.stack.pop().unwrap();
                    build_push(build, value)?;
                }
                EncodedInstruction::BuildComplete => {
                    let build = self.build_stack.pop().unwrap();
                    self.stack.push(build.into());
                }
                EncodedInstruction::IsNumeric => {
                    let is_numeric = self.pop_is_numeric()?;
                    self.stack.push(is_numeric.into());
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
        static_function_id: stack::StaticFunctionId,
        arity: u8,
        closure_values: &[stack::Value],
    ) -> error::Result<()> {
        let static_function = &self
            .dynamic_context
            .static_context
            .functions
            .get_by_index(static_function_id);
        let arguments = &self.stack[self.stack.len() - (arity as usize)..];
        let result = static_function.invoke(self.dynamic_context, arguments, closure_values)?;
        // truncate the stack to the base
        self.stack.truncate(self.stack.len() - (arity as usize + 1));
        self.stack.push(result.into());
        Ok(())
    }

    fn call_closure(&mut self, function_id: stack::FunctionId, arity: u8) -> error::Result<()> {
        // look up the function in order to access the parameters information
        let function = self.program.get_function_by_id(function_id);
        let params = &function.params;

        // TODO: fast path if no sequence types exist for parameters

        // now pop everything off the stack to do type matching, along
        // with sequence type conversion
        let mut arguments = Vec::with_capacity(arity as usize);
        for param in params.iter().rev() {
            let value = self.stack.pop().unwrap();
            if let Some(type_) = &param.type_ {
                let sequence: sequence::Sequence = value.into();
                // matching also takes care of function conversion rules
                let sequence = sequence
                    .sequence_type_matching_function_conversion(type_, self.dynamic_context)?;
                arguments.push(sequence.into())
            } else {
                // no need to do any checking or conversion
                arguments.push(value);
            }
        }
        // now we have a list of arguments that we want to push back onto the stack,
        // in reverse
        for arg in arguments.into_iter().rev() {
            self.stack.push(arg);
        }

        if self.frames.len() >= self.frames.capacity() {
            return Err(error::Error::StackOverflow);
        }
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: self.stack.len() - (arity as usize),
        });
        Ok(())
    }

    fn value_compare<O>(&mut self, _op: O) -> error::Result<()>
    where
        O: AtomicCompare,
    {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        // https://www.w3.org/TR/xpath-31/#id-value-comparisons
        // If an operand is the empty sequence, the result is the empty sequence
        if a.is_empty_sequence() || b.is_empty_sequence() {
            self.stack.push(stack::Value::Empty);
            return Ok(());
        }
        let mut atomized_a = a.atomized(self.dynamic_context.xot);
        let mut atomized_b = b.atomized(self.dynamic_context.xot);
        let a = atomized_a.one()?;
        let b = atomized_b.one()?;
        let collation = self.dynamic_context.static_context.default_collation()?;
        let result = O::atomic_compare(
            a,
            b,
            |a: &str, b: &str| collation.compare(a, b),
            self.dynamic_context.implicit_timezone(),
        )?;
        self.stack.push(result.into());
        Ok(())
    }

    fn general_compare<O>(&mut self, op: O) -> error::Result<()>
    where
        O: AtomicCompare,
    {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let value = a.general_comparison(b, self.dynamic_context, op)?.into();
        self.stack.push(value);
        Ok(())
    }

    fn arithmetic<F>(&mut self, op: F) -> error::Result<()>
    where
        F: Fn(atomic::Atomic, atomic::Atomic) -> error::Result<atomic::Atomic>,
    {
        self.arithmetic_with_offset(|a, b, _| op(a, b))
    }

    fn arithmetic_with_offset<F>(&mut self, op: F) -> error::Result<()>
    where
        F: Fn(atomic::Atomic, atomic::Atomic, chrono::FixedOffset) -> error::Result<atomic::Atomic>,
    {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        // https://www.w3.org/TR/xpath-31/#id-arithmetic
        // 2. If an operand is the empty sequence, the result is the empty sequence
        if a.is_empty_sequence() || b.is_empty_sequence() {
            self.stack.push(stack::Value::Empty);
            return Ok(());
        }
        let mut atomized_a = a.atomized(self.dynamic_context.xot);
        let mut atomized_b = b.atomized(self.dynamic_context.xot);
        let a = atomized_a.one()?;
        let b = atomized_b.one()?;
        let result = op(a, b, self.dynamic_context.implicit_timezone())?;
        self.stack.push(result.into());
        Ok(())
    }

    fn unary_arithmetic<F>(&mut self, op: F) -> error::Result<()>
    where
        F: Fn(atomic::Atomic) -> error::Result<atomic::Atomic>,
    {
        let a = self.stack.pop().unwrap();
        if a.is_empty_sequence() {
            self.stack.push(stack::Value::Empty);
            return Ok(());
        }
        let mut atomized_a = a.atomized(self.dynamic_context.xot);
        let a = atomized_a.one()?;
        let value = op(a)?;
        self.stack.push(value.into());
        Ok(())
    }

    fn pop_is_numeric(&mut self) -> error::Result<bool> {
        let value = self.stack.pop().unwrap();
        let mut atomized = value.atomized(self.dynamic_context.xot);
        let a = atomized.option()?;
        if let Some(a) = a {
            Ok(a.is_numeric())
        } else {
            Ok(false)
        }
    }

    fn pop_atomic(&mut self) -> error::Result<atomic::Atomic> {
        let value = self.stack.pop().unwrap();
        let mut atomized = value.atomized(self.dynamic_context.xot);
        atomized.one()
    }

    fn pop_atomic_option(&mut self) -> error::Result<Option<atomic::Atomic>> {
        let value = self.stack.pop().unwrap();
        let mut atomized = value.atomized(self.dynamic_context.xot);
        atomized.option()
    }

    fn pop_atomic2(&mut self) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
        let b = self.pop_atomic()?;
        let a = self.pop_atomic()?;
        Ok((a, b))
    }

    fn pop_effective_boolean(&mut self) -> error::Result<bool> {
        let a = self.stack.pop().unwrap();
        a.effective_boolean_value()
    }

    fn err(&self, value_error: error::Error) -> Error {
        value_error.with_span(self.program, self.current_span())
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

fn build_push(build: &mut Vec<sequence::Item>, value: stack::Value) -> error::Result<()> {
    match value {
        stack::Value::Empty => {}
        stack::Value::One(item) => build.push(item),
        stack::Value::Many(items) => build.extend(items.iter().cloned()),
        stack::Value::Absent => return Err(error::Error::ComponentAbsentInDynamicContext)?,
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use ibig::ibig;
//     use xee_xpath_ast::Namespaces;
//     use xot::Xot;

//     use crate::context::StaticContext;
//     use crate::interpreter::builder::{FunctionBuilder, JumpCondition};
//     use crate::interpreter::instruction::{decode_instructions, Instruction};

//     #[test]
//     fn test_interpreter() -> error::Result<()> {
//         let mut program = Program::new("".to_string());

//         let mut builder = FunctionBuilder::new(&mut program);
//         let empty_span = (0, 0).into();
//         builder.emit_constant(1i64.into(), empty_span);
//         builder.emit_constant(2i64.into(), empty_span);
//         builder.emit(Instruction::Add, empty_span);
//         let def = ir::FunctionDefinition { params: vec![], return_type: None, body: }
//         let function = builder.finish("main".to_string(), , empty_span);

//         let main_id = program.add_function(function);
//         let xot = Xot::new();
//         let namespaces = Namespaces::new(None, None);
//         let static_context = StaticContext::new(&namespaces);
//         let context = DynamicContext::new(&xot, &static_context);

//         let mut interpreter = Interpreter::new(&program, &context);
//         interpreter.start(
//             main_id,
//             Some(&sequence::Item::Atomic(atomic::Atomic::Integer(
//                 ibig!(0).into(),
//             ))),
//             vec![],
//         );
//         interpreter.run_actual()?;
//         assert_eq!(interpreter.stack, vec![3i64.into()]);
//         Ok(())
//     }

//     #[test]
//     fn test_emit_jump_forward() -> Result<(), Error> {
//         let mut program = Program::new("".to_string());

//         let mut builder = FunctionBuilder::new(&mut program);
//         let empty_span = (0, 0).into();
//         let jump = builder.emit_jump_forward(JumpCondition::Always, empty_span);
//         builder.emit_constant(3i64.into(), empty_span);
//         builder.patch_jump(jump);
//         builder.emit_constant(4i64.into(), empty_span);
//         let function = builder.finish("main".to_string(), vec![], empty_span);

//         let instructions = decode_instructions(&function.chunk);
//         program.add_function(function);
//         assert_eq!(
//             instructions,
//             vec![
//                 Instruction::Jump(3),
//                 Instruction::Const(0),
//                 Instruction::Const(1),
//                 Instruction::Return
//             ]
//         );
//         Ok(())
//     }

//     #[test]
//     fn test_condition_true() -> error::Result<()> {
//         let mut program = Program::new("".to_string());

//         let mut builder = FunctionBuilder::new(&mut program);
//         let empty_span = (0, 0).into();
//         builder.emit_constant(1i64.into(), empty_span);
//         builder.emit_constant(2i64.into(), empty_span);
//         builder.emit(Instruction::Lt, empty_span);
//         let lt_false = builder.emit_jump_forward(JumpCondition::False, empty_span);
//         builder.emit_constant(3i64.into(), empty_span);
//         let end = builder.emit_jump_forward(JumpCondition::Always, empty_span);
//         builder.patch_jump(lt_false);
//         builder.emit_constant(4i64.into(), empty_span);
//         builder.patch_jump(end);
//         let function = builder.finish("main".to_string(), vec![], empty_span);

//         let main_id = program.add_function(function);

//         let xot = Xot::new();
//         let namespaces = Namespaces::new(None, None);
//         let static_context = StaticContext::new(&namespaces);
//         let context = DynamicContext::new(&xot, &static_context);

//         let mut interpreter = Interpreter::new(&program, &context);
//         interpreter.start(
//             main_id,
//             Some(&sequence::Item::Atomic(atomic::Atomic::Integer(
//                 ibig!(0).into(),
//             ))),
//             vec![],
//         );
//         interpreter.run_actual()?;
//         assert_eq!(interpreter.stack, vec![3i64.into()]);
//         Ok(())
//     }

//     #[test]
//     fn test_condition_false() -> error::Result<()> {
//         let mut program = Program::new("".to_string());

//         let mut builder = FunctionBuilder::new(&mut program);
//         let empty_span = (0, 0).into();
//         builder.emit_constant(2i64.into(), empty_span);
//         builder.emit_constant(1i64.into(), empty_span);
//         builder.emit(Instruction::Lt, empty_span);
//         let lt_false = builder.emit_jump_forward(JumpCondition::False, empty_span);
//         builder.emit_constant(3i64.into(), empty_span);
//         let end = builder.emit_jump_forward(JumpCondition::Always, empty_span);
//         builder.patch_jump(lt_false);
//         builder.emit_constant(4i64.into(), empty_span);
//         builder.patch_jump(end);
//         let function = builder.finish("main".to_string(), vec![], empty_span);

//         let main_id = program.add_function(function);

//         let xot = Xot::new();
//         let namespaces = Namespaces::new(None, None);
//         let static_context = StaticContext::new(&namespaces);
//         let context = DynamicContext::new(&xot, &static_context);
//         let mut interpreter = Interpreter::new(&program, &context);
//         interpreter.start(
//             main_id,
//             Some(&sequence::Item::Atomic(atomic::Atomic::Integer(
//                 ibig!(0).into(),
//             ))),
//             vec![],
//         );
//         interpreter.run_actual()?;
//         assert_eq!(interpreter.stack, vec![4i64.into()]);
//         Ok(())
//     }
// }
