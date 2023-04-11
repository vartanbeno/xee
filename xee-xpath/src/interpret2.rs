use crate::builder::Program;
use crate::error::{Error, Result};
use crate::instruction::{decode_instruction, Instruction};
use crate::value::{Closure, FunctionId, Value};

#[derive(Debug, Clone)]
struct Frame {
    function: FunctionId,
    ip: usize,
    base: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Interpreter<'a> {
    program: &'a Program,
    stack: Vec<Value>,
    frames: Vec<Frame>,
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
                Instruction::Closure(function_id) => {
                    self.stack.push(Value::Closure(Closure {
                        function_id: FunctionId(function_id as usize),
                        values: Vec::new(),
                    }));
                }
                Instruction::Var(index) => {
                    self.stack.push(self.stack[base + index as usize].clone());
                }
                Instruction::ClosureVar(index) => {
                    // let closure = self.stack[base - 1].as_closure()?;
                    // self.stack.push(closure[index as usize].clone());
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
                    let closure =
                        self.stack[self.stack.len() - (arity as usize + 1)].as_closure()?;
                    let function_id = closure.function_id;
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
            }
        }
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
