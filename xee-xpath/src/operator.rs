#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BinaryOperator {
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
    IntDiv,
    Mod,
    // set
    Union,
    Intersect,
    Except,
    // Comma operator; only used in IR, not in AST
    Comma,
}
