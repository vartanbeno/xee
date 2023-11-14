use xee_xpath_ast::ast as xpath_ast;

type Expression = xpath_ast::XPath;
type EqName = String;
type QName = String;
type SequenceType = xpath_ast::SequenceType;
type Pattern = String;
type Token = String;
type Uri = String;
type Language = String;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Tokens(Vec<Token>);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EqNames(Vec<EqName>);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Template<V>
where
    V: Clone + PartialEq + Eq,
{
    pub value: V,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Accept {
    pub component: AcceptComponent,
    pub names: Tokens,
    pub visibility: VisibilityWithHidden,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AcceptComponent {
    Template,
    Function,
    AttributeSet,
    Variable,
    Mode,
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Accumulator {
    pub name: EqName,
    pub initial_value: Expression,
    pub as_: Option<SequenceType>,
    pub streamable: Option<bool>,
    pub content: Vec<AccumulatorRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AccumulatorRule {
    pub match_: Pattern,
    pub phase: Option<AccumulatorPhase>,
    pub select: Option<Expression>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AccumulatorPhase {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AnalyzeString {
    pub select: Expression,
    pub regex: Template<String>,
    pub flags: Option<Template<String>>,

    pub matching_substring: Option<SequenceConstructor>,
    pub non_matching_substring: Option<SequenceConstructor>,
    pub fallbacks: Vec<SequenceConstructor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ApplyImports {
    pub content: Vec<WithParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ApplyTemplates {
    pub select: Option<Expression>,
    pub mode: Option<Token>,
    pub content: Vec<ApplyTemplatesContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ApplyTemplatesContent {
    Sort(Sort),
    WithParam(WithParam),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Assert {
    pub test: Expression,
    pub select: Option<Expression>,
    pub error_code: Option<Template<EqName>>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Attribute {
    pub name: Template<QName>,
    pub namespace: Option<Template<Uri>>,
    pub select: Option<Expression>,
    pub separator: Option<Template<String>>,
    pub type_: Option<EqName>,
    pub validation: Option<AttributeValidation>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AttributeValidation {
    Strict,
    Lax,
    Preserve,
    Strip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AttributeSet {
    pub name: EqName,
    pub use_attribute_sets: Option<EqNames>,
    pub visibility: Option<Visibility>,
    pub streamable: Option<bool>,
    pub content: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Break {
    pub select: Option<Expression>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CallTemplate {
    pub name: EqName,
    pub content: Vec<WithParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Catch {
    pub errors: Option<Tokens>,
    pub select: Option<Expression>,
    pub content: Vec<SequenceConstructor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sort {
    pub select: Option<Expression>,
    pub lang: Option<Template<Language>>,
    pub order: Option<Template<SortOrder>>,
    pub collation: Option<Template<Uri>>,
    pub stable: Option<Template<bool>>,
    pub case_order: Option<Template<CaseOrder>>,
    pub data_type: Option<Template<DataType>>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CaseOrder {
    UpperFirst,
    LowerFirst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DataType {
    Text,
    Number,
    EQName(EqName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WithParam {
    pub name: EqName,
    pub select: Option<Expression>,
    pub as_: Option<SequenceType>,
    pub tunnel: Option<bool>,
    pub content: SequenceConstructor,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct If {
    pub(crate) test: Expression,
    pub(crate) content: Vec<SequenceConstructor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SequenceConstructor {
    Text(String),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Variable {
    pub name: EqName,
    // TODO: should this be subsumed into the
    // content, as a variable may either have
    // a select or a content, but not both
    pub select: Option<Expression>,
    // as_: Option<SequenceType>,
    // static_: bool,
    // visbility: Visibility,
    // // it's also possible to have an empty variable
    // // in case visibility is static; this could
    // // perhaps be modelled separately?
    pub content: Vec<SequenceConstructor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Visibility {
    Public,
    Private,
    Final,
    Abstract,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum VisibilityWithHidden {
    Public,
    Private,
    Final,
    Abstract,
    Hidden,
}

struct Function {
    name: EqName,
    as_: Option<SequenceType>,
    visbility: Visibility,
    override_extension_function: bool,
    new_each_time: NewEachTime,
    cache: bool,
    params: Vec<Param>,
    content: Vec<SequenceConstructor>,
}

struct Param {}

enum NewEachTime {
    Bool(bool),
    Maybe,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Instruction {
    Variable(Variable),
    If(If),
}
