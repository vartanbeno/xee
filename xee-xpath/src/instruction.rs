use num::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Instruction {
    // binary operators
    Add,
    Sub,
    Const(u16),
    Closure(u16),
    StaticFunction(u16),
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
    Union,
    Jump(i16),
    JumpIfTrue(i16),
    JumpIfFalse(i16),
    Call(u8),
    Return,
    Dup,
    Pop,
    LetDone,
    Range,
    SequenceNew,
    SequenceLen,
    SequenceGet,
    SequencePush,
    Step(u16),
    PrintTop,
    PrintStack,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToPrimitive, FromPrimitive)]
enum EncodedInstruction {
    Add,
    Sub,
    Const,
    Closure,
    StaticFunction,
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
    Union,
    Jump,
    JumpIfTrue,
    JumpIfFalse,
    Call,
    Return,
    Dup,
    Pop,
    LetDone,
    Range,
    SequenceNew,
    SequenceLen,
    SequenceGet,
    SequencePush,
    Step,
    PrintTop,
    PrintStack,
}

// decode a single instruction from the slice
pub(crate) fn decode_instruction(bytes: &[u8]) -> (Instruction, usize) {
    let encoded_instruction = EncodedInstruction::from_u8(bytes[0]).unwrap();
    match encoded_instruction {
        EncodedInstruction::Add => (Instruction::Add, 1),
        EncodedInstruction::Sub => (Instruction::Sub, 1),
        EncodedInstruction::Const => {
            let constant = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Const(constant), 3)
        }
        EncodedInstruction::Closure => {
            let function = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Closure(function), 3)
        }
        EncodedInstruction::StaticFunction => {
            let function = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::StaticFunction(function), 3)
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
        EncodedInstruction::Return => (Instruction::Return, 1),
        EncodedInstruction::Dup => (Instruction::Dup, 1),
        EncodedInstruction::Pop => (Instruction::Pop, 1),
        EncodedInstruction::LetDone => (Instruction::LetDone, 1),
        EncodedInstruction::Range => (Instruction::Range, 1),
        EncodedInstruction::SequenceNew => (Instruction::SequenceNew, 1),
        EncodedInstruction::SequenceLen => (Instruction::SequenceLen, 1),
        EncodedInstruction::SequenceGet => (Instruction::SequenceGet, 1),
        EncodedInstruction::SequencePush => (Instruction::SequencePush, 1),
        EncodedInstruction::Step => {
            let axis = u16::from_le_bytes([bytes[1], bytes[2]]);
            (Instruction::Step(axis), 3)
        }
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
        Instruction::Const(constant) => {
            bytes.push(EncodedInstruction::Const.to_u8().unwrap());
            bytes.extend_from_slice(&constant.to_le_bytes());
        }
        Instruction::Closure(function_id) => {
            bytes.push(EncodedInstruction::Closure.to_u8().unwrap());
            bytes.extend_from_slice(&function_id.to_le_bytes());
        }
        Instruction::StaticFunction(function_id) => {
            bytes.push(EncodedInstruction::StaticFunction.to_u8().unwrap());
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
        Instruction::Return => bytes.push(EncodedInstruction::Return.to_u8().unwrap()),
        Instruction::Dup => bytes.push(EncodedInstruction::Dup.to_u8().unwrap()),
        Instruction::Pop => bytes.push(EncodedInstruction::Pop.to_u8().unwrap()),
        Instruction::LetDone => bytes.push(EncodedInstruction::LetDone.to_u8().unwrap()),
        Instruction::Range => bytes.push(EncodedInstruction::Range.to_u8().unwrap()),
        Instruction::SequenceNew => bytes.push(EncodedInstruction::SequenceNew.to_u8().unwrap()),
        Instruction::SequenceLen => bytes.push(EncodedInstruction::SequenceLen.to_u8().unwrap()),
        Instruction::SequenceGet => bytes.push(EncodedInstruction::SequenceGet.to_u8().unwrap()),
        Instruction::SequencePush => bytes.push(EncodedInstruction::SequencePush.to_u8().unwrap()),
        Instruction::Step(step_id) => {
            bytes.push(EncodedInstruction::Step.to_u8().unwrap());
            bytes.extend_from_slice(&step_id.to_le_bytes());
        }
        Instruction::PrintTop => bytes.push(EncodedInstruction::PrintTop.to_u8().unwrap()),
        Instruction::PrintStack => bytes.push(EncodedInstruction::PrintStack.to_u8().unwrap()),
    }
}

pub(crate) fn encode_instructions(instructions: Vec<Instruction>, bytes: &mut Vec<u8>) {
    for instruction in instructions {
        encode_instruction(instruction, bytes);
    }
}
