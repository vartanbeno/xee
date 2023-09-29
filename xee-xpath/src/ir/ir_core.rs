use ibig::IBig;
// an Intermediate Representation in ANF - administrative normal form
// XXX is this really ANF? Maybe it is, though it doesn't support recursion
// (without function arguments), as XPath doesn't.
use ordered_float::OrderedFloat;
use rust_decimal::Decimal;

use xee_schema_type::Xs;
pub use xee_xpath_ast::ast::{BinaryOperator, SequenceType, SingleType, UnaryOperator};
use xee_xpath_ast::span::Spanned;

use crate::function::{CastType, StaticFunctionId};
use crate::xml;

pub(crate) type AtomS = Spanned<Atom>;
pub(crate) type ExprS = Spanned<Expr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Expr {
    Atom(AtomS),
    Let(Let),
    If(If),
    Binary(Binary),
    Unary(Unary),
    FunctionDefinition(FunctionDefinition),
    FunctionCall(FunctionCall),
    Lookup(Lookup),
    WildcardLookup(WildcardLookup),
    Step(Step),
    Deduplicate(Box<ExprS>),
    Map(Map),
    Filter(Filter),
    Quantified(Quantified),
    Cast(Cast),
    Castable(Castable),
    InstanceOf(InstanceOf),
    MapConstructor(MapConstructor),
    ArrayConstructor(ArrayConstructor),
}

// not to be confused with an XPath atom; this is a variable or a constant
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Atom {
    Const(Const),
    Variable(Name),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Const {
    Integer(IBig),
    String(String),
    Double(OrderedFloat<f64>),
    Decimal(Decimal),
    StaticFunctionReference(StaticFunctionId, Option<ContextNames>),
    // XXX replace this with a sequence constant? useful once we have constant folding
    EmptySequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Name(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextNames {
    pub(crate) item: Name,
    pub(crate) position: Name,
    pub(crate) last: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Let {
    pub(crate) name: Name,
    pub(crate) var_expr: Box<ExprS>,
    pub(crate) return_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct If {
    pub(crate) condition: AtomS,
    pub(crate) then: Box<ExprS>,
    pub(crate) else_: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binary {
    pub(crate) left: AtomS,
    pub(crate) op: BinaryOperator,
    pub(crate) right: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Unary {
    pub(crate) op: UnaryOperator,
    pub(crate) atom: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionDefinition {
    pub(crate) params: Vec<Param>,
    pub(crate) return_type: Option<SequenceType>,
    pub(crate) body: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Param {
    pub(crate) name: Name,
    pub(crate) type_: Option<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionCall {
    pub(crate) atom: AtomS,
    pub(crate) args: Vec<AtomS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Lookup {
    pub(crate) atom: AtomS,
    pub(crate) key: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WildcardLookup {
    pub(crate) atom: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    pub(crate) step: xml::Step,
    pub(crate) context: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Map {
    pub(crate) context_names: ContextNames,
    pub(crate) var_atom: AtomS,
    pub(crate) return_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Filter {
    pub(crate) context_names: ContextNames,
    pub(crate) var_atom: AtomS,
    pub(crate) return_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Quantified {
    pub(crate) quantifier: Quantifier,
    pub(crate) context_names: ContextNames,
    pub(crate) var_atom: AtomS,
    pub(crate) satisifies_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Quantifier {
    Some,
    Every,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cast {
    pub(crate) atom: AtomS,
    pub(crate) xs: Xs,
    pub(crate) empty_sequence_allowed: bool,
}

impl Cast {
    pub(crate) fn cast_type(&self) -> CastType {
        CastType {
            xs: self.xs,
            empty_sequence_allowed: self.empty_sequence_allowed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Castable {
    pub(crate) atom: AtomS,
    pub(crate) xs: Xs,
    pub(crate) empty_sequence_allowed: bool,
}

impl Castable {
    pub(crate) fn cast_type(&self) -> CastType {
        CastType {
            xs: self.xs,
            empty_sequence_allowed: self.empty_sequence_allowed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstanceOf {
    pub(crate) atom: AtomS,
    pub(crate) sequence_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MapConstructor {
    pub(crate) members: Vec<(AtomS, AtomS)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArrayConstructor {
    Square(Vec<AtomS>),
    Curly(AtomS),
}
