use std::rc::Rc;

use xee_xpath_ast::ast;

use crate::context::DynamicContext;
use crate::error::SpannedError;
use crate::function;
use crate::sequence;
use crate::stack;
use crate::xml;
use crate::Occurrence;
use crate::{error, Collation};

use super::Interpreter;
use super::Program;

#[derive(Debug, Clone)]
pub struct Runnable<'a> {
    program: &'a Program,
    map_signature: function::Signature,
    array_signature: function::Signature,
    // TODO: this should be private, but is needed right now
    // to implement call_static without lifetime issues.
    // We could possibly obtain context from the interpreter directly,
    // but this leads to lifetime issues right now.
    pub(crate) dynamic_context: &'a DynamicContext<'a>,
}

impl<'a> Runnable<'a> {
    pub(crate) fn new(program: &'a Program, dynamic_context: &'a DynamicContext<'a>) -> Self {
        Self {
            program,
            map_signature: function::Signature::map_signature(),
            array_signature: function::Signature::array_signature(),
            dynamic_context,
        }
    }

    pub(crate) fn run_value(
        &self,
        context_item: Option<&sequence::Item>,
    ) -> error::SpannedResult<stack::Value> {
        let mut interpreter = Interpreter::new(self);
        // TODO: the arguments aren't supplied to the function that are expected.
        // This should result in an error, preferrably the variable that is missing
        // underlined in the xpath expression. But that requires some more work to
        // accomplish, so for now we panic.
        let arguments = self.dynamic_context.arguments().unwrap();
        interpreter.start(context_item, arguments);
        interpreter.run(0)?;
        // the stack has to be 1 values and return the result of the expression
        // why 1 value if the context item is on the top of the stack? This is because
        // the outer main function will pop the context item; this code is there to
        // remove the function id from the stack but the main function has no function id
        assert_eq!(
            interpreter.state().stack().len(),
            1,
            "stack must only have 1 value but found {:?}",
            interpreter.state().stack()
        );
        let value = interpreter.state().stack().last().unwrap().clone();
        match value {
            stack::Value::Absent => Err(SpannedError {
                error: error::Error::XPDY0002,
                span: self.program.span().into(),
            }),
            _ => Ok(value),
        }
    }

    /// Run the program against a xot Node.
    pub fn many_xot_node(&self, node: xot::Node) -> error::SpannedResult<sequence::Sequence> {
        let node = xml::Node::Xot(node);
        let item = sequence::Item::Node(node);
        self.many(Some(&item))
    }

    /// Run the program against a sequence item.
    pub fn many(&self, item: Option<&sequence::Item>) -> error::SpannedResult<sequence::Sequence> {
        let value = self.run_value(item)?;
        Ok(value.into())
    }

    /// Run the program, expect a single item as the result.
    pub fn one(&self, item: Option<&sequence::Item>) -> error::SpannedResult<sequence::Item> {
        let sequence = self.many(item)?;
        sequence.items().one().map_err(|error| SpannedError {
            error,
            span: self.program.span().into(),
        })
    }

    /// Run the program, expect an optional single item as the result.
    pub fn option(
        &self,
        item: Option<&sequence::Item>,
    ) -> error::SpannedResult<Option<sequence::Item>> {
        let sequence = self.many(item)?;
        sequence.items().option().map_err(|error| SpannedError {
            error,
            span: self.program.span().into(),
        })
    }

    pub(crate) fn program(&self) -> &'a Program {
        self.program
    }

    pub fn dynamic_context(&self) -> &'a DynamicContext {
        self.dynamic_context
    }

    pub(crate) fn annotations(&self) -> &xml::Annotations {
        &self.dynamic_context.documents.annotations
    }

    pub fn xot(&self) -> &xot::Xot {
        self.dynamic_context.xot
    }

    pub fn default_collation_uri(&self) -> &str {
        self.dynamic_context.static_context.default_collation_uri()
    }

    pub fn default_collation(&self) -> error::Result<Rc<Collation>> {
        self.dynamic_context.static_context.default_collation()
    }

    pub fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.dynamic_context.implicit_timezone()
    }

    pub(crate) fn inline_function(
        &self,
        function_id: function::InlineFunctionId,
    ) -> &'a function::InlineFunction {
        &self.program.functions[function_id.0]
    }

    pub(crate) fn static_function(
        &self,
        function_id: function::StaticFunctionId,
    ) -> &'a function::StaticFunction {
        self.dynamic_context
            .static_context
            .functions
            .get_by_index(function_id)
    }

    pub(crate) fn function_info<'function>(
        &'a self,
        function: &'function function::Function,
    ) -> FunctionInfo<'a, 'function> {
        FunctionInfo::new(function, self)
    }

    pub fn signature(&'a self, function: &function::Function) -> &'a function::Signature {
        self.function_info(function).signature()
    }
}

pub(crate) struct FunctionInfo<'runnable, 'function> {
    function: &'function function::Function,
    runnable: &'runnable Runnable<'runnable>,
}

impl<'runnable, 'function> FunctionInfo<'runnable, 'function> {
    pub(crate) fn new(
        function: &'function function::Function,
        runnable: &'runnable Runnable<'runnable>,
    ) -> FunctionInfo<'runnable, 'function> {
        FunctionInfo { function, runnable }
    }

    pub(crate) fn arity(&self) -> usize {
        match self.function {
            function::Function::Inline {
                inline_function_id, ..
            } => self.runnable.inline_function(*inline_function_id).arity(),
            function::Function::Static {
                static_function_id, ..
            } => self.runnable.static_function(*static_function_id).arity(),
            function::Function::Array(_) => 1,
            function::Function::Map(_) => 1,
        }
    }

    pub(crate) fn name(&self) -> Option<ast::Name> {
        match self.function {
            function::Function::Static {
                static_function_id, ..
            } => {
                let static_function = self.runnable.static_function(*static_function_id);
                Some(static_function.name().clone())
            }
            _ => None,
        }
    }

    pub(crate) fn signature(&self) -> &'runnable function::Signature {
        match &self.function {
            function::Function::Static {
                static_function_id, ..
            } => {
                let static_function = self.runnable.static_function(*static_function_id);
                static_function.signature()
            }
            function::Function::Inline {
                inline_function_id, ..
            } => {
                let inline_function = self.runnable.inline_function(*inline_function_id);
                inline_function.signature()
            }
            function::Function::Map(_map) => &self.runnable.map_signature,
            function::Function::Array(_array) => &self.runnable.array_signature,
        }
    }
}
