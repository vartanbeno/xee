use xee_xpath_ast::ast as xpath_ast;

type Expression = xpath_ast::XPath;
type EqName = String;
type QName = String;
type SequenceType = xpath_ast::SequenceType;
type ItemType = xpath_ast::ItemType;
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Validation {
    Strict,
    Lax,
    Preserve,
    Strip,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Component {
    Template,
    Function,
    AttributeSet,
    Variable,
    Mode,
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Use {
    Required,
    Optional,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Accept {
    pub component: Component,
    pub names: Tokens,
    pub visibility: VisibilityWithHidden,
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
    pub validation: Option<Validation>,
    pub content: SequenceConstructor,
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
pub struct CharacterMap {
    pub name: EqName,
    pub use_character_maps: Option<EqNames>,
    pub content: Vec<OutputCharacter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Choose {
    when: Vec<When>,
    otherwise: Option<Otherwise>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Comment {
    pub select: Option<Expression>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContextItem {
    pub as_: Option<ItemType>,
    pub use_: Option<Use>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Copy {
    pub select: Option<Expression>,
    pub copy_namespaces: Option<bool>,
    pub inherit_namespaces: Option<bool>,
    pub use_attribute_sets: Option<EqNames>,
    pub type_: Option<EqName>,
    pub validation: Option<Validation>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CopyOf {
    pub select: Expression,
    pub copy_accumulators: Option<bool>,
    pub copy_namespaces: Option<bool>,
    pub type_: Option<EqName>,
    pub validation: Option<Validation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DecimalFormat {
    pub name: Option<EqName>,
    pub decimal_separator: Option<char>,
    pub grouping_separator: Option<char>,
    pub infinity: Option<String>,
    pub minus_sign: Option<char>,
    pub exponent_separator: Option<char>,
    pub nan: Option<String>,
    pub percent: Option<char>,
    pub per_mille: Option<char>,
    pub zero_digit: Option<char>,
    pub digit: Option<char>,
    pub pattern_separator: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Document {
    pub validation: Option<Validation>,
    pub type_: Option<EqName>,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Element {
    name: Template<EqName>,
    namespace: Option<Template<Uri>>,
    inherit_namespaces: Option<bool>,
    use_attribute_sets: Option<EqNames>,
    type_: Option<EqName>,
    validation: Option<Validation>,
    content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Evaluate {
    xpath: Expression,
    as_: Option<SequenceType>,
    base_uri: Option<Template<Uri>>,
    with_params: Option<Expression>,
    context_item: Option<Expression>,
    namespace_context: Option<Expression>,
    schema_aware: Option<Template<bool>>,
    content: Vec<EvaluateContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum EvaluateContent {
    WithParam(WithParam),
    Fallback(Fallback),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Expose {
    pub component: Component,
    pub names: Tokens,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Fallback {
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForEach {
    pub select: Expression,

    pub sort: Vec<Sort>,
    pub constructor: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForEachGroup {
    pub select: Expression,
    pub group_by: Option<Expression>,
    pub group_adjacent: Option<Expression>,
    pub group_starting_with: Option<Pattern>,
    pub group_ending_with: Option<Pattern>,
    pub composite: Option<bool>,
    pub collation: Option<Template<Uri>>,

    pub sort: Vec<Sort>,
    pub constructor: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Fork {
    pub content: ForkContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ForkContent {
    Fallback(Vec<Fallback>),
    SequenceFallbacks(Vec<(Sequence, Vec<Fallback>)>),
    ForEachGroup((ForEachGroup, Vec<Fallback>)),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Function {
    pub name: EqName,
    pub as_: Option<SequenceType>,
    pub visibility: Option<Visibility>,
    pub streamability: Option<Streamability>,
    pub override_extension_function: Option<bool>,
    pub new_each_time: Option<NewEachTime>,
    pub cache: Option<bool>,

    pub params: Vec<Param>,
    pub constructor: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Streamability {
    Unclassified,
    Absorbing,
    Inspection,
    Filter,
    ShallowDescent,
    DeepDescent,
    Ascent,
    EqName(EqName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NewEachTime {
    Yes,
    No,
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GlobalContextItem {
    as_: Option<ItemType>,
    use_: Option<Use>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct If {
    pub test: Expression,
    pub content: SequenceConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Otherwise {
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OutputCharacter {
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Param {
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sequence {
    // TODO
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
pub struct When {
    // TODO
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

pub type SequenceConstructor = Vec<SequenceConstructorItem>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SequenceConstructorItem {
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
    pub content: SequenceConstructor,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Instruction {
    Variable(Variable),
    If(If),
}
