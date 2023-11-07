type XPathExpr = String;
type EqName = String;
type SequenceType = String;

#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) struct If {
    pub(crate) test: XPathExpr,
    pub(crate) content: Vec<SequenceConstructor>,
}

#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) enum SequenceConstructor {
    Text(String),
}

struct Variable {
    name: EqName,
    // TODO: should this be subsumed into the
    // content, as a variable may either have
    // a select or a content, but not both
    select: Option<XPathExpr>,
    as_: Option<SequenceType>,
    static_: bool,
    visbility: Visibility,
    // it's also possible to have an empty variable
    // in case visibility is static; this could
    // perhaps be modelled separately?
    content: Vec<SequenceConstructor>,
}

enum Visibility {
    Public,
    Private,
    Final,
    Abstract,
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
