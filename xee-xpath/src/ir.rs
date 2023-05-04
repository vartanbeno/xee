// an Intermediate Representation in ANF - administrative normal form
// XXX is this really ANF? Maybe it is, though it doesn't support recursion
// (without function arguments), as XPath doesn't.
use std::rc::Rc;

use crate::span::Spanned;
use crate::value::{StaticFunctionId, Step};

pub(crate) type AtomS = Spanned<Atom>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Expr {
    Atom(AtomS),
    Let(Let),
    If(If),
    Binary(Binary),
    FunctionDefinition(FunctionDefinition),
    StaticFunctionReference(StaticFunctionId, Option<ContextNames>),
    FunctionCall(FunctionCall),
    Map(Map),
    Filter(Filter),
    Quantified(Quantified),
}

// not to be confused with an XPath atom; this is a variable or a constant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Atom {
    Const(Const),
    Variable(Name),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Const {
    Integer(i64),
    String(String),
    // XXX replace this with a sequence constant? useful once we have constant folding
    EmptySequence,
    // step is treated as a special function which takes the context node as
    // its argument
    Step(Rc<Step>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Name(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ContextNames {
    pub(crate) item: Name,
    pub(crate) position: Name,
    pub(crate) last: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Let {
    pub(crate) name: Name,
    pub(crate) var_expr: Box<Expr>,
    pub(crate) return_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct If {
    pub(crate) condition: AtomS,
    pub(crate) then: Box<Expr>,
    pub(crate) else_: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Binary {
    pub(crate) left: AtomS,
    pub(crate) op: BinaryOp,
    pub(crate) right: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Comma,
    Union,
    Range,
    Concat,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FunctionDefinition {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Param(pub(crate) Name);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FunctionCall {
    pub(crate) atom: AtomS,
    pub(crate) args: Vec<AtomS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Map {
    pub(crate) context_names: ContextNames,
    pub(crate) var_atom: AtomS,
    pub(crate) return_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Filter {
    pub(crate) context_names: ContextNames,
    pub(crate) var_atom: AtomS,
    pub(crate) return_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Quantified {
    pub(crate) quantifier: Quantifier,
    pub(crate) context_names: ContextNames,
    pub(crate) var_atom: AtomS,
    pub(crate) satisifies_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Quantifier {
    Some,
    Every,
}
