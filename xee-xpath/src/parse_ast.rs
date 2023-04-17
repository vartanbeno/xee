use crate::parse::XPathParser;
use ordered_float::OrderedFloat;
use pest::iterators::Pair;
use pest::Parser;

use crate::ast;
use crate::parse::Rule;

pub struct Error {}

fn struct_wrap<T, W>(pair: Pair<Rule>, outer_rule: Rule, inner_rule: Rule, wrap: W) -> T
where
    W: Fn(Pair<Rule>) -> T,
{
    debug_assert_eq!(pair.as_rule(), outer_rule);
    let pair = pair.into_inner().next().unwrap();
    if pair.as_rule() == inner_rule {
        wrap(pair)
    } else {
        panic!("unhandled {:?}", pair.as_rule())
    }
}

fn pair_to_path_expr(pair: Pair<Rule>) -> ast::PathExpr {
    let expr_single = expr_single(pair);
    match expr_single {
        ast::ExprSingle::Path(path_expr) => path_expr,
        _ => ast::PathExpr {
            steps: vec![ast::StepExpr::PrimaryExpr(ast::PrimaryExpr::Expr(vec![
                expr_single,
            ]))],
        },
    }
}

fn xpath(pair: Pair<Rule>) -> ast::XPath {
    // let pairs = pair.into_inner();
    // let exprs = pairs.map(expr_single).collect::<Vec<_>>();
    ast::XPath { exprs: exprs(pair) }
}

fn exprs(pair: Pair<Rule>) -> Vec<ast::ExprSingle> {
    let pairs = pair.into_inner();
    pairs.map(expr_single).collect::<Vec<_>>()
}

fn expr_single(pair: Pair<Rule>) -> ast::ExprSingle {
    match pair.as_rule() {
        Rule::PathExpr => ast::ExprSingle::Path(path_expr_to_path_expr(pair)),
        Rule::SimpleMapExpr => {
            let mut pairs = pair.into_inner();
            let path_expr_pair = pairs.next().unwrap();
            let simple_map_path_exprs = pairs.map(pair_to_path_expr).collect::<Vec<_>>();
            if !simple_map_path_exprs.is_empty() {
                let path_expr = pair_to_path_expr(path_expr_pair);
                ast::ExprSingle::Apply(ast::ApplyExpr {
                    path_expr,
                    operator: ast::ApplyOperator::SimpleMap(simple_map_path_exprs),
                })
            } else {
                expr_single(path_expr_pair)
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
                            return expr_single(pair);
                        }
                        let path_expr = pair_to_path_expr(pair);
                        return ast::ExprSingle::Apply(ast::ApplyExpr {
                            path_expr,
                            operator: ast::ApplyOperator::Unary(plus_minus),
                        });
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
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::CastExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::CastableExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::TreatExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::InstanceofExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)

            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::AdditiveExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let op = pairs.next();
            if let Some(op) = op {
                let operator = match op.as_rule() {
                    Rule::Plus => ast::Operator::Add,
                    Rule::Minus => ast::Operator::Sub,
                    _ => {
                        panic!("unhandled AdditiveExpr {:?}", op.as_rule())
                    }
                };
                let right_pair = pairs.next().unwrap();
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::MultiplicativeExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let op = pairs.next();
            if let Some(op) = op {
                let operator = match op.as_rule() {
                    Rule::Mult => ast::Operator::Mul,
                    Rule::Div => ast::Operator::Div,
                    Rule::IDiv => ast::Operator::IDiv,
                    Rule::Mod => ast::Operator::Mod,
                    _ => {
                        panic!("unhandled MultiplicativeExpr {:?}", op.as_rule())
                    }
                };
                let right_pair = pairs.next().unwrap();
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::OrExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let right_pair = pairs.next();
            if let Some(right_pair) = right_pair {
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator: ast::Operator::Or,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::AndExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let right_pair = pairs.next();
            if let Some(right_pair) = right_pair {
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator: ast::Operator::And,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::ComparisonExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let op = pairs.next();
            if let Some(op) = op {
                let operator = match op.as_rule() {
                    Rule::ValueEq => ast::Operator::ValueEq,
                    Rule::ValueNe => ast::Operator::ValueNe,
                    Rule::ValueLt => ast::Operator::ValueLt,
                    Rule::ValueLe => ast::Operator::ValueLe,
                    Rule::ValueGt => ast::Operator::ValueGt,
                    Rule::ValueGe => ast::Operator::ValueGe,
                    Rule::GenEq => ast::Operator::GenEq,
                    Rule::GenNe => ast::Operator::GenNe,
                    Rule::GenLt => ast::Operator::GenLt,
                    Rule::GenLe => ast::Operator::GenLe,
                    Rule::GenGt => ast::Operator::GenGt,
                    Rule::GenGe => ast::Operator::GenGe,
                    Rule::Is => ast::Operator::Is,
                    Rule::Precedes => ast::Operator::Precedes,
                    Rule::Follows => ast::Operator::Follows,
                    _ => {
                        panic!("unhandled ComparisonExpr {:?}", op.as_rule())
                    }
                };
                let right_pair = pairs.next().unwrap();
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::RangeExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let right_pair = pairs.next();
            if let Some(right_pair) = right_pair {
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator: ast::Operator::Range,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::StringConcatExpr => {
            let mut pairs = pair.into_inner();
            let left_pair = pairs.next().unwrap();
            let right_pair = pairs.next();
            if let Some(right_pair) = right_pair {
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator: ast::Operator::Concat,
                    left: pair_to_path_expr(left_pair),
                    right: pair_to_path_expr(right_pair),
                })
            } else {
                expr_single(left_pair)
            }
        }
        Rule::LetExpr => {
            let mut pairs = pair.into_inner();
            let let_clause = pairs.next().unwrap();
            let let_clause_pairs = let_clause.into_inner();
            let inner_return_expr = expr_single(pairs.next().unwrap());
            let mut return_expr = inner_return_expr;
            for let_clause_pair in let_clause_pairs.rev() {
                let mut let_binding = let_clause_pair.into_inner();
                let var_name = let_binding.next().unwrap();
                let var_expr = expr_single(let_binding.next().unwrap());
                let let_expr = ast::LetExpr {
                    var_name: var_name_to_name(var_name),
                    var_expr: Box::new(var_expr),
                    return_expr: Box::new(return_expr),
                };
                return_expr = ast::ExprSingle::Let(let_expr);
            }
            return_expr
        }
        Rule::ForExpr => {
            let mut pairs = pair.into_inner();
            let for_clause = pairs.next().unwrap();
            let for_clause_pairs = for_clause.into_inner();
            let inner_return_expr = expr_single(pairs.next().unwrap());
            let mut return_expr = inner_return_expr;
            for for_clause_pair in for_clause_pairs.rev() {
                let mut for_binding = for_clause_pair.into_inner();
                let var_name = for_binding.next().unwrap();
                let var_expr = expr_single(for_binding.next().unwrap());
                let for_expr = ast::ForExpr {
                    var_name: var_name_to_name(var_name),
                    var_expr: Box::new(var_expr),
                    return_expr: Box::new(return_expr),
                };
                return_expr = ast::ExprSingle::For(for_expr);
            }
            return_expr
        }
        Rule::QuantifiedExpr => {
            let mut pairs = pair.into_inner();
            let quantifier = pairs.next().unwrap();
            let quantifier = match quantifier.as_str() {
                "some" => ast::Quantifier::Some,
                "every" => ast::Quantifier::Every,
                _ => {
                    panic!("unhandled QuantifiedExpr {:?}", quantifier.as_str())
                }
            };
            let quantifier_clause = pairs.next().unwrap();
            let quantifier_clause_pairs = quantifier_clause.into_inner();
            let inner_satisfies_expr = expr_single(pairs.next().unwrap());
            let mut satisfies_expr = inner_satisfies_expr;
            for quantifier_clause_pair in quantifier_clause_pairs.rev() {
                let mut quantifier_binding = quantifier_clause_pair.into_inner();
                let var_name = quantifier_binding.next().unwrap();
                let var_expr = expr_single(quantifier_binding.next().unwrap());
                let quantified_expr = ast::QuantifiedExpr {
                    quantifier: quantifier.clone(),
                    var_name: var_name_to_name(var_name),
                    var_expr: Box::new(var_expr),
                    satisfies_expr: Box::new(satisfies_expr),
                };
                satisfies_expr = ast::ExprSingle::Quantified(quantified_expr);
            }
            satisfies_expr
        }
        Rule::IfExpr => {
            let mut pairs = pair.into_inner();
            let condition_pair = pairs.next().unwrap();
            let condition = exprs(condition_pair);
            let then = expr_single(pairs.next().unwrap());
            let else_ = expr_single(pairs.next().unwrap());
            ast::ExprSingle::If(ast::IfExpr {
                condition,
                then: Box::new(then),
                else_: Box::new(else_),
            })
        }
        Rule::UnionExpr | Rule::IntersectExceptExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)
            // ast::ExprSingle::Path(pair_to_path_expr(pair))
        }
        Rule::ValueExpr => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)
        }
        Rule::ParenthesizedExpr => {
            let pair = pair.into_inner().next().unwrap();
            // pass this along to Rule::Expr
            expr_single(pair)
        }
        Rule::ExprSingle => {
            let pair = pair.into_inner().next().unwrap();
            expr_single(pair)
        }
        Rule::Expr => {
            let exprs = exprs(pair);
            if exprs.len() == 1 {
                exprs[0].clone()
            } else {
                ast::ExprSingle::Path(ast::PathExpr {
                    steps: vec![ast::StepExpr::PrimaryExpr(ast::PrimaryExpr::Expr(exprs))],
                })
            }
        }

        _ => {
            panic!("unhandled ExprSingle {:?}", pair.as_rule())
        }
    }
}

fn path_expr_to_path_expr(pair: Pair<Rule>) -> ast::PathExpr {
    debug_assert_eq!(pair.as_rule(), Rule::PathExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::RelativePathExpr => ast::PathExpr {
            steps: relative_path_expr_to_steps(pair),
        },
        _ => {
            panic!("unhandled PathExpr: {:?}", pair.as_rule())
        }
    }
}

fn relative_path_expr_to_steps(pair: Pair<Rule>) -> Vec<ast::StepExpr> {
    debug_assert_eq!(pair.as_rule(), Rule::RelativePathExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::StepExpr => {
            vec![step_expr_to_step_expr(pair)]
        }
        _ => {
            panic!("unhandled RelativePathExpr: {:?}", pair.as_rule())
        }
    }
}

fn step_expr_to_step_expr(pair: Pair<Rule>) -> ast::StepExpr {
    debug_assert_eq!(pair.as_rule(), Rule::StepExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::PostfixExpr => {
            let mut pairs = pair.into_inner();
            let primary_pair = pairs.next().unwrap();
            let primary = primary_expr_to_primary(primary_pair);
            let postfixes = pairs.map(postfix_expr_to_postfix).collect::<Vec<_>>();
            if postfixes.is_empty() {
                ast::StepExpr::PrimaryExpr(primary)
            } else {
                postfixes_or_placeholdereds(primary, postfixes)
            }
        }
        Rule::AxisStep => {
            let mut pairs = pair.into_inner();
            let step_pair = pairs.next().unwrap();
            let predicates_pair = pairs.next().unwrap();
            let predicates = predicates_pair
                .into_inner()
                .map(predicate_to_expr)
                .collect::<Vec<_>>();
            let (axis, node_test) = match step_pair.as_rule() {
                Rule::ForwardStep => forward_step_to_axis_node_test(step_pair),
                _ => {
                    panic!("unhandled AxisStep: {:?}", step_pair.as_rule())
                }
            };

            let axis_step = ast::AxisStep {
                axis,
                node_test,
                predicates,
            };
            ast::StepExpr::AxisStep(axis_step)
        }
        // XXX handle axis step possibility
        _ => {
            panic!("unhandled StepExpr: {:?}", pair.as_rule())
        }
    }
}

fn forward_step_to_axis_node_test(pair: Pair<Rule>) -> (ast::Axis, ast::NodeTest) {
    debug_assert_eq!(pair.as_rule(), Rule::ForwardStep);
    let mut pairs = pair.into_inner();
    let first_pair = pairs.next().unwrap();
    if first_pair.as_rule() == Rule::ForwardAxis {
        let axis = forward_axis_to_axis(first_pair);
        let node_test_pair = pairs.next().unwrap();
        let node_test = node_test_to_node_test(node_test_pair);
        (axis, node_test)
    } else {
        // abbrev forward step
        todo!();
    }
}

fn forward_axis_to_axis(pair: Pair<Rule>) -> ast::Axis {
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

fn node_test_to_node_test(pair: Pair<Rule>) -> ast::NodeTest {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::KindTest => {
            todo!("not yet KindTest");
        }
        Rule::NameTest => {
            ast::NodeTest::NameTest(name_test_to_name_test(pair.into_inner().next().unwrap()))
        }
        _ => {
            panic!("unhandled NodeTest: {:?}", pair.as_rule())
        }
    }
}

fn name_test_to_name_test(pair: Pair<Rule>) -> ast::NameTest {
    match pair.as_rule() {
        Rule::Wildcard => {
            todo!("no nametest wildcard yet");
        }
        Rule::EQName => ast::NameTest::Name(eq_name_to_name(pair)),
        _ => {
            panic!("unhandled NameTest: {:?}", pair.as_rule())
        }
    }
}

fn primary_expr_to_primary(pair: Pair<Rule>) -> ast::PrimaryExpr {
    debug_assert_eq!(pair.as_rule(), Rule::PrimaryExpr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::Literal => ast::PrimaryExpr::Literal(literal_to_literal(pair)),
        Rule::ParenthesizedExpr => ast::PrimaryExpr::Expr(exprs(pair)),
        Rule::VarRef => {
            let pair = pair.into_inner().next().unwrap();
            ast::PrimaryExpr::VarRef(var_name_to_name(pair))
        }
        Rule::FunctionItemExpr => {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::InlineFunctionExpr => {
                    ast::PrimaryExpr::InlineFunction(inline_function_expr_to_inline_function(pair))
                }
                Rule::NamedFunctionRef => ast::PrimaryExpr::NamedFunctionRef(
                    named_function_ref_to_named_function_ref(pair),
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
            match argument_list_to_args(arguments) {
                ArgumentsOrPlaceholdered::Arguments(arguments) => {
                    ast::PrimaryExpr::FunctionCall(ast::FunctionCall {
                        name: eq_name_to_name(name),
                        arguments,
                    })
                }
                ArgumentsOrPlaceholdered::Placeholdered(arguments, params) => {
                    // construct an inline function that calls the underlying function,
                    // with the reduced placeholdered params
                    ast::PrimaryExpr::InlineFunction(ast::InlineFunction {
                        params,
                        return_type: None,
                        body: vec![ast::ExprSingle::Path(ast::PathExpr {
                            steps: vec![ast::StepExpr::PrimaryExpr(
                                ast::PrimaryExpr::FunctionCall(ast::FunctionCall {
                                    name: eq_name_to_name(name),
                                    arguments,
                                }),
                            )],
                        })],
                    })
                }
            }
        }
        Rule::ContextItemExpr => ast::PrimaryExpr::ContextItem,
        _ => {
            panic!("unhandled PrimaryExpr: {:?}", pair.as_rule())
        }
    }
}

fn postfix_expr_to_postfix(pair: Pair<Rule>) -> PostfixOrPlaceholdered {
    match pair.as_rule() {
        Rule::Predicate => {
            let pair = pair.into_inner().next().unwrap();
            PostfixOrPlaceholdered::Postfix(ast::Postfix::Predicate(exprs(pair)))
        }
        Rule::ArgumentList => match argument_list_to_args(pair) {
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
    primary: ast::PrimaryExpr,
    postfixes: Vec<PostfixOrPlaceholdered>,
) -> ast::StepExpr {
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
                primary = ast::PrimaryExpr::InlineFunction(ast::InlineFunction {
                    params,
                    return_type: None,
                    body: vec![ast::ExprSingle::Path(ast::PathExpr {
                        steps: vec![ast::StepExpr::PostfixExpr {
                            primary,
                            postfixes: normal_postfixes.clone(),
                        }],
                    })],
                });
                // collect more postfixes now
                normal_postfixes.clear();
            }
        }
    }
    if !normal_postfixes.is_empty() {
        ast::StepExpr::PostfixExpr {
            primary,
            postfixes: normal_postfixes,
        }
    } else {
        ast::StepExpr::PrimaryExpr(primary)
    }
}

enum PostfixOrPlaceholdered {
    Postfix(ast::Postfix),
    Placeholdered(Vec<ast::ExprSingle>, Vec<ast::Param>),
}

#[derive(Debug)]
enum ArgumentsOrPlaceholdered {
    Arguments(Vec<ast::ExprSingle>),
    Placeholdered(Vec<ast::ExprSingle>, Vec<ast::Param>),
}

fn argument_list_to_args(pair: Pair<Rule>) -> ArgumentsOrPlaceholdered {
    debug_assert_eq!(pair.as_rule(), Rule::ArgumentList);

    let mut args = vec![];
    let mut placeholder_index = 0;
    let mut params = vec![];
    for pair in pair.into_inner() {
        let expr_single = argument_to_expr_single(pair);
        if let Some(expr_single) = expr_single {
            args.push(expr_single);
        } else {
            // XXX what if someone uses this as a parameter name?
            let param_name = format!("placeholder{}", placeholder_index);
            let name = ast::Name {
                name: param_name,
                namespace: None,
            };
            params.push(ast::Param {
                name: name.clone(),
                type_: None,
            });
            args.push(ast::ExprSingle::Path(ast::PathExpr {
                steps: vec![ast::StepExpr::PrimaryExpr(ast::PrimaryExpr::VarRef(name))],
            }));
            placeholder_index += 1;
        }
    }
    if params.is_empty() {
        ArgumentsOrPlaceholdered::Arguments(args)
    } else {
        ArgumentsOrPlaceholdered::Placeholdered(args, params)
    }
}

fn argument_to_expr_single(pair: Pair<Rule>) -> Option<ast::ExprSingle> {
    debug_assert_eq!(pair.as_rule(), Rule::Argument);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::ExprSingle => Some(expr_single(pair)),
        Rule::ArgumentPlaceholder => None,
        _ => {
            panic!("unhandled argument: {:?}", pair.as_rule())
        }
    }
}

fn predicate_to_expr(pair: Pair<Rule>) -> Vec<ast::ExprSingle> {
    debug_assert_eq!(pair.as_rule(), Rule::Predicate);
    let pair = pair.into_inner().next().unwrap();
    exprs(pair)
}

fn var_name_to_name(pair: Pair<Rule>) -> ast::Name {
    debug_assert_eq!(pair.as_rule(), Rule::VarName);
    // XXX no support for namespaces yet
    ast::Name {
        name: pair.as_str().to_string(),
        namespace: None,
    }
}

fn eq_name_to_name(pair: Pair<Rule>) -> ast::Name {
    debug_assert_eq!(pair.as_rule(), Rule::EQName);
    // XXX no support for namespaces yet
    ast::Name {
        name: pair.as_str().to_string(),
        namespace: None,
    }
}

fn named_function_ref_to_named_function_ref(pair: Pair<Rule>) -> ast::NamedFunctionRef {
    debug_assert_eq!(pair.as_rule(), Rule::NamedFunctionRef);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap();
    let arity = pairs.next().unwrap();
    ast::NamedFunctionRef {
        name: eq_name_to_name(name),
        arity: arity.as_str().parse().unwrap(),
    }
}

fn inline_function_expr_to_inline_function(pair: Pair<Rule>) -> ast::InlineFunction {
    debug_assert_eq!(pair.as_rule(), Rule::InlineFunctionExpr);
    let mut pairs = pair.into_inner();
    let mut next = pairs.next().unwrap();
    let params = if next.as_rule() == Rule::ParamList {
        let params = param_list_to_params(next);
        next = pairs.next().unwrap();
        params
    } else {
        vec![]
    };
    let return_type = if next.as_rule() == Rule::SequenceType {
        panic!("unimplemented: return type");
        // let return_type = sequence_type(next);
        // next = pairs.next().unwrap();
        // Some(return_type)
    } else {
        None
    };
    let body = function_body_to_body(next);
    ast::InlineFunction {
        params,
        return_type,
        body,
    }
}

fn param_list_to_params(pair: Pair<Rule>) -> Vec<ast::Param> {
    debug_assert_eq!(pair.as_rule(), Rule::ParamList);
    let mut parameters = vec![];
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Param => {
                parameters.push(param_to_param(pair));
            }
            _ => {
                panic!("unhandled ParamList: {:?}", pair.as_rule())
            }
        }
    }
    parameters
}

fn param_to_param(pair: Pair<Rule>) -> ast::Param {
    debug_assert_eq!(pair.as_rule(), Rule::Param);
    let mut pairs = pair.into_inner();
    let name = eq_name_to_name(pairs.next().unwrap());
    let type_ = if let Some(pair) = pairs.next() {
        panic!("unhandled type annotation");
    } else {
        None
    };
    ast::Param { name, type_ }
}

fn function_body_to_body(pair: Pair<Rule>) -> Vec<ast::ExprSingle> {
    debug_assert_eq!(pair.as_rule(), Rule::FunctionBody);
    let pair = pair.into_inner().next().unwrap();
    exprs(pair)
}

fn literal_to_literal(pair: Pair<Rule>) -> ast::Literal {
    debug_assert_eq!(pair.as_rule(), Rule::Literal);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::StringLiteral => {
            let pair = pair.into_inner().next().unwrap();
            ast::Literal::String(pair.as_str().to_string())
        }
        Rule::NumericLiteral => numeric_literal_to_literal(pair),
        _ => {
            panic!("unhandled literal: {:?}", pair.as_rule())
        }
    }
}

fn numeric_literal_to_literal(pair: Pair<Rule>) -> ast::Literal {
    debug_assert_eq!(pair.as_rule(), Rule::NumericLiteral);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::IntegerLiteral => {
            let s = pair.as_str();
            // parser never delivers negative numbers
            let i = s.parse::<i64>().unwrap();
            ast::Literal::Integer(i)
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
            ast::Literal::Decimal(ast::DecimalLiteral {
                value: nr,
                fraction_digits: digits as u8,
            })
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

fn parse_rule<T, F>(rule: Rule, input: &str, f: F) -> T
where
    F: Fn(Pair<Rule>) -> T,
{
    let mut pairs = XPathParser::parse(rule, input).unwrap();
    let pair = pairs.next().unwrap();
    f(pair)
}

pub(crate) fn parse_expr_single(input: &str) -> ast::ExprSingle {
    parse_rule(Rule::ExprSingle, input, expr_single)
}

pub(crate) fn parse_xpath(input: &str) -> ast::XPath {
    parse_rule(Rule::Expr, input, xpath)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    fn parse_literal(input: &str) -> ast::Literal {
        parse_rule(Rule::Literal, input, literal_to_literal)
    }

    fn parse_primary_expr(input: &str) -> ast::PrimaryExpr {
        parse_rule(Rule::PrimaryExpr, input, primary_expr_to_primary)
    }

    fn parse_step_expr(input: &str) -> ast::StepExpr {
        parse_rule(Rule::StepExpr, input, step_expr_to_step_expr)
    }

    fn parse_relative_path_expr(input: &str) -> Vec<ast::StepExpr> {
        parse_rule(Rule::RelativePathExpr, input, relative_path_expr_to_steps)
    }

    fn parse_path_expr(input: &str) -> ast::PathExpr {
        parse_rule(Rule::PathExpr, input, path_expr_to_path_expr)
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
        assert_debug_snapshot!(parse_xpath("1 + 2"));
    }

    #[test]
    fn test_xpath_multi_expr() {
        assert_debug_snapshot!(parse_xpath("1 + 2, 3 + 4"));
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
        assert_debug_snapshot!(parse_xpath("1, 2"));
    }

    #[test]
    fn test_complex_comma() {
        assert_debug_snapshot!(parse_xpath("(1, 2), (3, 4)"));
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
}
