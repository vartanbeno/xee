use crate::ast;

pub use crate::ast::{NameTest, NodeTest};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Pattern {
    Predicate(PredicatePattern),
    Expr(ExprPattern),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PredicatePattern {
    pub predicates: Vec<ast::ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ExprPattern {
    Path(PathExpr),
    BinaryExpr(BinaryExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BinaryExpr {
    pub operator: Operator,
    pub left: Box<ExprPattern>,
    pub right: Box<ExprPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Operator {
    Union,
    Intersect,
    Except,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PathExpr {
    pub root: PathRoot,
    pub steps: Vec<StepExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PathRoot {
    Rooted {
        root: RootExpr,
        predicates: Vec<ast::ExprS>,
    },
    AbsoluteSlash,
    AbsoluteDoubleSlash,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RootExpr {
    VarRef(ast::Name),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionCall {
    pub name: OuterFunctionName,
    // one or more always
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum OuterFunctionName {
    Doc,
    Id,
    ElementWithId,
    Key,
    Root,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Argument {
    VarRef(ast::Name),
    Literal(ast::Literal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StepExpr {
    PostfixExpr(PostfixExpr),
    AxisStep(AxisStep),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PostfixExpr {
    pub expr: ExprPattern,
    pub predicates: Vec<ast::ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AxisStep {
    pub forward: ForwardAxis,
    pub node_test: ast::NodeTest,
    pub predicates: Vec<ast::ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForwardAxisNodeTest {
    pub axis: ForwardAxis,
    pub node_test: ast::NodeTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ForwardAxis {
    Child,
    Descendant,
    Attribute,
    Self_,
    DescendantOrSelf,
    Namespace,
}
