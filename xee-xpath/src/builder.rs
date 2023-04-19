use crate::ast;
use crate::instruction::{encode_instruction, Instruction};
use crate::value::{Function, FunctionId, StackValue, StaticFunctionId, Step};

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) functions: Vec<Function>,
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

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ForwardJumpRef(usize);

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BackwardJumpRef(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum JumpCondition {
    Always,
    True,
    False,
}

pub(crate) struct FunctionBuilder<'a> {
    program: &'a mut Program,
    compiled: Vec<u8>,
    constants: Vec<StackValue>,
    closure_names: Vec<ast::Name>,
    steps: Vec<Step>,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn new(program: &'a mut Program) -> Self {
        FunctionBuilder {
            program,
            compiled: Vec::new(),
            constants: Vec::new(),
            closure_names: Vec::new(),
            steps: Vec::new(),
        }
    }

    pub(crate) fn emit(&mut self, instruction: Instruction) {
        encode_instruction(instruction, &mut self.compiled);
    }

    pub(crate) fn emit_constant(&mut self, constant: StackValue) {
        let constant_id = self.constants.len();
        self.constants.push(constant);
        if constant_id > (u16::MAX as usize) {
            panic!("too many constants");
        }
        self.emit(Instruction::Const(constant_id as u16));
    }

    pub(crate) fn emit_step(&mut self, axis: ast::Axis, node_test: ast::NodeTest) {
        let step_id = self.steps.len();
        self.steps.push(Step { axis, node_test });
        if step_id > (u16::MAX as usize) {
            panic!("too many steps");
        }
        self.emit(Instruction::Step(step_id as u16));
    }

    pub(crate) fn emit_static_function(&mut self, static_function_id: StaticFunctionId) {
        self.emit(Instruction::StaticFunction(static_function_id.as_u16()));
    }

    pub(crate) fn add_closure_name(&mut self, name: &ast::Name) -> usize {
        let found = self.closure_names.iter().position(|n| n == name);
        if let Some(index) = found {
            return index;
        }
        let index = self.closure_names.len();
        self.closure_names.push(name.clone());
        if index > (u16::MAX as usize) {
            panic!("too many closure names");
        }
        index
    }

    pub(crate) fn emit_compare_value(&mut self, comparison: Comparison) {
        match comparison {
            Comparison::Eq => self.emit(Instruction::Eq),
            Comparison::Ne => self.emit(Instruction::Ne),
            Comparison::Lt => self.emit(Instruction::Lt),
            Comparison::Le => self.emit(Instruction::Le),
            Comparison::Gt => self.emit(Instruction::Gt),
            Comparison::Ge => self.emit(Instruction::Ge),
        }
    }

    pub(crate) fn loop_start(&self) -> BackwardJumpRef {
        BackwardJumpRef(self.compiled.len())
    }

    pub(crate) fn emit_jump_backward(
        &mut self,
        jump_ref: BackwardJumpRef,
        condition: JumpCondition,
    ) {
        let current = self.compiled.len() + 3;
        let offset = current - jump_ref.0;
        if jump_ref.0 > current {
            panic!("cannot jump forward");
        }
        if offset > (u16::MAX as usize) {
            panic!("jump too far");
        }

        match condition {
            JumpCondition::True => self.emit(Instruction::JumpIfTrue(-(offset as i16))),
            JumpCondition::False => self.emit(Instruction::JumpIfFalse(-(offset as i16))),
            JumpCondition::Always => self.emit(Instruction::Jump(-(offset as i16))),
        }
    }

    pub(crate) fn emit_jump_forward(&mut self, condition: JumpCondition) -> ForwardJumpRef {
        let index = self.compiled.len();
        match condition {
            JumpCondition::True => self.emit(Instruction::JumpIfTrue(0)),
            JumpCondition::False => self.emit(Instruction::JumpIfFalse(0)),
            JumpCondition::Always => self.emit(Instruction::Jump(0)),
        }
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
            closure_names: self.closure_names,
            constants: self.constants,
            steps: self.steps,
        }
    }

    pub(crate) fn builder(&mut self) -> FunctionBuilder {
        FunctionBuilder::new(self.program)
    }

    pub(crate) fn add_function(&mut self, function: Function) -> FunctionId {
        self.program.add_function(function)
    }
}
