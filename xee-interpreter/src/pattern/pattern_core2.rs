use xot::Xot;

use xee_xpath_ast::pattern;

use crate::{sequence::Item, xml};

struct Patterns<V> {
    patterns: Vec<(pattern::Pattern, V)>,
}

enum Backwards {
    NotFound,
    One,
    Any,
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
        let mut backwards = Backwards::One;
        for step in steps.iter().rev() {
            match backwards {
                Backwards::NotFound => return false,
                Backwards::One => {
                    if let Some(n) = node {
                        backwards = self.matches_step_expr(step, n, xot);
                        node = n.parent(xot);
                    } else {
                        return false;
                    }
                }
                Backwards::Any => loop {
                    if let Some(n) = node {
                        let new_backwards = self.matches_step_expr(step, n, xot);
                        match new_backwards {
                            Backwards::NotFound => {
                                // this parent wasn't it, so go up one more
                                node = n.parent(xot);
                            }
                            // we did find it
                            _ => {
                                backwards = new_backwards;
                                node = n.parent(xot);
                                break;
                            }
                        }
                    } else {
                        return false;
                    }
                },
            }
        }
        !matches!(backwards, Backwards::NotFound)
    }

    fn matches_step_expr(&self, step: &pattern::StepExpr, node: xml::Node, xot: &Xot) -> Backwards {
        match step {
            pattern::StepExpr::AxisStep(axis_step) => self.matches_axis_step(axis_step, node, xot),
            pattern::StepExpr::PostfixExpr(_) => todo!(),
        }
    }

    fn matches_axis_step(&self, step: &pattern::AxisStep, node: xml::Node, xot: &Xot) -> Backwards {
        if !step.predicates.is_empty() {
            todo!();
        }
        match &step.forward {
            pattern::ForwardAxis::Child => match node {
                xml::Node::Xot(_) => {
                    if self.matches_node_test(&step.node_test, node, xot) {
                        Backwards::One
                    } else {
                        Backwards::NotFound
                    }
                }
                xml::Node::Attribute(_, _) => Backwards::NotFound,
                xml::Node::Namespace(_, _) => Backwards::NotFound,
            },
            pattern::ForwardAxis::Descendant => match node {
                xml::Node::Xot(_) => {
                    if self.matches_node_test(&step.node_test, node, xot) {
                        Backwards::Any
                    } else {
                        Backwards::NotFound
                    }
                }
                xml::Node::Attribute(_, _) => Backwards::NotFound,
                xml::Node::Namespace(_, _) => Backwards::NotFound,
            },
            pattern::ForwardAxis::Attribute => match node {
                xml::Node::Attribute(_, _) => {
                    if self.matches_node_test(&step.node_test, node, xot) {
                        Backwards::One
                    } else {
                        Backwards::NotFound
                    }
                }
                _ => Backwards::NotFound,
            },
            pattern::ForwardAxis::Self_ => todo!(),
            pattern::ForwardAxis::DescendantOrSelf => todo!(),
            pattern::ForwardAxis::Namespace => todo!(),
        }
    }

    fn matches_node_test(&self, node_test: &pattern::NodeTest, node: xml::Node, xot: &Xot) -> bool {
        match node_test {
            pattern::NodeTest::NameTest(name_test) => self.matches_name_test(name_test, node, xot),
            pattern::NodeTest::KindTest(_kind_test) => todo!(),
        }
    }

    fn matches_name_test(&self, name_test: &pattern::NameTest, node: xml::Node, xot: &Xot) -> bool {
        match name_test {
            pattern::NameTest::Name(expected_name) => {
                if let Some(name) = node.node_name(xot) {
                    name == expected_name.value
                } else {
                    false
                }
            }
            pattern::NameTest::Star => true,
            pattern::NameTest::LocalName(_) => todo!(),
            pattern::NameTest::Namespace(_) => todo!(),
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
    fn test_match_name_not_found() {
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
        let pattern = pattern::Pattern::parse("notfound", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        assert_eq!(patterns.lookup(&item, &xot), None);
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

    #[test]
    fn test_match_name_nested_notfound() {
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
        let pattern =
            pattern::Pattern::parse("notfound/foo", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        assert_eq!(patterns.lookup(&item, &xot), None);
    }

    #[test]
    fn test_match_name_nested_with_explicit_descendant_axis() {
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
        let pattern =
            pattern::Pattern::parse("bar/descendant::foo", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        let found = patterns.lookup(&item, &xot).unwrap();
        assert_eq!(*found, 1);
    }

    #[test]
    fn test_match_name_nested_with_explicit_descendant_axis_not_found() {
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
        let pattern =
            pattern::Pattern::parse("notfound/descendant::foo", &namespaces, &variable_names)
                .unwrap();
        patterns.patterns.push((pattern, 1));
        assert_eq!(patterns.lookup(&item, &xot), None);
    }

    #[test]
    fn test_match_name_nested_actually_with_explicit_descendant_axis() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<root><qux><bar><foo/></bar></qux></root>"#)
            .unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let mut patterns = Patterns {
            patterns: Vec::new(),
        };
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        let pattern =
            pattern::Pattern::parse("qux/descendant::foo", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        let found = patterns.lookup(&item, &xot).unwrap();
        assert_eq!(*found, 1);
    }

    #[test]
    fn test_match_name_nested_actually_with_explicit_descendant_axis_not_found() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<root><qux><bar><foo/></bar></qux></root>"#)
            .unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let mut patterns = Patterns {
            patterns: Vec::new(),
        };
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        let pattern =
            pattern::Pattern::parse("notfound/descendant::foo", &namespaces, &variable_names)
                .unwrap();
        patterns.patterns.push((pattern, 1));
        assert_eq!(patterns.lookup(&item, &xot), None);
    }

    #[test]
    fn test_match_name_attribute() {
        let mut xot = Xot::new();
        let bar_name = xot.add_name("bar");
        let root = xot.parse(r#"<root><foo bar="BAR"/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Attribute(node, bar_name);
        let item: Item = node.into();

        let mut patterns = Patterns {
            patterns: Vec::new(),
        };
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        let pattern = pattern::Pattern::parse("@bar", &namespaces, &variable_names).unwrap();
        patterns.patterns.push((pattern, 1));
        let found = patterns.lookup(&item, &xot).unwrap();
        assert_eq!(*found, 1);
    }
}
