enum Expr {
    Literal(Literal),
    VarRef(VarRef),
    ContextItem(ContextItem),
    FunctionCall(FunctionCall),
}

enum PrimaryExpr {
    Literal(Literal),
    VarRef(VarRef),
    ContextItem(ContextItem),
    FunctionCall(FunctionCall),
}

struct DecimalLiteral {
    value: i64,
    fraction_digits: u8,
}

// derived from Decimal
struct IntegerLiteral {
    value: i64,
}

struct DoubleLiteral {
    value: f64,
}

struct StringLiteral {
    value: EQName,
}

enum Literal {
    DecimalLiteral(DecimalLiteral),
    IntegerLiteral(IntegerLiteral),
    DoubleLiteral(DoubleLiteral),
    StringLiteral(StringLiteral),
}

struct VarRef {
    name: String,
}

struct ContextItem {}

struct FunctionCall {
    name: EQName,
    arguments: Vec<Argument>,
}

enum Argument {
    Expr(Expr),
    ArgumentPlaceholder,
}

struct NamedFunctionRef {
    name: EQName,
    argument_count: u8,
}

struct InlineFunction {
    parameters: Vec<Param>,
    return_type: SequenceType,
    body: Expr,
}

struct Param {
    name: EQName,
    type_: SequenceType,
}

struct PostfixExpr {
    primary: PrimaryExpr,
    postfix: Postfix,
}

enum Postfix {
    Predicate(Expr),
    ArgumentList(Vec<Argument>),
    Lookup(Lookup),
}

enum Lookup {
    Name(String),
    IntegerLiteral(IntegerLiteral),
    Expr(Expr),
    Star,
}

struct PathExpr {
    steps: Vec<StepExpr>,
}

enum StepExpr {
    PostfixExpr(PostfixExpr),
    AxisStep(AxisStep),
}

struct AxisStep {
    axis: Axis,
    node_test: NodeTest,
    predicates: Vec<Expr>,
}

enum Axis {
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

enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

enum NameTest {
    EQName(EQName),
    Wildcard(Wildcard),
}

enum Wildcard {
    Star,
    LocalName(String),
    Prefix(String),
    BracedURILiteral(String),
}

enum EQName {
    QName(QName),
    URIQualifiedName(URIQualifiedName),
}

enum QName {
    PrefixedName(PrefixedName),
    UnprefixedName(UnprefixedName),
}

struct PrefixedName {
    prefix: String,
    local_part: String,
}

struct UnprefixedName {
    local_part: String,
}

struct URIQualifiedName {
    uri: String,
    local_part: String,
}

enum SequenceType {
    Empty,
    Item,
}

struct Item {
    item_type: ItemType,
    occurrence_indicator: Option<OccurrenceIndicator>,
}

enum ItemType {
    KindTest(KindTest),
    FunctionTest(FunctionTest),
    MapTest(MapTest),
    ArrayTest(ArrayTest),
    AtomicOrUnionType(EQName),
}

enum OccurrenceIndicator {
    QuestionMark,
    Asterisk,
    Plus,
}

enum KindTest {
    DocumentTest(Option<DocumentTest>),
    ElementTest(Option<ElementTest>),
    AttributeTest(Option<AttributeTest>),
    SchemaElementTest(SchemaElementTest),
    SchemaAttributeTest(SchemaAttributeTest),
    PITest(Option<PITest>),
    CommentTest,
    TextTest,
    NamespaceNodeTest,
    AnyKindTest,
}

enum DocumentTest {
    ElementTest(ElementTest),
    SchemaElementTest(SchemaElementTest),
    AnyKindTest,
}

struct ElementTest {
    name_test: ElementNameOrWildcard,
    type_name: Option<ElementTypeName>,
}

struct ElementTypeName {
    name: EQName,
    question_mark: bool,
}

enum ElementNameOrWildcard {
    EQName(EQName),
    Wildcard,
}

struct AttributeTest {
    name_test: AttribNameOrWildcard,
    type_name: Option<EQName>,
}

enum AttribNameOrWildcard {
    Name(EQName),
    Wildcard,
}

struct SchemaElementTest {
    name: EQName,
}

struct SchemaAttributeTest {
    name: EQName,
}

enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

struct TypedFunctionTest {
    parameter_types: Vec<SequenceType>,
    return_type: SequenceType,
}

enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

struct TypedMapTest {
    key_type: EQName,
    value_type: SequenceType,
}

enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

struct TypedArrayTest {
    item_type: SequenceType,
}

enum PITest {
    Name(String),
    StringLiteral(StringLiteral),
}
