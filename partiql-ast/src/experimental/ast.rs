//! An experimental PartiQL abstract syntax tree (AST) module, see the following for motivation:
//! <https://github.com/partiql/partiql-lang-rust/issues/52>
//!
//! This module contains the structures for the language AST.
//! Two main entities in the module are [`Item`] and [`ItemKind`]. `Item` represents an AST element
//! and `ItemKind` represents a concrete type with the data specific to the type of the item.

// Structures in this module are mostly from the current `partiql-ir-generator` spec. and comments
// are excerpts from the spec definition.
// https://github.com/partiql/partiql-lang-kotlin/blob/4bcfc7f73d3e6e54286bcc03a54d5f6425eec4cc/lang/resources/org/partiql/type-domains/partiql.ion

// TODO Add documentation.

use partiql_source_map::location::BytePosition;
use rust_decimal::Decimal as RustDecimal;
use std::fmt;

/// Provides the required methods for AstNode conversations.
pub trait ToAstNode {
    /// Wraps the `self` to an [AstNode] and returns an `AstNodeBuilder` for
    /// further [AstNode] construction.
    /// ## Example:
    /// ```
    /// use partiql_ast::experimental::ast::{Span, SymbolPrimitive, ToAstNode};
    /// use partiql_source_map::location::BytePosition;
    ///
    /// let p = SymbolPrimitive {
    ///     value: "symbol2".to_string()
    ///  };
    ///
    /// let node = p
    ///     .to_node()
    ///     .span(Span {
    ///         begin: BytePosition::from(12),
    ///             end: BytePosition::from(1),
    ///     })
    ///     .build()
    ///     .expect("Could not retrieve ast node");
    /// ```
    fn to_node(self) -> AstNodeBuilder<Self>
    where
        Self: Clone,
    {
        AstNodeBuilder::default().node(self).clone()
    }
}

/// Implements [ToAstNode] for all types within this crate, read further [here][1].
///
/// [1]: https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods
impl<T> ToAstNode for T {}

/// Represents an AST node. [AstNode] uses [derive_builder][1] to expose a Builder
/// for creating the node. See [ToAstNode] for more details on the usage.
///
/// [1]: https://crates.io/crates/derive_builder
#[derive(Builder, Clone, Eq, PartialEq, Debug)]
pub struct AstNode<T> {
    pub node: T,
    #[builder(setter(strip_option), default)]
    pub span: Option<Span>,
}

/// Represents the beginning and the end of a `Span` in the source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub begin: BytePosition,
    pub end: BytePosition,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub kind: ItemKind,
    // We can/require to extend the fields as we get more clarity on the path forward.
    // Candidate additional fields are `name: Ident`, `span: Span`, `attr: Vec<Attribute>`.
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatting for now
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    // Data Definition Language statements
    Ddl(Ddl),
    // Data Modification Language statements
    Dml(Dml),
    // Date retrieval statements
    Query(Query),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ddl {
    pub op: DdlOp,
}

/// A data definition operation.
#[derive(Clone, Debug, PartialEq)]
pub struct DdlOp {
    pub kind: DdlOpKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DdlOpKind {
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

#[derive(Clone, Debug, PartialEq)]
pub struct CreateTable {
    pub table_name: SymbolPrimitive,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropTable {
    pub table_name: SymbolPrimitive,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateIndex {
    pub index_name: Ident,
    pub fields: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropIndex {
    pub table: Ident,
    pub keys: Ident,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dml {
    pub op: DmlOp,
}

/// A Data Manipulation Operation.
#[derive(Clone, Debug, PartialEq)]
pub struct DmlOp {
    pub kind: DmlOpKind,
    pub from_clause: Option<FromClause>,
    pub where_clause: Option<Box<Expr>>,
    pub returning: Option<ReturningExpr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DmlOpKind {
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
pub struct Insert {
    pub target: Box<Expr>,
    pub values: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InsertValue {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub index: Option<Box<Expr>>,
    pub on_conflict: Option<OnConflict>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Set {
    pub assignment: Assignment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Remove {
    pub target: Box<Expr>,
}

/// `ON CONFLICT <expr> <conflict_action>`
#[derive(Clone, Debug, PartialEq)]
pub struct OnConflict {
    pub expr: Box<Expr>,
    pub conflict_action: ConflictAction,
}

/// `CONFLICT_ACTION <action>`
#[derive(Clone, Debug, PartialEq)]
pub enum ConflictAction {
    DoNothing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
}

/// The expressions that can result in values.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Lit(Lit),
    /// Variable reference
    VarRef(VarRef),
    /// A parameter, i.e. `?`
    Param(Param),
    /// Binary operator
    BinOp(BinOp),
    /// Unary operators
    UniOp(UniOp),
    /// Comparison operators
    Like(Like),
    Between(Between),
    In(In),
    Is(Is),
    /// CASE <expr> [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END
    SimpleCase(SimpleCase),
    /// CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END
    SearchedCase(SearchCase),
    /// Constructors
    Struct(Struct),
    Bag(Bag),
    List(List),
    Sexp(Sexp),
    /// Constructors for DateTime types
    Date(Date),
    LitTime(LitTime),
    /// Set operators
    Union(Union),
    Except(Except),
    Intersect(Intersect),
    /// Other expression types
    Path(Path),
    Call(Call),
    CallAgg(CallAgg),

    /// `SELECT` and its parts.
    Select(Select),

    /// Indicates an error occurred during query processing; The exact error details are out of band of the AST
    Error,
}

/// `Lit` is mostly inspired by SQL-92 Literals standard and PartiQL specification.
/// See section 5.3 in the following:
/// <https://www.contrib.andrew.cmu.edu/~shadow/sql/sql1992.txt>
/// and Section 2 of the following (Figure 1: BNF Grammar for PartiQL Values):
/// <https://partiql.org/assets/PartiQL-Specification.pdf>
#[derive(Clone, Debug, PartialEq)]
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
    DateTimeLit(DateTimeLit),
    CollectionLit(CollectionLit),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CollectionLit {
    ArrayLit(String),
    BagLit(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum DateTimeLit {
    DateLit(String),
    TimeLit(String),
    TimestampLit(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarRef {
    pub name: SymbolPrimitive,
    pub case: CaseSensitivity,
    pub qualifier: ScopeQualifier,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct UniOp {
    pub kind: UniOpKind,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UniOpKind {
    Pos,
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Like {
    pub value: Box<Expr>,
    pub pattern: Box<Expr>,
    pub escape: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Between {
    pub value: Box<Expr>,
    pub from: Box<Expr>,
    pub to: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct In {
    pub operands: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Is {
    pub operands: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleCase {
    pub expr: Box<Expr>,
    pub cases: ExprPairList,
    pub default: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchCase {
    pub cases: ExprPairList,
    pub default: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Struct {
    pub fields: Vec<ExprPair>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bag {
    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct List {
    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sexp {
    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Date {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LitTime {
    pub value: TimeValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Union {
    pub setq: SetQuantifier,
    pub operands: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Except {
    pub setq: SetQuantifier,
    pub operands: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Intersect {
    pub setq: SetQuantifier,
    pub operands: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub root: Box<Expr>,
    pub steps: Vec<PathStep>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub func_name: SymbolPrimitive,
    pub args: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallAgg {
    pub func_name: SymbolPrimitive,
    pub args: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cast {
    pub value: Box<Expr>,
    pub as_type: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanCast {
    pub value: Box<Expr>,
    pub as_type: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanLossLessCast {
    pub value: Box<Expr>,
    pub as_type: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NullIf {
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Coalesce {
    pub args: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    pub setq: Option<SetQuantifier>,
    pub project: Projection,
    pub from: Option<FromClause>,
    pub from_let: Option<Let>,
    pub where_clause: Option<Box<Expr>>,
    pub group_by: Option<Box<GroupByExpr>>,
    pub having: Option<Box<Expr>>,
    pub order_by: Option<Box<OrderByExpr>>,
    pub limit: Option<Box<Expr>>,
    pub offset: Option<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimeValue {
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub nano: i32,
    pub precision: i32,
    pub with_time_zone: bool,
    pub tz_minutes: Option<i32>,
}

/// A "step" within a path expression; that is the components of the expression following the root.
#[derive(Clone, Debug, PartialEq)]
pub enum PathStep {
    PathExpr(PathExpr),
    PathWildCard,
    PathUnpivot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathExpr {
    pub index: Box<Expr>,
    pub case: CaseSensitivity,
}

/// Is used to determine if variable lookup should be case-sensitive or not.
#[derive(Clone, Debug, PartialEq)]
pub enum CaseSensitivity {
    CaseSensitive,
    CaseInsensitive,
}

/// Indicates the type of projection in a SFW query.
#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    ProjectStar,
    ProjectList(Vec<ProjectItem>),
    ProjectPivot { key: Box<Expr>, value: Box<Expr> },
    ProjectValue(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectValue {
    pub value: Box<Expr>,
}

/// An item to be projected in a `SELECT`-list.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectItem {
    /// For `.*` in SELECT list
    ProjectAll(ProjectAll),
    /// For `<expr> [AS <id>]`
    ProjectExpr(ProjectExpr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectAll {
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectExpr {
    pub expr: Box<Expr>,
    pub as_alias: Option<SymbolPrimitive>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    /// A list of LET bindings
    pub let_bindings: Vec<LetBinding>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetBinding {
    pub expr: Box<Expr>,
    pub name: SymbolPrimitive,
}

/// FROM clause of an SFW query
#[derive(Clone, Debug, PartialEq)]
pub enum FromClause {
    FromLet(FromLet),
    /// <from_source> JOIN \[INNER | LEFT | RIGHT | FULL\] <from_source> ON <expr>
    Join(Join),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FromLet {
    pub expr: Box<Expr>,
    pub kind: FromLetKind,
    pub as_alias: Option<SymbolPrimitive>,
    pub at_alias: Option<SymbolPrimitive>,
    pub by_alias: Option<SymbolPrimitive>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    pub kind: JoinKind,
    pub left: Box<FromClause>,
    pub right: Box<FromClause>,
    pub predicate: Option<JoinSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JoinSpec {
    On(Box<Expr>),
    Using(Vec<Path>),
    Natural,
}

/// Indicates the type of FromLet, see the following for more details:
/// https:///github.com/partiql/partiql-lang-kotlin/issues/242
#[derive(Clone, Debug, PartialEq)]
pub enum FromLetKind {
    Scan,
    Unpivot,
}

/// Indicates the logical type of join.
#[derive(Clone, Debug, PartialEq)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// A generic pair of expressions. Used in the `pub struct`, `searched_case`
/// and `simple_case` expr variants above.
#[derive(Clone, Debug, PartialEq)]
pub struct ExprPair {
    pub first: Box<Expr>,
    pub second: Box<Expr>,
}

/// A list of expr_pair. Used in the `pub struct`, `searched_case` and `simple_case`
/// expr variants above.
#[derive(Clone, Debug, PartialEq)]
pub struct ExprPairList {
    pub pairs: Vec<ExprPair>,
}

/// GROUP BY <grouping_strategy> <group_key_list>... \[AS <symbol>\]
#[derive(Clone, Debug, PartialEq)]
pub struct GroupByExpr {
    pub strategy: GroupingStrategy,
    pub key_list: GroupKeyList,
    pub group_as_alias: Option<SymbolPrimitive>,
}

/// Desired grouping qualifier:  ALL or PARTIAL.  Note: the `group_` prefix is
/// needed to avoid naming clashes.
#[derive(Clone, Debug, PartialEq)]
pub enum GroupingStrategy {
    GroupFull,
    GroupPartial,
}

/// <group_key>[, <group_key>]...
#[derive(Clone, Debug, PartialEq)]
pub struct GroupKeyList {
    pub keys: Vec<GroupKey>,
}

/// <expr> [AS <symbol>]
#[derive(Clone, Debug, PartialEq)]
pub struct GroupKey {
    pub expr: Box<Expr>,
    pub as_alias: Option<SymbolPrimitive>,
}

/// ORDER BY <sort_spec>...
#[derive(Clone, Debug, PartialEq)]
pub struct OrderByExpr {
    pub sort_specs: Vec<SortSpec>,
}

/// <expr> [ASC | DESC] ?
#[derive(Clone, Debug, PartialEq)]
pub struct SortSpec {
    pub expr: Box<Expr>,
    pub ordering_spec: Option<OrderingSpec>,
    pub null_ordering_spec: Option<NullOrderingSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrderingSpec {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NullOrderingSpec {
    First,
    Last,
}

/// Indicates scope search order when resolving variables.
/// Has no effect except within `FROM` sources.
#[derive(Clone, Debug, PartialEq)]
pub enum ScopeQualifier {
    /// Use the default search order.
    Unqualified,
    /// Skip the globals, first check within FROM sources and resolve starting with the local scope.
    Qualified,
}

/// Indicates if a set should be reduced to its distinct elements or not.
#[derive(Clone, Debug, PartialEq)]
pub enum SetQuantifier {
    All,
    Distinct,
}

/// `RETURNING (<returning_elem> [, <returning_elem>]...)`
#[derive(Clone, Debug, PartialEq)]
pub struct ReturningExpr {
    pub elems: Vec<ReturningElem>,
}

/// `<returning mapping> (<expr> [, <expr>]...)`
#[derive(Clone, Debug, PartialEq)]
pub struct ReturningElem {
    pub mapping: ReturningMapping,
    pub column: ColumnComponent,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnComponent {
    ReturningWildcard,
    ReturningColumn(ReturningColumn),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturningColumn {
    pub expr: Box<Expr>,
}

/// ( MODIFIED | ALL ) ( NEW | OLD )
#[derive(Clone, Debug, PartialEq)]
pub enum ReturningMapping {
    ModifiedNew,
    ModifiedOld,
    AllNew,
    AllOld,
}

/// `Ident` can be used for names that need to be looked up with a notion of case-sensitivity.

/// For both `create_index` and `create_table`, there is no notion of case-sensitivity
/// for table Idents since they are *defining* new Idents.  However, for `drop_index` and
/// `drop_table` *do* have the notion of case sensitivity since they are referring to existing names.
/// Idents with case-sensitivity is already modeled with the `id` variant of `expr`,
/// but there is no way to specify to PIG that we want to only allow a single variant of a sum as
/// an element of a type.  (Even though in the Kotlin code each varaint is its own type.)  Hence, we
/// define an `Ident` type above which can be used without opening up an element's domain to
/// all of `expr`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ident {
    pub name: SymbolPrimitive,
    pub case: CaseSensitivity,
}

/// Represents `<expr> = <expr>` in a DML SET operation.  Note that in this case, `=` is representing
/// an assignment operation and *not* the equality operator.
#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
}

/// Represents all possible PartiQL data types.
#[derive(Clone, Debug, PartialEq)]
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
pub struct CharacterType {
    pub length: Option<LongPrimitive>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterVaryingType {
    pub length: Option<LongPrimitive>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomType {
    pub name: SymbolPrimitive,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolPrimitive {
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LongPrimitive {
    pub value: i32,
}
