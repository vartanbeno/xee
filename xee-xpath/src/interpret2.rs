use num::{FromPrimitive, ToPrimitive};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("integer overflow")]
    IntegerOverflow,
    #[error("type error")]
    TypeError,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct FunctionId(usize);

impl FunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Program {
    functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub(crate) struct Interpreter<'a> {
    program: &'a Program,
    stack: Vec<Value>,
    frames: Vec<Frame>,
}

#[derive(Debug, Clone)]
struct Frame {
    function: FunctionId,
    ip: usize,
    base: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    name: String,
    arity: usize,
    constants: Vec<Value>,
    chunk: Vec<u8>,
}

impl Function {
    pub(crate) fn decoded(&self) -> Vec<Instruction> {
        decode_instructions(&self.chunk)
    }
}

// TODO: could we shrink this by pointing to a value heap with a reference
// smaller than 64 bits?
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Integer(i64),
    Function(FunctionId),
}

impl Value {
    pub(crate) fn as_integer(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(Error::TypeError),
        }
    }

    pub(crate) fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Integer(i) => Ok(*i != 0),
            _ => Err(Error::TypeError),
        }
    }

    fn as_function(&self) -> Result<FunctionId> {
        match self {
            Value::Function(f) => Ok(*f),
            _ => Err(Error::TypeError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Instruction {
    // binary operators
    Add,
    Sub,
    Const(u16),
    Function(u16),
    Var(u16),
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Test,
    Jump(i16),
    Call(u8),
    Return,
    Dup,
    LetDone,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToPrimitive, FromPrimitive)]
enum EncodedInstruction {
    Add,
    Sub,
    Const,
    Function,
    Var,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Test,
    Jump,
    Call,
    Return,
    Dup,
    LetDone,
}

// decode a single instruction from the slice
fn decode_instruction(bytes: &[u8]) -> (Instruction, usize) {
    let encoded_instruction = EncodedInstruction::from_u8(bytes[0]).unwrap();
    match encoded_instruction {
        EncodedInstruction::Add => (Instruction::Add, 1),
        EncodedInstruction::Sub => (Instruction::Sub, 1),
        EncodedInstruction::Const => {
            let constant = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Const(constant), 3)
        }
        EncodedInstruction::Function => {
            let function = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Function(function), 3)
        }
        EncodedInstruction::Var => {
            let variable = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Var(variable), 3)
        }
        EncodedInstruction::Eq => (Instruction::Eq, 1),
        EncodedInstruction::Ne => (Instruction::Ne, 1),
        EncodedInstruction::Lt => (Instruction::Lt, 1),
        EncodedInstruction::Le => (Instruction::Le, 1),
        EncodedInstruction::Gt => (Instruction::Gt, 1),
        EncodedInstruction::Ge => (Instruction::Ge, 1),
        EncodedInstruction::Test => (Instruction::Test, 1),
        EncodedInstruction::Jump => {
            let displacement = i16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Jump(displacement), 3)
        }
        EncodedInstruction::Call => {
            let arity = bytes[1];
            (Instruction::Call(arity), 2)
        }
        EncodedInstruction::Return => (Instruction::Return, 1),
        EncodedInstruction::Dup => (Instruction::Dup, 1),
        EncodedInstruction::LetDone => (Instruction::LetDone, 1),
    }
}

pub(crate) fn decode_instructions(bytes: &[u8]) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut ip = 0;
    while ip < bytes.len() {
        let (instruction, instruction_size) = decode_instruction(&bytes[ip..]);
        instructions.push(instruction);
        ip += instruction_size;
    }
    instructions
}

fn encode_instruction(instruction: Instruction, bytes: &mut Vec<u8>) {
    match instruction {
        Instruction::Add => bytes.push(EncodedInstruction::Add.to_u8().unwrap()),
        Instruction::Sub => bytes.push(EncodedInstruction::Sub.to_u8().unwrap()),
        Instruction::Const(constant) => {
            bytes.push(EncodedInstruction::Const.to_u8().unwrap());
            bytes.extend_from_slice(&constant.to_le_bytes());
        }
        Instruction::Function(function_id) => {
            bytes.push(EncodedInstruction::Function.to_u8().unwrap());
            bytes.extend_from_slice(&function_id.to_le_bytes());
        }
        Instruction::Var(variable) => {
            bytes.push(EncodedInstruction::Var.to_u8().unwrap());
            bytes.extend_from_slice(&variable.to_le_bytes());
        }
        Instruction::Eq => bytes.push(EncodedInstruction::Eq.to_u8().unwrap()),
        Instruction::Ne => bytes.push(EncodedInstruction::Ne.to_u8().unwrap()),
        Instruction::Lt => bytes.push(EncodedInstruction::Lt.to_u8().unwrap()),
        Instruction::Le => bytes.push(EncodedInstruction::Le.to_u8().unwrap()),
        Instruction::Gt => bytes.push(EncodedInstruction::Gt.to_u8().unwrap()),
        Instruction::Ge => bytes.push(EncodedInstruction::Ge.to_u8().unwrap()),
        Instruction::Test => bytes.push(EncodedInstruction::Test.to_u8().unwrap()),
        Instruction::Jump(displacement) => {
            bytes.push(EncodedInstruction::Jump.to_u8().unwrap());
            bytes.extend_from_slice(&displacement.to_le_bytes());
        }
        Instruction::Call(arity) => {
            bytes.push(EncodedInstruction::Call.to_u8().unwrap());
            bytes.push(arity);
        }
        Instruction::Return => bytes.push(EncodedInstruction::Return.to_u8().unwrap()),
        Instruction::Dup => bytes.push(EncodedInstruction::Dup.to_u8().unwrap()),
        Instruction::LetDone => bytes.push(EncodedInstruction::LetDone.to_u8().unwrap()),
    }
}

fn encode_instructions(instructions: Vec<Instruction>, bytes: &mut Vec<u8>) {
    for instruction in instructions {
        encode_instruction(instruction, bytes);
    }
}

#[must_use]
pub(crate) struct ForwardJumpRef(usize);

#[must_use]
struct BackwardJumpRef(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub(crate) struct FunctionBuilder<'a> {
    program: &'a mut Program,
    compiled: Vec<u8>,
    constants: Vec<Value>,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn new(program: &'a mut Program) -> Self {
        FunctionBuilder {
            program,
            compiled: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub(crate) fn emit(&mut self, instruction: Instruction) {
        encode_instruction(instruction, &mut self.compiled);
    }

    pub(crate) fn emit_constant(&mut self, constant: Value) {
        let constant_id = self.constants.len();
        self.constants.push(constant);
        if constant_id > (u16::MAX as usize) {
            panic!("too many constants");
        }
        self.emit(Instruction::Const(constant_id as u16));
    }

    fn emit_compare(&mut self, comparison: Comparison) {
        match comparison {
            Comparison::Eq => self.emit(Instruction::Eq),
            Comparison::Ne => self.emit(Instruction::Ne),
            Comparison::Lt => self.emit(Instruction::Lt),
            Comparison::Le => self.emit(Instruction::Le),
            Comparison::Gt => self.emit(Instruction::Gt),
            Comparison::Ge => self.emit(Instruction::Ge),
        }
    }

    pub(crate) fn emit_compare_value(&mut self, comparison: Comparison) {
        let otherwise = self.emit_compare_forward(comparison);
        self.emit_constant(Value::Integer(1));
        let end = self.emit_jump_forward();
        self.patch_jump(otherwise);
        self.emit_constant(Value::Integer(0));
        self.patch_jump(end);
    }

    pub(crate) fn emit_compare_forward(&mut self, comparison: Comparison) -> ForwardJumpRef {
        self.emit_compare(comparison);
        self.emit_jump_forward()
    }

    fn emit_compare_backward(&mut self, comparison: Comparison, jump_ref: BackwardJumpRef) {
        self.emit_compare(comparison);
        self.emit_jump_backward(jump_ref);
    }

    pub(crate) fn emit_test_forward(&mut self) -> ForwardJumpRef {
        self.emit(Instruction::Test);
        self.emit_jump_forward()
    }

    fn loop_start(&self) -> BackwardJumpRef {
        BackwardJumpRef(self.compiled.len())
    }

    fn emit_jump_backward(&mut self, jump_ref: BackwardJumpRef) {
        let current = self.compiled.len() + 3;
        let offset = current - jump_ref.0;
        if jump_ref.0 > current {
            panic!("cannot jump forward");
        }
        if offset > (u16::MAX as usize) {
            panic!("jump too far");
        }
        self.emit(Instruction::Jump(-(offset as i16)));
    }

    pub(crate) fn emit_jump_forward(&mut self) -> ForwardJumpRef {
        let index = self.compiled.len();
        self.emit(Instruction::Jump(0));
        ForwardJumpRef(index)
    }

    pub(crate) fn patch_jump(&mut self, jump_ref: ForwardJumpRef) {
        let current = self.compiled.len();
        if jump_ref.0 > current {
            panic!("can only patch forward jumps");
        }
        let offset = current - jump_ref.0 - 3; // 3 for size of the jump
        if offset > (u16::MAX as usize) {
            panic!("jump too far");
        }
        let offset_bytes = offset.to_le_bytes();
        self.compiled[jump_ref.0 + 1] = offset_bytes[0];
        self.compiled[jump_ref.0 + 2] = offset_bytes[1];
    }

    pub(crate) fn finish(mut self, name: String, arity: usize) -> Function {
        self.emit(Instruction::Return);
        Function {
            name,
            arity,
            chunk: self.compiled,
            constants: self.constants,
        }
    }

    pub(crate) fn builder(&mut self) -> FunctionBuilder {
        FunctionBuilder::new(self.program)
    }

    pub(crate) fn add_function(&mut self, function: Function) -> FunctionId {
        self.program.add_function(function)
    }
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(program: &'a Program) -> Self {
        Interpreter {
            program,
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub(crate) fn stack(&self) -> &[Value] {
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
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    let result = a.checked_add(b).ok_or(Error::IntegerOverflow)?;
                    self.stack.push(Value::Integer(result));
                }
                Instruction::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    let result = a.checked_sub(b).ok_or(Error::IntegerOverflow)?;
                    self.stack.push(Value::Integer(result));
                }
                Instruction::Const(index) => {
                    self.stack.push(function.constants[index as usize].clone());
                }
                Instruction::Function(function_id) => {
                    self.stack
                        .push(Value::Function(FunctionId(function_id as usize)));
                }
                Instruction::Var(index) => {
                    self.stack.push(self.stack[base + index as usize].clone());
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
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
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
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
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
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
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
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    let result = a >= b;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                Instruction::Test => {
                    let a = self.stack.pop().unwrap();
                    let a = a.as_integer()?;
                    let result = a != 0;
                    // skip the next instruction, which by construction
                    // has to be a jump instruction, so we know its size
                    if result {
                        ip += 3;
                    }
                }
                // XXX do we need a TestFalse? in that case we make the previous
                // instruction TestTrue
                Instruction::Dup => {
                    let a = self.stack.last().unwrap().clone();
                    self.stack.push(a);
                }
                Instruction::Call(arity) => {
                    // store ip of next instruction in current frame
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip = ip;

                    // get function id from stack, by peeking back
                    let function_id =
                        self.stack[self.stack.len() - (arity as usize + 1)].as_function()?;
                    function = &self.program.functions[function_id.0];
                    let stack_size = self.stack.len();
                    base = stack_size - (arity as usize);
                    ip = 0;
                    self.frames.push(Frame {
                        function: function_id,
                        ip,
                        base,
                    });
                }
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap();

                    // truncate the stack to the base
                    self.stack.truncate(base);

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
            }
        }
        Ok(())
    }
}

impl Program {
    pub(crate) fn new() -> Self {
        Program {
            functions: Vec::new(),
        }
    }

    pub(crate) fn add_function(&mut self, function: Function) -> FunctionId {
        let id = self.functions.len();
        if id > u16::MAX as usize {
            panic!("too many functions");
        }
        self.functions.push(function);

        FunctionId(id)
    }

    pub(crate) fn get_function(&self, index: usize) -> &Function {
        &self.functions[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(Value::Integer(1));
        builder.emit_constant(Value::Integer(2));
        builder.emit(Instruction::Add);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let mut interpreter = Interpreter::new(&program);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(interpreter.stack, vec![Value::Integer(3)]);
        Ok(())
    }

    #[test]
    fn test_emit_jump_forward() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        let jump = builder.emit_jump_forward();
        builder.emit_constant(Value::Integer(3));
        builder.patch_jump(jump);
        builder.emit_constant(Value::Integer(4));
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
        builder.emit_constant(Value::Integer(1));
        builder.emit_constant(Value::Integer(2));
        let lt_false = builder.emit_compare_forward(Comparison::Lt);
        builder.emit_constant(Value::Integer(3));
        let end = builder.emit_jump_forward();
        builder.patch_jump(lt_false);
        builder.emit_constant(Value::Integer(4));
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let mut interpreter = Interpreter::new(&program);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(interpreter.stack, vec![Value::Integer(3)]);
        Ok(())
    }

    #[test]
    fn test_condition_false() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(Value::Integer(2));
        builder.emit_constant(Value::Integer(1));
        let lt_false = builder.emit_compare_forward(Comparison::Lt);
        builder.emit_constant(Value::Integer(3));
        let end = builder.emit_jump_forward();
        builder.patch_jump(lt_false);
        builder.emit_constant(Value::Integer(4));
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let mut interpreter = Interpreter::new(&program);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(interpreter.stack, vec![Value::Integer(4)]);
        Ok(())
    }

    #[test]
    fn test_loop() -> Result<()> {
        let mut program = Program::new();

        let mut builder = FunctionBuilder::new(&mut program);
        builder.emit_constant(Value::Integer(10));
        let loop_start = builder.loop_start();
        builder.emit(Instruction::Dup);
        builder.emit_constant(Value::Integer(5));
        let end = builder.emit_compare_forward(Comparison::Gt);
        builder.emit_constant(Value::Integer(1));
        builder.emit(Instruction::Sub);
        builder.emit_jump_backward(loop_start);
        builder.patch_jump(end);
        let function = builder.finish("main".to_string(), 0);

        let main_id = program.add_function(function);
        let mut interpreter = Interpreter::new(&program);
        interpreter.start(main_id);
        interpreter.run()?;
        assert_eq!(interpreter.stack, vec![Value::Integer(5)]);
        Ok(())
    }

    // #[test]
    // fn test_call() -> Result<()> {
    //     let mut program = Program::new();

    //     let mut builder = FunctionBuilder::new(&mut program);
    //     builder.emit_constant(Value::Integer(5));
    //     builder.emit_constant(Value::Integer(6));
    //     builder.emit(Instruction::Add);
    //     let inner = builder.finish("inner".to_string(), 0);
    //     let inner_id = program.add_function(inner);
    //     let mut builder = FunctionBuilder::new(&mut program);
    //     builder.emit_constant(Value::Integer(1));
    //     builder.emit_constant(Value::Function(inner_id));
    //     builder.emit(Instruction::Call(0));
    //     builder.emit(Instruction::Add);
    //     let outer = builder.finish("outer".to_string(), 0);
    //     let main_id = program.add_function(outer);
    //     let mut interpreter = Interpreter::new(&program);
    //     interpreter.start(main_id);
    //     interpreter.run()?;
    //     assert_eq!(interpreter.stack, vec![Value::Integer(12)]);
    //     Ok(())
    // }

    // #[test]
    // fn test_call_with_arity() -> Result<()> {
    //     let mut program = Program::new();

    //     let mut builder = FunctionBuilder::new(&mut program);
    //     builder.emit_constant(Value::Integer(5));
    //     builder.emit(Instruction::Add);
    //     let inner = builder.finish("inner".to_string(), 1);
    //     let inner_id = program.add_function(inner);

    //     let mut builder = FunctionBuilder::new(&mut program);
    //     builder.emit_constant(Value::Integer(1));
    //     builder.emit_constant(Value::Function(inner_id));
    //     builder.emit(Instruction::Call);
    //     let outer = builder.finish("outer".to_string(), 0);
    //     let main_id = program.add_function(outer);

    //     let mut interpreter = Interpreter::new(&program);
    //     interpreter.start(main_id);
    //     interpreter.run()?;
    //     assert_eq!(interpreter.stack, vec![Value::Integer(6)]);
    //     Ok(())
    // }
}
