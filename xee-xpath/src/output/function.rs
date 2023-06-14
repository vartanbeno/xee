use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub struct OutputClosure {
    pub(crate) function_id: stack::ClosureFunctionId,
    // pub(crate) values: Vec<Vec<OutputItem>>,
}
