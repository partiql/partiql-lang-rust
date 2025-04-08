use crate::ast::{AstNode, Expr, SymbolPrimitive};
use partiql_ast_macros::Visit;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphTable {
    pub graph_match: AstNode<GraphMatch>,
}

/// `<expr> MATCH <graph_pattern>`
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatch {
    pub expr: Box<Expr>,
    pub pattern: AstNode<GraphPattern>,
    #[visit(skip)]
    pub shape: GraphTableShape,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphPattern {
    #[visit(skip)]
    pub mode: Option<GraphMatchMode>,
    pub patterns: Vec<AstNode<GraphPathPattern>>,
    #[visit(skip)]
    pub keep: Option<GraphPathPrefix>,
    pub where_clause: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchMode {
    DifferentEdges,
    RepeatableElements,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphTableShape {
    pub rows: Option<AstNode<GraphTableRows>>,
    pub cols: Option<AstNode<GraphTableColumns>>,
    pub export: Option<AstNode<GraphTableExport>>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphTableRows {
    #[default]
    OneRowPerMatch,
    OneRowPerVertex {
        v: SymbolPrimitive,
        in_paths: Option<Vec<SymbolPrimitive>>,
    },
    OneRowPerStep {
        v1: SymbolPrimitive,
        e: SymbolPrimitive,
        v2: SymbolPrimitive,
        in_paths: Option<Vec<SymbolPrimitive>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphTableColumns {
    pub columns: Vec<GraphTableColumnDef>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphTableColumnDef {
    Expr(Box<Expr>, Option<SymbolPrimitive>),
    AllProperties(SymbolPrimitive),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphTableExport {
    AllSingletons {
        except: Option<Vec<SymbolPrimitive>>,
    },
    Singletons {
        exports: Vec<SymbolPrimitive>,
    },
    NoSingletons,
}

/// The direction of an edge
/// | Orientation               | Edge pattern | Abbreviation |
/// |---------------------------+--------------+--------------|
/// | Pointing left             | <−[ spec ]−  | <−           |
/// | Undirected                | ~[ spec ]~   | ~            |
/// | Pointing right            | −[ spec ]−>  | −>           |
/// | Left or undirected        | <~[ spec ]~  | <~           |
/// | Undirected or right       | ~[ spec ]~>  | ~>           |
/// | Left or right             | <−[ spec ]−> | <−>          |
/// | Left, undirected or right | −[ spec ]−   | −            |
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchDirection {
    Left,
    Undirected,
    Right,
    LeftOrUndirected,
    UndirectedOrRight,
    LeftOrRight,
    LeftOrUndirectedOrRight,
}

/// A quantifier for graph edges or patterns. (e.g., the `{2,5}` in `MATCH (x)->{2,5}(y)`)
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchQuantifier {
    pub lower: u32,
    pub upper: Option<NonZeroU32>,
}

/// A path mode
/// | Keyword        | Description
/// |----------------+--------------
/// | WALK           |
/// | TRAIL          | No repeated edges.
/// | ACYCLIC        | No repeated nodes.
/// | SIMPLE         | No repeated nodes, except that the ﬁrst and last nodes may be the same.

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphPathMode {
    Walk,
    Trail,
    Acyclic,
    Simple,
}

/// A single node in a graph pattern.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchNode {
    /// the optional element variable of the node match, e.g.: `x` in `MATCH (x)`
    #[visit(skip)]
    pub variable: Option<SymbolPrimitive>,
    /// the optional label(s) to match for the node, e.g.: `Entity` in `MATCH (x:Entity)`
    #[visit(skip)]
    pub label: Option<AstNode<GraphMatchLabel>>,
    /// an optional node where clause, e.g.: `WHERE c.name='Alarm'` in `MATCH (c WHERE c.name='Alarm')`
    pub where_clause: Option<Box<Expr>>,
}

/// A single edge in a graph pattern.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchEdge {
    /// edge direction
    #[visit(skip)]
    pub direction: GraphMatchDirection,
    /// the optional element variable of the edge match, e.g.: `t` in `MATCH −[t]−>`
    #[visit(skip)]
    pub variable: Option<SymbolPrimitive>,
    /// the optional label(s) to match for the edge. e.g.: `Target` in `MATCH −[t:Target]−>`
    #[visit(skip)]
    pub label: Option<AstNode<GraphMatchLabel>>,
    /// an optional edge where clause, e.g.: `WHERE t.capacity>100` in `MATCH −[t:hasSupply WHERE t.capacity>100]−>`
    pub where_clause: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchLabel {
    Name(SymbolPrimitive),
    Wildcard,
    Negated(Box<AstNode<GraphMatchLabel>>),
    Conjunction(Vec<AstNode<GraphMatchLabel>>),
    Disjunction(Vec<AstNode<GraphMatchLabel>>),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphPathPattern {
    /// the optional element variable of the pattern, e.g.: `p` in `MATCH p = (a) −[t]−> (b)`
    #[visit(skip)]
    pub variable: Option<SymbolPrimitive>,
    #[visit(skip)]
    pub prefix: Option<GraphPathPrefix>,
    /// the ordered pattern parts
    pub path: AstNode<GraphMatchPathPattern>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphPathSubPattern {
    /// the optional element variable of the pattern, e.g.: `p` in `MATCH p = (a) −[t]−> (b)`
    #[visit(skip)]
    pub variable: Option<SymbolPrimitive>,
    #[visit(skip)]
    pub mode: Option<GraphPathMode>,
    /// the ordered pattern parts
    pub path: AstNode<GraphMatchPathPattern>,
    /// an optional pattern where e.g.: `WHERE a.name=b.name` in `MATCH [(a)->(b) WHERE a.name=b.name]`
    pub where_clause: Option<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchPathPattern {
    Path(Vec<AstNode<GraphMatchPathPattern>>),
    Union(Vec<AstNode<GraphMatchPathPattern>>),
    Multiset(Vec<AstNode<GraphMatchPathPattern>>),

    Questioned(Box<AstNode<GraphMatchPathPattern>>),
    Quantified(GraphMatchPathPatternQuantified),

    Sub(Box<AstNode<GraphPathSubPattern>>),

    /// A single node in a graph pattern.
    Node(AstNode<GraphMatchNode>),

    /// A single edge in a graph pattern.
    Edge(AstNode<GraphMatchEdge>),

    #[visit(skip)]
    Simplified(AstNode<GraphMatchSimplified>),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchPathPatternQuantified {
    pub path: Box<AstNode<GraphMatchPathPattern>>,
    #[visit(skip)]
    pub quant: AstNode<GraphMatchQuantifier>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchElement {
    pub variable: Option<SymbolPrimitive>,
    pub label: Option<AstNode<GraphMatchLabel>>,
    pub where_clause: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchSimplified {
    pub dir: GraphMatchDirection,
    pub pattern: AstNode<GraphMatchSimplifiedPattern>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchSimplifiedPattern {
    Union(Vec<AstNode<GraphMatchSimplifiedPattern>>),
    Multiset(Vec<AstNode<GraphMatchSimplifiedPattern>>),

    Path(Vec<AstNode<GraphMatchSimplifiedPattern>>),
    Sub(Box<AstNode<GraphMatchSimplifiedPattern>>),

    Conjunction(Vec<AstNode<GraphMatchSimplifiedPattern>>),

    Questioned(Box<AstNode<GraphMatchSimplifiedPattern>>),
    Quantified(GraphMatchSimplifiedPatternQuantified),

    /// Direction override
    Direction(GraphMatchSimplifiedPatternDirected),

    Negated(Box<AstNode<GraphMatchSimplifiedPattern>>),
    Label(SymbolPrimitive),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchSimplifiedPatternQuantified {
    pub path: Box<AstNode<GraphMatchSimplifiedPattern>>,
    pub quant: AstNode<GraphMatchQuantifier>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchSimplifiedPatternDirected {
    pub dir: GraphMatchDirection,
    pub path: Box<AstNode<GraphMatchSimplifiedPattern>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphPathPrefix {
    Mode(GraphPathMode),
    Search(GraphPathSearchPrefix, Option<GraphPathMode>),
}

/// | Keyword
/// |------------------
/// | ALL
/// | Any
/// | ANY k
/// | ALL SHORTEST
/// | ANY SHORTEST
/// | SHORTEST k
/// | SHORTEST k GROUP
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphPathSearchPrefix {
    All,
    Any,
    AnyK(NonZeroU32),
    AllShortest,
    AnyShortest,
    ShortestK(NonZeroU32),
    ShortestKGroup(Option<NonZeroU32>),
}
