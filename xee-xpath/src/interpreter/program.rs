use crate::function;

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) src: String,
    pub(crate) functions: Vec<function::InlineFunction>,
}

impl Program {
    pub(crate) fn new(src: String) -> Self {
        Program {
            src,
            functions: Vec::new(),
        }
    }

    pub(crate) fn add_function(
        &mut self,
        function: function::InlineFunction,
    ) -> function::InlineFunctionId {
        let id = self.functions.len();
        if id > u16::MAX as usize {
            panic!("too many functions");
        }
        self.functions.push(function);

        function::InlineFunctionId(id)
    }

    pub(crate) fn get_function(&self, index: usize) -> &function::InlineFunction {
        &self.functions[index]
    }

    pub(crate) fn get_function_by_id(
        &self,
        id: function::InlineFunctionId,
    ) -> &function::InlineFunction {
        self.get_function(id.0)
    }

    pub(crate) fn main_id(&self) -> function::InlineFunctionId {
        function::InlineFunctionId(self.functions.len() - 1)
    }
}
