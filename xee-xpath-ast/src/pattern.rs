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
    pub left: Box<IntersectExceptExpr>,
    pub right: Box<IntersectExceptExpr>,
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
pub enum PathExpr {
    RootedPath(RootedPath),
    Slash(Option<RelativePathExpr>),
    DoubleSlash(RelativePathExpr),
    RelativePath(RelativePathExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Slash {
    Slash,
    DoubleSlash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RootedPathStart {
    VarRef(ast::Name),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RootedPath {
    pub start: RootedPathStart,
    pub predicates: Vec<ast::ExprS>,
    pub relative: Option<RootedPathRelative>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RootedPathRelative {
    Slash(RelativePathExpr),
    DoubleSlash(RelativePathExpr),
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
pub struct RelativePathExpr {
    pub first_step: StepExpr,
    pub steps: Vec<RelativePathStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RelativePathStep {
    Slash(StepExpr),
    DoubleSlash(StepExpr),
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
    expr: UnionExpr,
    predicates: Vec<ast::ExprS>,
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
