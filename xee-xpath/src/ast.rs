use ordered_float::OrderedFloat;

use crate::span::Spanned;

pub(crate) type ExprSingleS = Spanned<ExprSingle>;
pub(crate) type PrimaryExprS = Spanned<PrimaryExpr>;
pub(crate) type StepExprS = Spanned<StepExpr>;

pub(crate) type Expr = Vec<ExprSingleS>;
pub(crate) type ExprS = Spanned<Expr>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct XPath {
    // at least one entry
    pub(crate) exprs: ExprS,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ExprSingle {
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
pub(crate) struct ForExpr {
    pub(crate) var_name: Name,
    pub(crate) var_expr: Box<ExprSingleS>,
    pub(crate) return_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct QuantifiedExpr {
    pub(crate) quantifier: Quantifier,
    pub(crate) var_name: Name,
    pub(crate) var_expr: Box<ExprSingleS>,
    pub(crate) satisfies_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Name {
    pub(crate) name: String,
    pub(crate) namespace: Option<String>,
}

impl Name {
    pub(crate) fn new(name: String, namespace: Option<String>) -> Self {
        Name { name, namespace }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LetExpr {
    pub(crate) var_name: Name,
    pub(crate) var_expr: Box<ExprSingleS>,
    pub(crate) return_expr: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct IfExpr {
    pub(crate) condition: ExprS,
    pub(crate) then: Box<ExprSingleS>,
    pub(crate) else_: Box<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Quantifier {
    Some,
    Every,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PrimaryExpr {
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
pub(crate) enum UnaryLookup {
    Name(String),
    IntegerLiteral(i64),
    Expr(ExprS),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BinaryExpr {
    pub(crate) operator: Operator,
    pub(crate) left: PathExpr,
    pub(crate) right: PathExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Operator {
    // logical
    Or,
    And,
    // value comp
    ValueEq,
    ValueNe,
    ValueLt,
    ValueLe,
    ValueGt,
    ValueGe,
    // general comp
    GenEq,
    GenNe,
    GenLt,
    GenLe,
    GenGt,
    GenGe,
    // node comp
    Is,
    Precedes,
    Follows,
    // string concat
    Concat,
    // range
    Range,
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    IDiv,
    Mod,
    // set
    Union,
    Intersect,
    Except,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ApplyExpr {
    pub(crate) path_expr: PathExpr,
    pub(crate) operator: ApplyOperator,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ApplyOperator {
    SimpleMap(Vec<PathExpr>),
    Unary(Vec<UnaryOperator>),
    Arrow(Vec<(ArrowFunctionSpecifier, Vec<ExprSingleS>)>),
    Cast(SingleType),
    Castable(SingleType),
    Treat(SequenceType),
    InstanceOf(SequenceType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum UnaryOperator {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SingleType {
    pub(crate) name: EQName,
    pub(crate) question_mark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ArrowFunctionSpecifier {
    Name(EQName),
    VarRef(EQName),
    Expr(ExprS),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MapConstructor {
    pub(crate) entries: Vec<MapConstructorEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MapConstructorEntry {
    pub(crate) key: ExprSingleS,
    pub(crate) value: ExprSingleS,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ArrayConstructor {
    pub(crate) members: Vec<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DecimalLiteral {
    pub(crate) value: i64,
    pub(crate) fraction_digits: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Literal {
    Decimal(DecimalLiteral),
    Integer(i64),
    Double(OrderedFloat<f64>),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FunctionCall {
    pub(crate) name: Name,
    pub(crate) arguments: Vec<ExprSingleS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NamedFunctionRef {
    pub(crate) name: Name,
    pub(crate) arity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct InlineFunction {
    pub(crate) params: Vec<Param>,
    pub(crate) return_type: Option<SequenceType>,
    pub(crate) body: ExprS,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Param {
    pub(crate) name: Name,
    pub(crate) type_: Option<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Postfix {
    // vec contains at least 1 element
    Predicate(ExprS),
    ArgumentList(Vec<ExprSingleS>),
    Lookup(Lookup),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Lookup {
    Name(String),
    IntegerLiteral(i64),
    Expr(Vec<ExprSingleS>),
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PathExpr {
    pub(crate) steps: Vec<StepExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum StepExpr {
    PrimaryExpr(PrimaryExprS),
    PostfixExpr {
        primary: PrimaryExprS,
        postfixes: Vec<Postfix>,
    },
    AxisStep(AxisStep),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AxisStep {
    pub(crate) axis: Axis,
    pub(crate) node_test: NodeTest,
    pub(crate) predicates: Vec<ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Axis {
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
pub(crate) enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum NameTest {
    Name(Name),
    Star,
    LocalName(String),
    Namespace(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EQName {
    QName(QName),
    URIQualifiedName(URIQualifiedName),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum QName {
    PrefixedName(PrefixedName),
    UnprefixedName(UnprefixedName),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PrefixedName {
    pub(crate) prefix: String,
    pub(crate) local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct UnprefixedName {
    pub(crate) local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct URIQualifiedName {
    pub(crate) uri: String,
    pub(crate) local_part: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SequenceType {
    Empty,
    Item,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) item_type: ItemType,
    pub(crate) occurrence_indicator: Option<OccurrenceIndicator>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ItemType {
    KindTest(KindTest),
    FunctionTest(FunctionTest),
    MapTest(MapTest),
    ArrayTest(ArrayTest),
    AtomicOrUnionType(EQName),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OccurrenceIndicator {
    QuestionMark,
    Asterisk,
    Plus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KindTest {
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
pub(crate) enum DocumentTest {
    Element(ElementTest),
    SchemaElement(SchemaElementTest),
    AnyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ElementTest {
    pub(crate) name_test: ElementNameOrWildcard,
    pub(crate) type_name: Option<ElementTypeName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ElementTypeName {
    pub(crate) name: Name,
    pub(crate) question_mark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ElementNameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AttributeTest {
    pub(crate) name_test: AttribNameOrWildcard,
    pub(crate) type_name: Option<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AttribNameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SchemaElementTest {
    pub(crate) name: EQName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SchemaAttributeTest {
    pub(crate) name: EQName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedFunctionTest {
    parameter_types: Vec<SequenceType>,
    return_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedMapTest {
    pub(crate) key_type: EQName,
    pub(crate) value_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedArrayTest {
    pub(crate) item_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PITest {
    Name(String),
    StringLiteral(String),
}
