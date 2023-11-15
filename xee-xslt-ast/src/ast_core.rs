use xee_xpath_ast::ast as xpath_ast;

// TODO: standard attribute support such as expand-text during the parse, this
// should be respected and parse into the right thing, so the AST does not need
// to retain knowledge of expand-text

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<&xot::Span> for Span {
    fn from(span: &xot::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

type EqName = String;
type QName = String;
type NcName = String;
type SequenceType = xpath_ast::SequenceType;
type ItemType = xpath_ast::ItemType;
type Pattern = String;
type Token = String;
type Uri = String;
type Language = String;
type Prefix = String;
type Decimal = String; // HTML version
type NmToken = String;
type Id = String;
type PcData = String;

// type Expression = xpath_ast::XPath;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Expression {
    pub xpath: xpath_ast::XPath,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Templ<V>
where
    V: Clone + PartialEq + Eq,
{
    // TODO: this is not right; we need to produce a V from an
    // AttributeValueTemplate during runtime
    pub value: V,
    pub template: AttributeValueTemplate,
}

type AttributeValueTemplate = Vec<AttributeValueTemplateItem>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AttributeValueTemplateItem {
    // TODO: we probably need to store span information in here too
    Expression(Expression),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Visibility {
    Public,
    Private,
    Final,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum VisibilityWithAbstract {
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
pub enum Order {
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
pub struct Accept {
    pub component: Component,
    pub names: Vec<Token>,
    pub visibility: VisibilityWithHidden,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Accumulator {
    pub name: EqName,
    pub initial_value: Expression,
    pub as_: Option<SequenceType>,
    pub streamable: Option<bool>,

    pub rules: Vec<AccumulatorRule>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AccumulatorRule {
    pub match_: Pattern,
    pub phase: Option<AccumulatorPhase>,
    pub select: Option<Expression>,
    pub content: SequenceConstructor,

    pub span: Span,
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
    pub regex: Templ<String>,
    pub flags: Option<Templ<String>>,

    pub matching_substring: Option<MatchingSubstring>,
    pub non_matching_substring: Option<NonMatchingSubstring>,
    pub fallbacks: Vec<Fallback>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ApplyImports {
    pub with_params: Vec<WithParam>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ApplyTemplates {
    pub select: Option<Expression>,
    pub mode: Option<Token>,

    pub content: Vec<ApplyTemplatesContent>,

    pub span: Span,
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
    pub error_code: Option<Templ<EqName>>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Attribute {
    pub name: Templ<QName>,
    pub namespace: Option<Templ<Uri>>,
    pub select: Option<Expression>,
    pub separator: Option<Templ<String>>,
    pub type_: Option<EqName>,
    pub validation: Option<Validation>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AttributeSet {
    pub name: EqName,
    pub use_attribute_sets: Option<Vec<EqName>>,
    pub visibility: Option<VisibilityWithAbstract>,
    pub streamable: Option<bool>,

    pub content: Vec<Attribute>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Break {
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CallTemplate {
    pub name: EqName,

    pub content: Vec<WithParam>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Catch {
    pub errors: Option<Vec<Token>>,
    pub select: Option<Expression>,

    pub content: Vec<SequenceConstructor>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CharacterMap {
    pub name: EqName,
    pub use_character_maps: Option<Vec<EqName>>,

    pub content: Vec<OutputCharacter>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Choose {
    when: Vec<When>,
    otherwise: Option<Otherwise>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Comment {
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContextItem {
    pub as_: Option<ItemType>,
    pub use_: Option<Use>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Copy {
    pub select: Option<Expression>,
    pub copy_namespaces: bool,
    pub inherit_namespaces: bool,
    pub use_attribute_sets: Option<Vec<EqName>>,
    pub type_: Option<EqName>,
    pub validation: Validation,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CopyOf {
    pub select: Expression,
    pub copy_accumulators: Option<bool>,
    pub copy_namespaces: Option<bool>,
    pub type_: Option<EqName>,
    pub validation: Option<Validation>,

    pub span: Span,
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

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Document {
    pub validation: Option<Validation>,
    pub type_: Option<EqName>,
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Element {
    pub name: Templ<EqName>,
    pub namespace: Option<Templ<Uri>>,
    pub inherit_namespaces: Option<bool>,
    pub use_attribute_sets: Option<Vec<EqName>>,
    pub type_: Option<EqName>,
    pub validation: Option<Validation>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Evaluate {
    pub xpath: Expression,
    pub as_: Option<SequenceType>,
    pub base_uri: Option<Templ<Uri>>,
    pub with_params: Option<Expression>,
    pub context_item: Option<Expression>,
    pub namespace_context: Option<Expression>,
    pub schema_aware: Option<Templ<bool>>,

    pub content: Vec<EvaluateContent>,

    pub span: Span,
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
    pub names: Vec<Token>,
    pub visibility: VisibilityWithAbstract,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Fallback {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForEach {
    pub select: Expression,

    pub sort: Vec<Sort>,
    pub constructor: SequenceConstructor,

    pub span: Span,
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
    pub collation: Option<Templ<Uri>>,

    pub sort: Vec<Sort>,
    pub constructor: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Fork {
    pub content: ForkContent,

    pub span: Span,
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
    pub visibility: Option<VisibilityWithAbstract>,
    pub streamability: Option<Streamability>,
    pub override_extension_function: Option<bool>,
    pub new_each_time: Option<NewEachTime>,
    pub cache: Option<bool>,

    pub params: Vec<Param>,
    pub constructor: SequenceConstructor,

    pub span: Span,
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
    pub as_: Option<ItemType>,
    pub use_: Option<Use>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct If {
    pub test: Expression,
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Import {
    href: Uri,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportSchema {
    namespace: Option<Uri>,
    schema_location: Option<Uri>,

    content: Option<Schema>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Include {
    pub href: Uri,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Iterate {
    pub select: Expression,

    pub params: Vec<Param>,
    pub on_completion: Option<OnCompletion>,
    pub constructor: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Key {
    pub name: EqName,
    pub match_: Pattern,
    pub use_: Option<Expression>,
    pub composite: Option<bool>,
    pub collation: Option<Uri>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Map {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MapEntry {
    pub key: Expression,
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MatchingSubstring {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Merge {
    pub merge_source: Vec<MergeSource>,
    pub merge_action: MergeAction,
    pub fallback: Vec<Fallback>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MergeAction {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MergeKey {
    pub select: Option<Expression>,
    pub lang: Option<Templ<Language>>,
    pub order: Option<Templ<Order>>,
    pub collation: Option<Templ<Uri>>,
    pub case_order: Option<Templ<CaseOrder>>,
    pub data_type: Option<Templ<DataType>>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MergeSource {
    pub name: Option<NcName>,
    pub for_each_time: Option<Expression>,
    pub for_each_source: Option<Expression>,
    pub select: Expression,
    pub streamable: Option<bool>,
    pub use_accumlators: Option<Vec<Token>>,
    pub sort_before_merge: Option<bool>,
    pub validation: Option<Validation>,
    pub type_: Option<EqName>,

    pub content: Vec<MergeKey>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Message {
    pub select: Option<Expression>,
    pub terminate: Option<Templ<bool>>,
    pub error_code: Option<Templ<EqName>>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Mode {
    pub name: Option<EqName>,
    pub streamable: Option<bool>,
    pub use_accumulators: Option<Vec<Token>>,
    pub on_no_match: Option<OnNoMatch>,
    pub on_multiple_match: Option<OnMultipleMatch>,
    pub warning_on_no_match: Option<bool>,
    pub warning_on_multiple_match: Option<bool>,
    pub typed: Option<Typed>,
    pub visibility: Option<Visibility>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum OnNoMatch {
    DeepCopy,
    ShallowCopy,
    DeepSkip,
    ShallowSkip,
    TextOnlyCopy,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum OnMultipleMatch {
    UseLast,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Typed {
    Boolean,
    Strict,
    Lax,
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Namespace {
    pub name: Option<Templ<NcName>>,
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NamespaceAlias {
    pub stylesheet_prefix: PrefixOrDefault,
    pub result_prefix: PrefixOrDefault,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrefixOrDefault {
    Prefix(Prefix),
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NextIteration {
    pub params: Vec<Param>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NextMatch {
    pub content: Vec<NextMatchContent>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NextMatchContent {
    WithParam(WithParam),
    Fallback(Fallback),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NonMatchingSubstring {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Number {
    pub value: Option<Expression>,
    pub select: Option<Expression>,
    pub level: Option<NumberLevel>,
    pub count: Option<Pattern>,
    pub from: Option<Pattern>,
    pub format: Option<Templ<String>>,
    pub lang: Option<Templ<Language>>,
    pub letter_value: Option<Templ<LetterValue>>,
    pub ordinal: Option<Templ<String>>,
    pub start_at: Option<Templ<String>>,
    pub grouping_separator: Option<Templ<char>>,
    pub grouping_size: Option<Templ<usize>>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NumberLevel {
    Single,
    Multiple,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LetterValue {
    Alphabetic,
    Traditional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OnCompletion {
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OnEmpty {
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OnNonEmpty {
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Otherwise {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Output {
    pub name: Option<EqName>,
    pub method: Option<OutputMethod>,
    pub allow_duplicate_names: Option<bool>,
    pub build_tree: Option<bool>,
    pub byte_order_mark: Option<bool>,
    pub cdata_section_elements: Option<Vec<EqName>>,
    pub doctype_public: Option<String>,
    pub doctype_system: Option<String>,
    pub encoding: Option<String>,
    pub escape_uri_attributes: Option<bool>,
    pub include_content_type: Option<Decimal>,
    pub ident: Option<bool>,
    pub item_separator: Option<String>,
    pub json_node_output_method: Option<JsonNodeOutputMethod>,
    pub media_type: Option<String>,
    pub normalization_form: Option<NormalizationForm>,
    pub omit_xml_declaration: Option<bool>,
    pub parameter_document: Option<Uri>,
    pub standalone: Option<Standalone>,
    pub suppress_indentation: Option<Vec<EqName>>,
    pub undeclare_prefixes: Option<bool>,
    pub use_character_maps: Option<Vec<EqName>>,
    pub version: Option<NmToken>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum OutputMethod {
    Xml,
    Html,
    Xhtml,
    Text,
    Json,
    Adaptive,
    EqName(EqName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum JsonNodeOutputMethod {
    Xml,
    Html,
    Xhtml,
    Text,
    EqName(EqName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NormalizationForm {
    Nfc,
    Nfd,
    Nfkc,
    Nfkd,
    FullyNormalized,
    None,
    NmToken(NmToken),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Standalone {
    Bool(bool),
    Omit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OutputCharacter {
    pub character: char,
    pub string: String,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Override {
    pub content: Vec<OverrideContent>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum OverrideContent {
    Template(Template),
    Function(Function),
    Variable(Variable),
    Param(Param),
    AttributeSet(AttributeSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Package {
    pub id: Option<Id>,
    pub name: Option<Uri>,
    pub package_version: Option<String>,
    pub version: Decimal,
    pub input_type_annotations: Option<InputTypeAnnotations>,
    pub declared_modes: Option<bool>,
    pub default_mode: Option<DefaultMode>,
    pub default_validation: Option<DefaultValidation>,
    pub default_collation: Option<Vec<Uri>>,
    pub extension_element_prefixes: Option<Vec<Prefix>>,
    pub exclude_result_prefixes: Option<Vec<Prefix>>,
    pub expand_text: Option<bool>,
    pub use_when: Option<Expression>,
    pub xpath_default_namespace: Option<Uri>,

    pub content: Vec<PackageContent>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InputTypeAnnotations {
    Preserve,
    Strip,
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DefaultMode {
    EqName(EqName),
    Unnamed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DefaultValidation {
    Preserve,
    Strip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PackageContent {
    Expose(Expose),
    Declarations(Declarations),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Param {
    pub name: EqName,
    pub select: Option<Expression>,
    pub as_: Option<SequenceType>,
    pub required: Option<bool>,
    pub tunnel: Option<bool>,
    pub static_: Option<bool>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PerformSort {
    pub select: Option<Expression>,

    pub sorts: Vec<Sort>,
    pub constructor: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PreserveSpace {
    pub elements: Vec<Token>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ProcessingInstruction {
    pub name: Templ<NcName>,
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ResultDocument {
    pub format: Option<Templ<EqName>>,
    pub href: Option<Templ<Uri>>,
    pub validation: Option<Validation>,
    pub type_: EqName,
    pub method: Option<Templ<OutputMethod>>,
    pub allow_duplicate_names: Option<Templ<bool>>,
    pub build_tree: Option<Templ<bool>>,
    pub bye_order_mark: Option<Templ<bool>>,
    pub cdata_section_elements: Option<Templ<Vec<EqName>>>,
    pub doctype_public: Option<Templ<String>>,
    pub doctype_system: Option<Templ<String>>,
    pub encoding: Option<Templ<String>>,
    pub escape_uri_attributes: Option<Templ<bool>>,
    pub html_version: Option<Templ<Decimal>>,
    pub include_content_type: Option<Templ<bool>>,
    pub indent: Option<Templ<bool>>,
    pub item_separator: Option<Templ<String>>,
    pub json_node_output_method: Option<Templ<JsonNodeOutputMethod>>,
    pub media_type: Option<Templ<String>>,
    pub normalization_form: Option<Templ<NormalizationForm>>,
    pub omit_xml_declaration: Option<Templ<bool>>,
    pub parameter_document: Option<Templ<Uri>>,
    pub standalone: Option<Templ<Standalone>>,
    pub suppress_indentation: Option<Templ<Vec<EqName>>>,
    pub undeclare_prefixes: Option<Templ<bool>>,
    pub use_character_maps: Option<Vec<EqName>>,
    pub version: Option<Templ<NmToken>>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sequence {
    pub select: Option<Expression>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sort {
    pub select: Option<Expression>,
    pub lang: Option<Templ<Language>>,
    pub order: Option<Templ<Order>>,
    pub collation: Option<Templ<Uri>>,
    pub stable: Option<Templ<bool>>,
    pub case_order: Option<Templ<CaseOrder>>,
    pub data_type: Option<Templ<DataType>>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SourceDocument {
    pub href: Templ<Uri>,
    pub streamable: Option<bool>,
    pub use_accumulators: Option<Vec<Token>>,
    pub validation: Option<Validation>,
    pub type_: Option<EqName>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StripSpace {
    pub elements: Vec<Token>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Stylesheet {
    pub id: Option<Id>,
    pub version: Decimal,
    pub default_mode: Option<DefaultMode>,
    pub default_validation: Option<DefaultValidation>,
    pub input_type_annotations: Option<InputTypeAnnotations>,
    pub default_collation: Option<Vec<Uri>>,
    pub extension_element_prefixes: Option<Vec<Prefix>>,
    pub exclude_result_prefixes: Option<Vec<Prefix>>,
    pub expand_text: Option<bool>,
    pub use_when: Option<Expression>,
    pub xpath_default_namespace: Option<Uri>,

    pub declarations: Declarations,

    pub span: Span,
}

// Transform is an alias for Stylesheet. TODO: rename to Transform?

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Template {
    pub match_: Option<Pattern>,
    pub name: Option<EqName>,
    pub priority: Option<Decimal>,
    pub mode: Option<Vec<Token>>,
    pub as_: Option<SequenceType>,
    pub visibility: Option<VisibilityWithAbstract>,

    pub context_item: Option<ContextItem>,
    pub params: Vec<Param>,
    pub constructor: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Text {
    // DEPRECATED
    pub disable_output_escaping: Option<bool>,
    pub content: PcData,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Try {
    pub select: Option<Expression>,
    pub rollback_output: Option<bool>,

    pub constructor: SequenceConstructor,
    // TODO: at least one catch needs to be there, so could fold it into
    // the catches block
    pub catch: Catch,
    pub catches: Vec<TryCatchOrFinally>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TryCatchOrFinally {
    Catch(Catch),
    Fallback(Fallback),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UsePackage {
    pub name: Uri,
    pub package_version: Option<String>,

    pub content: Vec<UsePackageContent>,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum UsePackageContent {
    Accept(Accept),
    Override(Override),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ValueOf {
    pub select: Option<Expression>,
    pub separator: Option<Templ<String>>,
    // DEPRECATED
    pub disable_output_escaping: Option<bool>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Variable {
    pub name: EqName,
    pub select: Option<Expression>,
    pub as_: Option<SequenceType>,
    pub static_: bool,
    pub visibility: Option<VisibilityWithAbstract>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct When {
    pub test: Expression,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WherePopulated {
    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WithParam {
    pub name: EqName,
    pub select: Option<Expression>,
    pub as_: Option<SequenceType>,
    pub tunnel: Option<bool>,

    pub content: SequenceConstructor,

    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SequenceConstructorItem {
    // TODO: should support text value template
    TextNode(String),
    // TODO: to add: literal result element, which can contain sequence constructor
    // in turn as well
    AnalyzeString(Box<AnalyzeString>),
    ApplyImports(Box<ApplyImports>),
    ApplyTemplates(Box<ApplyTemplates>),
    Assert(Box<Assert>),
    Attribute(Box<Attribute>),
    Break(Box<Break>),
    CallTemplate(Box<CallTemplate>),
    Choose(Box<Choose>),
    Comment(Box<Comment>),
    Copy(Box<Copy>),
    CopyOf(Box<CopyOf>),
    Document(Box<Document>),
    Element(Box<Element>),
    Evaluate(Box<Evaluate>),
    Fallback(Box<Fallback>),
    ForEach(Box<ForEach>),
    ForEachGroup(Box<ForEachGroup>),
    Fork(Box<Fork>),
    If(Box<If>),
    Iterate(Box<Iterate>),
    Map(Box<Map>),
    MapEntry(Box<MapEntry>),
    Merge(Box<Merge>),
    Message(Box<Message>),
    Namespace(Box<Namespace>),
    NextIteration(Box<NextIteration>),
    NextMatch(Box<NextMatch>),
    Number(Box<Number>),
    OnEmpty(Box<OnEmpty>),
    OnNonEmpty(Box<OnNonEmpty>),
    PerformSort(Box<PerformSort>),
    ProcessingInstruction(Box<ProcessingInstruction>),
    ResultDocument(Box<ResultDocument>),
    Sequence(Box<Sequence>),
    SourceDocument(Box<SourceDocument>),
    Text(Box<Text>),
    Try(Box<Try>),
    ValueOf(Box<ValueOf>),
    Variable(Box<Variable>),
    WherePopulated(Box<WherePopulated>),
}

pub type SequenceConstructor = Vec<SequenceConstructorItem>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Declaration {
    Accumulator(Box<Accumulator>),
    CharacterMap(Box<CharacterMap>),
    DecimalFormat(Box<DecimalFormat>),
    Function(Box<Function>),
    GlobalContextItem(Box<GlobalContextItem>),
    Import(Box<Import>),
    ImportSchema(Box<ImportSchema>),
    Include(Box<Include>),
    Key(Box<Key>),
    Mode(Box<Mode>),
    NamespaceAlias(Box<NamespaceAlias>),
    Output(Box<Output>),
    Param(Box<Param>),
    PreserveSpace(Box<PreserveSpace>),
    StripSpace(Box<StripSpace>),
    Template(Box<Template>),
    UsePackage(Box<UsePackage>),
    Variable(Box<Variable>),
}

pub type Declarations = Vec<Declaration>;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Instruction {
    Variable(Variable),
    If(If),
    Copy(Copy),
}

// xs:schema
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Schema {
    // TODO
}
