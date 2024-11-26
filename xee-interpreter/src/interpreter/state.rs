use std::rc::Rc;

use arrayvec::ArrayVec;
use xot::Xot;

use crate::error;
use crate::function;
use crate::sequence;
use crate::stack;

const FRAMES_MAX: usize = 64;

#[derive(Debug, Clone)]
pub(crate) struct Frame {
    function: function::InlineFunctionId,
    base: usize,
    pub(crate) ip: usize,
}

impl Frame {
    pub(crate) fn function(&self) -> function::InlineFunctionId {
        self.function
    }
    pub(crate) fn base(&self) -> usize {
        self.base
    }
}

#[derive(Debug)]
pub struct State<'a> {
    stack: Vec<stack::Value>,
    build_stack: Vec<Vec<sequence::Item>>,
    frames: ArrayVec<Frame, FRAMES_MAX>,
    pub(crate) xot: &'a mut Xot,
}

impl<'a> State<'a> {
    pub(crate) fn new(xot: &'a mut Xot) -> Self {
        Self {
            stack: vec![],
            build_stack: vec![],
            frames: ArrayVec::new(),
            xot,
        }
    }

    pub(crate) fn push(&mut self, value: stack::Value) {
        self.stack.push(value);
    }

    pub(crate) fn build_new(&mut self) {
        self.build_stack.push(Vec::new());
    }

    pub(crate) fn build_push(&mut self) -> error::Result<()> {
        let build = &mut self.build_stack.last_mut().unwrap();
        let value = self.stack.pop().unwrap();
        Self::build_push_helper(build, value)
    }

    pub(crate) fn build_complete(&mut self) {
        let build = self.build_stack.pop().unwrap();
        self.stack.push(build.into());
    }

    fn build_push_helper(
        build: &mut Vec<sequence::Item>,
        value: stack::Value,
    ) -> error::Result<()> {
        match value {
            stack::Value::Empty => {}
            stack::Value::One(item) => build.push(item),
            stack::Value::Many(items) => build.extend(items.iter().cloned()),
            stack::Value::Absent => return Err(error::Error::XPDY0002)?,
        }
        Ok(())
    }

    pub(crate) fn push_var(&mut self, index: usize) {
        self.stack
            .push(self.stack[self.frame().base + index].clone());
    }

    pub(crate) fn push_closure_var(&mut self, index: usize) -> error::Result<()> {
        let function = self.function()?;
        let closure_vars = function.closure_vars();
        self.stack.push(closure_vars[index].clone().into());
        Ok(())
    }

    pub(crate) fn set_var(&mut self, index: usize) {
        let base = self.frame().base;
        self.stack[base + index] = self.stack.pop().unwrap();
    }

    pub(crate) fn pop(&mut self) -> error::Result<sequence::Sequence> {
        self.stack.pop().unwrap().try_into()
    }

    pub(crate) fn function(&self) -> error::Result<Rc<function::Function>> {
        // the function is always just below the base
        (&self.stack[self.frame().base - 1]).try_into()
    }

    pub(crate) fn push_start_frame(&mut self, function_id: function::InlineFunctionId) {
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: 0,
        });
    }

    pub(crate) fn push_frame(
        &mut self,
        function_id: function::InlineFunctionId,
        arity: usize,
    ) -> error::Result<()> {
        if self.frames.len() >= self.frames.capacity() {
            return Err(error::Error::StackOverflow);
        }
        self.frames.push(Frame {
            function: function_id,
            ip: 0,
            base: self.stack.len() - arity,
        });
        Ok(())
    }

    pub(crate) fn frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    pub(crate) fn frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    pub(crate) fn jump(&mut self, displacement: i32) {
        self.frame_mut().ip = (self.frame().ip as i32 + displacement) as usize;
    }

    pub(crate) fn callable(&self, arity: usize) -> error::Result<Rc<function::Function>> {
        let value = &self.stack[self.stack.len() - (arity + 1)];
        // TODO: check that arity of function matches arity of call
        value.try_into()
    }

    pub(crate) fn arguments(&self, arity: usize) -> &[stack::Value] {
        &self.stack[self.stack.len() - arity..]
    }

    pub(crate) fn inline_return(&mut self, start_base: usize) -> bool {
        let return_value = self.stack.pop().unwrap();

        // truncate the stack to the base
        let base = self.frame().base;
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

        // if the start base is the same as the base we just popped off,
        // we are done
        base == start_base
    }

    pub(crate) fn static_return(&mut self, arity: usize) {
        // truncate the stack to the base
        self.stack.truncate(self.stack.len() - (arity + 1));
    }

    pub(crate) fn top(&self) -> error::Result<sequence::Sequence> {
        self.stack.last().unwrap().try_into()
    }

    pub fn stack(&self) -> &[stack::Value] {
        &self.stack
    }

    pub fn xot(&self) -> &Xot {
        self.xot
    }

    pub fn xot_mut(&mut self) -> &mut Xot {
        self.xot
    }
}
