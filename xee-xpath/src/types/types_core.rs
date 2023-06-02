use crate::Name;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SequenceType {
    Empty,
    Item(Item),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) item_type: ItemType,
    pub(crate) occurrence: Occurrence,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ItemType {
    Item,
    AtomicOrUnionType(Name),
    KindTest(KindTest),
    FunctionTest(FunctionTest),
    MapTest(MapTest),
    ArrayTest(ArrayTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Occurrence {
    One,
    Option,
    Many,
    NonEmpty,
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
    pub(crate) name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SchemaAttributeTest {
    pub(crate) name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedFunctionTest {
    parameter_types: Vec<SequenceType>,
    return_type: Box<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedMapTest {
    pub(crate) key_type: Name,
    pub(crate) value_type: Box<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedArrayTest {
    pub(crate) item_type: Box<SequenceType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PITest {
    Name(String),
    StringLiteral(String),
}
