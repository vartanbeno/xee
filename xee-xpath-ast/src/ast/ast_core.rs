use ibig::IBig;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use xot::Xot;

pub use crate::operator::BinaryOperator;
use crate::{
    namespaces::Namespaces,
    span::{Spanned, WithSpan},
};

pub type ExprSingleS = Spanned<ExprSingle>;
pub type PrimaryExprS = Spanned<PrimaryExpr>;
pub type StepExprS = Spanned<StepExpr>;
pub type ExprS = Spanned<Expr>;
pub type NameS = Spanned<Name>;

impl WithSpan for ExprSingle {}
impl WithSpan for PrimaryExpr {}
impl WithSpan for StepExpr {}
impl WithSpan for Expr {}
impl WithSpan for Name {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Expr(pub Vec<ExprSingleS>);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct XPath(pub ExprS);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ExprSingle {
    // a path expression
    Path(PathExpr),
    // something applied to a path expression
    Apply(ApplyExpr),
    // combine two path expressions
    Let(LetExpr),
    If(IfExpr),
    Binary(BinaryExpr),
    For(ForExpr),
    Quantified(QuantifiedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ForExpr {
    pub var_name: NameS,
    pub var_expr: Box<ExprSingleS>,
    pub return_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct QuantifiedExpr {
    pub quantifier: Quantifier,
    pub var_name: NameS,
    pub var_expr: Box<ExprSingleS>,
    pub satisfies_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Name {
    name: String,
    prefix: Option<String>,
    namespace: Option<String>,
}

impl Name {
    pub fn new(name: String, namespace: Option<String>) -> Self {
        Name {
            name,
            namespace,
            prefix: None,
        }
    }

    pub fn prefixed(prefix: &str, name: &str, namespaces: &Namespaces) -> Option<Self> {
        let namespace = namespaces.by_prefix(prefix)?;
        Some(Name {
            name: name.to_string(),
            namespace: Some(namespace.to_string()),
            prefix: Some(prefix.to_string()),
        })
    }

    pub fn unprefixed(name: &str) -> Self {
        Name {
            name: name.to_string(),
            namespace: None,
            prefix: None,
        }
    }

    pub fn uri_qualified(uri: &str, name: &str) -> Self {
        Name {
            name: name.to_string(),
            namespace: Some(uri.to_string()),
            prefix: None,
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn local_name(&self) -> &str {
        &self.name
    }

    pub fn to_name_id(&self, xot: &Xot) -> Option<xot::NameId> {
        if let Some(namespace) = &self.namespace {
            let namespace_id = xot.namespace(namespace);
            if let Some(namespace_id) = namespace_id {
                xot.name_ns(&self.name, namespace_id)
            } else {
                None
            }
        } else {
            xot.name(&self.name)
        }
    }

    pub fn with_suffix(&self) -> Name {
        let mut name = self.name.clone();
        name.push('*');
        Name {
            name,
            namespace: self.namespace.clone(),
            prefix: self.prefix.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LetExpr {
    pub var_name: NameS,
    pub var_expr: Box<ExprSingleS>,
    pub return_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct IfExpr {
    pub condition: ExprS,
    pub then: Box<ExprSingleS>,
    pub else_: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Quantifier {
    Some,
    Every,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum PrimaryExpr {
    Literal(Literal),
    VarRef(Name),
    Expr(ExprS),
    ContextItem,
    FunctionCall(FunctionCall),
    NamedFunctionRef(NamedFunctionRef),
    InlineFunction(InlineFunction),
    MapConstructor(MapConstructor),
    ArrayConstructor(ArrayConstructor),
    UnaryLookup(UnaryLookup),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum UnaryLookup {
    Name(String),
    IntegerLiteral(i64),
    Expr(ExprS),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct BinaryExpr {
    pub operator: BinaryOperator,
    pub left: PathExpr,
    pub right: PathExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ApplyExpr {
    pub path_expr: PathExpr,
    pub operator: ApplyOperator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ApplyOperator {
    SimpleMap(Vec<PathExpr>),
    Unary(Vec<UnaryOperator>),
    Arrow(Vec<(ArrowFunctionSpecifier, Vec<ExprSingleS>)>),
    Cast(SingleType),
    Castable(SingleType),
    Treat(SequenceType),
    InstanceOf(SequenceType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum UnaryOperator {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SingleType {
    pub name: NameS,
    pub question_mark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ArrowFunctionSpecifier {
    Name(EQName),
    VarRef(EQName),
    Expr(ExprS),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct MapConstructor {
    pub entries: Vec<MapConstructorEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct MapConstructorEntry {
    pub key: ExprSingleS,
    pub value: ExprSingleS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ArrayConstructor {
    pub members: Vec<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Literal {
    Decimal(Decimal),
    Integer(IBig),
    Double(OrderedFloat<f64>),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct FunctionCall {
    pub name: Name,
    pub arguments: Vec<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct NamedFunctionRef {
    pub name: NameS,
    pub arity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct InlineFunction {
    pub params: Vec<Param>,
    pub return_type: Option<SequenceType>,
    pub body: Option<ExprS>,
}

// a function signature as described by:
// https://www.w3.org/TR/xpath-functions-31/#func-signatures
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Signature {
    pub name: Name,
    pub params: Vec<SignatureParam>,
    pub return_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Param {
    pub name: Name,
    pub type_: Option<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SignatureParam {
    pub name: Name,
    pub type_: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Postfix {
    // vec contains at least 1 element
    Predicate(ExprS),
    ArgumentList(Vec<ExprSingleS>),
    Lookup(Lookup),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Lookup {
    Name(String),
    IntegerLiteral(i64),
    Expr(Vec<ExprSingleS>),
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct PathExpr {
    pub steps: Vec<StepExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum StepExpr {
    PrimaryExpr(PrimaryExprS),
    PostfixExpr {
        primary: PrimaryExprS,
        postfixes: Vec<Postfix>,
    },
    AxisStep(AxisStep),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AxisStep {
    pub axis: Axis,
    pub node_test: NodeTest,
    pub predicates: Vec<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Axis {
    Ancestor,
    AncestorOrSelf,
    Attribute,
    Child,
    Descendant,
    DescendantOrSelf,
    Following,
    FollowingSibling,
    Namespace,
    Parent,
    Preceding,
    PrecedingSibling,
    Self_,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum NameTest {
    Name(Name),
    Star,
    LocalName(String),
    Namespace(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum EQName {
    QName(QName),
    URIQualifiedName(URIQualifiedName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum QName {
    PrefixedName(PrefixedName),
    UnprefixedName(UnprefixedName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct PrefixedName {
    pub prefix: String,
    pub local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct UnprefixedName {
    pub local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct URIQualifiedName {
    pub uri: String,
    pub local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum SequenceType {
    Empty,
    Item(Item),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Item {
    pub item_type: ItemType,
    pub occurrence: Occurrence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ItemType {
    Item,
    AtomicOrUnionType(NameS),
    KindTest(KindTest),
    FunctionTest(Box<FunctionTest>),
    MapTest(Box<MapTest>),
    ArrayTest(Box<ArrayTest>),
}

impl ItemType {
    pub fn is_generalized_atomic_type(&self) -> bool {
        matches!(self, ItemType::AtomicOrUnionType(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Occurrence {
    One,
    Option,
    Many,
    NonEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum KindTest {
    Document(Option<DocumentTest>),
    Element(Option<ElementTest>),
    Attribute(Option<AttributeTest>),
    SchemaElement(SchemaElementTest),
    SchemaAttribute(SchemaAttributeTest),
    PI(Option<PITest>),
    Comment,
    Text,
    NamespaceNode,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum DocumentTest {
    Element(ElementTest),
    SchemaElement(SchemaElementTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ElementTest {
    pub name_test: ElementNameOrWildcard,
    pub type_name: Option<ElementTypeName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ElementTypeName {
    pub name: Name,
    pub question_mark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ElementNameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AttributeTest {
    pub name_test: AttribNameOrWildcard,
    pub type_name: Option<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum AttribNameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SchemaElementTest {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SchemaAttributeTest {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct TypedFunctionTest {
    parameter_types: Vec<SequenceType>,
    return_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct TypedMapTest {
    pub key_type: Name,
    pub value_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct TypedArrayTest {
    pub item_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum PITest {
    Name(String),
    StringLiteral(String),
}
