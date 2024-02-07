use xee_xpath_type::ast::KindTest;
use xot::Xot;

use xee_xpath_ast::{ast, pattern};

use crate::function::InlineFunctionId;
use crate::sequence::{Item, Sequence};
use crate::xml;

// struct Patterns<V> {
//     patterns: Vec<(Pattern, V)>,
// }

enum NodeMatch {
    Match(Option<xml::Node>),
    NotMatch,
}

trait PredicateMatcher {
    fn match_predicate(&self, inline_function_id: InlineFunctionId, sequence: &Sequence) -> bool;
    fn xot(&self) -> &Xot;

    fn matches(&self, pattern: &pattern::Pattern<InlineFunctionId>, item: &Item) -> bool {
        match pattern {
            pattern::Pattern::Expr(expr_pattern) => self.matches_expr_pattern(expr_pattern, item),
            pattern::Pattern::Predicate(_predicate_pattern) => todo!(),
        }
    }

    fn matches_expr_pattern(
        &self,
        expr_pattern: &pattern::ExprPattern<InlineFunctionId>,
        item: &Item,
    ) -> bool {
        if let Item::Node(node) = item {
            match expr_pattern {
                pattern::ExprPattern::Path(path_expr) => self.matches_path_expr(path_expr, *node),
                pattern::ExprPattern::BinaryExpr(_binary_expr) => todo!(),
            }
        } else {
            false
        }
    }

    fn matches_path_expr(
        &self,
        path_expr: &pattern::PathExpr<InlineFunctionId>,
        node: xml::Node,
    ) -> bool {
        match &path_expr.root {
            pattern::PathRoot::Rooted {
                root: _,
                predicates: _,
            } => todo!(),
            pattern::PathRoot::AbsoluteSlash => self.matches_absolute_steps(&path_expr.steps, node),
            pattern::PathRoot::AbsoluteDoubleSlash => {
                self.matches_absolute_double_slash_steps(&path_expr.steps, node)
            }
            pattern::PathRoot::Relative => {
                match self.matches_relative_steps(&path_expr.steps, node) {
                    NodeMatch::Match(_) => true,
                    NodeMatch::NotMatch => false,
                }
            }
        }
    }

    fn matches_absolute_steps(
        &self,
        steps: &[pattern::StepExpr<InlineFunctionId>],
        node: xml::Node,
    ) -> bool {
        let node_match = self.matches_relative_steps(steps, node);
        if let NodeMatch::Match(Some(xml::Node::Xot(node))) = node_match {
            self.xot().is_root(node)
        } else {
            false
        }
    }

    fn matches_absolute_double_slash_steps(
        &self,
        steps: &[pattern::StepExpr<InlineFunctionId>],
        node: xml::Node,
    ) -> bool {
        let node_match = self.matches_relative_steps(steps, node);
        if let NodeMatch::Match(Some(xml::Node::Xot(node))) = node_match {
            // we need to be under root
            let mut current_node = node;
            loop {
                if self.xot().is_root(current_node) {
                    return true;
                }
                if let Some(parent) = self.xot().parent(current_node) {
                    current_node = parent;
                } else {
                    return false;
                }
            }
        } else {
            false
        }
    }

    fn matches_relative_steps(
        &self,
        steps: &[pattern::StepExpr<InlineFunctionId>],
        node: xml::Node,
    ) -> NodeMatch {
        let mut node = Some(node);
        let mut axis = pattern::ForwardAxis::Child;
        for step in steps.iter().rev() {
            loop {
                if let Some(n) = node {
                    let (matches, new_axis) = self.matches_step_expr(step, n);
                    match axis {
                        pattern::ForwardAxis::Descendant => {
                            if !matches {
                                node = n.parent(self.xot());
                                continue;
                            }
                            axis = new_axis;
                            break;
                        }
                        _ => {
                            if !matches {
                                return NodeMatch::NotMatch;
                            }
                            axis = new_axis;
                            break;
                        }
                    }
                } else {
                    return NodeMatch::NotMatch;
                }
            }
            if let Some(n) = node {
                node = n.parent(self.xot());
            } else {
                return NodeMatch::NotMatch;
            }
        }
        NodeMatch::Match(node)
    }

    fn matches_step_expr(
        &self,
        step: &pattern::StepExpr<InlineFunctionId>,
        node: xml::Node,
    ) -> (bool, pattern::ForwardAxis) {
        match step {
            pattern::StepExpr::AxisStep(axis_step) => self.matches_axis_step(axis_step, node),
            pattern::StepExpr::PostfixExpr(_) => todo!(),
        }
    }

    fn matches_axis_step(
        &self,
        step: &pattern::AxisStep<InlineFunctionId>,
        node: xml::Node,
    ) -> (bool, pattern::ForwardAxis) {
        if !step.predicates.is_empty() {
            todo!();
        }
        // if the forward axis is attribute based, we won't match with an element,
        // and vice versa
        match node {
            xml::Node::Attribute(_, _) => {
                if step.forward != pattern::ForwardAxis::Attribute {
                    return (false, step.forward);
                }
            }
            _ => {
                if step.forward == pattern::ForwardAxis::Attribute {
                    return (false, step.forward);
                }
            }
        }
        (
            Self::matches_node_test(&step.node_test, node, self.xot()),
            step.forward,
        )
    }

    fn matches_node_test(node_test: &pattern::NodeTest, node: xml::Node, xot: &Xot) -> bool {
        match node_test {
            pattern::NodeTest::NameTest(name_test) => Self::matches_name_test(name_test, node, xot),
            pattern::NodeTest::KindTest(kind_test) => Self::matches_kind_test(kind_test, node, xot),
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

    fn matches_kind_test(kind_test: &KindTest, node: xml::Node, xot: &Xot) -> bool {
        xml::kind_test(kind_test, xot, node)
    }
}

// impl Pattern {
//     fn matches(&self, item: &Item, predicate_matcher: &impl PredicateMatcher) -> bool {
//         match &self.0 {
//             pattern::Pattern::Expr(expr_pattern) => {
//                 Self::matches_expr_pattern(expr_pattern, item, predicate_matcher)
//             }
//             pattern::Pattern::Predicate(_predicate_pattern) => todo!(),
//         }
//     }

//     fn matches_expr_pattern(
//         expr_pattern: &pattern::ExprPattern<ast::ExprS>,
//         item: &Item,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> bool {
//         if let Item::Node(node) = item {
//             match expr_pattern {
//                 pattern::ExprPattern::Path(path_expr) => {
//                     Self::matches_path_expr(path_expr, *node, predicate_matcher)
//                 }
//                 pattern::ExprPattern::BinaryExpr(_binary_expr) => todo!(),
//             }
//         } else {
//             false
//         }
//     }

//     fn matches_path_expr(
//         path_expr: &pattern::PathExpr<ast::ExprS>,
//         node: xml::Node,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> bool {
//         match &path_expr.root {
//             pattern::PathRoot::Rooted {
//                 root: _,
//                 predicates: _,
//             } => todo!(),
//             pattern::PathRoot::AbsoluteSlash => {
//                 Self::matches_absolute_steps(&path_expr.steps, node, predicate_matcher)
//             }
//             pattern::PathRoot::AbsoluteDoubleSlash => {
//                 Self::matches_absolute_double_slash_steps(&path_expr.steps, node, predicate_matcher)
//             }
//             pattern::PathRoot::Relative => {
//                 match Self::matches_relative_steps(&path_expr.steps, node, predicate_matcher) {
//                     NodeMatch::Match(_) => true,
//                     NodeMatch::NotMatch => false,
//                 }
//             }
//         }
//     }

//     fn matches_absolute_steps(
//         steps: &[pattern::StepExpr<ast::ExprS>],
//         node: xml::Node,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> bool {
//         let node_match = Self::matches_relative_steps(steps, node, predicate_matcher);
//         if let NodeMatch::Match(Some(xml::Node::Xot(node))) = node_match {
//             predicate_matcher.xot().is_root(node)
//         } else {
//             false
//         }
//     }

//     fn matches_absolute_double_slash_steps(
//         steps: &[pattern::StepExpr<ast::ExprS>],
//         node: xml::Node,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> bool {
//         let node_match = Self::matches_relative_steps(steps, node, predicate_matcher);
//         if let NodeMatch::Match(Some(xml::Node::Xot(node))) = node_match {
//             // we need to be under root
//             let mut current_node = node;
//             loop {
//                 if predicate_matcher.xot().is_root(current_node) {
//                     return true;
//                 }
//                 if let Some(parent) = predicate_matcher.xot().parent(current_node) {
//                     current_node = parent;
//                 } else {
//                     return false;
//                 }
//             }
//         } else {
//             false
//         }
//     }

//     fn matches_relative_steps(
//         steps: &[pattern::StepExpr<ast::ExprS>],
//         node: xml::Node,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> NodeMatch {
//         let mut node = Some(node);
//         let mut axis = pattern::ForwardAxis::Child;
//         for step in steps.iter().rev() {
//             loop {
//                 if let Some(n) = node {
//                     let (matches, new_axis) = Self::matches_step_expr(step, n, predicate_matcher);
//                     match axis {
//                         pattern::ForwardAxis::Descendant => {
//                             if !matches {
//                                 node = n.parent(predicate_matcher.xot());
//                                 continue;
//                             }
//                             axis = new_axis;
//                             break;
//                         }
//                         _ => {
//                             if !matches {
//                                 return NodeMatch::NotMatch;
//                             }
//                             axis = new_axis;
//                             break;
//                         }
//                     }
//                 } else {
//                     return NodeMatch::NotMatch;
//                 }
//             }
//             if let Some(n) = node {
//                 node = n.parent(predicate_matcher.xot());
//             } else {
//                 return NodeMatch::NotMatch;
//             }
//         }
//         NodeMatch::Match(node)
//     }

//     fn matches_step_expr(
//         step: &pattern::StepExpr<ast::ExprS>,
//         node: xml::Node,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> (bool, pattern::ForwardAxis) {
//         match step {
//             pattern::StepExpr::AxisStep(axis_step) => {
//                 Self::matches_axis_step(axis_step, node, predicate_matcher)
//             }
//             pattern::StepExpr::PostfixExpr(_) => todo!(),
//         }
//     }

//     fn matches_axis_step(
//         step: &pattern::AxisStep<ast::ExprS>,
//         node: xml::Node,
//         predicate_matcher: &impl PredicateMatcher,
//     ) -> (bool, pattern::ForwardAxis) {
//         if !step.predicates.is_empty() {
//             todo!();
//         }
//         // if the forward axis is attribute based, we won't match with an element,
//         // and vice versa
//         match node {
//             xml::Node::Attribute(_, _) => {
//                 if step.forward != pattern::ForwardAxis::Attribute {
//                     return (false, step.forward);
//                 }
//             }
//             _ => {
//                 if step.forward == pattern::ForwardAxis::Attribute {
//                     return (false, step.forward);
//                 }
//             }
//         }
//         (
//             Self::matches_node_test(&step.node_test, node, predicate_matcher.xot()),
//             step.forward,
//         )
//     }

//     fn matches_node_test(node_test: &pattern::NodeTest, node: xml::Node, xot: &Xot) -> bool {
//         match node_test {
//             pattern::NodeTest::NameTest(name_test) => Self::matches_name_test(name_test, node, xot),
//             pattern::NodeTest::KindTest(kind_test) => Self::matches_kind_test(kind_test, node, xot),
//         }
//     }

//     fn matches_name_test(name_test: &pattern::NameTest, node: xml::Node, xot: &Xot) -> bool {
//         match name_test {
//             pattern::NameTest::Name(expected_name) => {
//                 if let Some(name) = node.node_name(xot) {
//                     name == expected_name.value
//                 } else {
//                     false
//                 }
//             }
//             pattern::NameTest::Star => true,
//             pattern::NameTest::LocalName(expected_local_name) => {
//                 if let Some(name) = node.node_name(xot) {
//                     name.local_name() == expected_local_name
//                 } else {
//                     false
//                 }
//             }
//             pattern::NameTest::Namespace(ns) => {
//                 if let Some(name) = node.node_name(xot) {
//                     let namespace = name.namespace();
//                     if let Some(namespace) = namespace {
//                         namespace == ns
//                     } else {
//                         ns.is_empty()
//                     }
//                 } else {
//                     false
//                 }
//             }
//         }
//     }

//     fn matches_kind_test(kind_test: &KindTest, node: xml::Node, xot: &Xot) -> bool {
//         xml::kind_test(kind_test, xot, node)
//     }
// }

// impl<V> Patterns<V> {
//     pub(crate) fn lookup(&self, item: &Item, xot: &Xot) -> Option<&V> {
//         for (pattern, value) in &self.patterns {
//             if pattern.matches(item, predicate_matcher) {
//                 return Some(value);
//             }
//         }
//         None
//     }
// }

#[cfg(test)]
mod tests {
    use std::prelude;

    use xee_xpath_ast::pattern::transform_pattern;

    use super::*;

    fn parse_pattern(pattern: &str) -> pattern::Pattern<InlineFunctionId> {
        let namespaces = xee_name::Namespaces::default();
        let variable_names = xee_name::VariableNames::default();
        parse_pattern_namespaces(pattern, &namespaces)
    }

    fn parse_pattern_namespaces(
        pattern: &str,
        namespaces: &xee_name::Namespaces,
    ) -> pattern::Pattern<InlineFunctionId> {
        let variable_names = xee_name::VariableNames::default();
        let pattern = pattern::Pattern::parse(pattern, namespaces, &variable_names).unwrap();

        transform_pattern(&pattern, |expr| dummy_inline_function_id()).unwrap()
    }

    fn dummy_inline_function_id() -> Result<InlineFunctionId, ()> {
        Ok(InlineFunctionId::new(0))
    }

    struct BasicPredicateMatcher<'a> {
        xot: &'a Xot,
    }

    impl<'a> BasicPredicateMatcher<'a> {
        fn new(xot: &'a Xot) -> Self {
            Self { xot }
        }
    }

    impl<'a> PredicateMatcher for BasicPredicateMatcher<'a> {
        fn match_predicate(
            &self,
            _inline_function_id: InlineFunctionId,
            _sequence: &Sequence,
        ) -> bool {
            false
        }

        fn xot(&self) -> &Xot {
            self.xot
        }
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

        let pm = BasicPredicateMatcher::new(&xot);
        assert!(pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        assert!(pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("*:foo");
        assert!(pm.matches(&pattern, &item));
        let pattern = parse_pattern("foo");
        assert!(!pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern_namespaces("d:*", &namespaces);
        assert!(pm.matches(&pattern, &item));
        let pattern = parse_pattern_namespaces("d:foo", &namespaces);
        assert!(pm.matches(&pattern, &item));
        let pattern = parse_pattern_namespaces("o:*", &namespaces);
        assert!(!pm.matches(&pattern, &item));
    }

    #[test]
    fn test_not_match_name() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("notfound");
        assert!(!pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("bar/foo");
        assert!(pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("notfound/foo");
        assert!(!pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("bar/descendant::foo");
        assert!(pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("notfound/descendant::foo");
        assert!(!pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("qux/descendant::foo");
        assert!(pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("notfound/descendant::foo");
        assert!(!pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("@bar");
        assert!(pm.matches(&pattern, &item));
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

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("@qux");
        assert!(!pm.matches(&pattern, &item));
    }

    #[test]
    fn test_not_match_name_element_because_its_attribute() {
        let mut xot = Xot::new();
        let bar_name = xot.add_name("bar");
        let root = xot.parse(r#"<root><foo bar="BAR"/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Attribute(node, bar_name);
        let item: Item = node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("bar");
        assert!(!pm.matches(&pattern, &item));
    }

    #[test]
    fn test_not_match_name_element_descendant_because_its_attribute() {
        let mut xot = Xot::new();
        let bar_name = xot.add_name("bar");
        let root = xot.parse(r#"<root><foo bar="BAR"/></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Attribute(node, bar_name);
        let item: Item = node.into();

        let pm = BasicPredicateMatcher::new(&xot);

        let pattern = parse_pattern("root//bar");
        assert!(!pm.matches(&pattern, &item));
    }

    #[test]
    fn test_not_match_name_attribute_because_its_element() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><bar /></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let node = xml::Node::Xot(node);
        let item: Item = node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("@bar");
        assert!(!pm.matches(&pattern, &item));
    }

    #[test]
    fn test_matches_kind_test_any() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo bar="BAR" /></root>"#).unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let element_node = xml::Node::Xot(node);
        let element_item: Item = element_node.into();
        let attribute_name = xot.add_name("bar");
        let attribute_node = xml::Node::Attribute(node, attribute_name);
        let attribute_item: Item = attribute_node.into();

        // the axis determines whether we match. This is a bit
        // counter intuitive, but the spec affirms it.

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("node()");
        assert!(pm.matches(&pattern, &element_item));
        assert!(!pm.matches(&pattern, &attribute_item));
        let pattern = parse_pattern("attribute::node()");
        assert!(!pm.matches(&pattern, &element_item));
        assert!(pm.matches(&pattern, &attribute_item));
    }

    #[test]
    fn test_matches_kind_test_element() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<root><foo bar="BAR">text</foo></root>"#)
            .unwrap();
        let document_element = xot.document_element(root).unwrap();
        let node = xot.first_child(document_element).unwrap();
        let element_node = xml::Node::Xot(node);
        let element_item: Item = element_node.into();
        let attribute_name = xot.add_name("bar");
        let attribute_node = xml::Node::Attribute(node, attribute_name);
        let attribute_item: Item = attribute_node.into();
        let text_node = xot.first_child(node).unwrap();
        let text_node = xml::Node::Xot(text_node);
        let text_item: Item = text_node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("element()");
        assert!(pm.matches(&pattern, &element_item));
        assert!(!pm.matches(&pattern, &text_item));
        assert!(!pm.matches(&pattern, &attribute_item));
    }

    #[test]
    fn test_matches_absolute_slash() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root/>"#).unwrap();
        let node = xml::Node::Xot(root);
        let document_element = xot.document_element(root).unwrap();
        let document_element_node = xml::Node::Xot(document_element);
        let item: Item = node.into();
        let document_element_item: Item = document_element_node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("/");
        assert!(pm.matches(&pattern, &item));
        assert!(!pm.matches(&pattern, &document_element_item));
    }

    #[test]
    fn test_matches_absolute_slash_with_element() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root/>"#).unwrap();
        let node = xml::Node::Xot(root);
        let document_element = xot.document_element(root).unwrap();
        let document_element_node = xml::Node::Xot(document_element);
        let item: Item = node.into();
        let document_element_item: Item = document_element_node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("/root");
        assert!(!pm.matches(&pattern, &item));
        assert!(pm.matches(&pattern, &document_element_item));
    }

    #[test]
    fn test_matches_absolute_double_slash() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root/>"#).unwrap();
        let node = xml::Node::Xot(root);
        let document_element = xot.document_element(root).unwrap();
        let document_element_node = xml::Node::Xot(document_element);
        let item: Item = node.into();
        let document_element_item: Item = document_element_node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("//root");
        assert!(!pm.matches(&pattern, &item));
        assert!(pm.matches(&pattern, &document_element_item));
    }

    #[test]
    fn test_matches_absolute_double_slash_nesting() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<root><foo/></root>"#).unwrap();
        let node = xml::Node::Xot(root);
        let document_element = xot.document_element(root).unwrap();
        let document_element_node = xml::Node::Xot(document_element);
        let item: Item = node.into();
        let document_element_item: Item = document_element_node.into();
        let foo_node = xot.first_child(document_element).unwrap();
        let foo_node = xml::Node::Xot(foo_node);
        let foo_item: Item = foo_node.into();

        let pm = BasicPredicateMatcher::new(&xot);
        let pattern = parse_pattern("//root/foo");
        assert!(!pm.matches(&pattern, &item));
        assert!(!pm.matches(&pattern, &document_element_item));
        assert!(pm.matches(&pattern, &foo_item));
    }
}
