use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use xot::Xot;

pub use crate::operator::BinaryOperator;
use crate::span::Spanned;

pub type ExprSingleS = Spanned<ExprSingle>;
pub type PrimaryExprS = Spanned<PrimaryExpr>;
pub type StepExprS = Spanned<StepExpr>;

pub type Expr = Vec<ExprSingleS>;
pub type ExprS = Spanned<Expr>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XPath {
    // at least one entry
    pub exprs: ExprS,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForExpr {
    pub var_name: Name,
    pub var_expr: Box<ExprSingleS>,
    pub return_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuantifiedExpr {
    pub quantifier: Quantifier,
    pub var_name: Name,
    pub var_expr: Box<ExprSingleS>,
    pub satisfies_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    name: String,
    namespace: Option<String>,
}

impl Name {
    pub fn new(name: String, namespace: Option<String>) -> Self {
        Name { name, namespace }
    }

    pub fn without_ns(name: &str) -> Self {
        Name {
            name: name.to_string(),
            namespace: None,
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
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
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetExpr {
    pub var_name: Name,
    pub var_expr: Box<ExprSingleS>,
    pub return_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfExpr {
    pub condition: ExprS,
    pub then: Box<ExprSingleS>,
    pub else_: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Quantifier {
    Some,
    Every,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryLookup {
    Name(String),
    IntegerLiteral(i64),
    Expr(ExprS),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryExpr {
    pub operator: BinaryOperator,
    pub left: PathExpr,
    pub right: PathExpr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplyExpr {
    pub path_expr: PathExpr,
    pub operator: ApplyOperator,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApplyOperator {
    SimpleMap(Vec<PathExpr>),
    Unary(Vec<UnaryOperator>),
    Arrow(Vec<(ArrowFunctionSpecifier, Vec<ExprSingleS>)>),
    Cast(SingleType),
    Castable(SingleType),
    Treat(SequenceType),
    InstanceOf(SequenceType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SingleType {
    pub name: EQName,
    pub question_mark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArrowFunctionSpecifier {
    Name(EQName),
    VarRef(EQName),
    Expr(ExprS),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapConstructor {
    pub entries: Vec<MapConstructorEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapConstructorEntry {
    pub key: ExprSingleS,
    pub value: ExprSingleS,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayConstructor {
    pub members: Vec<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Decimal(Decimal),
    Integer(String),
    Double(OrderedFloat<f64>),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionCall {
    pub name: Name,
    pub arguments: Vec<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedFunctionRef {
    pub name: Name,
    pub arity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineFunction {
    pub params: Vec<Param>,
    pub return_type: Option<SequenceType>,
    pub body: ExprS,
}

// a function signature as described by:
// https://www.w3.org/TR/xpath-functions-31/#func-signatures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub name: Name,
    pub params: Vec<SignatureParam>,
    pub return_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param {
    pub name: Name,
    pub type_: Option<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignatureParam {
    pub name: Name,
    pub type_: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Postfix {
    // vec contains at least 1 element
    Predicate(ExprS),
    ArgumentList(Vec<ExprSingleS>),
    Lookup(Lookup),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lookup {
    Name(String),
    IntegerLiteral(i64),
    Expr(Vec<ExprSingleS>),
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathExpr {
    pub steps: Vec<StepExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StepExpr {
    PrimaryExpr(PrimaryExprS),
    PostfixExpr {
        primary: PrimaryExprS,
        postfixes: Vec<Postfix>,
    },
    AxisStep(AxisStep),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AxisStep {
    pub axis: Axis,
    pub node_test: NodeTest,
    pub predicates: Vec<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameTest {
    Name(Name),
    Star,
    LocalName(String),
    Namespace(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EQName {
    QName(QName),
    URIQualifiedName(URIQualifiedName),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QName {
    PrefixedName(PrefixedName),
    UnprefixedName(UnprefixedName),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixedName {
    pub prefix: String,
    pub local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnprefixedName {
    pub local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct URIQualifiedName {
    pub uri: String,
    pub local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequenceType {
    Empty,
    Item(Item),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub item_type: ItemType,
    pub occurrence: Occurrence,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemType {
    Item,
    AtomicOrUnionType(Name),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Occurrence {
    One,
    Option,
    Many,
    NonEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentTest {
    Element(ElementTest),
    SchemaElement(SchemaElementTest),
    AnyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementTest {
    pub name_test: ElementNameOrWildcard,
    pub type_name: Option<ElementTypeName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementTypeName {
    pub name: Name,
    pub question_mark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementNameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeTest {
    pub name_test: AttribNameOrWildcard,
    pub type_name: Option<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttribNameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaElementTest {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaAttributeTest {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedFunctionTest {
    parameter_types: Vec<SequenceType>,
    return_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedMapTest {
    pub key_type: Name,
    pub value_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedArrayTest {
    pub item_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PITest {
    Name(String),
    StringLiteral(String),
}
