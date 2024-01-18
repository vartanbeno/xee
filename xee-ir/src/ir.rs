// an Intermediate Representation in ANF - administrative normal form
// XXX is this really ANF? Maybe it is, though it doesn't support recursion
// (without function arguments), as XPath doesn't.

use ibig::IBig;
use ordered_float::OrderedFloat;
use rust_decimal::Decimal;

pub use xee_interpreter::function::Name;
use xee_interpreter::function::{CastType, Signature, StaticFunctionId};
use xee_interpreter::xml;
use xee_schema_type::Xs;
pub use xee_xpath_ast::ast::{BinaryOperator, SequenceType, UnaryOperator};
use xee_xpath_ast::span::Spanned;

pub type AtomS = Spanned<Atom>;
pub type ExprS = Spanned<Expr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Atom(AtomS),
    Let(Let),
    If(If),
    Match,
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
    Treat(Treat),
    MapConstructor(MapConstructor),
    ArrayConstructor(ArrayConstructor),
    XmlName(XmlName),
    Root(XmlRoot),
    Element(XmlElement),
    Attribute(XmlAttribute),
    Prefix(XmlPrefix),
    Text(XmlText),
    Comment(XmlComment),
    ProcessingInstruction(XmlProcessingInstruction),
}

// not to be confused with an XPath atom; this is a variable or a constant
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    Const(Const),
    Variable(Name),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Const {
    Integer(IBig),
    String(String),
    Double(OrderedFloat<f64>),
    Decimal(Decimal),
    StaticFunctionReference(StaticFunctionId, Option<ContextNames>),
    // XXX replace this with a sequence constant? useful once we have constant folding
    EmptySequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextNames {
    pub item: Name,
    pub position: Name,
    pub last: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Let {
    pub name: Name,
    pub var_expr: Box<ExprS>,
    pub return_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub condition: AtomS,
    pub then: Box<ExprS>,
    pub else_: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub left: AtomS,
    pub op: BinaryOperator,
    pub right: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unary {
    pub op: UnaryOperator,
    pub atom: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub params: Vec<Param>,
    pub return_type: Option<SequenceType>,
    pub body: Box<ExprS>,
}

impl FunctionDefinition {
    pub fn signature(&self) -> Signature {
        Signature {
            parameter_types: self
                .params
                .iter()
                .map(|param| param.type_.clone())
                .collect(),
            return_type: self.return_type.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: Name,
    pub type_: Option<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub atom: AtomS,
    pub args: Vec<AtomS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lookup {
    pub atom: AtomS,
    pub arg_atom: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WildcardLookup {
    pub atom: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub step: xml::Step,
    pub context: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub context_names: ContextNames,
    pub var_atom: AtomS,
    pub return_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub context_names: ContextNames,
    pub var_atom: AtomS,
    pub return_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantified {
    pub quantifier: Quantifier,
    pub context_names: ContextNames,
    pub var_atom: AtomS,
    pub satisifies_expr: Box<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quantifier {
    Some,
    Every,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cast {
    pub atom: AtomS,
    pub xs: Xs,
    pub empty_sequence_allowed: bool,
}

impl Cast {
    pub fn cast_type(&self) -> CastType {
        CastType {
            xs: self.xs,
            empty_sequence_allowed: self.empty_sequence_allowed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Castable {
    pub atom: AtomS,
    pub xs: Xs,
    pub empty_sequence_allowed: bool,
}

impl Castable {
    pub fn cast_type(&self) -> CastType {
        CastType {
            xs: self.xs,
            empty_sequence_allowed: self.empty_sequence_allowed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceOf {
    pub atom: AtomS,
    pub sequence_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Treat {
    pub atom: AtomS,
    pub sequence_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapConstructor {
    pub members: Vec<(AtomS, AtomS)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayConstructor {
    Square(Vec<AtomS>),
    Curly(AtomS),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlName {
    pub local_name: AtomS,
    pub namespace: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlRoot {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement {
    // element or root to which to add the new element
    pub element: AtomS,
    pub name: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlAttribute {
    pub element: AtomS,
    pub name: AtomS,
    pub value: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlPrefix {
    pub element: AtomS,
    pub name: AtomS,
    pub uri: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlText {
    pub element: AtomS,
    pub value: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlComment {
    pub element: AtomS,
    pub value: AtomS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlProcessingInstruction {
    pub element: AtomS,
    pub target: AtomS,
    pub content: AtomS,
}
