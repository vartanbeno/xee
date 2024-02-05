use xee_name::Name;
use xot::Xot;

use xee_xpath_ast::pattern;

use crate::{sequence::Item, xml};

struct Patterns<V> {
    patterns: Vec<(pattern::Pattern, V)>,
}

impl<V> Patterns<V> {
    pub(crate) fn lookup(&self, item: &Item, xot: &Xot) -> Option<&V> {
        for (pattern, value) in &self.patterns {
            if self.matches(pattern, item, xot) {
                return Some(value);
            }
        }
        None
    }

    fn matches(&self, pattern: &pattern::Pattern, item: &Item, xot: &Xot) -> bool {
        match pattern {
            pattern::Pattern::Expr(expr_pattern) => {
                self.matches_expr_pattern(expr_pattern, item, xot)
            }
            pattern::Pattern::Predicate(_predicate_pattern) => todo!(),
        }
    }

    fn matches_expr_pattern(
        &self,
        expr_pattern: &pattern::ExprPattern,
        item: &Item,
        xot: &Xot,
    ) -> bool {
        if let Item::Node(node) = item {
            match expr_pattern {
                pattern::ExprPattern::Path(path_expr) => {
                    self.matches_path_expr(path_expr, *node, xot)
                }
                pattern::ExprPattern::BinaryExpr(_binary_expr) => todo!(),
            }
        } else {
            false
        }
    }

    fn matches_path_expr(&self, path_expr: &pattern::PathExpr, node: xml::Node, xot: &Xot) -> bool {
        match &path_expr.root {
            pattern::PathRoot::Rooted {
                root: _,
                predicates: _,
            } => todo!(),
            pattern::PathRoot::AbsoluteSlash => todo!(),
            pattern::PathRoot::AbsoluteDoubleSlash => todo!(),
            pattern::PathRoot::Relative => self.matches_relative_steps(&path_expr.steps, node, xot),
        }
    }

    fn matches_relative_steps(
        &self,
        steps: &[pattern::StepExpr],
        node: xml::Node,
        xot: &Xot,
    ) -> bool {
        let mut node = Some(node);
        for step in steps.iter().rev() {
            if let Some(n) = node {
                if !self.matches_step_expr(step, n, xot) {
                    return false;
                }
                node = n.parent(xot);
            } else {
                return false;
            }
        }
        true
    }

    fn matches_step_expr(&self, step: &pattern::StepExpr, node: xml::Node, xot: &Xot) -> bool {
        match node {
            xml::Node::Xot(node) => match step {
                pattern::StepExpr::AxisStep(axis_step) => {
                    self.matches_axis_step(axis_step, node, xot)
                }
                pattern::StepExpr::PostfixExpr(_) => todo!(),
            },
            xml::Node::Attribute(_, _) => todo!(),
            xml::Node::Namespace(_, _) => todo!(),
        }
    }

    fn matches_axis_step(&self, step: &pattern::AxisStep, node: xot::Node, xot: &Xot) -> bool {
        if step.forward != pattern::ForwardAxis::Child {
            todo!();
        }
        if !step.predicates.is_empty() {
            todo!();
        }
        self.matches_node_test(&step.node_test, node, xot)
    }

    fn matches_node_test(&self, node_test: &pattern::NodeTest, node: xot::Node, xot: &Xot) -> bool {
        match node_test {
            pattern::NodeTest::NameTest(name_test) => self.matches_name_test(name_test, node, xot),
            pattern::NodeTest::KindTest(_kind_test) => todo!(),
        }
    }

    fn matches_name_test(&self, name_test: &pattern::NameTest, node: xot::Node, xot: &Xot) -> bool {
        if let Some(element) = xot.element(node) {
            match name_test {
                pattern::NameTest::Name(name) => Name::from_xot(element.name(), xot) == name.value,
                pattern::NameTest::Star => true,
                pattern::NameTest::LocalName(_) => todo!(),
                pattern::NameTest::Namespace(_) => todo!(),
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_name() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let mut patterns = Patterns {
            patterns: Vec::new(),
        };
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        let pattern = pattern::Pattern::parse("foo", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        let found = patterns.lookup(&item, &xot).unwrap();
        assert_eq!(*found, 1);
    }

    #[test]
    fn test_match_name_nested() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><bar><foo/></bar></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let mut patterns = Patterns {
            patterns: Vec::new(),
        };
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        let pattern = pattern::Pattern::parse("bar/foo", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        let found = patterns.lookup(&item, &xot).unwrap();
        assert_eq!(*found, 1);
    }
}
