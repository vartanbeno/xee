// an implementation of https://www.w3.org/TR/xslt-30/#default-priority

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use xee_xpath_ast::{ast, pattern};

pub(crate) fn default_priority<'a>(
    pattern: &'a pattern::Pattern,
) -> Box<dyn Iterator<Item = (&'a pattern::Pattern, Decimal)> + 'a> {
    match pattern {
        pattern::Pattern::Predicate(predicate) => {
            if !predicate.predicates.is_empty() {
                Box::new(std::iter::once((pattern, dec!(1))))
            } else {
                Box::new(std::iter::once((pattern, dec!(-1))))
            }
        }
        pattern::Pattern::Expr(expr) => {
            Box::new(default_priority_expr_pattern(expr).map(move |priority| (pattern, priority)))
        }
    }
}

fn default_priority_expr_pattern(
    expr_pattern: &pattern::ExprPattern,
) -> Box<dyn Iterator<Item = Decimal>> {
    match expr_pattern {
        pattern::ExprPattern::Path(path) => {
            Box::new(std::iter::once(default_priority_path_expr(path)))
        }
        _ => todo!(),
    }
}

fn default_priority_path_expr(path: &pattern::PathExpr) -> Decimal {
    let default = dec!(0.5);

    match path.root {
        pattern::PathRoot::AbsoluteSlash => {
            if path.steps.is_empty() {
                dec!(-0.5)
            } else {
                default
            }
        }
        pattern::PathRoot::Relative => {
            if path.steps.is_empty() || path.steps.len() > 1 {
                default
            } else {
                let step = &path.steps[0];
                match step {
                    pattern::StepExpr::AxisStep(axis_step) => {
                        if axis_step.predicates.is_empty() {
                            match &axis_step.node_test {
                                pattern::NodeTest::NameTest(name_test) => match name_test {
                                    pattern::NameTest::Name(_) => dec!(0),
                                    pattern::NameTest::LocalName(_)
                                    | pattern::NameTest::Namespace(_) => dec!(-0.25),
                                    pattern::NameTest::Star => dec!(-0.5),
                                },
                                pattern::NodeTest::KindTest(kind_test) => {
                                    default_priority_kind_test(kind_test)
                                }
                            }
                        } else {
                            default
                        }
                    }
                    pattern::StepExpr::PostfixExpr(_) => default,
                }
            }
        }
        _ => default,
    }
}

fn default_priority_kind_test(kind_test: &ast::KindTest) -> Decimal {
    match kind_test {
        ast::KindTest::Element(test) | ast::KindTest::Attribute(test) => {
            if let Some(test) = test {
                if test.type_name.is_some() {
                    match test.name_or_wildcard {
                        ast::NameOrWildcard::Name(_) => {
                            dec!(0.25)
                        }
                        ast::NameOrWildcard::Wildcard => {
                            dec!(0)
                        }
                    }
                } else {
                    match test.name_or_wildcard {
                        ast::NameOrWildcard::Name(_) => {
                            dec!(0)
                        }
                        ast::NameOrWildcard::Wildcard => {
                            dec!(-0.5)
                        }
                    }
                }
            } else {
                dec!(-0.5)
            }
        }
        ast::KindTest::PI(pi_test) => {
            if let Some(_pi_test) = pi_test {
                dec!(0)
            } else {
                dec!(-0.5)
            }
        }
        ast::KindTest::SchemaAttribute(_) => dec!(0.25),
        ast::KindTest::SchemaElement(_) => dec!(0.25),
        ast::KindTest::Document(document_test) => {
            if let Some(document_test) = document_test {
                match document_test {
                    ast::DocumentTest::Element(element_or_attribute_test) => {
                        default_priority_kind_test(&ast::KindTest::Element(
                            element_or_attribute_test.clone(),
                        ))
                    }
                    ast::DocumentTest::SchemaElement(_schema_element_test) => dec!(0.25),
                }
            } else {
                dec!(-0.5)
            }
        }
        _ => dec!(-0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xee_name::{Namespaces, VariableNames};
    use xee_xpath_ast::pattern::Pattern;

    fn parse(expr: &str) -> Pattern {
        let namespaces = Namespaces::default();
        let variable_names = VariableNames::default();
        Pattern::parse(expr, &namespaces, &variable_names).unwrap()
    }

    fn one_default_priority(pattern: &Pattern) -> Decimal {
        let v = default_priority(pattern).collect::<Vec<_>>();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, pattern);
        v[0].1
    }

    #[test]
    fn test_4_predicate_pattern_non_empty_predicate_list() {
        let pattern = parse(".[1]");

        assert_eq!(one_default_priority(&pattern), dec!(1));
    }

    #[test]
    fn test_4_predicate_pattern_empty_predicate_list() {
        let pattern = parse(".");

        assert_eq!(one_default_priority(&pattern), dec!(-1));
    }

    #[test]
    fn test_5_path_root() {
        let pattern = parse("/");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_6_eqname() {
        let pattern = parse("foo");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_6_eqname_with_forward_axis() {
        let pattern = parse("descendant::foo");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_6_processing_instruction_string_literal() {
        let pattern = parse("processing-instruction('foo')");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_6_processing_instruction_name() {
        let pattern = parse("processing-instruction(foo)");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_element_test() {
        let pattern = parse("element()");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_element_test_star() {
        let pattern = parse("element(*)");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_attribute_test() {
        let pattern = parse("attribute()");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_attribute_test_start() {
        let pattern = parse("attribute(*)");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_element_test_name() {
        let pattern = parse("element(foo)");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_element_test_type() {
        let pattern = parse("element(*, xs:integer)");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_attribute_test_name() {
        let pattern = parse("attribute(foo)");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_attribute_test_type() {
        let pattern = parse("attribute(*, xs:integer)");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_element_test_name_type() {
        let pattern = parse("element(foo, xs:integer)");

        assert_eq!(one_default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_7_schema_element_test() {
        let pattern = parse("schema-element(foo)");

        assert_eq!(one_default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_7_attribute_test_name_type() {
        let pattern = parse("attribute(foo, xs:integer)");

        assert_eq!(one_default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_7_schema_attribute_test() {
        let pattern = parse("schema-attribute(foo)");

        assert_eq!(one_default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_8_document_test() {
        let pattern = parse("document-node()");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_8_document_test_with_element_test() {
        let pattern = parse("document-node(element(foo))");

        assert_eq!(one_default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_8_document_test_with_element_test_and_type() {
        let pattern = parse("document-node(element(foo, xs:integer))");

        assert_eq!(one_default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_8_document_test_with_schema_element() {
        let pattern = parse("document-node(schema-element(foo))");

        assert_eq!(one_default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_9_ncname_star() {
        let pattern = parse("fn:*");

        assert_eq!(one_default_priority(&pattern), dec!(-0.25));
    }

    #[test]
    fn test_9_star_ncname() {
        let pattern = parse("*:foo");

        assert_eq!(one_default_priority(&pattern), dec!(-0.25));
    }

    #[test]
    fn test_10_any_other_node_test_node() {
        let pattern = parse("node()");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_10_any_other_node_test_star() {
        let pattern = parse("*");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_10_processing_instruction_without_arguments() {
        let pattern = parse("processing-instruction()");

        assert_eq!(one_default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_pattern_with_predicate() {
        let pattern = parse("foo[1]");

        assert_eq!(one_default_priority(&pattern), dec!(0.5));
    }

    #[test]
    fn test_multi_step_pattern() {
        let pattern = parse("foo/bar");

        assert_eq!(one_default_priority(&pattern), dec!(0.5));
    }

    // #[test]
    // fn test_top_level_union() {
    //     let pattern = parse("foo | bar");

    //     assert_eq!(one_default_priority(&pattern), dec!(0.5));
    // }
}
