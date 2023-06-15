use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub(crate) function_id: stack::ClosureFunctionId,
    // pub(crate) values: Vec<Vec<OutputItem>>,
}
