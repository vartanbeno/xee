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
        .map(|o| o.unwrap_or(ast::Occurrence::One));

    let item_type = recursive(|item_type| {
        just(Token::Item)
            .ignore_then(empty_call)
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

        let predicate = expr
            .clone()
            .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
            .boxed();

        let postfix = predicate.map(ast::Postfix::Predicate).boxed();

        let string_literal = select! {
            Token::StringLiteral(s) => s,
        }
        .map(|s| ast::Literal::String(s.to_string()))
        .boxed();

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
            .boxed();

        let postfix_expr = primary_expr
            .then(postfix.repeated().collect::<Vec<_>>())
            .map_with_span(|(primary, postfixes), span| {
                if postfixes.is_empty() {
                    ast::StepExpr::PrimaryExpr(primary).with_span(span)
                } else {
                    ast::StepExpr::PostfixExpr { primary, postfixes }.with_span(span)
                }
            })
            .boxed();

        let step_expr = postfix_expr;

        let relative_path_expr = step_expr
            .clone()
            .separated_by(just(Token::Slash))
            .at_least(1)
            .collect::<Vec<_>>()
            .boxed();

        let slash_prefix_path_expr = just(Token::Slash)
            .map_with_span(|_, span| span)
            .then(relative_path_expr.clone())
            .map(|(slash_span, steps)| {
                let root_step = root_step(slash_span);
                let all_steps = once(root_step).chain(steps.into_iter()).collect();
                ast::PathExpr { steps: all_steps }
            })
            .boxed();

        let doubleslash_prefix_path_expr = just(Token::DoubleSlash)
            .map_with_span(|_, span| span)
            .then(relative_path_expr.clone())
            .map(|(double_slash_span, steps)| {
                let root_step = root_step(double_slash_span);
                let descendant_step = ast::StepExpr::AxisStep(ast::AxisStep {
                    axis: ast::Axis::DescendantOrSelf,
                    node_test: ast::NodeTest::KindTest(ast::KindTest::Any),
                    predicates: vec![],
                })
                .with_span(double_slash_span);
                let all_steps = once(root_step)
                    .chain(once(descendant_step).chain(steps.into_iter()))
                    .collect();
                ast::PathExpr { steps: all_steps }
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
            name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
            arguments: vec![path_arg],
        })
        .with_empty_span(),
    )
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

    // #[test]
    // fn test_dynamic_function_call() {
    //     assert_ron_snapshot!(parse_expr_single("$foo()"));
    // }

    // #[test]
    // fn test_dynamic_function_call_args() {
    //     assert_ron_snapshot!(parse_expr_single("$foo(1 + 1, 3)"));
    // }

    // #[test]
    // fn test_static_function_call() {
    //     assert_ron_snapshot!(parse_expr_single("my_function()"));
    // }

    // #[test]
    // fn test_static_function_call_fn_prefix() {
    //     assert_ron_snapshot!(parse_expr_single("fn:root()"));
    // }

    // #[test]
    // fn test_static_function_call_q() {
    //     assert_ron_snapshot!(parse_expr_single("Q{http://example.com}something()"));
    // }

    // #[test]
    // fn test_static_function_call_args() {
    //     assert_ron_snapshot!(parse_expr_single("my_function(1, 2)"));
    // }

    // #[test]
    // fn test_named_function_ref() {
    //     assert_ron_snapshot!(parse_expr_single("my_function#2"));
    // }

    // #[test]
    // fn test_dynamic_function_call_placeholder() {
    //     assert_ron_snapshot!(parse_expr_single("$foo(1, ?)"));
    // }

    // #[test]
    // fn test_static_function_call_placeholder() {
    //     assert_ron_snapshot!(parse_expr_single("my_function(?, 1)"));
    // }

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

    // #[test]
    // fn test_axis() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo"));
    // }

    // #[test]
    // fn test_multiple_steps() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo/child::bar"));
    // }

    // #[test]
    // fn test_with_predicate() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo[1]"));
    // }

    // #[test]
    // fn test_axis_with_predicate() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo[1]"));
    // }

    // #[test]
    // fn test_axis_star() {
    //     assert_ron_snapshot!(parse_expr_single("child::*"));
    // }

    // #[test]
    // fn test_axis_wildcard_prefix() {
    //     assert_ron_snapshot!(parse_expr_single("child::*:foo"));
    // }

    // #[test]
    // fn test_axis_wildcard_local_name() {
    //     assert_ron_snapshot!(parse_expr_single("child::fn:*"));
    // }

    // #[test]
    // fn test_axis_wildcard_q_name() {
    //     assert_ron_snapshot!(parse_expr_single("child::Q{http://example.com}*"));
    // }

    // #[test]
    // fn test_reverse_axis() {
    //     assert_ron_snapshot!(parse_expr_single("parent::foo"));
    // }

    // #[test]
    // fn test_node_test() {
    //     assert_ron_snapshot!(parse_expr_single("self::node()"));
    // }

    // #[test]
    // fn test_text_test() {
    //     assert_ron_snapshot!(parse_expr_single("self::text()"));
    // }

    // #[test]
    // fn test_comment_test() {
    //     assert_ron_snapshot!(parse_expr_single("self::comment()"));
    // }

    // #[test]
    // fn test_namespace_node_test() {
    //     assert_ron_snapshot!(parse_expr_single("self::namespace-node()"));
    // }

    // #[test]
    // fn test_attribute_test_no_args() {
    //     assert_ron_snapshot!(parse_expr_single("self::attribute()"));
    // }

    // #[test]
    // fn test_attribute_test_star_arg() {
    //     assert_ron_snapshot!(parse_expr_single("self::attribute(*)"));
    // }

    // #[test]
    // fn test_attribute_test_name_arg() {
    //     assert_ron_snapshot!(parse_expr_single("self::attribute(foo)"));
    // }

    // #[test]
    // fn test_attribute_test_name_arg_type_arg() {
    //     assert_ron_snapshot!(parse_expr_single("self::attribute(foo, bar)"));
    // }

    // #[test]
    // fn test_element_test() {
    //     assert_ron_snapshot!(parse_expr_single("self::element()"));
    // }

    // #[test]
    // fn test_abbreviated_forward_step() {
    //     assert_ron_snapshot!(parse_expr_single("foo"));
    // }

    // #[test]
    // fn test_abbreviated_forward_step_with_attribute_test() {
    //     assert_ron_snapshot!(parse_expr_single("foo/attribute()"));
    // }

    // XXX should test for attribute axis for SchemaAttributeTest too

    // #[test]
    // fn test_namespace_node_default_axis() {
    //     assert_ron_snapshot!(parse_expr_single("foo/namespace-node()"));
    // }

    // #[test]
    // fn test_abbreviated_forward_step_attr() {
    //     assert_ron_snapshot!(parse_expr_single("@foo"));
    // }

    // #[test]
    // fn test_abbreviated_reverse_step() {
    //     assert_ron_snapshot!(parse_expr_single("foo/.."));
    // }

    // #[test]
    // fn test_abbreviated_reverse_step_with_predicates() {
    //     assert_ron_snapshot!(parse_expr_single("..[1]"));
    // }

    // #[test]
    // fn test_starts_single_slash() {
    //     assert_ron_snapshot!(parse_expr_single("/child::foo"));
    // }

    // #[test]
    // fn test_single_slash_by_itself() {
    //     assert_ron_snapshot!(parse_expr_single("/"));
    // }

    // #[test]
    // fn test_starts_double_slash() {
    //     assert_ron_snapshot!(parse_expr_single("//child::foo"));
    // }

    // #[test]
    // fn test_double_slash_middle() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo//child::bar"));
    // }

    // #[test]
    // fn test_union() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo | child::bar"));
    // }

    // #[test]
    // fn test_intersect() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo intersect child::bar"));
    // }

    // #[test]
    // fn test_except() {
    //     assert_ron_snapshot!(parse_expr_single("child::foo except child::bar"));
    // }

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
