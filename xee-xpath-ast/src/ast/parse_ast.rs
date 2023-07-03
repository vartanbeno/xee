use miette::SourceSpan;
use ordered_float::OrderedFloat;
use pest::error::InputLocation;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use rust_decimal::Decimal;

use crate::error::Error;
use crate::namespaces::{Namespaces, FN_NAMESPACE};
use crate::parser::{Rule, XPathParser};
use crate::span::{not_spanned, Spanned};

use super::{ast_core as ast, rename::unique_names};

struct AstParser<'a> {
    namespaces: &'a Namespaces<'a>,
}

enum PostfixOrPlaceholdered {
    Postfix(ast::Postfix),
    Placeholdered(Vec<ast::ExprSingleS>, Vec<ast::Param>),
}

#[derive(Debug)]
enum ArgumentsOrPlaceholdered {
    Arguments(Vec<ast::ExprSingleS>),
    Placeholdered(Vec<ast::ExprSingleS>, Vec<ast::Param>),
}

impl<'a> AstParser<'a> {
    fn new(namespaces: &'a Namespaces<'a>) -> Self {
        AstParser { namespaces }
    }

    // fn struct_wrap<T, W>(&self, pair: Pair<Rule>, outer_rule: Rule, inner_rule: Rule, wrap: W) -> T
    // where
    //     W: Fn(Pair<Rule>) -> T,
    // {
    //     debug_assert_eq!(pair.as_rule(), outer_rule);
    //     let pair = pair.into_inner().next().unwrap();
    //     if pair.as_rule() == inner_rule {
    //         wrap(pair)
    //     } else {
    //         panic!("unhandled {:?}", pair.as_rule())
    //     }
    // }

    fn pair_to_path_expr(&self, pair: Pair<Rule>) -> ast::PathExpr {
        let expr_single = self.expr_single(pair);
        let span = expr_single.span;
        match expr_single.value {
            ast::ExprSingle::Path(path_expr) => path_expr,
            _ => ast::PathExpr {
                steps: vec![Spanned::new(
                    ast::StepExpr::PrimaryExpr(Spanned::new(
                        ast::PrimaryExpr::Expr(Spanned::new(vec![expr_single], span)),
                        span,
                    )),
                    span,
                )],
            },
        }
    }

    fn xpath(&self, pair: Pair<Rule>) -> ast::XPath {
        ast::XPath {
            exprs: self.exprs(pair),
        }
    }

    fn exprs(&self, pair: Pair<Rule>) -> ast::ExprS {
        let span = pair.as_span();
        let pairs = pair.into_inner();
        spanned(
            pairs.map(|pair| self.expr_single(pair)).collect::<Vec<_>>(),
            &span,
        )
    }

    fn expr_single(&self, pair: Pair<Rule>) -> ast::ExprSingleS {
        let span = &pair.as_span();
        match pair.as_rule() {
            Rule::PathExpr => spanned(
                ast::ExprSingle::Path(self.path_expr_to_path_expr(pair)),
                span,
            ),
            Rule::SimpleMapExpr => {
                let mut pairs = pair.into_inner();
                let path_expr_pair = pairs.next().unwrap();
                let simple_map_path_exprs = pairs
                    .map(|pair| self.pair_to_path_expr(pair))
                    .collect::<Vec<_>>();
                if !simple_map_path_exprs.is_empty() {
                    let path_expr = self.pair_to_path_expr(path_expr_pair);
                    spanned(
                        ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::SimpleMap(simple_map_path_exprs),
                        }),
                        span,
                    )
                } else {
                    self.expr_single(path_expr_pair)
                }
            }
            Rule::UnaryExpr => {
                let mut plus_minus = vec![];
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::Minus => {
                            plus_minus.push(ast::UnaryOperator::Minus);
                        }
                        Rule::Plus => {
                            plus_minus.push(ast::UnaryOperator::Plus);
                        }
                        Rule::ValueExpr => {
                            if plus_minus.is_empty() {
                                return self.expr_single(pair);
                            }
                            let path_expr = self.pair_to_path_expr(pair);
                            return spanned(
                                ast::ExprSingle::Apply(ast::ApplyExpr {
                                    path_expr,
                                    operator: ast::ApplyOperator::Unary(plus_minus),
                                }),
                                span,
                            );
                        }
                        _ => {
                            panic!("unhandled unary {:?}", pair.as_rule())
                        }
                    }
                }
                unreachable!();
            }
            Rule::ArrowExpr => {
                let pair = pair.into_inner().next().unwrap();
                self.expr_single(pair)
            }
            Rule::CastExpr => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let single_type_pair = pairs.next();
                if let Some(single_type_pair) = single_type_pair {
                    let path_expr = self.pair_to_path_expr(pair);
                    let single_type = self.single_type(single_type_pair);
                    spanned(
                        ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::Cast(single_type),
                        }),
                        span,
                    )
                } else {
                    self.expr_single(pair)
                }
            }
            Rule::CastableExpr => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let single_type_pair = pairs.next();
                if let Some(single_type_pair) = single_type_pair {
                    let path_expr = self.pair_to_path_expr(pair);
                    let single_type = self.single_type(single_type_pair);
                    spanned(
                        ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::Castable(single_type),
                        }),
                        span,
                    )
                } else {
                    self.expr_single(pair)
                }
            }
            Rule::TreatExpr => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let sequence_type_pair = pairs.next();
                if let Some(sequence_type_pair) = sequence_type_pair {
                    let path_expr = self.pair_to_path_expr(pair);
                    let sequence_type = self.sequence_type(sequence_type_pair);
                    spanned(
                        ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::Treat(sequence_type),
                        }),
                        span,
                    )
                } else {
                    self.expr_single(pair)
                }
            }
            Rule::InstanceofExpr => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let sequence_type_pair = pairs.next();
                if let Some(sequence_type_pair) = sequence_type_pair {
                    let path_expr = self.pair_to_path_expr(pair);
                    let sequence_type = self.sequence_type(sequence_type_pair);
                    spanned(
                        ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::InstanceOf(sequence_type),
                        }),
                        span,
                    )
                } else {
                    self.expr_single(pair)
                }
            }
            Rule::AdditiveExpr => self.binary_op(pair, |r| match r {
                Rule::Plus => ast::BinaryOperator::Add,
                Rule::Minus => ast::BinaryOperator::Sub,
                _ => {
                    unreachable!("unknown AdditiveExpr {:?}", r)
                }
            }),
            Rule::MultiplicativeExpr => self.binary_op(pair, |r| match r {
                Rule::Mult => ast::BinaryOperator::Mul,
                Rule::Div => ast::BinaryOperator::Div,
                Rule::IDiv => ast::BinaryOperator::IntDiv,
                Rule::Mod => ast::BinaryOperator::Mod,
                _ => {
                    unreachable!("unknown MultiplicativeExpr {:?}", r)
                }
            }),
            Rule::OrExpr => self.binary(pair, ast::BinaryOperator::Or),
            Rule::AndExpr => self.binary(pair, ast::BinaryOperator::And),
            Rule::ComparisonExpr => self.binary_op(pair, |r| match r {
                Rule::ValueEq => ast::BinaryOperator::ValueEq,
                Rule::ValueNe => ast::BinaryOperator::ValueNe,
                Rule::ValueLt => ast::BinaryOperator::ValueLt,
                Rule::ValueLe => ast::BinaryOperator::ValueLe,
                Rule::ValueGt => ast::BinaryOperator::ValueGt,
                Rule::ValueGe => ast::BinaryOperator::ValueGe,
                Rule::GenEq => ast::BinaryOperator::GenEq,
                Rule::GenNe => ast::BinaryOperator::GenNe,
                Rule::GenLt => ast::BinaryOperator::GenLt,
                Rule::GenLe => ast::BinaryOperator::GenLe,
                Rule::GenGt => ast::BinaryOperator::GenGt,
                Rule::GenGe => ast::BinaryOperator::GenGe,
                Rule::Is => ast::BinaryOperator::Is,
                Rule::Precedes => ast::BinaryOperator::Precedes,
                Rule::Follows => ast::BinaryOperator::Follows,
                _ => {
                    unreachable!("unknown ComparisonExpr {:?}", r)
                }
            }),
            Rule::RangeExpr => self.binary(pair, ast::BinaryOperator::Range),
            Rule::StringConcatExpr => self.binary(pair, ast::BinaryOperator::Concat),
            Rule::LetExpr => {
                let span = &pair.as_span();
                let mut pairs = pair.into_inner();
                let let_clause = pairs.next().unwrap();
                let let_clause_pairs = let_clause.into_inner();
                let inner_return_expr = self.expr_single(pairs.next().unwrap());
                let mut return_expr = inner_return_expr;
                for let_clause_pair in let_clause_pairs.rev() {
                    let mut let_binding = let_clause_pair.into_inner();
                    let var_name = let_binding.next().unwrap();
                    let var_expr = self.expr_single(let_binding.next().unwrap());
                    let let_expr = ast::LetExpr {
                        var_name: self.var_name_to_name(var_name),
                        var_expr: Box::new(var_expr),
                        return_expr: Box::new(return_expr),
                    };
                    return_expr = spanned(ast::ExprSingle::Let(let_expr), span);
                }
                return_expr
            }
            Rule::ForExpr => {
                let span = &pair.as_span();
                let mut pairs = pair.into_inner();
                let for_clause = pairs.next().unwrap();
                let for_clause_pairs = for_clause.into_inner();
                let inner_return_expr = self.expr_single(pairs.next().unwrap());
                let mut return_expr = inner_return_expr;
                for for_clause_pair in for_clause_pairs.rev() {
                    let mut for_binding = for_clause_pair.into_inner();
                    let var_name = for_binding.next().unwrap();
                    let var_expr = self.expr_single(for_binding.next().unwrap());
                    let for_expr = ast::ForExpr {
                        var_name: self.var_name_to_name(var_name),
                        var_expr: Box::new(var_expr),
                        return_expr: Box::new(return_expr),
                    };
                    return_expr = spanned(ast::ExprSingle::For(for_expr), span);
                }
                return_expr
            }
            Rule::QuantifiedExpr => {
                let span = &pair.as_span();
                let mut pairs = pair.into_inner();
                let quantifier = pairs.next().unwrap();
                let quantifier = match quantifier.as_str() {
                    "some" => ast::Quantifier::Some,
                    "every" => ast::Quantifier::Every,
                    _ => {
                        unreachable!("unhandled QuantifiedExpr {:?}", quantifier.as_str())
                    }
                };
                let quantifier_clause = pairs.next().unwrap();
                let quantifier_clause_pairs = quantifier_clause.into_inner();
                let inner_satisfies_expr = self.expr_single(pairs.next().unwrap());
                let mut satisfies_expr = inner_satisfies_expr;
                for quantifier_clause_pair in quantifier_clause_pairs.rev() {
                    let mut quantifier_binding = quantifier_clause_pair.into_inner();
                    let var_name = quantifier_binding.next().unwrap();
                    let var_expr = self.expr_single(quantifier_binding.next().unwrap());
                    let quantified_expr = ast::QuantifiedExpr {
                        quantifier: quantifier.clone(),
                        var_name: self.var_name_to_name(var_name),
                        var_expr: Box::new(var_expr),
                        satisfies_expr: Box::new(satisfies_expr),
                    };
                    satisfies_expr = spanned(ast::ExprSingle::Quantified(quantified_expr), span);
                }
                satisfies_expr
            }
            Rule::IfExpr => {
                let span = &pair.as_span();
                let mut pairs = pair.into_inner();
                let condition_pair = pairs.next().unwrap();
                let condition = self.exprs(condition_pair);
                let then = self.expr_single(pairs.next().unwrap());
                let else_ = self.expr_single(pairs.next().unwrap());
                spanned(
                    ast::ExprSingle::If(ast::IfExpr {
                        condition,
                        then: Box::new(then),
                        else_: Box::new(else_),
                    }),
                    span,
                )
            }
            Rule::UnionExpr => self.binary(pair, ast::BinaryOperator::Union),
            Rule::IntersectExceptExpr => self.binary_op(pair, |r| match r {
                Rule::Intersect => ast::BinaryOperator::Intersect,
                Rule::Except => ast::BinaryOperator::Except,
                _ => {
                    unreachable!("unknown IntersectExceptExpr {:?}", r)
                }
            }),
            Rule::ValueExpr => {
                let pair = pair.into_inner().next().unwrap();
                self.expr_single(pair)
            }
            Rule::ParenthesizedExpr => {
                let pair = pair.into_inner().next().unwrap();
                // pass this along to Rule::Expr
                self.expr_single(pair)
            }
            Rule::ExprSingle => {
                let pair = pair.into_inner().next().unwrap();
                self.expr_single(pair)
            }
            Rule::Expr => {
                let span = &pair.as_span();
                let exprs = self.exprs(pair);
                if exprs.value.len() == 1 {
                    exprs.value[0].clone()
                } else {
                    spanned(
                        ast::ExprSingle::Path(ast::PathExpr {
                            steps: vec![spanned(
                                ast::StepExpr::PrimaryExpr(spanned(
                                    ast::PrimaryExpr::Expr(exprs),
                                    span,
                                )),
                                span,
                            )],
                        }),
                        span,
                    )
                }
            }
            _ => {
                panic!("unhandled ExprSingle {:?}", pair.as_rule())
            }
        }
    }

    fn binary_get_operator<F>(&self, pair: Pair<Rule>, get_operator: F) -> ast::ExprSingleS
    where
        F: Fn(&mut Pairs<Rule>) -> Option<ast::BinaryOperator>,
    {
        let mut pairs = pair.into_inner();
        let left_pair = pairs.next().unwrap();
        let span_start = left_pair.as_span().start();
        let mut binary = self.expr_single(left_pair);

        while let Some(operator) = get_operator(&mut pairs) {
            let right_pair = pairs.next().expect("operator but no right pair");
            let span_end = right_pair.as_span().end();
            binary = Spanned::new(
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator,
                    left: expr_single_to_path_expr(binary),
                    right: self.pair_to_path_expr(right_pair),
                }),
                (span_start, span_end - span_start).into(),
            )
        }
        binary
    }

    fn binary_op<F>(&self, pair: Pair<Rule>, get_operator: F) -> ast::ExprSingleS
    where
        F: Fn(Rule) -> ast::BinaryOperator,
    {
        self.binary_get_operator(pair, |pairs| {
            let op = pairs.next();
            op.map(|op| get_operator(op.as_rule()))
        })
    }

    fn binary(&self, pair: Pair<Rule>, operator: ast::BinaryOperator) -> ast::ExprSingleS {
        self.binary_get_operator(pair, |pairs| {
            if pairs.peek().is_some() {
                Some(operator)
            } else {
                None
            }
        })
    }

    fn path_expr_to_path_expr(&self, pair: Pair<Rule>) -> ast::PathExpr {
        debug_assert_eq!(pair.as_rule(), Rule::PathExpr);
        let mut pairs = pair.into_inner();
        let first_pair = pairs.next().unwrap();
        let first_pair_span = first_pair.as_span();
        match first_pair.as_rule() {
            Rule::Slash => {
                let mut steps = vec![root_from_context(&first_pair_span)];
                let next_pair = pairs.next();
                if let Some(next_pair) = next_pair {
                    steps.extend(self.relative_path_expr_to_steps(next_pair));
                }
                ast::PathExpr { steps }
            }
            Rule::DoubleSlash => {
                let mut steps = vec![
                    root_from_context(&first_pair_span),
                    spanned(
                        ast::StepExpr::AxisStep(ast::AxisStep {
                            axis: ast::Axis::DescendantOrSelf,
                            node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                            predicates: vec![],
                        }),
                        &first_pair_span,
                    ),
                ];
                steps.extend(self.relative_path_expr_to_steps(pairs.next().unwrap()));
                ast::PathExpr { steps }
            }
            Rule::RelativePathExpr => ast::PathExpr {
                steps: self.relative_path_expr_to_steps(first_pair),
            },
            _ => {
                panic!("unhandled PathExpr {:?}", first_pair.as_rule())
            }
        }
    }

    fn relative_path_expr_to_steps(&self, pair: Pair<Rule>) -> Vec<ast::StepExprS> {
        debug_assert_eq!(pair.as_rule(), Rule::RelativePathExpr);
        let pairs = pair.into_inner();
        let mut result = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::Slash => {
                    // do nothing
                }
                Rule::DoubleSlash => {
                    let span = pair.as_span();
                    result.push(spanned(
                        ast::StepExpr::AxisStep(ast::AxisStep {
                            axis: ast::Axis::DescendantOrSelf,
                            node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                            predicates: vec![],
                        }),
                        &span,
                    ));
                }
                Rule::StepExpr => {
                    result.push(self.step_expr_to_step_expr(pair));
                }
                _ => {
                    unreachable!("unhandled {:?}", pair.as_rule());
                }
            }
        }
        result
    }

    fn step_expr_to_step_expr(&self, pair: Pair<Rule>) -> ast::StepExprS {
        debug_assert_eq!(pair.as_rule(), Rule::StepExpr);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::PostfixExpr => {
                let span = pair.as_span();
                let mut pairs = pair.into_inner();
                let primary_pair = pairs.next().unwrap();
                let primary_pair_span = primary_pair.as_span();
                let primary = self.primary_expr_to_primary(primary_pair);
                let postfixes = pairs
                    .map(|pair| self.postfix_expr_to_postfix(pair))
                    .collect::<Vec<_>>();
                if postfixes.is_empty() {
                    spanned(ast::StepExpr::PrimaryExpr(primary), &primary_pair_span)
                } else {
                    self.postfixes_or_placeholdereds(primary, postfixes, &span)
                }
            }
            Rule::AxisStep => {
                let span = pair.as_span();
                let mut pairs = pair.into_inner();
                let step_pair = pairs.next().unwrap();
                let predicates_pair = pairs.next().unwrap();
                let predicates = predicates_pair
                    .into_inner()
                    .map(|pair| self.predicate_to_expr(pair))
                    .collect::<Vec<_>>();
                let (axis, node_test) = match step_pair.as_rule() {
                    Rule::ForwardStep => self.forward_step_to_axis_node_test(step_pair),
                    Rule::ReverseStep => self.reverse_step_to_axis_node_test(step_pair),
                    _ => {
                        panic!("unhandled AxisStep: {:?}", step_pair.as_rule())
                    }
                };

                let axis_step = ast::AxisStep {
                    axis,
                    node_test,
                    predicates,
                };
                spanned(ast::StepExpr::AxisStep(axis_step), &span)
            }
            Rule::AbbrevReverseStep => {
                let span = pair.as_span();
                let mut pairs = pair.into_inner();
                let _double_dot_pair = pairs.next().unwrap();
                let predicates_pair = pairs.next().unwrap();
                let predicates = predicates_pair
                    .into_inner()
                    .map(|pair| self.predicate_to_expr(pair))
                    .collect::<Vec<_>>();
                spanned(
                    ast::StepExpr::AxisStep(ast::AxisStep {
                        axis: ast::Axis::Parent,
                        node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                        predicates,
                    }),
                    &span,
                )
            }
            _ => {
                panic!("unhandled StepExpr: {:?}", pair.as_rule())
            }
        }
    }

    fn forward_step_to_axis_node_test(&self, pair: Pair<Rule>) -> (ast::Axis, ast::NodeTest) {
        debug_assert_eq!(pair.as_rule(), Rule::ForwardStep);
        let mut pairs = pair.into_inner();
        let first_pair = pairs.next().unwrap();
        if first_pair.as_rule() == Rule::ForwardAxis {
            let axis = self.forward_axis_to_axis(first_pair);
            let is_attribute = matches!(axis, ast::Axis::Attribute);
            let node_test_pair = pairs.next().unwrap();
            let node_test = self.node_test_to_node_test(node_test_pair, is_attribute);
            (axis, node_test)
        } else {
            let mut pairs = first_pair.into_inner();
            let first = pairs.next().unwrap();
            match first.as_rule() {
                Rule::AbbrevAtSign => {
                    let node_test = self.node_test_to_node_test(pairs.next().unwrap(), true);
                    (ast::Axis::Attribute, node_test)
                }
                Rule::NodeTest => {
                    let node_test = self.node_test_to_node_test(first, false);
                    // https://www.w3.org/TR/xpath-31/#abbrev
                    let axis = match &node_test {
                        ast::NodeTest::KindTest(t) => match t {
                            ast::KindTest::Attribute(_) | ast::KindTest::SchemaAttribute(_) => {
                                ast::Axis::Attribute
                            }
                            ast::KindTest::NamespaceNode => ast::Axis::Namespace,
                            _ => ast::Axis::Child,
                        },
                        _ => ast::Axis::Child,
                    };
                    (axis, node_test)
                }
                _ => {
                    unreachable!("unhandled AbbrevForwardStep: {:?}", first.as_rule())
                }
            }
        }
    }

    fn reverse_step_to_axis_node_test(&self, pair: Pair<Rule>) -> (ast::Axis, ast::NodeTest) {
        debug_assert_eq!(pair.as_rule(), Rule::ReverseStep);
        let mut pairs = pair.into_inner();
        let first_pair = pairs.next().unwrap();
        if first_pair.as_rule() == Rule::ReverseAxis {
            let axis = self.reverse_axis_to_axis(first_pair);
            let node_test_pair = pairs.next().unwrap();
            let node_test = self.node_test_to_node_test(node_test_pair, false);
            (axis, node_test)
        } else {
            // abbrev reverse step
            todo!("abbrev reverse step");
        }
    }

    fn forward_axis_to_axis(&self, pair: Pair<Rule>) -> ast::Axis {
        match pair.as_str() {
            "child::" => ast::Axis::Child,
            "descendant::" => ast::Axis::Descendant,
            "attribute::" => ast::Axis::Attribute,
            "self::" => ast::Axis::Self_,
            "descendant-or-self::" => ast::Axis::DescendantOrSelf,
            "following-sibling::" => ast::Axis::FollowingSibling,
            "following::" => ast::Axis::Following,
            "namespace::" => ast::Axis::Namespace,
            _ => {
                panic!("unhandled ForwardAxis: {:?}", pair.as_rule())
            }
        }
    }

    fn reverse_axis_to_axis(&self, pair: Pair<Rule>) -> ast::Axis {
        match pair.as_str() {
            "parent::" => ast::Axis::Parent,
            "ancestor::" => ast::Axis::Ancestor,
            "preceding-sibling::" => ast::Axis::PrecedingSibling,
            "preceding::" => ast::Axis::Preceding,
            "ancestor-or-self::" => ast::Axis::AncestorOrSelf,
            _ => {
                panic!("unhandled ReverseAxis: {:?}", pair.as_rule())
            }
        }
    }

    fn node_test_to_node_test(&self, pair: Pair<Rule>, is_attribute: bool) -> ast::NodeTest {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::KindTest => ast::NodeTest::KindTest(self.kind_test_to_kind_test(pair)),
            Rule::NameTest => ast::NodeTest::NameTest(
                self.name_test_to_name_test(pair.into_inner().next().unwrap(), is_attribute),
            ),
            _ => {
                panic!("unhandled NodeTest: {:?}", pair.as_rule())
            }
        }
    }

    fn name_test_to_name_test(&self, pair: Pair<Rule>, is_attribute: bool) -> ast::NameTest {
        match pair.as_rule() {
            Rule::Wildcard => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::WildcardStar => ast::NameTest::Star,
                    // any local name with a particular prefix
                    Rule::WildcardLocalName => {
                        let prefix = pair.into_inner().next().unwrap().as_str();
                        let namespace = self.namespaces.by_prefix(prefix).unwrap();
                        ast::NameTest::Namespace(namespace.to_string())
                    }
                    // any prefix with a particular local name
                    Rule::WildcardPrefix => {
                        let local_name = pair.into_inner().next().unwrap().as_str();
                        ast::NameTest::LocalName(local_name.to_string())
                    }
                    // any local name with a particular namespace URI
                    Rule::WildcardBracedURILiteral => {
                        let braced_pair = pair.into_inner().next().unwrap();
                        let uri_literal_pair = braced_pair.into_inner().next().unwrap();
                        ast::NameTest::Namespace(uri_literal_pair.as_str().to_string())
                    }
                    _ => {
                        panic!("unhandled Wildcard: {:?}", pair.as_rule())
                    }
                }
            }
            Rule::EQName => {
                if is_attribute {
                    // attributes are not in any namespace by default
                    ast::NameTest::Name(self.eq_name_to_name(pair, None))
                } else {
                    ast::NameTest::Name(
                        self.eq_name_to_name(pair, self.namespaces.default_element_namespace),
                    )
                }
            }
            _ => {
                panic!("unhandled NameTest: {:?}", pair.as_rule())
            }
        }
    }

    fn kind_test_to_kind_test(&self, pair: Pair<Rule>) -> ast::KindTest {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::AnyKindTest => ast::KindTest::Any,
            Rule::TextTest => ast::KindTest::Text,
            Rule::CommentTest => ast::KindTest::Comment,
            Rule::NamespaceNodeTest => ast::KindTest::NamespaceNode,
            Rule::ElementTest => {
                let mut pairs = pair.into_inner();
                if let Some(_pair) = pairs.next() {
                    todo!("no arguments for element test yet")
                } else {
                    ast::KindTest::Element(None)
                }
            }
            Rule::AttributeTest => {
                let mut pairs = pair.into_inner();
                let first_pair = pairs.next();
                if let Some(_pair) = first_pair {
                    // XXX this should not use a default element namespace
                    todo!("no arguments for attribute test yet")
                } else {
                    ast::KindTest::Attribute(None)
                }
            }
            _ => {
                panic!("unhandled KindTest: {:?}", pair.as_rule());
            }
        }
    }

    fn primary_expr_to_primary(&self, pair: Pair<Rule>) -> ast::PrimaryExprS {
        debug_assert_eq!(pair.as_rule(), Rule::PrimaryExpr);
        let pair = pair.into_inner().next().unwrap();
        let span = &pair.as_span();
        spanned(
            match pair.as_rule() {
                Rule::Literal => ast::PrimaryExpr::Literal(self.literal_to_literal(pair)),
                Rule::ParenthesizedExpr => ast::PrimaryExpr::Expr(self.exprs(pair)),
                Rule::VarRef => {
                    let pair = pair.into_inner().next().unwrap();
                    ast::PrimaryExpr::VarRef(self.var_name_to_name(pair))
                }
                Rule::FunctionItemExpr => {
                    let pair = pair.into_inner().next().unwrap();
                    match pair.as_rule() {
                        Rule::InlineFunctionExpr => ast::PrimaryExpr::InlineFunction(
                            self.inline_function_expr_to_inline_function(pair),
                        ),
                        Rule::NamedFunctionRef => ast::PrimaryExpr::NamedFunctionRef(
                            self.named_function_ref_to_named_function_ref(pair),
                        ),
                        _ => {
                            panic!("unhandled FunctionItemExpr: {:?}", pair.as_rule())
                        }
                    }
                }
                Rule::FunctionCall => {
                    let mut pairs = pair.into_inner();
                    let name = pairs.next().unwrap();
                    // unwrap NonReservedFunctionName
                    let name = name.into_inner().next().unwrap();
                    let arguments = pairs.next().unwrap();
                    match self.argument_list_to_args(arguments) {
                        ArgumentsOrPlaceholdered::Arguments(arguments) => {
                            ast::PrimaryExpr::FunctionCall(ast::FunctionCall {
                                name: self.eq_name_to_name(
                                    name,
                                    self.namespaces.default_function_namespace,
                                ),
                                arguments,
                            })
                        }
                        ArgumentsOrPlaceholdered::Placeholdered(arguments, params) => {
                            // construct an inline function that calls the underlying function,
                            // with the reduced placeholdered params
                            ast::PrimaryExpr::InlineFunction(ast::InlineFunction {
                                params,
                                return_type: None,
                                body: not_spanned(vec![not_spanned(ast::ExprSingle::Path(
                                    ast::PathExpr {
                                        steps: vec![not_spanned(ast::StepExpr::PrimaryExpr(
                                            not_spanned(ast::PrimaryExpr::FunctionCall(
                                                ast::FunctionCall {
                                                    name: self.eq_name_to_name(
                                                        name,
                                                        self.namespaces.default_function_namespace,
                                                    ),
                                                    arguments,
                                                },
                                            )),
                                        ))],
                                    },
                                ))]),
                            })
                        }
                    }
                }
                Rule::ContextItemExpr => ast::PrimaryExpr::ContextItem,
                _ => {
                    panic!("unhandled PrimaryExpr: {:?}", pair.as_rule())
                }
            },
            span,
        )
    }

    fn postfix_expr_to_postfix(&self, pair: Pair<Rule>) -> PostfixOrPlaceholdered {
        match pair.as_rule() {
            Rule::Predicate => {
                let pair = pair.into_inner().next().unwrap();
                PostfixOrPlaceholdered::Postfix(ast::Postfix::Predicate(self.exprs(pair)))
            }
            Rule::ArgumentList => match self.argument_list_to_args(pair) {
                ArgumentsOrPlaceholdered::Arguments(arguments) => {
                    PostfixOrPlaceholdered::Postfix(ast::Postfix::ArgumentList(arguments))
                }
                ArgumentsOrPlaceholdered::Placeholdered(arguments, params) => {
                    PostfixOrPlaceholdered::Placeholdered(arguments, params)
                }
            },
            Rule::Lookup => {
                panic!("lookup not handled yet");
            }
            _ => {
                panic!("unhandled postfix: {:?}", pair.as_rule())
            }
        }
    }

    fn postfixes_or_placeholdereds(
        &self,
        primary: ast::PrimaryExprS,
        postfixes: Vec<PostfixOrPlaceholdered>,
        span: &pest::Span,
    ) -> ast::StepExprS {
        let mut normal_postfixes = Vec::new();
        let mut primary = primary;
        for postfix in postfixes {
            match postfix {
                PostfixOrPlaceholdered::Postfix(postfix) => {
                    normal_postfixes.push(postfix);
                }
                PostfixOrPlaceholdered::Placeholdered(arguments, params) => {
                    // we want to add a postfix to the primary with placeholdered params
                    normal_postfixes.push(ast::Postfix::ArgumentList(arguments));
                    primary = not_spanned(ast::PrimaryExpr::InlineFunction(ast::InlineFunction {
                        params,
                        return_type: None,
                        body: not_spanned(vec![not_spanned(ast::ExprSingle::Path(
                            ast::PathExpr {
                                steps: vec![not_spanned(ast::StepExpr::PostfixExpr {
                                    primary,
                                    postfixes: normal_postfixes.clone(),
                                })],
                            },
                        ))]),
                    }));
                    // collect more postfixes now
                    normal_postfixes.clear();
                }
            }
        }
        if !normal_postfixes.is_empty() {
            spanned(
                ast::StepExpr::PostfixExpr {
                    primary,
                    postfixes: normal_postfixes,
                },
                span,
            )
        } else {
            spanned(ast::StepExpr::PrimaryExpr(primary), span)
        }
    }

    fn argument_list_to_args(&self, pair: Pair<Rule>) -> ArgumentsOrPlaceholdered {
        debug_assert_eq!(pair.as_rule(), Rule::ArgumentList);

        let mut args = vec![];
        let mut placeholder_index = 0;
        let mut params = vec![];
        for pair in pair.into_inner() {
            let expr_single = self.argument_to_expr_single(pair);
            if let Some(expr_single) = expr_single {
                args.push(expr_single);
            } else {
                // XXX what if someone uses this as a parameter name?
                let param_name = format!("placeholder{}", placeholder_index);
                let name = ast::Name::without_ns(&param_name);
                params.push(ast::Param {
                    name: name.clone(),
                    type_: None,
                });
                args.push(not_spanned(ast::ExprSingle::Path(ast::PathExpr {
                    steps: vec![not_spanned(ast::StepExpr::PrimaryExpr(not_spanned(
                        ast::PrimaryExpr::VarRef(name),
                    )))],
                })));
                placeholder_index += 1;
            }
        }
        if params.is_empty() {
            ArgumentsOrPlaceholdered::Arguments(args)
        } else {
            ArgumentsOrPlaceholdered::Placeholdered(args, params)
        }
    }

    fn argument_to_expr_single(&self, pair: Pair<Rule>) -> Option<ast::ExprSingleS> {
        debug_assert_eq!(pair.as_rule(), Rule::Argument);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::ExprSingle => Some(self.expr_single(pair)),
            Rule::ArgumentPlaceholder => None,
            _ => {
                panic!("unhandled argument: {:?}", pair.as_rule())
            }
        }
    }

    fn predicate_to_expr(&self, pair: Pair<Rule>) -> ast::ExprS {
        debug_assert_eq!(pair.as_rule(), Rule::Predicate);
        let pair = pair.into_inner().next().unwrap();
        self.exprs(pair)
    }

    fn var_name_to_name(&self, pair: Pair<Rule>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::VarName);
        // XXX no support for namespaces yet
        ast::Name::without_ns(pair.as_str())
    }

    fn eq_name_to_name(&self, pair: Pair<Rule>, default_namespace: Option<&'a str>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::EQName);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::QName => self.q_name_to_name(pair, default_namespace),
            Rule::URIQualifiedName => self.uri_qualified_name_to_name(pair),
            _ => {
                panic!("unhandled EQName: {:?}", pair.as_rule());
            }
        }
    }

    fn q_name_to_name(&self, pair: Pair<Rule>, default_namespace: Option<&'a str>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::QName);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::PrefixedName => self.prefixed_name_to_name(pair),
            Rule::UnprefixedName => self.unprefixed_name_to_name(pair, default_namespace),
            _ => {
                panic!("unhandled QName: {:?}", pair.as_rule())
            }
        }
    }

    fn uri_qualified_name_to_name(&self, pair: Pair<Rule>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::URIQualifiedName);
        let mut pairs = pair.into_inner();
        let uri = pairs.next().unwrap();
        let uri = uri.into_inner().next().unwrap();
        let local_part = pairs.next().unwrap();
        ast::Name::new(
            local_part.as_str().to_string(),
            Some(uri.as_str().to_string()),
        )
    }

    fn prefixed_name_to_name(&self, pair: Pair<Rule>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::PrefixedName);
        let mut pairs = pair.into_inner();
        let prefix = pairs.next().unwrap().as_str();
        // XXX unwrap should be an compile time error
        let namespace = self.namespaces.by_prefix(prefix).unwrap();
        let local_part = pairs.next().unwrap();
        ast::Name::new(local_part.as_str().to_string(), Some(namespace.to_string()))
    }

    fn unprefixed_name_to_name(
        &self,
        pair: Pair<Rule>,
        default_namespace: Option<&'a str>,
    ) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::UnprefixedName);
        let pair = pair.into_inner().next().unwrap();
        ast::Name::new(
            pair.as_str().to_string(),
            default_namespace.map(|s| s.to_string()),
        )
    }

    fn named_function_ref_to_named_function_ref(&self, pair: Pair<Rule>) -> ast::NamedFunctionRef {
        debug_assert_eq!(pair.as_rule(), Rule::NamedFunctionRef);
        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap();
        let arity = pairs.next().unwrap();
        ast::NamedFunctionRef {
            name: self.eq_name_to_name(name, self.namespaces.default_function_namespace),
            arity: arity.as_str().parse().unwrap(),
        }
    }

    fn inline_function_expr_to_inline_function(&self, pair: Pair<Rule>) -> ast::InlineFunction {
        debug_assert_eq!(pair.as_rule(), Rule::InlineFunctionExpr);
        let mut pairs = pair.into_inner();
        let mut next = pairs.next().unwrap();
        let params = if next.as_rule() == Rule::ParamList {
            let params = self.params(next);
            next = pairs.next().unwrap();
            params
        } else {
            vec![]
        };
        let return_type = if next.as_rule() == Rule::SequenceType {
            let sequence_type = Some(self.sequence_type(next));
            next = pairs.next().unwrap();
            sequence_type
        } else {
            None
        };
        let body = self.function_body_to_body(next);
        ast::InlineFunction {
            params,
            return_type,
            body,
        }
    }

    fn signature(&self, pair: Pair<Rule>) -> ast::Signature {
        debug_assert_eq!(pair.as_rule(), Rule::Signature);
        let mut pairs = pair.into_inner();
        let name = self.eq_name_to_name(pairs.next().unwrap(), None);
        let mut next = pairs.next().unwrap();
        let params = if next.as_rule() == Rule::SignatureParamList {
            let params = self.signature_params(next);
            next = pairs.next().unwrap();
            params
        } else {
            vec![]
        };

        let return_type = self.sequence_type(next);

        ast::Signature {
            name,
            params,
            return_type,
        }
    }

    fn params(&self, pair: Pair<Rule>) -> Vec<ast::Param> {
        debug_assert_eq!(pair.as_rule(), Rule::ParamList);
        let mut parameters = vec![];
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::Param => {
                    parameters.push(self.param(pair));
                }
                _ => {
                    panic!("unhandled ParamList: {:?}", pair.as_rule())
                }
            }
        }
        parameters
    }

    fn param(&self, pair: Pair<Rule>) -> ast::Param {
        debug_assert_eq!(pair.as_rule(), Rule::Param);
        let mut pairs = pair.into_inner();
        let name = self.eq_name_to_name(pairs.next().unwrap(), None);
        let next = pairs.next();
        let type_ = next.map(|next| self.type_declaration(next));
        ast::Param { name, type_ }
    }

    fn signature_params(&self, pair: Pair<Rule>) -> Vec<ast::SignatureParam> {
        debug_assert_eq!(pair.as_rule(), Rule::SignatureParamList);
        let mut parameters = vec![];
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::SignatureParam => {
                    parameters.push(self.signature_param(pair));
                }
                _ => {
                    panic!("unhandled ParamList: {:?}", pair.as_rule())
                }
            }
        }
        parameters
    }

    fn signature_param(&self, pair: Pair<Rule>) -> ast::SignatureParam {
        debug_assert_eq!(pair.as_rule(), Rule::SignatureParam);
        let mut pairs = pair.into_inner();
        let name = self.eq_name_to_name(pairs.next().unwrap(), None);
        let next = pairs.next().unwrap();
        let type_ = self.type_declaration(next);
        ast::SignatureParam { name, type_ }
    }

    fn type_declaration(&self, pair: Pair<Rule>) -> ast::SequenceType {
        debug_assert_eq!(pair.as_rule(), Rule::TypeDeclaration);
        let mut pairs = pair.into_inner();
        let next = pairs.next().unwrap();
        self.sequence_type(next)
    }

    fn sequence_type(&self, pair: Pair<Rule>) -> ast::SequenceType {
        debug_assert_eq!(pair.as_rule(), Rule::SequenceType);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::EmptySequenceType => ast::SequenceType::Empty,
            Rule::Item => ast::SequenceType::Item(self.item(pair)),
            _ => {
                panic!("unhandled SequenceType: {:?}", pair.as_rule())
            }
        }
    }

    fn item(&self, pair: Pair<Rule>) -> ast::Item {
        debug_assert_eq!(pair.as_rule(), Rule::Item);
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let item_type = self.item_type(pair);
        let next = pairs.next();
        let occurrence = if let Some(next) = next {
            match next.as_str() {
                "?" => ast::Occurrence::Option,
                "+" => ast::Occurrence::NonEmpty,
                "*" => ast::Occurrence::Many,
                _ => {
                    panic!("unhandled ItemType: {:?}", next.as_str())
                }
            }
        } else {
            ast::Occurrence::One
        };
        ast::Item {
            item_type,
            occurrence,
        }
    }

    fn item_type(&self, pair: Pair<Rule>) -> ast::ItemType {
        debug_assert_eq!(pair.as_rule(), Rule::ItemType);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::KindTest => ast::ItemType::KindTest(self.kind_test_to_kind_test(pair)),
            Rule::AnyItemType => ast::ItemType::Item,
            Rule::AtomicOrUnionType => {
                ast::ItemType::AtomicOrUnionType(self.atomic_or_union_type(pair))
            }
            _ => {
                panic!("unhandled ItemType: {:?}", pair.as_rule())
            }
        }
    }

    fn atomic_or_union_type(&self, pair: Pair<Rule>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::AtomicOrUnionType);
        let pair = pair.into_inner().next().unwrap();
        self.eq_name_to_name(pair, None)
    }

    fn single_type(&self, pair: Pair<Rule>) -> ast::SingleType {
        debug_assert_eq!(pair.as_rule(), Rule::SingleType);
        let mut pairs = pair.into_inner();
        let name = self.simple_type_name(pairs.next().unwrap());
        let question_mark = pairs.next().is_some();
        ast::SingleType {
            name,
            question_mark,
        }
    }

    fn simple_type_name(&self, pair: Pair<Rule>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::SimpleTypeName);
        let pair = pair.into_inner().next().unwrap();
        self.type_name(pair)
    }

    fn type_name(&self, pair: Pair<Rule>) -> ast::Name {
        debug_assert_eq!(pair.as_rule(), Rule::TypeName);
        let pair = pair.into_inner().next().unwrap();
        self.eq_name_to_name(pair, None)
    }

    fn function_body_to_body(&self, pair: Pair<Rule>) -> ast::ExprS {
        debug_assert_eq!(pair.as_rule(), Rule::FunctionBody);
        let pair = pair.into_inner().next().unwrap();
        self.exprs(pair)
    }

    fn literal_to_literal(&self, pair: Pair<Rule>) -> ast::Literal {
        debug_assert_eq!(pair.as_rule(), Rule::Literal);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::StringLiteral => {
                let pair = pair.into_inner().next().unwrap();
                ast::Literal::String(pair.as_str().to_string())
            }
            Rule::NumericLiteral => self.numeric_literal_to_literal(pair),
            _ => {
                panic!("unhandled literal: {:?}", pair.as_rule())
            }
        }
    }

    fn numeric_literal_to_literal(&self, pair: Pair<Rule>) -> ast::Literal {
        debug_assert_eq!(pair.as_rule(), Rule::NumericLiteral);
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::IntegerLiteral => {
                let s = pair.as_str();
                ast::Literal::Integer(s.to_string())
            }
            Rule::DecimalLiteral => {
                let s = pair.as_str();
                let period_index = s.find('.').unwrap();
                let (before, after) = s.split_at(period_index);
                let after = &after[1..];
                let before_nr = if !before.is_empty() {
                    before.parse::<i64>().unwrap()
                } else {
                    0
                };
                let after_nr = if !after.is_empty() {
                    after.parse::<i64>().unwrap()
                } else {
                    0
                };
                let digits = after.len();
                // to get positive number
                let factor = 10i64.pow(digits as u32);
                let before_nr = before_nr * factor;
                let nr = before_nr + after_nr;
                ast::Literal::Decimal(Decimal::new(nr, digits as u32))
            }
            Rule::DoubleLiteral => {
                let s = pair.as_str();
                let f = s.parse::<f64>().unwrap();
                ast::Literal::Double(OrderedFloat(f))
            }
            _ => {
                panic!("unhandled numeric literal: {:?}", pair.as_rule())
            }
        }
    }
}

fn expr_single_to_path_expr(expr: ast::ExprSingleS) -> ast::PathExpr {
    let span = expr.span;
    match expr.value {
        ast::ExprSingle::Path(path) => path,
        _ => ast::PathExpr {
            steps: vec![Spanned::new(
                ast::StepExpr::PrimaryExpr(Spanned::new(
                    ast::PrimaryExpr::Expr(Spanned::new(vec![expr], span)),
                    span,
                )),
                span,
            )],
        },
    }
}

fn root_from_context(span: &pest::Span) -> ast::StepExprS {
    spanned(
        ast::StepExpr::PrimaryExpr(not_spanned(ast::PrimaryExpr::FunctionCall(
            ast::FunctionCall {
                name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
                arguments: vec![not_spanned(ast::ExprSingle::Path(ast::PathExpr {
                    steps: vec![not_spanned(ast::StepExpr::AxisStep(ast::AxisStep {
                        axis: ast::Axis::Self_,
                        node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                        predicates: vec![],
                    }))],
                }))],
            },
        ))),
        span,
    )
}

#[allow(clippy::result_large_err)]
fn parse_rule_start_end<T, F>(rule: Rule, input: &str, f: F) -> Result<T, pest::error::Error<Rule>>
where
    F: Fn(Pair<Rule>) -> T,
{
    let mut pairs = XPathParser::parse(rule, input)?;
    let mut pairs = pairs.next().unwrap().into_inner();
    let pair = pairs.next().unwrap();
    Ok(f(pair))
}

pub fn parse_expr_single(input: &str) -> ast::ExprSingleS {
    let namespaces = Namespaces::new(None, None);
    let ast_parser = AstParser::new(&namespaces);
    parse_rule_start_end(Rule::OuterExprSingle, input, |p| ast_parser.expr_single(p)).unwrap()
}

pub fn parse_xpath(
    input: &str,
    namespaces: &Namespaces,
    variables: &[ast::Name],
) -> Result<ast::XPath, Error> {
    let ast_parser = AstParser::new(namespaces);
    let result = parse_rule_start_end(Rule::Xpath, input, |p| ast_parser.xpath(p));

    match result {
        Ok(mut xpath) => {
            // rename all variables to unique names
            unique_names(&mut xpath, variables);
            Ok(xpath)
        }
        Err(e) => {
            let src = input.to_string();
            let location = e.location;
            let span: SourceSpan = match location {
                InputLocation::Pos(pos) => (pos - 1, 0).into(),
                InputLocation::Span((start, end)) => (start, end).into(),
            };
            Err(Error::ParseError { src, span })
        }
    }
}

pub fn parse_signature(input: &str, namespaces: &Namespaces) -> Result<ast::Signature, Error> {
    let ast_parser = AstParser::new(namespaces);
    let result = parse_rule_start_end(Rule::OuterSignature, input, |p| ast_parser.signature(p));

    match result {
        Ok(signature) => Ok(signature),
        Err(e) => {
            let src = input.to_string();
            let location = e.location;
            let span: SourceSpan = match location {
                InputLocation::Pos(pos) => (pos, 0).into(),
                InputLocation::Span((start, end)) => (start, end).into(),
            };
            Err(Error::ParseError { src, span })
        }
    }
}

pub fn parse_sequence_type(
    input: &str,
    namespaces: &Namespaces,
) -> Result<ast::SequenceType, Error> {
    let ast_parser = AstParser::new(namespaces);
    let result = parse_rule_start_end(Rule::OuterSequenceType, input, |p| {
        ast_parser.sequence_type(p)
    });

    match result {
        Ok(sequence_type) => Ok(sequence_type),
        Err(e) => {
            let src = input.to_string();
            let location = e.location;
            let span: SourceSpan = match location {
                InputLocation::Pos(pos) => (pos, 0).into(),
                InputLocation::Span((start, end)) => (start, end).into(),
            };
            Err(Error::ParseError { src, span })
        }
    }
}

pub(crate) fn spanned<T>(value: T, span: &pest::Span) -> Spanned<T> {
    Spanned::new(value, (span.start(), span.end() - span.start()).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    fn parse_rule<T, F>(rule: Rule, input: &str, f: F) -> T
    where
        F: Fn(Pair<Rule>) -> T,
    {
        let mut pairs = XPathParser::parse(rule, input).unwrap();
        let pair = pairs.next().unwrap();
        f(pair)
    }

    fn parse_xpath_no_default_ns(input: &str) -> Result<ast::XPath, Error> {
        let namespaces = Namespaces::new(None, None);
        parse_xpath(input, &namespaces, &[])
    }

    fn parse_literal(input: &str) -> ast::Literal {
        let namespaces = Namespaces::new(None, None);
        let ast_parser = AstParser::new(&namespaces);
        parse_rule(Rule::Literal, input, |p| ast_parser.literal_to_literal(p))
    }

    fn parse_primary_expr(input: &str) -> ast::PrimaryExprS {
        let namespaces = Namespaces::new(None, None);
        let ast_parser = AstParser::new(&namespaces);
        parse_rule(Rule::PrimaryExpr, input, |p| {
            ast_parser.primary_expr_to_primary(p)
        })
    }

    fn parse_step_expr(input: &str) -> ast::StepExprS {
        let namespaces = Namespaces::new(None, None);
        let ast_parser = AstParser::new(&namespaces);
        parse_rule(Rule::StepExpr, input, |p| {
            ast_parser.step_expr_to_step_expr(p)
        })
    }

    fn parse_relative_path_expr(input: &str) -> Vec<ast::StepExprS> {
        let namespaces = Namespaces::new(None, None);
        let ast_parser = AstParser::new(&namespaces);
        parse_rule(Rule::RelativePathExpr, input, |p| {
            ast_parser.relative_path_expr_to_steps(p)
        })
    }

    fn parse_path_expr(input: &str) -> ast::PathExpr {
        let namespaces = Namespaces::new(None, None);
        let ast_parser = AstParser::new(&namespaces);
        parse_rule(Rule::PathExpr, input, |p| {
            ast_parser.path_expr_to_path_expr(p)
        })
    }

    #[test]
    fn test_string_literal() {
        assert_debug_snapshot!(parse_literal("'foo'"));
    }

    #[test]
    fn test_integer_literal() {
        assert_debug_snapshot!(parse_literal("1"));
    }

    #[test]
    fn test_decimal_literal() {
        assert_debug_snapshot!(parse_literal("1.5"));
    }

    #[test]
    fn test_decimal_literal_no_after() {
        assert_debug_snapshot!(parse_literal("1."));
    }

    #[test]
    fn test_decimal_literal_no_before() {
        assert_debug_snapshot!(parse_literal(".5"));
    }

    #[test]
    fn test_float_lowercase_e() {
        assert_debug_snapshot!(parse_literal("1.5e0"));
    }

    #[test]
    fn test_float_upper_e() {
        assert_debug_snapshot!(parse_literal("1.5E0"));
    }

    #[test]
    fn test_primary_expr_literal() {
        assert_debug_snapshot!(parse_primary_expr("1"));
    }

    #[test]
    fn test_step_expr() {
        assert_debug_snapshot!(parse_step_expr("1"));
    }

    #[test]
    fn test_relative_path() {
        assert_debug_snapshot!(parse_relative_path_expr("1"));
    }

    #[test]
    fn test_path_expr() {
        assert_debug_snapshot!(parse_path_expr("1"));
    }

    #[test]
    fn test_integer_expr_single() {
        assert_debug_snapshot!(parse_expr_single("1"));
    }

    #[test]
    fn test_simple_map_expr() {
        assert_debug_snapshot!(parse_expr_single("1 ! 2"));
    }

    #[test]
    fn test_unary_expr() {
        assert_debug_snapshot!(parse_expr_single("-1"));
    }

    #[test]
    fn test_additive_expr() {
        assert_debug_snapshot!(parse_expr_single("1 + 2"));
    }

    #[test]
    fn test_additive_expr_repeat() {
        assert_debug_snapshot!(parse_expr_single("1 + 2 + 3"));
    }

    #[test]
    fn test_or_expr() {
        assert_debug_snapshot!(parse_expr_single("1 or 2"));
    }

    #[test]
    fn test_and_expr() {
        assert_debug_snapshot!(parse_expr_single("1 and 2"));
    }

    #[test]
    fn test_comparison_expr() {
        assert_debug_snapshot!(parse_expr_single("1 < 2"));
    }

    #[test]
    fn test_concat_expr() {
        assert_debug_snapshot!(parse_expr_single("'a' || 'b'"));
    }

    #[test]
    fn test_nested_expr() {
        assert_debug_snapshot!(parse_expr_single("1 + (2 * 3)"));
    }

    #[test]
    fn test_xpath_single_expr() {
        assert_debug_snapshot!(parse_xpath_no_default_ns("1 + 2"));
    }

    #[test]
    fn test_xpath_multi_expr() {
        assert_debug_snapshot!(parse_xpath_no_default_ns("1 + 2, 3 + 4"));
    }

    #[test]
    fn test_single_let_expr() {
        assert_debug_snapshot!(parse_expr_single("let $x := 1 return 5"));
    }

    #[test]
    fn test_single_let_expr_var_ref() {
        assert_debug_snapshot!(parse_expr_single("let $x := 1 return $x"));
    }

    #[test]
    fn test_nested_let_expr() {
        assert_debug_snapshot!(parse_expr_single("let $x := 1, $y := 2 return 5"));
    }

    #[test]
    fn test_single_for_expr() {
        assert_debug_snapshot!(parse_expr_single("for $x in 1 return 5"));
    }

    #[test]
    fn test_for_loop() {
        assert_debug_snapshot!(parse_expr_single("for $x in 1 to 2 return $x"));
    }

    #[test]
    fn test_if_expr() {
        assert_debug_snapshot!(parse_expr_single("if (1) then 2 else 3"));
    }

    #[test]
    fn test_inline_function() {
        assert_debug_snapshot!(parse_expr_single("function($x) { $x }"));
    }

    #[test]
    fn test_inline_function_with_param_types() {
        assert_debug_snapshot!(parse_expr_single("function($x as xs:integer) { $x }"));
    }

    #[test]
    fn test_inline_function_with_return_type() {
        assert_debug_snapshot!(parse_expr_single("function($x) as xs:integer { $x }"));
    }

    #[test]
    fn test_inline_function2() {
        assert_debug_snapshot!(parse_expr_single("function($x, $y) { $x + $y }"));
    }

    #[test]
    fn test_dynamic_function_call() {
        assert_debug_snapshot!(parse_expr_single("$foo()"));
    }

    #[test]
    fn test_dynamic_function_call_args() {
        assert_debug_snapshot!(parse_expr_single("$foo(1 + 1, 3)"));
    }

    #[test]
    fn test_static_function_call() {
        assert_debug_snapshot!(parse_expr_single("my_function()"));
    }

    #[test]
    fn test_static_function_call_fn_prefix() {
        assert_debug_snapshot!(parse_expr_single("fn:root()"));
    }

    #[test]
    fn test_static_function_call_q() {
        assert_debug_snapshot!(parse_expr_single("Q{http://example.com}something()"));
    }

    #[test]
    fn test_static_function_call_args() {
        assert_debug_snapshot!(parse_expr_single("my_function(1, 2)"));
    }

    #[test]
    fn test_named_function_ref() {
        assert_debug_snapshot!(parse_expr_single("my_function#2"));
    }

    #[test]
    fn test_dynamic_function_call_placeholder() {
        assert_debug_snapshot!(parse_expr_single("$foo(1, ?)"));
    }

    #[test]
    fn test_static_function_call_placeholder() {
        assert_debug_snapshot!(parse_expr_single("my_function(?, 1)"));
    }

    #[test]
    fn test_simple_comma() {
        assert_debug_snapshot!(parse_xpath_no_default_ns("1, 2"));
    }

    #[test]
    fn test_complex_comma() {
        assert_debug_snapshot!(parse_xpath_no_default_ns("(1, 2), (3, 4)"));
    }

    #[test]
    fn test_range() {
        assert_debug_snapshot!(parse_expr_single("1 to 2"));
    }

    #[test]
    fn test_simple_map() {
        assert_debug_snapshot!(parse_expr_single("(1, 2) ! (. * 2)"));
    }

    #[test]
    fn test_quantified() {
        assert_debug_snapshot!(parse_expr_single("every $x in (1, 2) satisfies $x > 0"));
    }

    #[test]
    fn test_quantified_nested() {
        assert_debug_snapshot!(parse_expr_single(
            "every $x in (1, 2), $y in (3, 4) satisfies $x > 0 and $y > 0"
        ));
    }

    #[test]
    fn test_predicate() {
        assert_debug_snapshot!(parse_expr_single("(1, 2)[2]"));
    }

    #[test]
    fn test_axis() {
        assert_debug_snapshot!(parse_expr_single("child::foo"));
    }

    #[test]
    fn test_multiple_steps() {
        assert_debug_snapshot!(parse_expr_single("child::foo/child::bar"));
    }

    #[test]
    fn test_with_predicate() {
        assert_debug_snapshot!(parse_expr_single("child::foo[1]"));
    }

    #[test]
    fn test_axis_with_predicate() {
        assert_debug_snapshot!(parse_expr_single("child::foo[1]"));
    }

    #[test]
    fn test_axis_star() {
        assert_debug_snapshot!(parse_expr_single("child::*"));
    }

    #[test]
    fn test_axis_wildcard_prefix() {
        assert_debug_snapshot!(parse_expr_single("child::*:foo"));
    }

    #[test]
    fn test_axis_wildcard_local_name() {
        assert_debug_snapshot!(parse_expr_single("child::fn:*"));
    }

    #[test]
    fn test_axis_wildcard_q_name() {
        assert_debug_snapshot!(parse_expr_single("child::Q{http://example.com}*"));
    }

    #[test]
    fn test_reverse_axis() {
        assert_debug_snapshot!(parse_expr_single("parent::foo"));
    }

    #[test]
    fn test_node_test() {
        assert_debug_snapshot!(parse_expr_single("self::node()"));
    }

    #[test]
    fn test_text_test() {
        assert_debug_snapshot!(parse_expr_single("self::text()"));
    }

    #[test]
    fn test_comment_test() {
        assert_debug_snapshot!(parse_expr_single("self::comment()"));
    }

    #[test]
    fn test_namespace_node_test() {
        assert_debug_snapshot!(parse_expr_single("self::namespace-node()"));
    }

    #[test]
    fn test_attribute_test_no_args() {
        assert_debug_snapshot!(parse_expr_single("self::attribute()"));
    }

    // #[test]
    // fn test_attribute_test_star_arg() {
    //     assert_debug_snapshot!(parse_expr_single("self::attribute(*)"));
    // }

    // #[test]
    // fn test_attribute_test_name_arg() {
    //     assert_debug_snapshot!(parse_expr_single("self::attribute(foo)"));
    // }

    // #[test]
    // fn test_attribute_test_name_arg_type_arg() {
    //     assert_debug_snapshot!(parse_expr_single("self::attribute(foo, bar)"));
    // }

    #[test]
    fn test_element_test() {
        assert_debug_snapshot!(parse_expr_single("self::element()"));
    }

    #[test]
    fn test_abbreviated_forward_step() {
        assert_debug_snapshot!(parse_expr_single("foo"));
    }

    #[test]
    fn test_abbreviated_forward_step_with_attribute_test() {
        assert_debug_snapshot!(parse_expr_single("foo/attribute()"));
    }

    // XXX should test for attribute axis for SchemaAttributeTest too

    #[test]
    fn test_namespace_node_default_axis() {
        assert_debug_snapshot!(parse_expr_single("foo/namespace-node()"));
    }

    #[test]
    fn test_abbreviated_forward_step_attr() {
        assert_debug_snapshot!(parse_expr_single("@foo"));
    }

    #[test]
    fn test_abbreviated_reverse_step() {
        assert_debug_snapshot!(parse_expr_single("foo/.."));
    }

    #[test]
    fn test_abbreviated_reverse_step_with_predicates() {
        assert_debug_snapshot!(parse_expr_single("..[1]"));
    }

    #[test]
    fn test_starts_single_slash() {
        assert_debug_snapshot!(parse_expr_single("/child::foo"));
    }

    #[test]
    fn test_single_slash_by_itself() {
        assert_debug_snapshot!(parse_expr_single("/"));
    }

    #[test]
    fn test_starts_double_slash() {
        assert_debug_snapshot!(parse_expr_single("//child::foo"));
    }

    #[test]
    fn test_double_slash_middle() {
        assert_debug_snapshot!(parse_expr_single("child::foo//child::bar"));
    }

    #[test]
    fn test_union() {
        assert_debug_snapshot!(parse_expr_single("child::foo | child::bar"));
    }

    #[test]
    fn test_intersect() {
        assert_debug_snapshot!(parse_expr_single("child::foo intersect child::bar"));
    }

    #[test]
    fn test_except() {
        assert_debug_snapshot!(parse_expr_single("child::foo except child::bar"));
    }

    #[test]
    fn test_xpath_parse_error() {
        assert_debug_snapshot!(parse_xpath_no_default_ns("1 + 2 +"));
    }

    #[test]
    fn test_xpath_ge() {
        assert_debug_snapshot!(parse_xpath_no_default_ns("1 >= 2"));
    }

    #[test]
    fn test_signature_without_params() {
        let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
        assert_debug_snapshot!(parse_signature("fn:foo() as xs:integer", &namespaces));
    }

    #[test]
    fn test_signature_without_params2() {
        let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
        assert_debug_snapshot!(parse_signature("fn:foo() as xs:integer*", &namespaces));
    }

    #[test]
    fn test_signature_with_params() {
        let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
        assert_debug_snapshot!(parse_signature(
            "fn:foo($a as xs:decimal*) as xs:integer",
            &namespaces
        ));
    }

    #[test]
    fn test_signature_with_node_param() {
        let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
        assert_debug_snapshot!(parse_signature(
            "fn:foo($a as node()) as xs:integer",
            &namespaces
        ));
    }

    #[test]
    fn test_unary_multiple() {
        assert_debug_snapshot!(parse_expr_single("+-1"));
    }

    #[test]
    fn test_cast_as() {
        assert_debug_snapshot!(parse_expr_single("1 cast as xs:integer"));
    }

    #[test]
    fn test_cast_as_with_question_mark() {
        assert_debug_snapshot!(parse_expr_single("1 cast as xs:integer?"));
    }

    #[test]
    fn test_castable_as() {
        assert_debug_snapshot!(parse_expr_single("1 castable as xs:integer"));
    }

    #[test]
    fn test_castable_as_with_question_mark() {
        assert_debug_snapshot!(parse_expr_single("1 castable as xs:integer?"));
    }

    #[test]
    fn test_instance_of() {
        assert_debug_snapshot!(parse_expr_single("1 instance of xs:integer"));
    }

    #[test]
    fn test_instance_of_with_star() {
        assert_debug_snapshot!(parse_expr_single("1 instance of xs:integer*"));
    }

    #[test]
    fn test_treat() {
        assert_debug_snapshot!(parse_expr_single("1 treat as xs:integer"));
    }

    #[test]
    fn test_treat_with_star() {
        assert_debug_snapshot!(parse_expr_single("1 treat as xs:integer*"));
    }
}
