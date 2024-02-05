use xot::Xot;

use xee_xpath_ast::pattern;

use crate::{sequence::Item, xml};

struct Pattern(pattern::Pattern);

struct Patterns<V> {
    patterns: Vec<(Pattern, V)>,
}

enum Backwards {
    NotFound,
    One,
    Any,
}

impl Pattern {
    fn matches(&self, item: &Item, xot: &Xot) -> bool {
        match &self.0 {
            pattern::Pattern::Expr(expr_pattern) => {
                Self::matches_expr_pattern(expr_pattern, item, xot)
            }
            pattern::Pattern::Predicate(_predicate_pattern) => todo!(),
        }
    }

    fn matches_expr_pattern(expr_pattern: &pattern::ExprPattern, item: &Item, xot: &Xot) -> bool {
        if let Item::Node(node) = item {
            match expr_pattern {
                pattern::ExprPattern::Path(path_expr) => {
                    Self::matches_path_expr(path_expr, *node, xot)
                }
                pattern::ExprPattern::BinaryExpr(_binary_expr) => todo!(),
            }
        } else {
            false
        }
    }

    fn matches_path_expr(path_expr: &pattern::PathExpr, node: xml::Node, xot: &Xot) -> bool {
        match &path_expr.root {
            pattern::PathRoot::Rooted {
                root: _,
                predicates: _,
            } => todo!(),
            pattern::PathRoot::AbsoluteSlash => todo!(),
            pattern::PathRoot::AbsoluteDoubleSlash => todo!(),
            pattern::PathRoot::Relative => {
                Self::matches_relative_steps(&path_expr.steps, node, xot)
            }
        }
    }

    fn matches_relative_steps(steps: &[pattern::StepExpr], node: xml::Node, xot: &Xot) -> bool {
        let mut node = Some(node);
        let mut backwards = Backwards::One;
        for step in steps.iter().rev() {
            match backwards {
                Backwards::NotFound => return false,
                Backwards::One => {
                    if let Some(n) = node {
                        backwards = Self::matches_step_expr(step, n, xot);
                        node = n.parent(xot);
                    } else {
                        return false;
                    }
                }
                Backwards::Any => loop {
                    if let Some(n) = node {
                        let new_backwards = Self::matches_step_expr(step, n, xot);
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

    fn matches_step_expr(step: &pattern::StepExpr, node: xml::Node, xot: &Xot) -> Backwards {
        match step {
            pattern::StepExpr::AxisStep(axis_step) => Self::matches_axis_step(axis_step, node, xot),
            pattern::StepExpr::PostfixExpr(_) => todo!(),
        }
    }

    fn matches_axis_step(step: &pattern::AxisStep, node: xml::Node, xot: &Xot) -> Backwards {
        if !step.predicates.is_empty() {
            todo!();
        }
        match &step.forward {
            pattern::ForwardAxis::Child => match node {
                xml::Node::Xot(_) => {
                    if Self::matches_node_test(&step.node_test, node, xot) {
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
                    if Self::matches_node_test(&step.node_test, node, xot) {
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
                    if Self::matches_node_test(&step.node_test, node, xot) {
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

    fn matches_node_test(node_test: &pattern::NodeTest, node: xml::Node, xot: &Xot) -> bool {
        match node_test {
            pattern::NodeTest::NameTest(name_test) => Self::matches_name_test(name_test, node, xot),
            pattern::NodeTest::KindTest(_kind_test) => todo!(),
        }
    }

    fn matches_name_test(name_test: &pattern::NameTest, node: xml::Node, xot: &Xot) -> bool {
        match name_test {
            pattern::NameTest::Name(expected_name) => {
                if let Some(name) = node.node_name(xot) {
                    name == expected_name.value
                } else {
                    false
                }
            }
            pattern::NameTest::Star => true,
            pattern::NameTest::LocalName(expected_local_name) => {
                if let Some(name) = node.node_name(xot) {
                    name.local_name() == expected_local_name
                } else {
                    false
                }
            }
            pattern::NameTest::Namespace(ns) => {
                if let Some(name) = node.node_name(xot) {
                    let namespace = name.namespace();
                    if let Some(namespace) = namespace {
                        namespace == ns
                    } else {
                        ns.is_empty()
                    }
                } else {
                    false
                }
            }
        }
    }
}

impl<V> Patterns<V> {
    pub(crate) fn lookup(&self, item: &Item, xot: &Xot) -> Option<&V> {
        for (pattern, value) in &self.patterns {
            if pattern.matches(item, xot) {
                return Some(value);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_pattern(pattern: &str) -> Pattern {
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        Pattern(pattern::Pattern::parse(pattern, &namespaces, &variable_names).unwrap())
    }

    fn parse_pattern_namespaces(pattern: &str, namespaces: &xee_name::Namespaces) -> Pattern {
        let variable_names = xee_name::VariableNames::default();
        Pattern(pattern::Pattern::parse(pattern, namespaces, &variable_names).unwrap())
    }

    #[test]
    fn test_match_name() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("foo");
        assert!(pattern.matches(&item, &xot));
    }

    #[test]
    fn test_match_star() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("*");
        assert!(pattern.matches(&item, &xot));
    }

    #[test]
    fn test_match_local_name() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<root><foo xmlns="different"/></root>"#)
            .unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("*:foo");
        assert!(pattern.matches(&item, &xot));
        let pattern = parse_pattern("foo");
        assert!(!pattern.matches(&item, &xot));
    }

    #[test]
    fn test_match_namespace_name() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<root><foo xmlns="different"/></root>"#)
            .unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let namespaces = xee_name::Namespaces::new(
            vec![("d", "different"), ("o", "other")]
                .into_iter()
                .collect(),
            None,
            None,
        );

        let pattern = parse_pattern_namespaces("d:*", &namespaces);
        assert!(pattern.matches(&item, &xot));
        let pattern = parse_pattern_namespaces("d:foo", &namespaces);
        assert!(pattern.matches(&item, &xot));
        let pattern = parse_pattern_namespaces("o:*", &namespaces);
        assert!(!pattern.matches(&item, &xot));
    }

    #[test]
    fn test_not_match_name() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("notfound");
        assert!(!pattern.matches(&item, &xot));
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

        let pattern = parse_pattern("bar/foo");
        assert!(pattern.matches(&item, &xot));
    }

    #[test]
    fn test_not_match_name_nested() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><bar><foo/></bar></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("notfound/foo");
        assert!(!pattern.matches(&item, &xot));
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

        let pattern = parse_pattern("bar/descendant::foo");
        assert!(pattern.matches(&item, &xot));
    }

    #[test]
    fn test_not_match_name_nested_with_explicit_descendant_axis() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><bar><foo/></bar></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xot.first_child(node).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("notfound/descendant::foo");
        assert!(!pattern.matches(&item, &xot));
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

        let pattern = parse_pattern("qux/descendant::foo");
        assert!(pattern.matches(&item, &xot));
    }

    #[test]
    fn test_not_match_name_nested_actually_with_explicit_descendant_axis() {
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

        let pattern = parse_pattern("notfound/descendant::foo");
        assert!(!pattern.matches(&item, &xot));
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

        let pattern = parse_pattern("@bar");
        assert!(pattern.matches(&item, &xot));
    }

    #[test]
    fn test_not_match_name_attribute() {
        let mut xot = Xot::new();
        let bar_name = xot.add_name("bar");
        let root = xot.parse(r#"<root><foo bar="BAR"/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Attribute(node, bar_name);
        let item: Item = node.into();

        let pattern = parse_pattern("@qux");
        assert!(!pattern.matches(&item, &xot));
    }

    #[test]
    fn test_not_match_name_attribute_because_its_element() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><bar /></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pattern = parse_pattern("@bar");
        assert!(!pattern.matches(&item, &xot));
    }
}
