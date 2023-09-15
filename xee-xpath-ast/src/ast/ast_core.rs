use chumsky::prelude::SimpleSpan;
use ibig::IBig;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use xot::Xot;

use crate::namespaces::NamespaceLookup;
pub use crate::operator::BinaryOperator;
use crate::span::{Spanned, WithSpan};

pub type Span = SimpleSpan;

pub type ExprSingleS = Spanned<ExprSingle>;
pub type PrimaryExprS = Spanned<PrimaryExpr>;
pub type StepExprS = Spanned<StepExpr>;
pub type ExprS = Spanned<Expr>;
pub type ExprOrEmpty = Option<Expr>;
pub type ExprOrEmptyS = Spanned<ExprOrEmpty>;
pub type NameS = Spanned<Name>;

impl WithSpan for ExprSingle {}
impl WithSpan for PrimaryExpr {}
impl WithSpan for StepExpr {}
impl WithSpan for Expr {}
impl WithSpan for Name {}
impl WithSpan for ExprOrEmpty {}

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

#[derive(Debug, Clone, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Name {
    name: String,
    prefix: Option<String>,
    namespace: Option<String>,
}

// a custom hasher that ignores the prefix
impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.namespace.hash(state);
    }
}

// and partial eq that ignores the prefix
impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.namespace == other.namespace
    }
}

impl Name {
    pub fn new(name: String, namespace: Option<String>, prefix: Option<String>) -> Self {
        Name {
            name,
            namespace,
            prefix,
        }
    }

    pub fn prefixed(prefix: &str, name: &str, namespaces: impl NamespaceLookup) -> Option<Self> {
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

    pub fn with_default_namespace(self, uri: Option<&str>) -> Self {
        if let Some(uri) = uri {
            if self.namespace.is_none() {
                return Name {
                    name: self.name,
                    namespace: Some(uri.to_string()),
                    prefix: None,
                };
            }
        }
        self
    }

    pub fn has_namespace_without_prefix(&self) -> bool {
        self.namespace.is_some() && self.prefix.is_none()
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn local_name(&self) -> &str {
        &self.name
    }

    pub fn to_full_name(&self) -> String {
        if let Some(prefix) = &self.prefix {
            if !prefix.is_empty() {
                format!("{}:{}", prefix, self.name)
            } else {
                self.name.clone()
            }
        } else {
            self.name.clone()
        }
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
    Expr(ExprOrEmptyS),
    ContextItem,
    FunctionCall(FunctionCall),
    NamedFunctionRef(NamedFunctionRef),
    InlineFunction(InlineFunction),
    MapConstructor(MapConstructor),
    ArrayConstructor(ArrayConstructor),
    UnaryLookup(KeySpecifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum KeySpecifier {
    NcName(String),
    Integer(IBig),
    Expr(ExprOrEmptyS),
    Star,
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
    pub optional: bool,
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
pub enum ArrayConstructor {
    Square(ExprS),
    Curly(ExprOrEmptyS),
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
    pub name: NameS,
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
    pub body: ExprOrEmptyS,
}

// a function signature as described by:
// https://www.w3.org/TR/xpath-functions-31/#func-signatures
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Signature {
    pub name: NameS,
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
    Lookup(KeySpecifier),
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
    Name(NameS),
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
#[cfg_attr(test, derive(serde::Serialize))]
pub enum DocumentTest {
    Element(Option<ElementOrAttributeTest>),
    SchemaElement(SchemaElementTest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ElementOrAttributeTest {
    pub name_or_wildcard: NameOrWildcard,
    pub type_name: Option<TypeName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct TypeName {
    pub name: NameS,
    // only relevant for elements; for attributes it's always true
    pub can_be_nilled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum NameOrWildcard {
    Name(NameS),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SchemaElementTest {
    pub name: NameS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SchemaAttributeTest {
    pub name: NameS,
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
    pub parameter_types: Vec<SequenceType>,
    pub return_type: SequenceType,
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
