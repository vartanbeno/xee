use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::ir;

use super::static_function::FunctionKind;

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub(crate) parameter_types: Vec<Option<ast::SequenceType>>,
    pub(crate) return_type: Option<ast::SequenceType>,
}

impl Signature {
    pub(crate) fn map_signature() -> Self {
        let key = ast::SequenceType::Item(ast::Item {
            item_type: ast::ItemType::AtomicOrUnionType(Xs::AnyAtomicType),
            occurrence: ast::Occurrence::One,
        });

        let return_type = ast::SequenceType::Item(ast::Item {
            item_type: ast::ItemType::Item,
            occurrence: ast::Occurrence::Many,
        });
        Self {
            parameter_types: vec![Some(key)],
            return_type: Some(return_type),
        }
    }

    pub(crate) fn array_signature() -> Self {
        let position = ast::SequenceType::Item(ast::Item {
            item_type: ast::ItemType::AtomicOrUnionType(Xs::AnyAtomicType),
            occurrence: ast::Occurrence::One,
        });

        let return_type = ast::SequenceType::Item(ast::Item {
            item_type: ast::ItemType::Item,
            occurrence: ast::Occurrence::Many,
        });
        Self {
            parameter_types: vec![Some(position)],
            return_type: Some(return_type),
        }
    }

    pub(crate) fn alternative_signatures(
        &self,
        function_kind: FunctionKind,
    ) -> Vec<(Signature, Option<FunctionKind>)> {
        match function_kind {
            FunctionKind::ItemFirst => vec![
                (
                    Self {
                        parameter_types: self.parameter_types[1..].to_vec(),
                        return_type: self.return_type.clone(),
                    },
                    Some(function_kind),
                ),
                (self.clone(), None),
            ],
            FunctionKind::ItemLast => vec![
                (
                    Self {
                        parameter_types: self.parameter_types[..self.parameter_types.len() - 1]
                            .to_vec(),
                        return_type: self.return_type.clone(),
                    },
                    Some(function_kind),
                ),
                (self.clone(), None),
            ],
            FunctionKind::ItemLastOptional => vec![(
                Self {
                    parameter_types: self.parameter_types[..self.parameter_types.len() - 1]
                        .to_vec(),
                    return_type: self.return_type.clone(),
                },
                Some(function_kind),
            )],
            FunctionKind::Position => vec![(self.clone(), Some(function_kind))],
            FunctionKind::Size => vec![(self.clone(), Some(function_kind))],
            FunctionKind::Collation => vec![
                (
                    Self {
                        parameter_types: self.parameter_types[..self.parameter_types.len() - 1]
                            .to_vec(),
                        return_type: self.return_type.clone(),
                    },
                    Some(function_kind),
                ),
                (self.clone(), None),
            ],
        }
    }

    pub(crate) fn arity(&self) -> usize {
        self.parameter_types.len()
    }
}

impl From<ast::Signature> for Signature {
    fn from(signature: ast::Signature) -> Self {
        Self {
            parameter_types: signature
                .params
                .into_iter()
                .map(|p| Some(p.type_))
                .collect(),
            return_type: Some(signature.return_type),
        }
    }
}

impl From<&ir::FunctionDefinition> for Signature {
    fn from(function_definition: &ir::FunctionDefinition) -> Self {
        Self {
            parameter_types: function_definition
                .params
                .iter()
                .map(|param| param.type_.clone())
                .collect(),
            return_type: function_definition.return_type.clone(),
        }
    }
}
