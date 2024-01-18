use ahash::{HashMap, HashMapExt};
use xot::Xot;

use xee_xpath_ast::pattern;

use crate::{sequence::Item, xml};

#[derive(Debug, Default)]
pub struct PatternLookup<V> {
    by_name: HashMap<xee_name::Name, V>,
}

impl<V> PatternLookup<V> {
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
        }
    }

    pub fn add(&mut self, pattern: &pattern::Pattern, value: V) {
        match pattern {
            pattern::Pattern::Expr(expr_pattern) => {
                self.add_expr_pattern(expr_pattern, value);
            }
            pattern::Pattern::Predicate(_predicate_pattern) => {
                todo!()
            }
        }
    }

    fn add_expr_pattern(&mut self, expr_pattern: &pattern::ExprPattern, value: V) {
        match expr_pattern {
            pattern::ExprPattern::Path(path_expr) => {
                self.add_path_expr(path_expr, value);
            }
            pattern::ExprPattern::BinaryExpr(_binary_expr) => {
                todo!();
            }
        }
    }

    fn add_path_expr(&mut self, path_expr: &pattern::PathExpr, value: V) {
        match &path_expr.root {
            pattern::PathRoot::Rooted {
                root: _,
                predicates: _,
            } => {
                todo!()
            }
            pattern::PathRoot::AbsoluteSlash => {
                todo!();
            }
            pattern::PathRoot::AbsoluteDoubleSlash => {
                todo!();
            }
            pattern::PathRoot::Relative => {
                self.add_relative_steps(&path_expr.steps, value);
            }
        }
    }

    fn add_relative_steps(&mut self, steps: &[pattern::StepExpr], value: V) {
        if steps.len() != 1 {
            todo!();
        }
        let step = &steps[0];
        match step {
            pattern::StepExpr::AxisStep(axis_step) => {
                self.add_single_axis_step(axis_step, value);
            }
            pattern::StepExpr::PostfixExpr(_postfix_expr) => {
                todo!()
            }
        }
    }

    fn add_single_axis_step(&mut self, step: &pattern::AxisStep, value: V) {
        if step.forward != pattern::ForwardAxis::Child {
            todo!();
        }
        if !step.predicates.is_empty() {
            todo!();
        }
        match &step.node_test {
            pattern::NodeTest::NameTest(name_test) => match name_test {
                pattern::NameTest::Name(name) => {
                    self.by_name.insert(name.value.clone(), value);
                }
                _ => {
                    todo!();
                }
            },
            pattern::NodeTest::KindTest(_kind_test) => {
                todo!();
            }
        }
    }

    pub(crate) fn lookup(&self, item: Item, xot: &Xot) -> Option<&V> {
        match item {
            Item::Node(node) => match node {
                xml::Node::Xot(node) => {
                    if let Some(element) = xot.element(node) {
                        self.by_name
                            .get(&xee_name::Name::from_xot(element.name(), xot))
                    } else {
                        None
                    }
                }
                _ => {
                    todo!();
                }
            },
            _ => {
                todo!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xee_name::{Namespaces, VariableNames};
    use xot::Xot;

    use crate::xml;

    #[test]
    fn test_lookup_by_name() {
        let mut xot = Xot::new();
        let name = xot.add_name("foo");
        let node = xot.new_element(name);
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let mut lookup = PatternLookup::new();
        let namespaces = Namespaces::default();
        let variable_names = VariableNames::default();
        let pattern = pattern::Pattern::parse("foo", &namespaces, &variable_names).unwrap();
        lookup.add(&pattern, 1);
        let found = lookup.lookup(item, &xot).unwrap();
        assert_eq!(*found, 1);
    }
}
