use ahash::{HashMap, HashMapExt};
use std::fmt::{Debug, Formatter};
use xee_xpath_ast::ast;
use xee_xpath_ast::Namespaces;

use crate::error;
use crate::func::static_function_descriptions;
use crate::interpreter;
use crate::sequence;
use crate::stack;

use super::dynamic_context::DynamicContext;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum FunctionKind {
    // generate a function with one less arity that takes the
    // item as the first argument
    ItemFirst,
    // generate a function with one less arity that takes the item
    // as the last argument
    ItemLast,
    // this function takes position as the implicit only argument
    Position,
    // this function takes size as the implicit only argument
    Size,
    // generate a function with one less arity that takes the collation
    // as the last argument
    Collation,
}

impl FunctionKind {
    pub(crate) fn parse(s: &str) -> Option<FunctionKind> {
        match s {
            "" => None,
            "context_first" => Some(FunctionKind::ItemFirst),
            "context_last" => Some(FunctionKind::ItemLast),
            "position" => Some(FunctionKind::Position),
            "size" => Some(FunctionKind::Size),
            "collation" => Some(FunctionKind::Collation),
            _ => panic!("Unknown function kind {}", s),
        }
    }
}

pub(crate) type StaticFunctionType = fn(
    context: &DynamicContext,
    interpreter: &mut interpreter::Interpreter,
    arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence>;

pub(crate) struct StaticFunctionDescription {
    pub(crate) name: ast::Name,
    pub(crate) arity: usize,
    pub(crate) function_kind: Option<FunctionKind>,
    pub(crate) func: StaticFunctionType,
}

// Wraps a Rust function annotated with `#[xpath_fn]` and turns it
// into a StaticFunctionDescription
#[macro_export]
macro_rules! wrap_xpath_fn {
    ($function:path) => {{
        use $function as wrapped_function;
        let namespaces = xee_xpath_ast::Namespaces::default();
        $crate::context::StaticFunctionDescription::new(
            wrapped_function::WRAPPER,
            wrapped_function::SIGNATURE,
            $crate::context::FunctionKind::parse(wrapped_function::KIND),
            &namespaces,
        )
    }};
}

impl StaticFunctionDescription {
    pub(crate) fn new(
        func: StaticFunctionType,
        signature: &str,
        function_kind: Option<FunctionKind>,
        namespaces: &Namespaces,
    ) -> Self {
        // XXX reparse signature; the macro could have stored the parsed
        // version as code, but that's more work than I'm prepared to do
        // right now.
        let signature = ast::Signature::parse(signature, namespaces)
            .expect("Signature parse failed unexpectedly");

        Self {
            name: signature.name.value,
            arity: signature.params.len(),
            function_kind,
            func,
        }
    }

    fn functions(&self) -> Vec<StaticFunction> {
        if let Some(function_kind) = &self.function_kind {
            match function_kind {
                FunctionKind::ItemFirst => {
                    vec![
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity - 1,
                            function_rule: Some(FunctionRule::ItemFirst),
                            func: self.func,
                        },
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity,
                            function_rule: None,
                            func: self.func,
                        },
                    ]
                }
                FunctionKind::ItemLast => {
                    vec![
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity - 1,
                            function_rule: Some(FunctionRule::ItemLast),
                            func: self.func,
                        },
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity,
                            function_rule: None,
                            func: self.func,
                        },
                    ]
                }
                FunctionKind::Position => {
                    vec![StaticFunction {
                        name: self.name.clone(),
                        arity: self.arity,
                        function_rule: Some(FunctionRule::PositionFirst),
                        func: self.func,
                    }]
                }
                FunctionKind::Size => {
                    vec![StaticFunction {
                        name: self.name.clone(),
                        arity: self.arity,
                        function_rule: Some(FunctionRule::SizeFirst),
                        func: self.func,
                    }]
                }
                FunctionKind::Collation => {
                    vec![
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity - 1,
                            function_rule: Some(FunctionRule::Collation),
                            func: self.func,
                        },
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity,
                            function_rule: None,
                            func: self.func,
                        },
                    ]
                }
            }
        } else {
            vec![StaticFunction {
                name: self.name.clone(),
                arity: self.arity,
                function_rule: None,
                func: self.func,
            }]
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum FunctionRule {
    ItemFirst,
    ItemLast,
    PositionFirst,
    SizeFirst,
    Collation,
}

pub(crate) struct StaticFunction {
    name: ast::Name,
    arity: usize,
    pub(crate) function_rule: Option<FunctionRule>,
    func: StaticFunctionType,
}

impl Debug for StaticFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .field("function_rule", &self.function_rule)
            .finish()
    }
}

impl StaticFunction {
    pub(crate) fn needs_context(&self) -> bool {
        match self.function_rule {
            None | Some(FunctionRule::Collation) => false,
            Some(_) => true,
        }
    }

    pub(crate) fn invoke(
        &self,
        context: &DynamicContext,
        interpreter: &mut interpreter::Interpreter,
        closure_values: &[sequence::Sequence],
        arity: u8,
    ) -> error::Result<sequence::Sequence> {
        let arguments = &interpreter.arguments(arity);
        if arguments.len() != self.arity {
            return Err(error::Error::Type);
        }
        let arguments = into_sequences(arguments);
        if let Some(function_rule) = &self.function_rule {
            match function_rule {
                FunctionRule::ItemFirst | FunctionRule::PositionFirst | FunctionRule::SizeFirst => {
                    let mut new_arguments = vec![closure_values[0].clone()];
                    new_arguments.extend_from_slice(&arguments);
                    (self.func)(context, interpreter, &new_arguments)
                }
                FunctionRule::ItemLast => {
                    let mut new_arguments = arguments.to_vec();
                    new_arguments.push(closure_values[0].clone());
                    (self.func)(context, interpreter, &new_arguments)
                }
                FunctionRule::Collation => {
                    let mut new_arguments = arguments.to_vec();
                    // the default collation query
                    new_arguments.push(context.static_context.default_collation_uri().into());
                    (self.func)(context, interpreter, &new_arguments)
                }
            }
        } else {
            (self.func)(context, interpreter, &arguments)
        }
    }
}

fn into_sequences(values: &[stack::Value]) -> Vec<sequence::Sequence> {
    values.iter().map(|v| v.into()).collect()
}

#[derive(Debug)]
pub(crate) struct StaticFunctions {
    by_name: HashMap<(ast::Name, u8), stack::StaticFunctionId>,
    by_index: Vec<StaticFunction>,
}

impl StaticFunctions {
    pub(crate) fn new() -> Self {
        let mut by_name = HashMap::new();
        let descriptions = static_function_descriptions();
        let mut by_index = Vec::new();
        for description in descriptions {
            by_index.extend(description.functions());
        }

        for (i, static_function) in by_index.iter().enumerate() {
            by_name.insert(
                (static_function.name.clone(), static_function.arity as u8),
                stack::StaticFunctionId(i),
            );
        }
        Self { by_name, by_index }
    }

    pub(crate) fn get_by_name(
        &self,
        name: &ast::Name,
        arity: u8,
    ) -> Option<stack::StaticFunctionId> {
        // TODO annoying clone
        self.by_name.get(&(name.clone(), arity)).copied()
    }

    pub(crate) fn get_by_index(
        &self,
        static_function_id: stack::StaticFunctionId,
    ) -> &StaticFunction {
        &self.by_index[static_function_id.0]
    }
}
