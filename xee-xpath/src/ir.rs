// an Intermediate Representation in ANF - administrative normal form

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Expr {
    Atom(Atom),
    Let(Let),
    If(If),
    Binary(Binary),
    FunctionDefinition(FunctionDefinition),
    FunctionCall(FunctionCall),
    // XXX a context map and filter are likely required,
    // which place a full Context item on the stack
    Map(Map),
    Filter(Filter),
    Quantified(Quantified),
}

// not to be confused by an XPath atom; this is a variable or a constant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Atom {
    Const(Const),
    Variable(Name),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Const {
    Integer(i64),
    String(String),
    EmptySequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Name(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Let {
    pub(crate) name: Name,
    pub(crate) var_expr: Box<Expr>,
    pub(crate) return_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct If {
    pub(crate) condition: Atom,
    pub(crate) then: Box<Expr>,
    pub(crate) else_: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Binary {
    pub(crate) left: Atom,
    pub(crate) binary_op: BinaryOp,
    pub(crate) right: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BinaryOp {
    Add,
    Eq,
    Ne,
    Comma,
    Gt,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FunctionDefinition {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Param(pub(crate) Name);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FunctionCall {
    pub(crate) atom: Atom,
    pub(crate) args: Vec<Atom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Map {
    pub(crate) var_name: Name,
    pub(crate) var_atom: Atom,
    pub(crate) return_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Filter {
    pub(crate) var_name: Name,
    pub(crate) var_atom: Atom,
    pub(crate) return_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Quantified {
    pub(crate) quantifier: Quantifier,
    pub(crate) var_name: Name,
    pub(crate) var_atom: Atom,
    pub(crate) satisifies_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Quantifier {
    Some,
    Every,
}
