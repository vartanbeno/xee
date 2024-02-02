// an implementation of https://www.w3.org/TR/xslt-30/#default-priority

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use xee_xpath_ast::{ast, pattern};

pub(crate) fn default_priority(pattern: &pattern::Pattern) -> Decimal {
    match pattern {
        pattern::Pattern::Predicate(predicate) => {
            if !predicate.predicates.is_empty() {
                dec!(1)
            } else {
                dec!(-1)
            }
        }
        pattern::Pattern::Expr(expr) => match expr {
            pattern::ExprPattern::Path(path) => match path.root {
                pattern::PathRoot::AbsoluteSlash => {
                    if path.steps.is_empty() {
                        dec!(-0.5)
                    } else {
                        todo!();
                    }
                }
                pattern::PathRoot::Relative => {
                    if path.steps.is_empty() {
                        todo!()
                    } else if path.steps.len() > 1 {
                        todo!()
                    } else {
                        let step = &path.steps[0];
                        match step {
                            pattern::StepExpr::AxisStep(axis_step) => {
                                if axis_step.predicates.is_empty() {
                                    match &axis_step.node_test {
                                        pattern::NodeTest::NameTest(name_test) => match name_test {
                                            pattern::NameTest::Name(_) => dec!(0),
                                            _ => todo!(),
                                        },
                                        pattern::NodeTest::KindTest(kind_test) => match kind_test {
                                            ast::KindTest::Element(test)
                                            | ast::KindTest::Attribute(test) => {
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
                                                    todo!()
                                                }
                                            }
                                            ast::KindTest::SchemaAttribute(_) => dec!(0.25),
                                            ast::KindTest::SchemaElement(_) => dec!(0.25),
                                            _ => todo!(),
                                        },
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            pattern::StepExpr::PostfixExpr(_) => todo!(),
                        }
                    }
                }
                _ => todo!(),
            },
            pattern::ExprPattern::BinaryExpr(_) => todo!(),
        },
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

    #[test]
    fn test_4_predicate_pattern_non_empty_predicate_list() {
        let pattern = parse(".[1]");

        assert_eq!(default_priority(&pattern), dec!(1));
    }

    #[test]
    fn test_4_predicate_pattern_empty_predicate_list() {
        let pattern = parse(".");

        assert_eq!(default_priority(&pattern), dec!(-1));
    }

    #[test]
    fn test_5_path_root() {
        let pattern = parse("/");

        assert_eq!(default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_6_eqname() {
        let pattern = parse("foo");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_6_eqname_with_forward_axis() {
        let pattern = parse("descendant::foo");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_6_processing_instruction_string_literal() {
        let pattern = parse("processing-instruction('foo')");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_6_processing_instruction_name() {
        let pattern = parse("processing-instruction(foo)");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_element_test() {
        let pattern = parse("element()");

        assert_eq!(default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_element_test_star() {
        let pattern = parse("element(*)");

        assert_eq!(default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_attribute_test() {
        let pattern = parse("attribute()");

        assert_eq!(default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_attribute_test_start() {
        let pattern = parse("attribute(*)");

        assert_eq!(default_priority(&pattern), dec!(-0.5));
    }

    #[test]
    fn test_7_element_test_name() {
        let pattern = parse("element(foo)");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_element_test_type() {
        let pattern = parse("element(*, xs:integer)");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_attribute_test_name() {
        let pattern = parse("attribute(foo)");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_attribute_test_type() {
        let pattern = parse("attribute(*, xs:integer)");

        assert_eq!(default_priority(&pattern), dec!(0));
    }

    #[test]
    fn test_7_element_test_name_type() {
        let pattern = parse("element(foo, xs:integer)");

        assert_eq!(default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_7_schema_element_test() {
        let pattern = parse("schema-element(foo)");

        assert_eq!(default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_7_attribute_test_name_type() {
        let pattern = parse("attribute(foo, xs:integer)");

        assert_eq!(default_priority(&pattern), dec!(0.25));
    }

    #[test]
    fn test_7_schema_attribute_test() {
        let pattern = parse("schema-attribute(foo)");

        assert_eq!(default_priority(&pattern), dec!(0.25));
    }

    // #[test]
    // fn test_processing_instruction_without_arguments() {
    //     let pattern = parse("processing-instruction()");

    //     assert_eq!(default_priority(&pattern), dec!(0));
    // }
}
