use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Pattern {
    PredicatePattern(PredicatePattern),
    UnionExpr(UnionExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PredicatePattern {
    pub predicates: Vec<ast::ExprS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnionExpr {
    pub intersect_exprs: Vec<IntersectExceptExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum IntersectExceptOperator {
    Intersect,
    Except,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IntersectExceptExpr {
    pub operator: IntersectExceptOperator,
    pub left: Box<PathExpr>,
    pub right: Box<PathExpr>,
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
    PostfixExprP(PostfixExpr),
    AxisStep(AxisStep),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PostfixExpr {
    pub expr: UnionExpr,
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
