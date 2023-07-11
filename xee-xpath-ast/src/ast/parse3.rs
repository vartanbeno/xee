use chumsky::input::Stream;
use chumsky::{extra::Full, input::ValueInput, prelude::*};
use ordered_float::OrderedFloat;
use std::borrow::Cow;
use std::iter::once;

use crate::error::Error;
use crate::lexer::{lexer, Token};
use crate::namespaces::Namespaces;
use crate::span::WithSpan;
use crate::FN_NAMESPACE;

use super::ast_core as ast;

type Span = SimpleSpan;

pub(crate) struct State<'a> {
    namespaces: Cow<'a, Namespaces<'a>>,
}

pub(crate) type Extra<'a, T> = Full<Rich<'a, T>, State<'a>, ()>;

type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a, Token<'a>>>;

#[derive(Clone)]
struct ParserOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    name: BoxedParser<'a, I, ast::NameS>,
    expr_single: BoxedParser<'a, I, ast::ExprSingleS>,
    xpath: BoxedParser<'a, I, ast::XPath>,
}

fn parser<'a, I>() -> ParserOutput<'a, I>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let ncname = select! {
        Token::NCName(s) => s,

    };

    let braced_uri_literal = select! {
        Token::BracedURILiteral(s) => s,
    };

    // PrefixedName ::= Prefix ':' LocalPart
    let prefixed_name = ncname
        .then_ignore(just(Token::Colon))
        .then(ncname)
        .try_map_with_state(|(prefix, local_name), span, state: &mut State| {
            ast::Name::prefixed(prefix, local_name, state.namespaces.as_ref())
                .map(|name| name.with_span(span))
                .ok_or_else(|| Rich::custom(span, format!("Unknown prefix: {}", prefix)))
        });

    // QName ::= PrefixedName | UnprefixedName
    let qname = prefixed_name
        .or(ncname
            .map_with_span(|local_name, span| ast::Name::unprefixed(local_name).with_span(span)))
        .boxed();

    let uri_qualified_name =
        braced_uri_literal
            .then(ncname)
            .map_with_span(|(uri, local_name), span| {
                ast::Name::uri_qualified(uri, local_name).with_span(span)
            });

    let eqname = qname.or(uri_qualified_name).boxed();

    let string = select! {
        Token::StringLiteral(s) => s,
    };
    let string_literal = string.map(|s| ast::Literal::String(s.to_string())).boxed();

    let integer = select! {
        Token::IntegerLiteral(i) => i,
    };

    let integer_literal = integer.map(ast::Literal::Integer).boxed();

    let decimal_literal = select! {
        Token::DecimalLiteral(d) => d,
    }
    .map(ast::Literal::Decimal)
    .boxed();

    let double_literal = select! {
        Token::DoubleLiteral(d) => d,
    }
    .map(|d| ast::Literal::Double(OrderedFloat(d)))
    .boxed();

    let literal = string_literal
        .or(integer_literal.clone())
        .or(decimal_literal)
        .or(double_literal)
        .map_with_span(|literal, span| ast::PrimaryExpr::Literal(literal).with_span(span))
        .boxed();

    let var_ref = just(Token::Dollar)
        .ignore_then(eqname.clone())
        .map_with_span(|name, span| ast::PrimaryExpr::VarRef(name.value).with_span(span))
        .boxed();

    let context_item_expr = just(Token::Dot)
        .map_with_span(|_, span| ast::PrimaryExpr::ContextItem.with_span(span))
        .boxed();

    let single_type = eqname
        .clone()
        .then(just(Token::QuestionMark).or_not())
        .map_with_span(|(name, question_mark), _span| ast::SingleType {
            name,
            question_mark: question_mark.is_some(),
        });

    let empty_call = just(Token::LeftParen)
        .ignore_then(just(Token::RightParen))
        .boxed();

    let empty = just(Token::EmptySequence)
        .ignore_then(empty_call.clone())
        .to(ast::SequenceType::Empty);
    let occurrence = one_of([Token::QuestionMark, Token::Asterisk, Token::Plus])
        .map(|c| match c {
            Token::QuestionMark => ast::Occurrence::Option,
            Token::Asterisk => ast::Occurrence::Many,
            Token::Plus => ast::Occurrence::NonEmpty,
            _ => unreachable!(),
        })
        .or_not()
        .map(|o| o.unwrap_or(ast::Occurrence::One))
        .boxed();

    let item_type = recursive(|item_type| {
        just(Token::Item)
            .ignore_then(empty_call.clone())
            .to(ast::ItemType::Item)
            .or(eqname
                .clone()
                .map_with_span(|name, _span| ast::ItemType::AtomicOrUnionType(name)))
            .or(item_type.delimited_by(just(Token::LeftParen), just(Token::RightParen)))
    })
    .boxed();

    let item = item_type
        .clone()
        .then(occurrence)
        .map(|(item_type, occurrence)| ast::Item {
            item_type,
            occurrence,
        });

    let sequence_type = empty.or(item.map(ast::SequenceType::Item)).boxed();

    let element_declaration = eqname.clone();
    let schema_element_test = just(Token::SchemaElement)
        .ignore_then(
            element_declaration.delimited_by(just(Token::LeftParen), just(Token::RightParen)),
        )
        .map(|name| ast::SchemaElementTest { name });

    let element_name_or_wildcard = just(Token::Asterisk)
        .to(ast::ElementNameOrWildcard::Wildcard)
        .or(eqname.clone().map(ast::ElementNameOrWildcard::Name));

    let type_name = eqname.clone();

    let element_type_name = type_name
        .clone()
        .then(just(Token::QuestionMark).or_not())
        .map(|(name, question_mark)| ast::ElementTypeName {
            name,
            question_mark: question_mark.is_some(),
        });

    let element_test_content = element_name_or_wildcard
        .then((just(Token::Comma).ignore_then(element_type_name)).or_not())
        .map(|(name_test, type_name)| ast::ElementTest {
            name_test,
            type_name,
        });

    let element_test = just(Token::Element)
        .ignore_then(
            element_test_content
                .or_not()
                .delimited_by(just(Token::LeftParen), just(Token::RightParen)),
        )
        .boxed();

    let document_test_content = element_test
        .clone()
        .map(ast::DocumentTest::Element)
        .or(schema_element_test
            .clone()
            .map(ast::DocumentTest::SchemaElement))
        .boxed();

    let document_test = just(Token::DocumentNode)
        .ignore_then(
            document_test_content
                .or_not()
                .delimited_by(just(Token::LeftParen), just(Token::RightParen)),
        )
        .boxed();

    let attrib_name_or_wildcard = just(Token::Asterisk)
        .to(ast::AttribNameOrWildcard::Wildcard)
        .or(eqname.clone().map(ast::AttribNameOrWildcard::Name));

    let attribute_test_content = attrib_name_or_wildcard
        .then((just(Token::Comma).ignore_then(type_name)).or_not())
        .map(|(name_test, type_name)| ast::AttributeTest {
            name_test,
            type_name,
        });

    let attribute_test = just(Token::Attribute).ignore_then(
        attribute_test_content
            .or_not()
            .delimited_by(just(Token::LeftParen), just(Token::RightParen)),
    );

    let any_test = just(Token::Node)
        .ignore_then(empty_call.clone())
        .to(ast::KindTest::Any)
        .boxed();

    let attribute_name = eqname.clone();
    let attribute_declaration = attribute_name;
    let schema_attribute_test = just(Token::SchemaAttribute)
        .ignore_then(
            attribute_declaration.delimited_by(just(Token::LeftParen), just(Token::RightParen)),
        )
        .map(|name| ast::SchemaAttributeTest { name });

    let pi_test_content = ncname
        .map(|s| ast::PITest::Name(s.to_string()))
        .or(string.map(|s| ast::PITest::StringLiteral(s.to_string())));

    let pi_test = just(Token::ProcessingInstruction).ignore_then(
        pi_test_content
            .or_not()
            .delimited_by(just(Token::LeftParen), just(Token::RightParen)),
    );

    let text_test = just(Token::Text)
        .ignore_then(empty_call.clone())
        .to(ast::KindTest::Text);
    let comment_test = just(Token::Comment)
        .ignore_then(empty_call.clone())
        .to(ast::KindTest::Comment);
    let namespace_node_test = just(Token::NamespaceNode)
        .ignore_then(empty_call.clone())
        .to(ast::KindTest::NamespaceNode);

    let kind_test = document_test
        .map(ast::KindTest::Document)
        .or(element_test.map(ast::KindTest::Element))
        .or(attribute_test.map(ast::KindTest::Attribute))
        .or(schema_element_test.map(ast::KindTest::SchemaElement))
        .or(schema_attribute_test.map(ast::KindTest::SchemaAttribute))
        .or(pi_test.map(ast::KindTest::PI))
        .or(comment_test)
        .or(text_test)
        .or(namespace_node_test)
        .or(any_test)
        .boxed();

    let wildcard_ncname = ncname
        .then_ignore(just(Token::ColonAsterisk))
        .try_map_with_state(|prefix, span, state: &mut State| {
            let namespace = state
                .namespaces
                .by_prefix(prefix)
                .ok_or_else(|| Rich::custom(span, format!("Unknown prefix: {}", prefix)))?;
            Ok(ast::NameTest::Namespace(namespace.to_string()))
        });
    let wildcard_braced_uri_literal = braced_uri_literal
        .then_ignore(just(Token::Asterisk))
        .map(|uri| ast::NameTest::Namespace(uri.to_string()));
    let wildcard_localname = just(Token::AsteriskColon)
        .ignore_then(ncname)
        .map(|name| ast::NameTest::LocalName(name.to_string()));
    let wildcard_star = just(Token::Asterisk).to(ast::NameTest::Star);

    let wildcard = wildcard_ncname
        .or(wildcard_braced_uri_literal)
        .or(wildcard_localname)
        .or(wildcard_star)
        .boxed();

    let name_test = wildcard.or(eqname.clone().map(ast::NameTest::Name));

    let parent_axis = just(Token::Parent)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Parent);
    let ancestor_axis = just(Token::Ancestor)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Ancestor);
    let preceding_sibling_axis = just(Token::PrecedingSibling)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::PrecedingSibling);
    let preceding_axis = just(Token::Preceding)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Preceding);
    let ancestor_or_self_axis = just(Token::AncestorOrSelf)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::AncestorOrSelf);

    let reverse_axis = choice::<_>([
        parent_axis,
        ancestor_axis,
        preceding_sibling_axis,
        preceding_axis,
        ancestor_or_self_axis,
    ])
    .boxed();

    let node_test = name_test
        .map(ast::NodeTest::NameTest)
        .or(kind_test.map(ast::NodeTest::KindTest));

    let abbrev_reverse_step = just(Token::DotDot).to((
        ast::Axis::Parent,
        ast::NodeTest::KindTest(ast::KindTest::Any),
    ));

    let reverse_axis_with_node_test = reverse_axis.then(node_test.clone()).boxed();
    let reverse_step = reverse_axis_with_node_test.or(abbrev_reverse_step).boxed();

    let child_axis = just(Token::Child)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Child);
    let descendant_axis = just(Token::Descendant)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Descendant);
    let attribute_axis = just(Token::Attribute)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Attribute);
    let self_axis = just(Token::Self_)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Self_);
    let descendant_or_self_axis = just(Token::DescendantOrSelf)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::DescendantOrSelf);
    let following_sibling_axis = just(Token::FollowingSibling)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::FollowingSibling);
    let following_axis = just(Token::Following)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Following);
    let namespace_axis = just(Token::Namespace)
        .ignore_then(just(Token::DoubleColon))
        .to(ast::Axis::Namespace);

    let forward_axis = choice::<_>([
        child_axis,
        descendant_axis,
        attribute_axis,
        self_axis,
        descendant_or_self_axis,
        following_sibling_axis,
        following_axis,
        namespace_axis,
    ])
    .boxed();

    let forward_step_with_node_test = forward_axis.then(node_test.clone()).boxed();

    let abbrev_forward_step = just(Token::At)
        .or_not()
        .then(node_test.clone())
        .map(|(at, node_test)| {
            if at.is_some() {
                (ast::Axis::Attribute, node_test)
            } else {
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
        })
        .boxed();

    let forward_step = forward_step_with_node_test.or(abbrev_forward_step).boxed();

    // ugly way to get expr out of recursive
    let mut expr_ = None;

    let expr_single = recursive(|expr_single| {
        let expr = expr_single
            .clone()
            .separated_by(just(Token::Comma))
            .at_least(1)
            .collect::<Vec<_>>()
            .map_with_span(|exprs, span| ast::Expr(exprs).with_span(span))
            .boxed();

        expr_ = Some(expr.clone());

        // TODO: handle empty parenthesized expr which means empty sequence
        let parenthesized_expr = expr
            .clone()
            .delimited_by(just(Token::LeftParen), just(Token::RightParen))
            .boxed()
            .map_with_span(|expr, span| ast::PrimaryExpr::Expr(expr).with_span(span));

        let argument_placeholder = just(Token::QuestionMark)
            .map(|_| ArgumentOrPlaceholder::Placeholder)
            .boxed();
        let argument = expr_single
            .clone()
            .map(ArgumentOrPlaceholder::Argument)
            .or(argument_placeholder)
            .boxed();
        let argument_list = argument
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LeftParen), just(Token::RightParen));

        enum PostfixOrPlaceholderWrapper {
            Postfix(ast::Postfix),
            PlaceholderWrapper(Vec<ast::ExprSingleS>, Vec<ast::Param>, Span),
        }

        let predicate = expr
            .clone()
            .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
            .map(ast::Postfix::Predicate)
            .map(PostfixOrPlaceholderWrapper::Postfix)
            .boxed();

        let argument_list_postfix = argument_list
            .clone()
            .map_with_span(|arguments, span| {
                let (arguments, params) = placeholder_arguments(&arguments);
                if params.is_empty() {
                    PostfixOrPlaceholderWrapper::Postfix(ast::Postfix::ArgumentList(arguments))
                } else {
                    PostfixOrPlaceholderWrapper::PlaceholderWrapper(arguments, params, span)
                }
            })
            .boxed();

        let postfix = predicate.or(argument_list_postfix).boxed();

        let function_call = eqname
            .clone()
            .then(argument_list)
            .map_with_span(move |(name, arguments), span| {
                let (arguments, params) = placeholder_arguments(&arguments);
                if params.is_empty() {
                    ast::PrimaryExpr::FunctionCall(ast::FunctionCall { name, arguments })
                        .with_span(span)
                } else {
                    let inner_function_call =
                        ast::PrimaryExpr::FunctionCall(ast::FunctionCall { name, arguments })
                            .with_empty_span();
                    let step_expr =
                        ast::StepExpr::PrimaryExpr(inner_function_call).with_empty_span();
                    placeholder_wrapper_function(step_expr, params, span)
                }
            })
            .boxed();

        let named_function_ref = eqname
            .clone()
            .then_ignore(just(Token::Hash))
            .then(integer)
            .map_with_span(|(name, arity), span| {
                ast::PrimaryExpr::NamedFunctionRef(ast::NamedFunctionRef {
                    name,
                    // TODO: handle overflow
                    arity: arity.try_into().unwrap(),
                })
                .with_span(span)
            })
            .boxed();

        let type_declaration = just(Token::As).ignore_then(sequence_type.clone());

        let param = just(Token::Dollar)
            .ignore_then(eqname.clone())
            .then(type_declaration.or_not())
            .map(|(name, type_)| ast::Param {
                name: name.value,
                type_,
            });

        let param_list = param
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .boxed();

        let enclosed_expr =
            (expr.clone().or_not()).delimited_by(just(Token::LeftBrace), just(Token::RightBrace));

        let function_body = enclosed_expr;

        let inline_function_expr = just(Token::Function)
            .ignore_then(param_list.delimited_by(just(Token::LeftParen), just(Token::RightParen)))
            .then(just(Token::As).ignore_then(sequence_type.clone()).or_not())
            .then(function_body)
            .map_with_span(|((params, return_type), body), span| {
                ast::PrimaryExpr::InlineFunction(ast::InlineFunction {
                    params,
                    return_type,
                    body,
                })
                .with_span(span)
            })
            .boxed();

        let primary_expr = parenthesized_expr
            .or(literal)
            .or(var_ref)
            .or(context_item_expr)
            .or(named_function_ref)
            .or(inline_function_expr)
            .or(function_call)
            .boxed();

        let postfix_expr = primary_expr
            .then(postfix.repeated().collect::<Vec<_>>())
            .map_with_span(|(primary, postfixes), span| {
                // in case of a placeholder argument list we need to
                // wrap the existing primary
                let mut normal_postfixes = Vec::new();
                let mut primary = primary;
                for postfix in postfixes {
                    match postfix {
                        PostfixOrPlaceholderWrapper::Postfix(postfix) => {
                            normal_postfixes.push(postfix)
                        }
                        PostfixOrPlaceholderWrapper::PlaceholderWrapper(
                            arguments,
                            params,
                            span,
                        ) => {
                            normal_postfixes.push(ast::Postfix::ArgumentList(arguments));
                            let step_expr = ast::StepExpr::PostfixExpr {
                                primary,
                                postfixes: normal_postfixes.clone(),
                            }
                            .with_empty_span();
                            // replace primary with a placeholder wrapper function
                            primary = placeholder_wrapper_function(step_expr, params, span);
                            // now collect more postfixes
                            normal_postfixes.clear();
                        }
                    }
                }
                if normal_postfixes.is_empty() {
                    ast::StepExpr::PrimaryExpr(primary).with_span(span)
                } else {
                    ast::StepExpr::PostfixExpr {
                        primary,
                        postfixes: normal_postfixes,
                    }
                    .with_span(span)
                }
            })
            .boxed();

        let predicate = expr
            .clone()
            .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
            .boxed();

        let predicate_list = predicate.repeated().collect::<Vec<_>>().boxed();

        let axis_step = (reverse_step.or(forward_step))
            .then(predicate_list)
            .map_with_span(|((axis, node_test), predicates), span| {
                ast::StepExpr::AxisStep(ast::AxisStep {
                    axis,
                    node_test,
                    predicates,
                })
                .with_span(span)
            })
            .boxed();

        let step_expr = postfix_expr.or(axis_step).boxed();

        let relative_path_expr = step_expr
            .clone()
            .then(
                just(Token::Slash)
                    .or(just(Token::DoubleSlash))
                    .then(step_expr.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(first_step, rest_steps)| {
                let mut steps = vec![first_step];
                for (token, step) in rest_steps {
                    match token {
                        Token::Slash => {}
                        Token::DoubleSlash => {
                            steps.push(
                                ast::StepExpr::AxisStep(ast::AxisStep {
                                    axis: ast::Axis::DescendantOrSelf,
                                    node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                                    predicates: vec![],
                                })
                                .with_empty_span(),
                            );
                        }
                        _ => unreachable!(),
                    }
                    steps.push(step);
                }
                steps
            })
            .boxed();

        let slash_prefix_path_expr = just(Token::Slash)
            .map_with_span(|_, span| span)
            .then(relative_path_expr.clone().or_not())
            .map(|(slash_span, steps)| {
                let root_step = root_step(slash_span);
                if let Some(steps) = steps {
                    let all_steps = once(root_step).chain(steps.into_iter()).collect();
                    ast::PathExpr { steps: all_steps }
                } else {
                    ast::PathExpr {
                        steps: vec![root_step],
                    }
                }
            })
            .boxed();

        let doubleslash_prefix_path_expr = just(Token::DoubleSlash)
            .map_with_span(|_, span| span)
            .then(relative_path_expr.clone().or_not())
            .map(|(double_slash_span, steps)| {
                let root_step = root_step(double_slash_span);
                let descendant_step = ast::StepExpr::AxisStep(ast::AxisStep {
                    axis: ast::Axis::DescendantOrSelf,
                    node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                    predicates: vec![],
                })
                .with_span(double_slash_span);
                if let Some(steps) = steps {
                    let all_steps = once(root_step)
                        .chain(once(descendant_step).chain(steps.into_iter()))
                        .collect();
                    ast::PathExpr { steps: all_steps }
                } else {
                    ast::PathExpr {
                        steps: vec![root_step, descendant_step],
                    }
                }
            })
            .boxed();

        let path_expr = doubleslash_prefix_path_expr
            .or(slash_prefix_path_expr)
            .or(relative_path_expr.map(|steps| ast::PathExpr { steps }))
            .boxed();

        let value_expr = path_expr
            .clone()
            .separated_by(just(Token::ExclamationMark))
            .at_least(1)
            .collect::<Vec<_>>()
            .map_with_span(|path_exprs, span| {
                if path_exprs.len() == 1 {
                    ast::ExprSingle::Path(path_exprs[0].clone()).with_span(span)
                } else {
                    ast::ExprSingle::Apply(ast::ApplyExpr {
                        operator: ast::ApplyOperator::SimpleMap(path_exprs[1..].to_vec()),
                        path_expr: path_exprs[0].clone(),
                    })
                    .with_span(span)
                }
            })
            .boxed();

        let unary_operator = just(Token::Minus)
            .to(ast::UnaryOperator::Minus)
            .or(just(Token::Plus).to(ast::UnaryOperator::Plus));

        let unary_expr = unary_operator
            .repeated()
            .collect::<Vec<_>>()
            .then(value_expr.clone())
            .map_with_span(|(unary_operators, expr), span| {
                if unary_operators.is_empty() {
                    expr
                } else {
                    ast::ExprSingle::Apply(ast::ApplyExpr {
                        operator: ast::ApplyOperator::Unary(unary_operators),
                        path_expr: expr_single_to_path_expr(expr),
                    })
                    .with_span(span)
                }
            })
            .boxed();

        // // TODO
        let arrow_expr = unary_expr;
        let cast_expr = arrow_expr
            .then(
                just(Token::Cast)
                    .ignore_then(just(Token::As))
                    .ignore_then(single_type.clone())
                    .or_not(),
            )
            .map_with_span(|(expr, single_type), span| {
                if let Some(single_type) = single_type {
                    ast::ExprSingle::Apply(ast::ApplyExpr {
                        path_expr: expr_single_to_path_expr(expr),
                        operator: ast::ApplyOperator::Cast(single_type),
                    })
                    .with_span(span)
                } else {
                    expr
                }
            })
            .boxed();

        let castable_expr = cast_expr
            .then(
                just(Token::Castable)
                    .ignore_then(just(Token::As))
                    .ignore_then(single_type)
                    .or_not(),
            )
            .map_with_span(|(expr, single_type), span| {
                if let Some(single_type) = single_type {
                    ast::ExprSingle::Apply(ast::ApplyExpr {
                        path_expr: expr_single_to_path_expr(expr),
                        operator: ast::ApplyOperator::Castable(single_type),
                    })
                    .with_span(span)
                } else {
                    expr
                }
            })
            .boxed();

        let treat_expr = castable_expr
            .then(
                just(Token::Treat)
                    .ignore_then(just(Token::As))
                    .ignore_then(sequence_type.clone())
                    .or_not(),
            )
            .map_with_span(|(expr, sequence_type), span| {
                if let Some(sequence_type) = sequence_type {
                    ast::ExprSingle::Apply(ast::ApplyExpr {
                        path_expr: expr_single_to_path_expr(expr),
                        operator: ast::ApplyOperator::Treat(sequence_type),
                    })
                    .with_span(span)
                } else {
                    expr
                }
            })
            .boxed();

        let instance_of_expr = treat_expr
            .then(
                just(Token::Instance)
                    .ignore_then(just(Token::Of))
                    .ignore_then(sequence_type)
                    .or_not(),
            )
            .map_with_span(|(expr, sequence_type), span| {
                if let Some(sequence_type) = sequence_type {
                    ast::ExprSingle::Apply(ast::ApplyExpr {
                        path_expr: expr_single_to_path_expr(expr),
                        operator: ast::ApplyOperator::InstanceOf(sequence_type),
                    })
                    .with_span(span)
                } else {
                    expr
                }
            })
            .boxed();

        let intersect_except_operator = just(Token::Intersect)
            .to(ast::BinaryOperator::Intersect)
            .or(just(Token::Except).to(ast::BinaryOperator::Except))
            .boxed();

        let intersect_except_expr =
            binary_expr_op(instance_of_expr, intersect_except_operator).boxed();

        let union_operator = just(Token::Pipe)
            .map(|_| ast::BinaryOperator::Union)
            .or(just(Token::Union).map(|_| ast::BinaryOperator::Union))
            .boxed();

        let union_expr = binary_expr_op(intersect_except_expr, union_operator).boxed();

        let multiplicative_operator = choice::<_>([
            just(Token::Asterisk).to(ast::BinaryOperator::Mul),
            just(Token::Div).to(ast::BinaryOperator::Div),
            just(Token::Idiv).to(ast::BinaryOperator::IntDiv),
            just(Token::Mod).to(ast::BinaryOperator::Mod),
        ])
        .boxed();

        let multiplicative_expr = binary_expr_op(union_expr, multiplicative_operator).boxed();

        let additive_operator = one_of([Token::Plus, Token::Minus])
            .map(|c| match c {
                Token::Plus => ast::BinaryOperator::Add,
                Token::Minus => ast::BinaryOperator::Sub,
                _ => unreachable!(),
            })
            .boxed();
        let additive_expr = binary_expr_op(multiplicative_expr, additive_operator).boxed();

        let range_expr = binary_expr(additive_expr, Token::To, ast::BinaryOperator::Range).boxed();
        let string_concat_expr =
            binary_expr(range_expr, Token::DoublePipe, ast::BinaryOperator::Concat).boxed();
        use ast::BinaryOperator::*;

        let comparison_operator = choice::<_>([
            just(Token::Equal).to(GenEq),
            just(Token::NotEqual).to(GenNe),
            just(Token::LessThan).to(GenLt),
            just(Token::LessThanEqual).to(GenLe),
            just(Token::GreaterThan).to(GenGt),
            just(Token::GreaterThanEqual).to(GenGe),
            just(Token::Eq).to(ValueEq),
            just(Token::Ne).to(ValueNe),
            just(Token::Lt).to(ValueLt),
            just(Token::Le).to(ValueLe),
            just(Token::Gt).to(ValueGt),
            just(Token::Ge).to(ValueGe),
            just(Token::Is).to(Is),
            just(Token::Precedes).to(Precedes),
            just(Token::Follows).to(Follows),
        ])
        .boxed();

        let comparison_expr = binary_expr_op(string_concat_expr, comparison_operator).boxed();
        let and_expr = binary_expr(comparison_expr, Token::And, ast::BinaryOperator::And).boxed();
        let or_expr = binary_expr(and_expr, Token::Or, ast::BinaryOperator::Or).boxed();

        let path_expr = or_expr
            .map_with_span(|expr_single, span| {
                ast::ExprSingle::Path(expr_single_to_path_expr(expr_single)).with_span(span)
            })
            .boxed();

        let simple_let_binding = just(Token::Dollar)
            .ignore_then(eqname.clone())
            .then_ignore(just(Token::ColonEqual))
            .then(expr_single.clone())
            .boxed();

        let simple_let_clause = just(Token::Let).ignore_then(
            simple_let_binding
                .clone()
                .separated_by(just(Token::Comma))
                .at_least(1)
                .collect::<Vec<_>>(),
        );

        let let_expr = simple_let_clause
            .then_ignore(just(Token::Return))
            .then(expr_single.clone())
            .map_with_span(|(bindings, return_expr), span| {
                bindings
                    .iter()
                    .rev()
                    .fold(return_expr, |return_expr, (var_name, var_expr)| {
                        ast::ExprSingle::Let(ast::LetExpr {
                            var_name: var_name.clone(),
                            var_expr: Box::new(var_expr.clone()),
                            return_expr: Box::new(return_expr),
                        })
                        .with_span(span)
                    })
            })
            .boxed();

        let simple_for_binding = just(Token::Dollar)
            .ignore_then(eqname.clone())
            .then_ignore(just(Token::In))
            .then(expr_single.clone())
            .boxed();

        let for_bindings = simple_for_binding
            .clone()
            .separated_by(just(Token::Comma))
            .at_least(1)
            .collect::<Vec<_>>();

        let simple_for_clause = just(Token::For).ignore_then(for_bindings.clone());

        let for_expr = simple_for_clause
            .clone()
            .then_ignore(just(Token::Return))
            .then(expr_single.clone())
            .map_with_span(|(bindings, return_expr), span| {
                bindings
                    .iter()
                    .rev()
                    .fold(return_expr, |return_expr, (var_name, var_expr)| {
                        ast::ExprSingle::For(ast::ForExpr {
                            var_name: var_name.clone(),
                            var_expr: Box::new(var_expr.clone()),
                            return_expr: Box::new(return_expr),
                        })
                        .with_span(span)
                    })
            })
            .boxed();

        let if_expr = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr_single.clone())
            .then_ignore(just(Token::Else))
            .then(expr_single.clone())
            .map_with_span(|((condition, then), else_), span| {
                ast::ExprSingle::If(ast::IfExpr {
                    condition,
                    then: Box::new(then),
                    else_: Box::new(else_),
                })
                .with_span(span)
            })
            .boxed();

        let quantified_expr = choice::<_>([
            just(Token::Some).to(ast::Quantifier::Some),
            just(Token::Every).to(ast::Quantifier::Every),
        ])
        .then(for_bindings.clone())
        .then_ignore(just(Token::Satisfies))
        .then(expr_single)
        .map_with_span(|((quantifier, bindings), satisfies_expr), span| {
            bindings
                .iter()
                .rev()
                .fold(satisfies_expr, |satisfies_expr, (var_name, var_expr)| {
                    ast::ExprSingle::Quantified(ast::QuantifiedExpr {
                        quantifier: quantifier.clone(),
                        var_name: var_name.clone(),
                        var_expr: Box::new(var_expr.clone()),
                        satisfies_expr: Box::new(satisfies_expr),
                    })
                    .with_span(span)
                })
        })
        .boxed();

        let expr_single_ = path_expr
            .or(let_expr)
            .or(for_expr)
            .or(if_expr)
            .or(quantified_expr)
            .boxed();

        expr_single_
    })
    .boxed();

    let name = eqname.clone().then_ignore(end()).boxed();
    let expr_single = expr_single.then_ignore(end()).boxed();
    let xpath = expr_.unwrap().then_ignore(end()).map(ast::XPath).boxed();

    ParserOutput {
        name,
        expr_single,
        xpath,
    }
}

fn binary_expr<'a, I>(
    sub_expr: BoxedParser<'a, I, ast::ExprSingleS>,
    operator_token: Token<'a>,
    operator: ast::BinaryOperator,
) -> BoxedParser<'a, I, ast::ExprSingleS>
where
    I: Input<'a, Token = Token<'a>, Span = Span> + ValueInput<'a>,
{
    binary_expr_op(
        sub_expr,
        just(operator_token).map(move |_| operator).boxed(),
    )
}

fn binary_expr_op<'a, I>(
    sub_expr: BoxedParser<'a, I, ast::ExprSingleS>,
    operator: BoxedParser<'a, I, ast::BinaryOperator>,
) -> BoxedParser<'a, I, ast::ExprSingleS>
where
    I: Input<'a, Token = Token<'a>, Span = Span> + ValueInput<'a>,
{
    sub_expr
        .clone()
        .foldl(
            operator.then(sub_expr).repeated(),
            move |left, (operator, right)| {
                let span: SimpleSpan = (left.span.start..right.span.end).into();
                ast::ExprSingle::Binary(ast::BinaryExpr {
                    operator,
                    left: expr_single_to_path_expr(left),
                    right: expr_single_to_path_expr(right),
                })
                .with_span(span)
            },
        )
        .boxed()
}

fn expr_single_to_path_expr(expr: ast::ExprSingleS) -> ast::PathExpr {
    let span = expr.span;
    match expr.value {
        ast::ExprSingle::Path(path) => path,
        _ => ast::PathExpr {
            steps: vec![ast::StepExpr::PrimaryExpr(
                ast::PrimaryExpr::Expr(ast::Expr(vec![expr]).with_span(span)).with_span(span),
            )
            .with_span(span)],
        },
    }
}

fn root_step(span: Span) -> ast::StepExprS {
    let path_arg = ast::ExprSingle::Path(ast::PathExpr {
        steps: vec![ast::StepExpr::AxisStep(ast::AxisStep {
            axis: ast::Axis::Self_,
            node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
            predicates: vec![],
        })
        .with_empty_span()],
    })
    .with_empty_span();

    ast::StepExpr::PrimaryExpr(
        ast::PrimaryExpr::FunctionCall(ast::FunctionCall {
            name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string()))
                .with_empty_span(),
            arguments: vec![path_arg],
        })
        .with_empty_span(),
    )
    .with_span(span)
}

enum ArgumentOrPlaceholder {
    Argument(ast::ExprSingleS),
    Placeholder,
}

// given a list of entries, each an argument or a placeholder, split this into
// a list of real arguments and a list of parameters to construct for the new
// function without the placeholders. If this list of parameters is empty, no
// wrapping placeholder function is constructed.
fn placeholder_arguments(
    aps: &[ArgumentOrPlaceholder],
) -> (Vec<ast::ExprSingleS>, Vec<ast::Param>) {
    let mut placeholder_index = 0;
    let mut arguments = Vec::new();
    let mut params = Vec::new();
    for argument_or_placeholder in aps.iter() {
        match argument_or_placeholder {
            ArgumentOrPlaceholder::Argument(expr) => {
                arguments.push(expr.clone());
            }
            ArgumentOrPlaceholder::Placeholder => {
                // XXX what if someone uses this as a parameter name?
                let param_name = format!("placeholder{}", placeholder_index);
                placeholder_index += 1;
                let name = ast::Name::unprefixed(&param_name);
                let param = ast::Param {
                    name: name.clone(),
                    type_: None,
                };
                params.push(param);
                arguments.push(
                    ast::ExprSingle::Path(ast::PathExpr {
                        steps: vec![ast::StepExpr::PrimaryExpr(
                            ast::PrimaryExpr::VarRef(name).with_empty_span(),
                        )
                        .with_empty_span()],
                    })
                    .with_empty_span(),
                );
            }
        }
    }
    (arguments, params)
}

// construct an inline function that calls the underlying
// function with the reduced placeholdered params
fn placeholder_wrapper_function(
    step_expr: ast::StepExprS,
    params: Vec<ast::Param>,
    span: Span,
) -> ast::PrimaryExprS {
    let path_expr = ast::PathExpr {
        steps: vec![step_expr],
    };
    let expr_single = ast::ExprSingle::Path(path_expr).with_empty_span();
    let body = ast::Expr(vec![expr_single]).with_empty_span();
    ast::PrimaryExpr::InlineFunction(ast::InlineFunction {
        params,
        return_type: None,
        body: Some(body),
    })
    .with_span(span)
}

fn create_token_iter(src: &str) -> impl Iterator<Item = (Token, SimpleSpan)> + '_ {
    lexer(src).map(|(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Token::Error, span.into()),
    })
}

fn tokens(src: &str) -> impl ValueInput<'_, Token = Token<'_>, Span = Span> {
    Stream::from_iter(create_token_iter(src)).spanned((src.len()..src.len()).into())
}

#[derive(Debug)]
pub struct ParseError<'a> {
    errors: Vec<Rich<'a, Token<'a>>>,
}

#[cfg(test)]
impl serde::Serialize for ParseError<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let formatted = format!("{:?}", self.errors);
        serializer.serialize_str(&formatted)

        // let mut errors = serializer.serialize_struct("ParseError", 1)?;
        // now output formatted as serialized
        // use serde::ser::SerializeStruct;
        // // errors.serialize_field("errors", &formatted)?;
        // errors.end()
    }
}

fn parse<'a, I, T>(
    parser: BoxedParser<'a, I, T>,
    input: I,
    namespaces: Cow<'a, Namespaces<'a>>,
) -> Result<T, ParseError<'a>>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    T: std::fmt::Debug,
{
    let mut state = State { namespaces };
    parser
        .parse_with_state(input, &mut state)
        .into_result()
        .map_err(|errors| ParseError { errors })
}

pub fn parse_xpath<'a>(
    input: &'a str,
    namespaces: &'a Namespaces,
    variables: &'a [ast::Name],
) -> Result<ast::XPath, ParseError<'a>> {
    todo!();
}

pub fn parse_signature(input: &str, namespaces: &Namespaces) -> Result<ast::Signature, Error> {
    todo!();
}

pub fn parse_sequence_type(
    input: &str,
    namespaces: &Namespaces,
) -> Result<ast::SequenceType, Error> {
    todo!();
}

pub fn parse_kind_test(input: &str, namespaces: &Namespaces) -> Result<ast::KindTest, Error> {
    todo!();
}

#[cfg(test)]
mod tests {
    use crate::FN_NAMESPACE;

    use super::*;

    use insta::assert_ron_snapshot;

    fn parse_expr_single(src: &str) -> Result<ast::ExprSingleS, ParseError> {
        let namespaces = Namespaces::default();
        parse(parser().expr_single, tokens(src), Cow::Owned(namespaces))
    }

    fn parse_name(src: &str) -> Result<ast::NameS, ParseError> {
        let namespaces = Namespaces::default();
        parse(parser().name, tokens(src), Cow::Owned(namespaces))
    }

    fn parse_xpath_simple(src: &str) -> Result<ast::XPath, ParseError> {
        let namespaces = Namespaces::default();
        parse(parser().xpath, tokens(src), Cow::Owned(namespaces))
    }

    #[test]
    fn test_unprefixed_name() {
        assert_ron_snapshot!(parse_name("foo"));
    }

    #[test]
    fn test_prefixed_name() {
        assert_ron_snapshot!(parse_name("xs:foo"));
    }

    #[test]
    fn test_qualified_name() {
        assert_ron_snapshot!(parse_name("Q{http://example.com}foo"));
    }

    #[test]
    fn test_literal() {
        assert_ron_snapshot!(parse_expr_single("1"));
    }

    #[test]
    fn test_var_ref() {
        assert_ron_snapshot!(parse_expr_single("$foo"));
    }

    #[test]
    fn test_expr_single_addition() {
        assert_ron_snapshot!(parse_expr_single("1 + 2"));
    }

    #[test]
    fn test_simple_map_expr() {
        assert_ron_snapshot!(parse_expr_single("1 ! 2"));
    }

    #[test]
    fn test_unary_expr() {
        assert_ron_snapshot!(parse_expr_single("-1"));
    }

    #[test]
    fn test_additive_expr() {
        assert_ron_snapshot!(parse_expr_single("1 + 2"));
    }

    #[test]
    fn test_additive_expr_repeat() {
        assert_ron_snapshot!(parse_expr_single("1 + 2 + 3"));
    }

    #[test]
    fn test_or_expr() {
        assert_ron_snapshot!(parse_expr_single("1 or 2"));
    }

    #[test]
    fn test_and_expr() {
        assert_ron_snapshot!(parse_expr_single("1 and 2"));
    }

    #[test]
    fn test_comparison_expr() {
        assert_ron_snapshot!(parse_expr_single("1 < 2"));
    }

    #[test]
    fn test_concat_expr() {
        assert_ron_snapshot!(parse_expr_single("'a' || 'b'"));
    }

    #[test]
    fn test_nested_expr() {
        assert_ron_snapshot!(parse_expr_single("1 + (2 * 3)"));
    }

    #[test]
    fn test_xpath_single_expr() {
        assert_ron_snapshot!(parse_expr_single("1 + 2"));
    }

    #[test]
    fn test_xpath_multi_expr() {
        assert_ron_snapshot!(parse_xpath_simple("1 + 2, 3 + 4"));
    }

    #[test]
    fn test_single_let_expr() {
        assert_ron_snapshot!(parse_expr_single("let $x := 1 return 5"));
    }

    #[test]
    fn test_single_let_expr_var_ref() {
        assert_ron_snapshot!(parse_expr_single("let $x := 1 return $x"));
    }

    #[test]
    fn test_nested_let_expr() {
        assert_ron_snapshot!(parse_expr_single("let $x := 1, $y := 2 return 5"));
    }

    #[test]
    fn test_single_for_expr() {
        assert_ron_snapshot!(parse_expr_single("for $x in 1 return 5"));
    }

    #[test]
    fn test_for_loop() {
        assert_ron_snapshot!(parse_expr_single("for $x in 1 to 2 return $x"));
    }

    #[test]
    fn test_if_expr() {
        assert_ron_snapshot!(parse_expr_single("if (1) then 2 else 3"));
    }

    #[test]
    fn test_quantified() {
        assert_ron_snapshot!(parse_expr_single("every $x in (1, 2) satisfies $x > 0"));
    }

    #[test]
    fn test_quantified_nested() {
        assert_ron_snapshot!(parse_expr_single(
            "every $x in (1, 2), $y in (3, 4) satisfies $x > 0 and $y > 0"
        ));
    }

    #[test]
    fn test_inline_function() {
        assert_ron_snapshot!(parse_expr_single("function($x) { $x }"));
    }

    #[test]
    fn test_inline_function_with_param_types() {
        assert_ron_snapshot!(parse_expr_single("function($x as xs:integer) { $x }"));
    }

    #[test]
    fn test_inline_function_with_return_type() {
        assert_ron_snapshot!(parse_expr_single("function($x) as xs:integer { $x }"));
    }

    #[test]
    fn test_inline_function2() {
        assert_ron_snapshot!(parse_expr_single("function($x, $y) { $x + $y }"));
    }

    #[test]
    fn test_dynamic_function_call() {
        assert_ron_snapshot!(parse_expr_single("$foo()"));
    }

    #[test]
    fn test_dynamic_function_call_args() {
        assert_ron_snapshot!(parse_expr_single("$foo(1 + 1, 3)"));
    }

    #[test]
    fn test_dynamic_function_call_placeholder() {
        assert_ron_snapshot!(parse_expr_single("$foo(1, ?)"));
    }

    #[test]
    fn test_static_function_call() {
        assert_ron_snapshot!(parse_expr_single("my_function()"));
    }

    #[test]
    fn test_static_function_call_fn_prefix() {
        assert_ron_snapshot!(parse_expr_single("fn:root()"));
    }

    #[test]
    fn test_static_function_call_q() {
        assert_ron_snapshot!(parse_expr_single("Q{http://example.com}something()"));
    }

    #[test]
    fn test_static_function_call_args() {
        assert_ron_snapshot!(parse_expr_single("my_function(1, 2)"));
    }

    #[test]
    fn test_named_function_ref() {
        assert_ron_snapshot!(parse_expr_single("my_function#2"));
    }

    #[test]
    fn test_static_function_call_placeholder() {
        assert_ron_snapshot!(parse_expr_single("my_function(?, 1)"));
    }

    #[test]
    fn test_simple_comma() {
        assert_ron_snapshot!(parse_xpath_simple("1, 2"));
    }

    #[test]
    fn test_complex_comma() {
        assert_ron_snapshot!(parse_xpath_simple("(1, 2), (3, 4)"));
    }

    #[test]
    fn test_range() {
        assert_ron_snapshot!(parse_expr_single("1 to 2"));
    }

    #[test]
    fn test_simple_map() {
        assert_ron_snapshot!(parse_expr_single("(1, 2) ! (. * 2)"));
    }

    #[test]
    fn test_predicate() {
        assert_ron_snapshot!(parse_expr_single("(1, 2)[2]"));
    }

    #[test]
    fn test_axis() {
        assert_ron_snapshot!(parse_expr_single("child::foo"));
    }

    #[test]
    fn test_multiple_steps() {
        assert_ron_snapshot!(parse_expr_single("child::foo/child::bar"));
    }

    #[test]
    fn test_with_predicate() {
        assert_ron_snapshot!(parse_expr_single("child::foo[1]"));
    }

    #[test]
    fn test_axis_with_predicate() {
        assert_ron_snapshot!(parse_expr_single("child::foo[1]"));
    }

    #[test]
    fn test_axis_star() {
        assert_ron_snapshot!(parse_expr_single("child::*"));
    }

    #[test]
    fn test_axis_wildcard_prefix() {
        assert_ron_snapshot!(parse_expr_single("child::*:foo"));
    }

    #[test]
    fn test_axis_wildcard_local_name() {
        assert_ron_snapshot!(parse_expr_single("child::fn:*"));
    }

    #[test]
    fn test_axis_wildcard_q_name() {
        assert_ron_snapshot!(parse_expr_single("child::Q{http://example.com}*"));
    }

    #[test]
    fn test_reverse_axis() {
        assert_ron_snapshot!(parse_expr_single("parent::foo"));
    }

    #[test]
    fn test_node_test() {
        assert_ron_snapshot!(parse_expr_single("self::node()"));
    }

    #[test]
    fn test_text_test() {
        assert_ron_snapshot!(parse_expr_single("self::text()"));
    }

    #[test]
    fn test_comment_test() {
        assert_ron_snapshot!(parse_expr_single("self::comment()"));
    }

    #[test]
    fn test_namespace_node_test() {
        assert_ron_snapshot!(parse_expr_single("self::namespace-node()"));
    }

    #[test]
    fn test_attribute_test_no_args() {
        assert_ron_snapshot!(parse_expr_single("self::attribute()"));
    }

    #[test]
    fn test_attribute_test_star_arg() {
        assert_ron_snapshot!(parse_expr_single("self::attribute(*)"));
    }

    #[test]
    fn test_attribute_test_name_arg() {
        assert_ron_snapshot!(parse_expr_single("self::attribute(foo)"));
    }

    #[test]
    fn test_attribute_test_name_arg_type_arg() {
        assert_ron_snapshot!(parse_expr_single("self::attribute(foo, bar)"));
    }

    #[test]
    fn test_element_test() {
        assert_ron_snapshot!(parse_expr_single("self::element()"));
    }

    #[test]
    fn test_abbreviated_forward_step() {
        assert_ron_snapshot!(parse_expr_single("foo"));
    }

    #[test]
    fn test_abbreviated_forward_step_with_attribute_test() {
        assert_ron_snapshot!(parse_expr_single("foo/attribute()"));
    }

    // XXX should test for attribute axis for SchemaAttributeTest too

    #[test]
    fn test_namespace_node_default_axis() {
        assert_ron_snapshot!(parse_expr_single("foo/namespace-node()"));
    }

    #[test]
    fn test_abbreviated_forward_step_attr() {
        assert_ron_snapshot!(parse_expr_single("@foo"));
    }

    #[test]
    fn test_abbreviated_reverse_step() {
        assert_ron_snapshot!(parse_expr_single("foo/.."));
    }

    #[test]
    fn test_abbreviated_reverse_step_with_predicates() {
        assert_ron_snapshot!(parse_expr_single("..[1]"));
    }

    #[test]
    fn test_starts_single_slash() {
        assert_ron_snapshot!(parse_expr_single("/child::foo"));
    }

    #[test]
    fn test_single_slash_by_itself() {
        assert_ron_snapshot!(parse_expr_single("/"));
    }

    #[test]
    fn test_double_slash_by_itself() {
        assert_ron_snapshot!(parse_expr_single("//"));
    }

    #[test]
    fn test_starts_double_slash() {
        assert_ron_snapshot!(parse_expr_single("//child::foo"));
    }

    #[test]
    fn test_double_slash_middle() {
        assert_ron_snapshot!(parse_expr_single("child::foo//child::bar"));
    }

    #[test]
    fn test_union() {
        assert_ron_snapshot!(parse_expr_single("child::foo | child::bar"));
    }

    #[test]
    fn test_intersect() {
        assert_ron_snapshot!(parse_expr_single("child::foo intersect child::bar"));
    }

    #[test]
    fn test_except() {
        assert_ron_snapshot!(parse_expr_single("child::foo except child::bar"));
    }

    #[test]
    fn test_xpath_parse_error() {
        assert_ron_snapshot!(parse_expr_single("1 + 2 +"));
    }

    #[test]
    fn test_xpath_ge() {
        assert_ron_snapshot!(parse_expr_single("1 >= 2"));
    }

    // #[test]
    // fn test_signature_without_params() {
    //     let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
    //     assert_ron_snapshot!(parse_signature("fn:foo() as xs:integer", &namespaces));
    // }

    // #[test]
    // fn test_signature_without_params2() {
    //     let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
    //     assert_ron_snapshot!(parse_signature("fn:foo() as xs:integer*", &namespaces));
    // }

    // #[test]
    // fn test_signature_with_params() {
    //     let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
    //     assert_ron_snapshot!(parse_signature(
    //         "fn:foo($a as xs:decimal*) as xs:integer",
    //         &namespaces
    //     ));
    // }

    // #[test]
    // fn test_signature_with_node_param() {
    //     let namespaces = Namespaces::new(None, Some(FN_NAMESPACE));
    //     assert_ron_snapshot!(parse_signature(
    //         "fn:foo($a as node()) as xs:integer",
    //         &namespaces
    //     ));
    // }

    #[test]
    fn test_unary_multiple() {
        assert_ron_snapshot!(parse_expr_single("+-1"));
    }

    #[test]
    fn test_cast_as() {
        assert_ron_snapshot!(parse_expr_single("1 cast as xs:integer"));
    }

    #[test]
    fn test_cast_as_with_question_mark() {
        assert_ron_snapshot!(parse_expr_single("1 cast as xs:integer?"));
    }

    #[test]
    fn test_castable_as() {
        assert_ron_snapshot!(parse_expr_single("1 castable as xs:integer"));
    }

    #[test]
    fn test_castable_as_with_question_mark() {
        assert_ron_snapshot!(parse_expr_single("1 castable as xs:integer?"));
    }

    #[test]
    fn test_instance_of() {
        assert_ron_snapshot!(parse_expr_single("1 instance of xs:integer"));
    }

    #[test]
    fn test_instance_of_with_star() {
        assert_ron_snapshot!(parse_expr_single("1 instance of xs:integer*"));
    }

    #[test]
    fn test_treat() {
        assert_ron_snapshot!(parse_expr_single("1 treat as xs:integer"));
    }

    #[test]
    fn test_treat_with_star() {
        assert_ron_snapshot!(parse_expr_single("1 treat as xs:integer*"));
    }
}
