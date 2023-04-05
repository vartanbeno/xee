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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct FunctionId(usize);

#[derive(Debug, Clone)]
struct Interpreter {
    functions: Vec<Function>,
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
struct Function {
    name: String,
    arity: usize,
    constants: Vec<Value>,
    chunk: Vec<u8>,
}

// TODO: could we shrink this by pointing to a value heap with a reference
// smaller than 64 bits?
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Integer(i64),
    Function(FunctionId),
}

impl Value {
    fn as_integer(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(Error::TypeError),
        }
    }
}

enum Instruction {
    // binary operators
    Add,
    Sub,
    Const(u16),
    Eq,
    Lt,
    Le,
    Test,
    Jump(i16),
    Call,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToPrimitive, FromPrimitive)]
enum EncodedInstruction {
    // binary operators
    Add,
    Sub,
    // next 2 bytes is constant reference
    Const,
    // comparison & control flow
    Eq,
    Lt,
    Le,
    Test,
    Jump, // displacement encoded as 16 bit signed integer
    // functions
    Call,
    Return,
}

// decode a single instruction from the slice
fn decode_instruction(bytes: &[u8]) -> (Instruction, usize) {
    let encoded_instruction = EncodedInstruction::from_u8(bytes[0]).unwrap();
    match encoded_instruction {
        EncodedInstruction::Add => (Instruction::Add, 1),
        EncodedInstruction::Sub => (Instruction::Sub, 1),
        EncodedInstruction::Const => {
            let constant = u16::from_be_bytes([bytes[1], bytes[2]]);
            (Instruction::Const(constant), 3)
        }
        EncodedInstruction::Eq => (Instruction::Eq, 1),
        EncodedInstruction::Lt => (Instruction::Lt, 1),
        EncodedInstruction::Le => (Instruction::Le, 1),
        EncodedInstruction::Test => (Instruction::Test, 1),
        EncodedInstruction::Jump => {
            let displacement = i16::from_be_bytes([bytes[1], bytes[2]]);
            (Instruction::Jump(displacement), 3)
        }
        EncodedInstruction::Call => (Instruction::Call, 1),
        EncodedInstruction::Return => (Instruction::Return, 1),
    }
}

fn encode_instruction(instruction: Instruction, bytes: &mut Vec<u8>) {
    match instruction {
        Instruction::Add => bytes.push(EncodedInstruction::Add.to_u8().unwrap()),
        Instruction::Sub => bytes.push(EncodedInstruction::Sub.to_u8().unwrap()),
        Instruction::Const(constant) => {
            bytes.push(EncodedInstruction::Const.to_u8().unwrap());
            bytes.extend_from_slice(&constant.to_be_bytes());
        }
        Instruction::Eq => bytes.push(EncodedInstruction::Eq.to_u8().unwrap()),
        Instruction::Lt => bytes.push(EncodedInstruction::Lt.to_u8().unwrap()),
        Instruction::Le => bytes.push(EncodedInstruction::Le.to_u8().unwrap()),
        Instruction::Test => bytes.push(EncodedInstruction::Test.to_u8().unwrap()),
        Instruction::Jump(displacement) => {
            bytes.push(EncodedInstruction::Jump.to_u8().unwrap());
            bytes.extend_from_slice(&displacement.to_be_bytes());
        }
        Instruction::Call => bytes.push(EncodedInstruction::Call.to_u8().unwrap()),
        Instruction::Return => bytes.push(EncodedInstruction::Return.to_u8().unwrap()),
    }
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            functions: Vec::new(),
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn run(&mut self) -> Result<()> {
        let frame = self.frames.last_mut().unwrap();
        let function = &self.functions[frame.function.0];

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
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}
