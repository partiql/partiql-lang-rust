//! A PartiQL abstract syntax tree (AST).
//!
//! This module contains the structures for the language AST.
//! Two main entities in the module are [`Item`] and [`ItemKind`]. `Item` represents an AST element
//! and `ItemKind` represents a concrete type with the data specific to the type of the item.

// As more changes to this AST are expected, unless explicitly advised, using the structures exposed
// in this crate directly is not recommended.

use rust_decimal::Decimal as RustDecimal;

use std::fmt;
use std::num::NonZeroU32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeId(pub u32);

/// Represents an AST node.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AstNode<T> {
    pub id: NodeId,
    pub node: T,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Item {
    // Data Definition Language statements
    Ddl(Ddl),
    // Data Modification Language statements
    Dml(Dml),
    // Data retrieval statements
    Query(Query),
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatting for now
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ddl {
    pub op: DdlOp,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DdlOp {
    /// `CREATE TABLE <symbol>`
    CreateTable(CreateTable),
    /// `DROP TABLE <Ident>`
    DropTable(DropTable),
    /// `CREATE INDEX ON <Ident> (<expr> [, <expr>]...)`
    CreateIndex(CreateIndex),
    /// DROP INDEX <Ident> ON <Ident>
    /// In Statement, first <Ident> represents keys, second represents table
    DropIndex(DropIndex),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateTable {
    pub table_name: SymbolPrimitive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DropTable {
    pub table_name: SymbolPrimitive,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateIndex {
    pub index_name: SymbolPrimitive,
    pub fields: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DropIndex {
    pub table: SymbolPrimitive,
    pub keys: SymbolPrimitive,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Dml {
    pub op: DmlOp,
    pub from_clause: Option<FromClause>,
    pub where_clause: Option<Box<Expr>>,
    pub returning: Option<ReturningExpr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DmlOp {
    /// `INSERT INTO <expr> <expr>`
    Insert(Insert),
    /// `INSERT INTO <expr> VALUE <expr> [AT <expr>]` [ON CONFLICT WHERE <expr> DO NOTHING]`
    InsertValue(InsertValue),
    /// `SET <assignment>...`
    Set(Set),
    /// `REMOVE <expr>`
    Remove(Remove),
    /// DELETE
    Delete,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Insert {
    pub target: Box<Expr>,
    pub values: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InsertValue {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub index: Option<Box<Expr>>,
    pub on_conflict: Option<OnConflict>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Set {
    pub assignment: Assignment,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Remove {
    pub target: Box<Expr>,
}

/// `ON CONFLICT <expr> <conflict_action>`
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OnConflict {
    pub expr: Box<Expr>,
    pub conflict_action: ConflictAction,
}

/// `CONFLICT_ACTION <action>`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ConflictAction {
    DoNothing,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Query {
    pub set: AstNode<QuerySet>,
    pub order_by: Option<Box<AstNode<OrderByExpr>>>,
    pub limit: Option<Box<Expr>>,
    pub offset: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QuerySet {
    SetOp(Box<AstNode<SetExpr>>),
    Select(Box<AstNode<Select>>),
    Expr(Box<Expr>),
    Values(Vec<Box<Expr>>),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SetExpr {
    pub setop: SetOperator,
    pub setq: SetQuantifier,
    pub lhs: Box<AstNode<QuerySet>>,
    pub rhs: Box<AstNode<QuerySet>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SetOperator {
    Union,
    Except,
    Intersect,
}

/// The expressions that can result in values.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Expr {
    Lit(AstNode<Lit>),
    /// Variable reference
    VarRef(AstNode<VarRef>),
    /// Binary operator
    BinOp(AstNode<BinOp>),
    /// Unary operators
    UniOp(AstNode<UniOp>),
    /// Comparison operators
    Like(AstNode<Like>),
    Between(AstNode<Between>),
    In(AstNode<In>),
    Case(AstNode<Case>),
    /// Constructors
    Struct(AstNode<Struct>),
    Bag(AstNode<Bag>),
    List(AstNode<List>),
    Sexp(AstNode<Sexp>),
    /// Other expression types
    Path(AstNode<Path>),
    Call(AstNode<Call>),
    CallAgg(AstNode<CallAgg>),

    /// Query, e.g. `UNION` | `EXCEPT` | `INTERSECT` | `SELECT` and their parts.
    Query(AstNode<Query>),

    /// Indicates an error occurred during query processing; The exact error details are out of band of the AST
    Error,
}

/// `Lit` is mostly inspired by SQL-92 Literals standard and PartiQL specification.
/// See section 5.3 in the following:
/// <https://www.contrib.andrew.cmu.edu/~shadow/sql/sql1992.txt>
/// and Section 2 of the following (Figure 1: BNF Grammar for PartiQL Values):
/// <https://partiql.org/assets/PartiQL-Specification.pdf>
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Lit {
    Null,
    Missing,
    Int8Lit(i8),
    Int16Lit(i16),
    Int32Lit(i32),
    Int64Lit(i64),
    DecimalLit(RustDecimal),
    NumericLit(RustDecimal),
    RealLit(f32),
    FloatLit(f32),
    DoubleLit(f64),
    BoolLit(bool),
    IonStringLit(String),
    CharStringLit(String),
    NationalCharStringLit(String),
    BitStringLit(String),
    HexStringLit(String),
    CollectionLit(CollectionLit),
    /// E.g. `TIME WITH TIME ZONE` in `SELECT TIME WITH TIME ZONE '12:00' FROM ...`
    TypedLit(String, Type),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CollectionLit {
    ArrayLit(String),
    BagLit(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VarRef {
    pub name: SymbolPrimitive,
    pub qualifier: ScopeQualifier,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BinOp {
    pub kind: BinOpKind,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BinOpKind {
    // Arithmetic
    Add,
    Div,
    Exp,
    Mod,
    Mul,
    Neg,
    // Logical
    And,
    Or,
    // String
    Concat,
    // Comparison
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Ne,
    Is,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UniOp {
    pub kind: UniOpKind,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UniOpKind {
    Pos,
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Like {
    pub value: Box<Expr>,
    pub pattern: Box<Expr>,
    pub escape: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Between {
    pub value: Box<Expr>,
    pub from: Box<Expr>,
    pub to: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct In {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Case {
    /// CASE <expr> [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END
    SimpleCase(SimpleCase),
    /// CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END
    SearchedCase(SearchedCase),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SimpleCase {
    pub expr: Box<Expr>,
    pub cases: Vec<ExprPair>,
    pub default: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchedCase {
    pub cases: Vec<ExprPair>,
    pub default: Option<Box<Expr>>,
}

/// A generic pair of expressions. Used in the `pub struct`, `searched_case`
/// and `simple_case` expr variants above.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExprPair {
    pub first: Box<Expr>,
    pub second: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Struct {
    pub fields: Vec<ExprPair>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bag {
    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct List {
    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sexp {
    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Call {
    pub func_name: SymbolPrimitive,
    pub args: Vec<AstNode<CallArg>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CallArg {
    /// `*` used as an argument to a function call (e.g., in `count(*)`)
    Star(),
    /// positional argument to a function call (e.g., all arguments in `foo(1, 'a', 3)`)
    Positional(Box<Expr>),

    /// E.g. `INT` in `foo(INT)`
    PositionalType(Type),
    /// named argument to a function call (e.g., the `"from" : 2` in `substring(a, "from":2)`
    Named {
        name: SymbolPrimitive,
        value: Box<Expr>,
    },

    /// E.g. `AS: VARCHAR` in `CAST('abc' AS VARCHAR`
    NamedType { name: SymbolPrimitive, ty: Type },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallAgg {
    pub func_name: SymbolPrimitive,
    pub setq: Option<SetQuantifier>,
    pub args: Vec<AstNode<CallArg>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Select {
    pub project: AstNode<Projection>,
    pub from: Option<AstNode<FromClause>>,
    pub from_let: Option<AstNode<Let>>,
    pub where_clause: Option<Box<Expr>>,
    pub group_by: Option<Box<AstNode<GroupByExpr>>>,
    pub having: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Path {
    pub root: Box<Expr>,
    pub steps: Vec<PathStep>,
}

/// A "step" within a path expression; that is the components of the expression following the root.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathStep {
    PathExpr(PathExpr),
    PathWildCard,
    PathUnpivot,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PathExpr {
    pub index: Box<Expr>,
}

/// Is used to determine if variable lookup should be case-sensitive or not.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CaseSensitivity {
    CaseSensitive,
    CaseInsensitive,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Projection {
    pub kind: ProjectionKind,
    pub setq: Option<SetQuantifier>,
}

/// Indicates the type of projection in a SFW query.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProjectionKind {
    ProjectStar,
    ProjectList(Vec<AstNode<ProjectItem>>),
    ProjectPivot { key: Box<Expr>, value: Box<Expr> },
    ProjectValue(Box<Expr>),
}

/// An item to be projected in a `SELECT`-list.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProjectItem {
    /// For `.*` in SELECT list
    ProjectAll(ProjectAll), // TODO remove this?
    /// For `<expr> [AS <id>]`
    ProjectExpr(ProjectExpr),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProjectAll {
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProjectExpr {
    pub expr: Box<Expr>,
    pub as_alias: Option<SymbolPrimitive>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Let {
    /// A list of LET bindings
    pub let_bindings: Vec<LetBinding>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LetBinding {
    pub expr: Box<Expr>,
    pub as_alias: SymbolPrimitive,
}

/// FROM clause of an SFW query
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FromClause {
    FromLet(AstNode<FromLet>),
    /// <from_source> JOIN \[INNER | LEFT | RIGHT | FULL\] <from_source> ON <expr>
    Join(AstNode<Join>),
    /// <expr> MATCH <graph_pattern>
    GraphMatch(AstNode<GraphMatch>),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FromLet {
    pub expr: Box<Expr>,
    pub kind: FromLetKind,
    pub as_alias: Option<SymbolPrimitive>,
    pub at_alias: Option<SymbolPrimitive>,
    pub by_alias: Option<SymbolPrimitive>,
}

/// Indicates the type of FromLet, see the following for more details:
/// https:///github.com/partiql/partiql-lang-kotlin/issues/242
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FromLetKind {
    Scan,
    Unpivot,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Join {
    pub kind: JoinKind,
    pub left: Box<AstNode<FromClause>>,
    pub right: Box<AstNode<FromClause>>,
    pub predicate: Option<AstNode<JoinSpec>>,
}

/// Indicates the logical type of join.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JoinSpec {
    On(Box<Expr>),
    Using(Vec<Path>),
    Natural,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatch {
    pub expr: Box<Expr>,
    pub graph_expr: Box<AstNode<GraphMatchExpr>>,
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
///
/// Fig. 5. Table of edge patterns:
/// https://arxiv.org/abs/2112.06217
#[derive(Clone, Debug, PartialEq)]
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

/// A part of a graph pattern
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchPatternPart {
    /// A single node in a graph pattern.
    Node(AstNode<GraphMatchNode>),

    /// A single edge in a graph pattern.
    Edge(AstNode<GraphMatchEdge>),

    /// A sub-pattern.
    Pattern(AstNode<GraphMatchPattern>),
}

/// A quantifier for graph edges or patterns. (e.g., the `{2,5}` in `MATCH (x)->{2,5}(y)`)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchQuantifier {
    pub lower: u32,
    pub upper: Option<NonZeroU32>,
}

/// A path restrictor
/// | Keyword        | Description
/// |----------------+--------------
/// | TRAIL          | No repeated edges.
/// | ACYCLIC        | No repeated nodes.
/// | SIMPLE         | No repeated nodes, except that the ﬁrst and last nodes may be the same.
///
/// Fig. 7. Table of restrictors:
/// https://arxiv.org/abs/2112.06217
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchRestrictor {
    Trail,
    Acyclic,
    Simple,
}

/// A single node in a graph pattern.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchNode {
    /// an optional node pre-filter, e.g.: `WHERE c.name='Alarm'` in `MATCH (c WHERE c.name='Alarm')`
    pub prefilter: Option<Box<Expr>>,
    /// the optional element variable of the node match, e.g.: `x` in `MATCH (x)`
    pub variable: Option<SymbolPrimitive>,
    /// the optional label(s) to match for the node, e.g.: `Entity` in `MATCH (x:Entity)`
    pub label: Option<Vec<SymbolPrimitive>>,
}

/// A single edge in a graph pattern.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchEdge {
    /// edge direction
    pub direction: GraphMatchDirection,
    /// an optional quantifier for the edge match
    pub quantifier: Option<AstNode<GraphMatchQuantifier>>,
    /// an optional edge pre-filter, e.g.: `WHERE t.capacity>100` in `MATCH −[t:hasSupply WHERE t.capacity>100]−>`
    pub prefilter: Option<Box<Expr>>,
    /// the optional element variable of the edge match, e.g.: `t` in `MATCH −[t]−>`
    pub variable: Option<SymbolPrimitive>,
    /// the optional label(s) to match for the edge. e.g.: `Target` in `MATCH −[t:Target]−>`
    pub label: Option<Vec<SymbolPrimitive>>,
}

/// A single graph match pattern.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchPattern {
    pub restrictor: Option<GraphMatchRestrictor>,
    /// an optional quantifier for the entire pattern match
    pub quantifier: Option<AstNode<GraphMatchQuantifier>>,
    /// an optional pattern pre-filter, e.g.: `WHERE a.name=b.name` in `MATCH [(a)->(b) WHERE a.name=b.name]`
    pub prefilter: Option<Box<Expr>>,
    /// the optional element variable of the pattern, e.g.: `p` in `MATCH p = (a) −[t]−> (b)`
    pub variable: Option<SymbolPrimitive>,
    /// the ordered pattern parts
    pub parts: Vec<GraphMatchPatternPart>,
}

/// A path selector
/// | Keyword
/// |------------------
/// | ANY SHORTEST
/// | ALL SHORTEST
/// | ANY
/// | ANY k
/// | SHORTEST k
/// | SHORTEST k GROUP
///
/// Fig. 8. Table of restrictors:
/// https://arxiv.org/abs/2112.06217
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GraphMatchSelector {
    AnyShortest,
    AllShortest,
    Any,
    AnyK(NonZeroU32),
    ShortestK(NonZeroU32),
    ShortestKGroup(NonZeroU32),
}

/// A graph match clause as defined in GPML
/// See https://arxiv.org/abs/2112.06217
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphMatchExpr {
    pub selector: Option<GraphMatchSelector>,
    pub patterns: Vec<AstNode<GraphMatchPattern>>,
}

/// GROUP BY <grouping_strategy> <group_key_list>... \[AS <symbol>\]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupByExpr {
    pub strategy: GroupingStrategy,
    pub keys: Vec<AstNode<GroupKey>>,
    pub group_as_alias: Option<SymbolPrimitive>,
}

/// Desired grouping qualifier:  ALL or PARTIAL.  Note: the `group_` prefix is
/// needed to avoid naming clashes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GroupingStrategy {
    GroupFull,
    GroupPartial,
}

/// <expr> [AS <symbol>]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupKey {
    pub expr: Box<Expr>,
    pub as_alias: Option<SymbolPrimitive>,
}

/// ORDER BY <sort_spec>...
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrderByExpr {
    pub sort_specs: Vec<AstNode<SortSpec>>,
}

/// <expr> [ASC | DESC] ?
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortSpec {
    pub expr: Box<Expr>,
    pub ordering_spec: Option<OrderingSpec>,
    pub null_ordering_spec: Option<NullOrderingSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrderingSpec {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NullOrderingSpec {
    First,
    Last,
}

/// Indicates scope search order when resolving variables.
/// Has no effect except within `FROM` sources.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ScopeQualifier {
    /// Use the default search order.
    Unqualified,
    /// Skip the globals, first check within FROM sources and resolve starting with the local scope.
    Qualified,
}

/// Indicates if a set should be reduced to its distinct elements or not.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SetQuantifier {
    All,
    Distinct,
}

/// `RETURNING (<returning_elem> [, <returning_elem>]...)`
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReturningExpr {
    pub elems: Vec<ReturningElem>,
}

/// `<returning mapping> (<expr> [, <expr>]...)`
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReturningElem {
    pub mapping: ReturningMapping,
    pub column: ColumnComponent,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ColumnComponent {
    ReturningWildcard,
    ReturningColumn(ReturningColumn),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReturningColumn {
    pub expr: Box<Expr>,
}

/// ( MODIFIED | ALL ) ( NEW | OLD )
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReturningMapping {
    ModifiedNew,
    ModifiedOld,
    AllNew,
    AllOld,
}

/// Represents `<expr> = <expr>` in a DML SET operation.  Note that in this case, `=` is representing
/// an assignment operation and *not* the equality operator.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Assignment {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
}

/// Represents all possible PartiQL data types.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Type {
    NullType,
    BooleanType,
    Integer2Type,
    Integer4Type,
    Integer8Type,
    DecimalType,
    NumericType,
    RealType,
    DoublePrecisionType,
    TimestampType,
    CharacterType,
    CharacterVaryingType,
    MissingType,
    StringType,
    SymbolType,
    BlobType,
    ClobType,
    DateType,
    TimeType,
    ZonedTimestampType,
    StructType,
    TupleType,
    ListType,
    SexpType,
    BagType,
    AnyType,

    CustomType(CustomType),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CustomTypeParam {
    /// E.g. `2` in `VARCHAR(2)`
    Lit(Lit),
    /// E.g. `INT` in `FooType(INT)`
    Type(Type),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CustomTypePart {
    /// E.g. any of `WITH`, `TIME`, and`ZONE` in `TIME(20) WITH TIME ZONE`
    Name(SymbolPrimitive),
    /// E.g. `TIME(20) in `TIME(20) WITH TIME ZONE`
    Parameterized(SymbolPrimitive, Vec<CustomTypeParam>),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CustomType {
    pub parts: Vec<CustomTypePart>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SymbolPrimitive {
    pub value: String,
    pub case: CaseSensitivity,
}
