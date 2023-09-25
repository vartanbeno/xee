use xee_xpath_ast::ast;

use super::static_function::FunctionKind;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Signature {
    pub(crate) parameter_types: Vec<ast::SequenceType>,
    pub(crate) return_type: ast::SequenceType,
}

impl Signature {
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
            parameter_types: signature.params.into_iter().map(|p| p.type_).collect(),
            return_type: signature.return_type,
        }
    }
}
