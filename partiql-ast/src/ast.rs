//! A `PartiQL` abstract syntax tree (AST).
//!
//! This module contains the structures for the language AST.
//! Two main entities in the module are [`Item`] and [`AstNode`]. `AstNode` represents an AST node
//! and `Item` represents a `PartiQL` statement type, e.g. query, data definition language (DDL)
//! data manipulation language (DML).

// As more changes to this AST are expected, unless explicitly advised, using the structures exposed
// in this crate directly is not recommended.

use indexmap::IndexMap;
use rust_decimal::Decimal as RustDecimal;

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use partiql_ast_macros::Visit;

pub type AstTypeMap<T> = IndexMap<NodeId, T>;

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

#[derive(Visit, Clone, Debug, PartialEq)]
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
        write!(f, "{self:?}")
    }
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ddl {
    pub op: DdlOp,
}

#[derive(Visit, Clone, Debug, PartialEq)]
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

#[derive(Visit, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateTable {
    #[visit(skip)]
    pub table_name: SymbolPrimitive,
}

#[derive(Visit, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DropTable {
    #[visit(skip)]
    pub table_name: SymbolPrimitive,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateIndex {
    #[visit(skip)]
    pub index_name: SymbolPrimitive,
    pub fields: Vec<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DropIndex {
    #[visit(skip)]
    pub table: SymbolPrimitive,
    #[visit(skip)]
    pub keys: SymbolPrimitive,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Dml {
    pub op: DmlOp,
    pub from_clause: Option<FromClause>,
    pub where_clause: Option<Box<Expr>>,
    pub returning: Option<ReturningExpr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
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
    Delete(Delete),
}

/// `RETURNING (<returning_elem> [, <returning_elem>]...)`
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReturningExpr {
    pub elems: Vec<ReturningElem>,
}

/// `<returning mapping> (<expr> [, <expr>]...)`
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReturningElem {
    #[visit(skip)]
    pub mapping: ReturningMapping,
    #[visit(skip)]
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

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Insert {
    pub target: Box<Expr>,
    pub values: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InsertValue {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub index: Option<Box<Expr>>,
    pub on_conflict: Option<OnConflict>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Set {
    pub assignment: Assignment,
}

/// Represents `<expr> = <expr>` in a DML SET operation.  Note that in this case, `=` is representing
/// an assignment operation and *not* the equality operator.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Assignment {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Remove {
    pub target: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Delete {}

/// `ON CONFLICT <expr> <conflict_action>`
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OnConflict {
    pub expr: Box<Expr>,
    #[visit(skip)]
    pub conflict_action: ConflictAction,
}

/// `CONFLICT_ACTION <action>`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ConflictAction {
    DoNothing,
}

// Evaluation order
// WITH,
// FROM,
// LET,
// WHERE,
// GROUP BY,
// HAVING,
// LETTING (which is special to PartiQL),
// ORDER BY,
// LIMIT / OFFSET
// SELECT (or SELECT VALUE or PIVOT, which are both special to ion PartiQL).

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TopLevelQuery {
    pub with: Option<AstNode<WithClause>>,
    pub query: AstNode<Query>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Query {
    pub set: AstNode<QuerySet>,
    pub order_by: Option<Box<AstNode<OrderByExpr>>>,
    pub limit_offset: Option<Box<AstNode<LimitOffsetClause>>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WithClause {
    #[visit(skip)]
    pub recursive: bool,
    pub withs: Vec<AstNode<WithElement>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WithElement {
    #[visit(skip)]
    pub query_name: SymbolPrimitive,
    #[visit(skip)]
    pub columns: Option<Vec<SymbolPrimitive>>,
    pub subquery: AstNode<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QuerySet {
    BagOp(Box<AstNode<BagOpExpr>>),
    Select(Box<AstNode<Select>>),
    Expr(Box<Expr>),
    Values(Vec<Box<Expr>>),
    Table(QueryTable),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BagOpExpr {
    #[visit(skip)]
    pub bag_op: BagOperator,
    #[visit(skip)]
    pub setq: Option<SetQuantifier>,
    pub lhs: Box<AstNode<Query>>,
    pub rhs: Box<AstNode<Query>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BagOperator {
    Union,
    Except,
    Intersect,
    OuterUnion,
    OuterExcept,
    OuterIntersect,
}

/// Indicates if a set should be reduced to its distinct elements or not.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SetQuantifier {
    All,
    Distinct,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Select {
    pub project: AstNode<Projection>,
    pub exclude: Option<AstNode<Exclusion>>,
    pub from: Option<AstNode<FromClause>>,
    pub from_let: Option<AstNode<Let>>,
    pub where_clause: Option<Box<AstNode<WhereClause>>>,
    pub group_by: Option<Box<AstNode<GroupByExpr>>>,
    pub having: Option<Box<AstNode<HavingClause>>>,
}

#[derive(Visit, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QueryTable {
    #[visit(skip)]
    pub table_name: SymbolPrimitive,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Projection {
    pub kind: ProjectionKind,
    #[visit(skip)]
    pub setq: Option<SetQuantifier>,
}

/// Indicates the type of projection in a SFW query.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProjectionKind {
    #[visit(skip)]
    ProjectStar,
    ProjectList(Vec<AstNode<ProjectItem>>),
    ProjectPivot(ProjectPivot),
    ProjectValue(Box<Expr>),
}

/// An item to be projected in a `SELECT`-list.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProjectItem {
    /// For `.*` in SELECT list
    ProjectAll(ProjectAll), // TODO remove this?
    /// For `<expr> [AS <id>]`
    ProjectExpr(ProjectExpr),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProjectPivot {
    pub key: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProjectAll {
    pub expr: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProjectExpr {
    pub expr: Box<Expr>,
    #[visit(skip)]
    pub as_alias: Option<SymbolPrimitive>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Exclusion {
    pub items: Vec<AstNode<ExcludePath>>,
}

/// The expressions that can result in values.
#[derive(Visit, Clone, Debug, PartialEq)]
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
    #[visit(skip)]
    Error,
}

/// `Lit` is mostly inspired by SQL-92 Literals standard and `PartiQL` specification.
/// See section 5.3 in the following:
/// <https://www.contrib.andrew.cmu.edu/~shadow/sql/sql1992.txt>
/// and Section 2 of the following (Figure 1: BNF Grammar for `PartiQL` Values):
/// <https://partiql.org/assets/PartiQL-Specification.pdf>
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[visit(skip_recurse)]
pub enum Lit {
    #[visit(skip)]
    Null,
    #[visit(skip)]
    Missing,
    #[visit(skip)]
    Int8Lit(i8),
    #[visit(skip)]
    Int16Lit(i16),
    #[visit(skip)]
    Int32Lit(i32),
    #[visit(skip)]
    Int64Lit(i64),
    #[visit(skip)]
    DecimalLit(RustDecimal),
    #[visit(skip)]
    NumericLit(RustDecimal),
    #[visit(skip)]
    RealLit(f32),
    #[visit(skip)]
    FloatLit(f32),
    #[visit(skip)]
    DoubleLit(f64),
    #[visit(skip)]
    BoolLit(bool),
    #[visit(skip)]
    IonStringLit(String),
    #[visit(skip)]
    CharStringLit(String),
    #[visit(skip)]
    NationalCharStringLit(String),
    #[visit(skip)]
    BitStringLit(String),
    #[visit(skip)]
    HexStringLit(String),
    #[visit(skip)]
    StructLit(AstNode<Struct>),
    #[visit(skip)]
    BagLit(AstNode<Bag>),
    #[visit(skip)]
    ListLit(AstNode<List>),
    /// E.g. `TIME WITH TIME ZONE` in `SELECT TIME WITH TIME ZONE '12:00' FROM ...`
    #[visit(skip)]
    TypedLit(String, Type),
}

#[derive(Visit, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VarRef {
    #[visit(skip)]
    pub name: SymbolPrimitive,
    #[visit(skip)]
    pub qualifier: ScopeQualifier,
}

/// Indicates scope search order when resolving variables.
/// Has no effect except within `FROM` sources.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ScopeQualifier {
    /// The variable was *NOT* prefixed with `@`.
    /// Resolve the variable by looking first in the database environment, then in the 'lexical' scope.
    Unqualified,
    /// The variable *WAS* prefixed with `@`.
    /// Resolve the variable by looking first in the 'lexical' scope, then in the database environment.
    Qualified,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BinOp {
    #[visit(skip)]
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
    Sub,
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

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UniOp {
    #[visit(skip)]
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

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Like {
    pub value: Box<Expr>,
    pub pattern: Box<Expr>,
    pub escape: Option<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Between {
    pub value: Box<Expr>,
    pub from: Box<Expr>,
    pub to: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct In {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Case {
    /// CASE <expr> [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END
    SimpleCase(SimpleCase),
    /// CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END
    SearchedCase(SearchedCase),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SimpleCase {
    pub expr: Box<Expr>,
    pub cases: Vec<ExprPair>,
    pub default: Option<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchedCase {
    pub cases: Vec<ExprPair>,
    pub default: Option<Box<Expr>>,
}

/// A generic pair of expressions. Used in the `pub struct`, `searched_case`
/// and `simple_case` expr variants above.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExprPair {
    pub first: Box<Expr>,
    pub second: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Struct {
    pub fields: Vec<ExprPair>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bag {
    pub values: Vec<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct List {
    pub values: Vec<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sexp {
    pub values: Vec<Box<Expr>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallAgg {
    #[visit(skip)]
    pub func_name: SymbolPrimitive,
    pub args: Vec<AstNode<CallArg>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Call {
    #[visit(skip)]
    pub func_name: SymbolPrimitive,
    pub args: Vec<AstNode<CallArg>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CallArg {
    /// `*` used as an argument to a function call (e.g., in `count(*)`)
    #[visit(skip)]
    Star(),
    /// positional argument to a function call (e.g., all arguments in `foo(1, 'a', 3)`)
    Positional(Box<Expr>),
    /// E.g. `INT` in `foo(INT)`
    #[visit(skip)]
    PositionalType(Type),
    /// named argument to a function call (e.g., the `"from" : 2` in `substring(a, "from":2)`
    Named(CallArgNamed),
    /// E.g. `AS: VARCHAR` in `CAST('abc' AS VARCHAR)`
    NamedType(CallArgNamedType),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallArgNamed {
    #[visit(skip)]
    pub name: SymbolPrimitive,
    pub value: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallArgNamedType {
    #[visit(skip)]
    pub name: SymbolPrimitive,
    #[visit(skip)]
    pub ty: Type,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Path {
    pub root: Box<Expr>,
    pub steps: Vec<PathStep>,
}

/// A "step" within a path expression; that is the components of the expression following the root.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathStep {
    PathProject(PathExpr),
    PathIndex(PathExpr),
    #[visit(skip)]
    PathForEach,
    #[visit(skip)]
    PathUnpivot,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PathExpr {
    pub index: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExcludePath {
    pub root: AstNode<VarRef>,
    pub steps: Vec<ExcludePathStep>,
}

/// A "step" within an exclude path; that is the components of the exclude path following the root.
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ExcludePathStep {
    #[visit(skip)]
    PathProject(AstNode<SymbolPrimitive>),
    #[visit(skip)]
    PathIndex(AstNode<Lit>),
    #[visit(skip)]
    PathForEach,
    #[visit(skip)]
    PathUnpivot,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Let {
    /// A list of LET bindings
    pub let_bindings: Vec<LetBinding>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LetBinding {
    pub expr: Box<Expr>,
    #[visit(skip)]
    pub as_alias: SymbolPrimitive,
}

/// FROM clause of an SFW query
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FromClause {
    pub source: FromSource,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FromSource {
    FromLet(AstNode<FromLet>),
    /// <from_source> JOIN \[INNER | LEFT | RIGHT | FULL\] <from_source> ON <expr>
    Join(AstNode<Join>),
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WhereClause {
    pub expr: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HavingClause {
    pub expr: Box<Expr>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FromLet {
    pub expr: Box<Expr>,
    #[visit(skip)]
    pub kind: FromLetKind,
    #[visit(skip)]
    pub as_alias: Option<SymbolPrimitive>,
    #[visit(skip)]
    pub at_alias: Option<SymbolPrimitive>,
    #[visit(skip)]
    pub by_alias: Option<SymbolPrimitive>,
}

/// Indicates the type of `FromLet`, see the following for more details:
/// https:///github.com/partiql/partiql-lang-kotlin/issues/242
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FromLetKind {
    Scan,
    Unpivot,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Join {
    #[visit(skip)]
    pub kind: JoinKind,
    pub left: Box<FromSource>,
    pub right: Box<FromSource>,
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

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JoinSpec {
    On(Box<Expr>),
    Using(Vec<Path>),
    #[visit(skip)]
    Natural,
}

/// GROUP BY <`grouping_strategy`> <`group_key`>[, <`group_key`>]... \[AS <symbol>\]
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupByExpr {
    #[visit(skip)]
    pub strategy: Option<GroupingStrategy>,
    pub keys: Vec<AstNode<GroupKey>>,
    #[visit(skip)]
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
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupKey {
    pub expr: Box<Expr>,
    #[visit(skip)]
    pub as_alias: Option<SymbolPrimitive>,
}

/// ORDER BY <`sort_spec`>...
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrderByExpr {
    pub sort_specs: Vec<AstNode<SortSpec>>,
}

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LimitOffsetClause {
    pub limit: Option<Box<Expr>>,
    pub offset: Option<Box<Expr>>,
}

/// <expr> [ASC | DESC] ?
#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortSpec {
    pub expr: Box<Expr>,
    #[visit(skip)]
    pub ordering_spec: Option<OrderingSpec>,
    #[visit(skip)]
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

/// Represents all possible `PartiQL` data types.
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

#[derive(Visit, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CustomType {
    #[visit(skip)]
    pub parts: Vec<CustomTypePart>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SymbolPrimitive {
    pub value: String,
    pub case: CaseSensitivity,
}

/// Is used to determine if variable lookup should be case-sensitive or not.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CaseSensitivity {
    CaseSensitive,
    CaseInsensitive,
}
