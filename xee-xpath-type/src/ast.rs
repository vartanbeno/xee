use xee_name::Name;
use xee_schema_type::Xs;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SequenceType {
    Empty,
    Item(Item),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Item {
    pub item_type: ItemType,
    pub occurrence: Occurrence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ItemType {
    Item,
    AtomicOrUnionType(Xs),
    KindTest(KindTest),
    FunctionTest(FunctionTest),
    MapTest(MapTest),
    ArrayTest(ArrayTest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Occurrence {
    One,
    Option,
    Many,
    NonEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum KindTest {
    Document(Option<DocumentTest>),
    Element(Option<ElementOrAttributeTest>),
    Attribute(Option<ElementOrAttributeTest>),
    SchemaElement(SchemaElementTest),
    SchemaAttribute(SchemaAttributeTest),
    PI(Option<PITest>),
    Comment,
    Text,
    NamespaceNode,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DocumentTest {
    Element(Option<ElementOrAttributeTest>),
    SchemaElement(SchemaElementTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ElementOrAttributeTest {
    pub name_or_wildcard: NameOrWildcard,
    pub type_name: Option<TypeName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeName {
    pub name: Xs,
    // only relevant for elements; for attributes it's always true
    pub can_be_nilled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NameOrWildcard {
    Name(Name),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SchemaElementTest {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SchemaAttributeTest {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(Box<TypedFunctionTest>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypedFunctionTest {
    pub parameter_types: Vec<SequenceType>,
    pub return_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MapTest {
    AnyMapTest,
    TypedMapTest(Box<TypedMapTest>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypedMapTest {
    pub key_type: Xs,
    pub value_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(Box<TypedArrayTest>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypedArrayTest {
    pub item_type: SequenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PITest {
    Name(String),
    StringLiteral(String),
}
