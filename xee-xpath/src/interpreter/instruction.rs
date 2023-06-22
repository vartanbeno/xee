use num::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Instruction {
    // binary operators
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    // unary operators
    Plus,
    Minus,
    //
    Concat,
    Const(u16),
    Closure(u16),
    StaticClosure(u16),
    Var(u16),
    Set(u16),
    ClosureVar(u16),
    Comma,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    GenEq,
    GenNe,
    GenLt,
    GenLe,
    GenGt,
    GenGe,
    Union,
    Jump(i16),
    JumpIfTrue(i16),
    JumpIfFalse(i16),
    Call(u8),
    Step(u16),
    Return,
    Dup,
    Pop,
    LetDone,

    Range,
    SequenceNew,
    SequenceLen,
    SequenceGet,
    SequencePush,
    IsNumeric,
    PrintTop,
    PrintStack,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToPrimitive, FromPrimitive)]
pub(crate) enum EncodedInstruction {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Plus,
    Minus,
    Concat,
    Const,
    Closure,
    StaticClosure,
    Var,
    Set,
    ClosureVar,
    Comma,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    GenEq,
    GenNe,
    GenLt,
    GenLe,
    GenGt,
    GenGe,
    Union,
    Jump,
    JumpIfTrue,
    JumpIfFalse,
    Call,
    Step,
    Return,
    Dup,
    Pop,
    LetDone,
    Range,
    SequenceNew,
    SequenceLen,
    SequenceGet,
    SequencePush,
    IsNumeric,
    PrintTop,
    PrintStack,
}

// decode a single instruction from the slice
pub(crate) fn decode_instruction(bytes: &[u8]) -> (Instruction, usize) {
    let encoded_instruction = EncodedInstruction::from_u8(bytes[0]).unwrap();
    match encoded_instruction {
        EncodedInstruction::Add => (Instruction::Add, 1),
        EncodedInstruction::Sub => (Instruction::Sub, 1),
        EncodedInstruction::Mul => (Instruction::Mul, 1),
        EncodedInstruction::Div => (Instruction::Div, 1),
        EncodedInstruction::IntDiv => (Instruction::IntDiv, 1),
        EncodedInstruction::Mod => (Instruction::Mod, 1),
        EncodedInstruction::Plus => (Instruction::Plus, 1),
        EncodedInstruction::Minus => (Instruction::Minus, 1),
        EncodedInstruction::Concat => (Instruction::Concat, 1),
        EncodedInstruction::Const => {
            let constant = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Const(constant), 3)
        }
        EncodedInstruction::Closure => {
            let function = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Closure(function), 3)
        }
        EncodedInstruction::StaticClosure => {
            let function = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::StaticClosure(function), 3)
        }
        EncodedInstruction::Var => {
            let variable = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Var(variable), 3)
        }
        EncodedInstruction::Set => {
            let variable = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Set(variable), 3)
        }
        EncodedInstruction::ClosureVar => {
            let variable = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::ClosureVar(variable), 3)
        }
        EncodedInstruction::Comma => (Instruction::Comma, 1),
        EncodedInstruction::Eq => (Instruction::Eq, 1),
        EncodedInstruction::Ne => (Instruction::Ne, 1),
        EncodedInstruction::Lt => (Instruction::Lt, 1),
        EncodedInstruction::Le => (Instruction::Le, 1),
        EncodedInstruction::Gt => (Instruction::Gt, 1),
        EncodedInstruction::Ge => (Instruction::Ge, 1),
        EncodedInstruction::GenEq => (Instruction::GenEq, 1),
        EncodedInstruction::GenNe => (Instruction::GenNe, 1),
        EncodedInstruction::GenLt => (Instruction::GenLt, 1),
        EncodedInstruction::GenLe => (Instruction::GenLe, 1),
        EncodedInstruction::GenGt => (Instruction::GenGt, 1),
        EncodedInstruction::GenGe => (Instruction::GenGe, 1),
        EncodedInstruction::Union => (Instruction::Union, 1),
        EncodedInstruction::Jump => {
            let displacement = i16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Jump(displacement), 3)
        }
        EncodedInstruction::JumpIfTrue => {
            let displacement = i16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::JumpIfTrue(displacement), 3)
        }
        EncodedInstruction::JumpIfFalse => {
            let displacement = i16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::JumpIfFalse(displacement), 3)
        }
        EncodedInstruction::Call => {
            let arity = bytes[1];
            (Instruction::Call(arity), 2)
        }
        EncodedInstruction::Step => {
            let step = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Step(step), 3)
        }
        EncodedInstruction::Return => (Instruction::Return, 1),
        EncodedInstruction::Dup => (Instruction::Dup, 1),
        EncodedInstruction::Pop => (Instruction::Pop, 1),
        EncodedInstruction::LetDone => (Instruction::LetDone, 1),
        EncodedInstruction::Range => (Instruction::Range, 1),
        EncodedInstruction::SequenceNew => (Instruction::SequenceNew, 1),
        EncodedInstruction::SequenceLen => (Instruction::SequenceLen, 1),
        EncodedInstruction::SequenceGet => (Instruction::SequenceGet, 1),
        EncodedInstruction::SequencePush => (Instruction::SequencePush, 1),
        EncodedInstruction::IsNumeric => (Instruction::IsNumeric, 1),
        EncodedInstruction::PrintTop => (Instruction::PrintTop, 1),
        EncodedInstruction::PrintStack => (Instruction::PrintStack, 1),
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

pub(crate) fn encode_instruction(instruction: Instruction, bytes: &mut Vec<u8>) {
    match instruction {
        Instruction::Add => bytes.push(EncodedInstruction::Add.to_u8().unwrap()),
        Instruction::Sub => bytes.push(EncodedInstruction::Sub.to_u8().unwrap()),
        Instruction::Mul => bytes.push(EncodedInstruction::Mul.to_u8().unwrap()),
        Instruction::Div => bytes.push(EncodedInstruction::Div.to_u8().unwrap()),
        Instruction::IntDiv => bytes.push(EncodedInstruction::IntDiv.to_u8().unwrap()),
        Instruction::Mod => bytes.push(EncodedInstruction::Mod.to_u8().unwrap()),
        Instruction::Plus => bytes.push(EncodedInstruction::Plus.to_u8().unwrap()),
        Instruction::Minus => bytes.push(EncodedInstruction::Minus.to_u8().unwrap()),
        Instruction::Concat => bytes.push(EncodedInstruction::Concat.to_u8().unwrap()),
        Instruction::Const(constant) => {
            bytes.push(EncodedInstruction::Const.to_u8().unwrap());
            bytes.extend_from_slice(&constant.to_le_bytes());
        }
        Instruction::Closure(function_id) => {
            bytes.push(EncodedInstruction::Closure.to_u8().unwrap());
            bytes.extend_from_slice(&function_id.to_le_bytes());
        }
        Instruction::StaticClosure(function_id) => {
            bytes.push(EncodedInstruction::StaticClosure.to_u8().unwrap());
            bytes.extend_from_slice(&function_id.to_le_bytes());
        }
        Instruction::Var(variable) => {
            bytes.push(EncodedInstruction::Var.to_u8().unwrap());
            bytes.extend_from_slice(&variable.to_le_bytes());
        }
        Instruction::Set(variable) => {
            bytes.push(EncodedInstruction::Set.to_u8().unwrap());
            bytes.extend_from_slice(&variable.to_le_bytes());
        }
        Instruction::ClosureVar(variable) => {
            bytes.push(EncodedInstruction::ClosureVar.to_u8().unwrap());
            bytes.extend_from_slice(&variable.to_le_bytes());
        }
        Instruction::Comma => bytes.push(EncodedInstruction::Comma.to_u8().unwrap()),
        Instruction::Eq => bytes.push(EncodedInstruction::Eq.to_u8().unwrap()),
        Instruction::Ne => bytes.push(EncodedInstruction::Ne.to_u8().unwrap()),
        Instruction::Lt => bytes.push(EncodedInstruction::Lt.to_u8().unwrap()),
        Instruction::Le => bytes.push(EncodedInstruction::Le.to_u8().unwrap()),
        Instruction::Gt => bytes.push(EncodedInstruction::Gt.to_u8().unwrap()),
        Instruction::Ge => bytes.push(EncodedInstruction::Ge.to_u8().unwrap()),
        Instruction::GenEq => bytes.push(EncodedInstruction::GenEq.to_u8().unwrap()),
        Instruction::GenNe => bytes.push(EncodedInstruction::GenNe.to_u8().unwrap()),
        Instruction::GenLt => bytes.push(EncodedInstruction::GenLt.to_u8().unwrap()),
        Instruction::GenLe => bytes.push(EncodedInstruction::GenLe.to_u8().unwrap()),
        Instruction::GenGt => bytes.push(EncodedInstruction::GenGt.to_u8().unwrap()),
        Instruction::GenGe => bytes.push(EncodedInstruction::GenGe.to_u8().unwrap()),
        Instruction::Union => bytes.push(EncodedInstruction::Union.to_u8().unwrap()),
        Instruction::Jump(displacement) => {
            bytes.push(EncodedInstruction::Jump.to_u8().unwrap());
            bytes.extend_from_slice(&displacement.to_le_bytes());
        }
        Instruction::JumpIfTrue(displacement) => {
            bytes.push(EncodedInstruction::JumpIfTrue.to_u8().unwrap());
            bytes.extend_from_slice(&displacement.to_le_bytes());
        }
        Instruction::JumpIfFalse(displacement) => {
            bytes.push(EncodedInstruction::JumpIfFalse.to_u8().unwrap());
            bytes.extend_from_slice(&displacement.to_le_bytes());
        }
        Instruction::Call(arity) => {
            bytes.push(EncodedInstruction::Call.to_u8().unwrap());
            bytes.push(arity);
        }
        Instruction::Step(step_id) => {
            bytes.push(EncodedInstruction::Step.to_u8().unwrap());
            bytes.extend_from_slice(&step_id.to_le_bytes());
        }
        Instruction::Return => bytes.push(EncodedInstruction::Return.to_u8().unwrap()),
        Instruction::Dup => bytes.push(EncodedInstruction::Dup.to_u8().unwrap()),
        Instruction::Pop => bytes.push(EncodedInstruction::Pop.to_u8().unwrap()),
        Instruction::LetDone => bytes.push(EncodedInstruction::LetDone.to_u8().unwrap()),
        Instruction::Range => bytes.push(EncodedInstruction::Range.to_u8().unwrap()),
        Instruction::SequenceNew => bytes.push(EncodedInstruction::SequenceNew.to_u8().unwrap()),
        Instruction::SequenceLen => bytes.push(EncodedInstruction::SequenceLen.to_u8().unwrap()),
        Instruction::SequenceGet => bytes.push(EncodedInstruction::SequenceGet.to_u8().unwrap()),
        Instruction::SequencePush => bytes.push(EncodedInstruction::SequencePush.to_u8().unwrap()),
        Instruction::IsNumeric => bytes.push(EncodedInstruction::IsNumeric.to_u8().unwrap()),
        Instruction::PrintTop => bytes.push(EncodedInstruction::PrintTop.to_u8().unwrap()),
        Instruction::PrintStack => bytes.push(EncodedInstruction::PrintStack.to_u8().unwrap()),
    }
}

pub(crate) fn encode_instructions(instructions: Vec<Instruction>, bytes: &mut Vec<u8>) {
    for instruction in instructions {
        encode_instruction(instruction, bytes);
    }
}

// size in bytes for an instruction
pub(crate) fn instruction_size(instruction: &Instruction) -> usize {
    match instruction {
        Instruction::Add
        | Instruction::Sub
        | Instruction::Mul
        | Instruction::Div
        | Instruction::IntDiv
        | Instruction::Mod
        | Instruction::Plus
        | Instruction::Minus
        | Instruction::Concat
        | Instruction::Comma
        | Instruction::Eq
        | Instruction::Ne
        | Instruction::Lt
        | Instruction::Le
        | Instruction::Gt
        | Instruction::Ge
        | Instruction::GenEq
        | Instruction::GenNe
        | Instruction::GenLt
        | Instruction::GenLe
        | Instruction::GenGt
        | Instruction::GenGe
        | Instruction::Union
        | Instruction::Return
        | Instruction::Dup
        | Instruction::Pop
        | Instruction::LetDone
        | Instruction::Range
        | Instruction::SequenceNew
        | Instruction::SequenceLen
        | Instruction::SequenceGet
        | Instruction::SequencePush
        | Instruction::IsNumeric
        | Instruction::PrintTop
        | Instruction::PrintStack => 1,
        Instruction::Call(_) => 2,
        Instruction::Const(_)
        | Instruction::Closure(_)
        | Instruction::StaticClosure(_)
        | Instruction::Var(_)
        | Instruction::Set(_)
        | Instruction::ClosureVar(_)
        | Instruction::Jump(_)
        | Instruction::JumpIfTrue(_)
        | Instruction::Step(_)
        | Instruction::JumpIfFalse(_) => 3,
    }
}

pub(crate) fn read_instruction(bytes: &[u8], ip: &mut usize) -> EncodedInstruction {
    let byte = bytes[*ip];
    *ip += 1;
    EncodedInstruction::from_u8(byte).unwrap()
}

pub(crate) fn read_u16(bytes: &[u8], ip: &mut usize) -> u16 {
    let bytes = &bytes[*ip..*ip + 2];
    *ip += 2;
    u16::from_le_bytes([bytes[0], bytes[1]])
}

pub(crate) fn read_i16(bytes: &[u8], ip: &mut usize) -> i16 {
    let bytes = &bytes[*ip..*ip + 2];
    *ip += 2;
    i16::from_le_bytes([bytes[0], bytes[1]])
}

pub(crate) fn read_u8(bytes: &[u8], ip: &mut usize) -> u8 {
    let byte = bytes[*ip];
    *ip += 1;
    byte
}

#[cfg(test)]
use crate::stack::Function;

#[cfg(test)]
impl Function {
    pub(crate) fn decoded(&self) -> Vec<Instruction> {
        decode_instructions(&self.chunk)
    }
}
