//! A PartiQL logical plan.
//!
//! This module contains the structures for a PartiQL logical plan. Three main entities in the
//! module are [`LogicalPlan`], [`BindingsOp`], and [`ValueExpr`].
//! `LogicalPlan` represents a graph based logical plan. `BindingsOp` represent operations that
//! operate on binding tuples and `ValueExpr` represents PartiQL expressions that produce PartiQL
//! values; all as specified in [PartiQL Specification 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
//!
//! Plan graph nodes are called _operators_ and edges are called _flows_ re-instating the fact that
//! the plan captures data flows for a given PartiQL statement.
//!
/// # Examples
/// ```
/// use partiql_logical::{BinaryOp, BindingsOp, LogicalPlan, PathComponent, ProjectValue, Scan, ValueExpr};
/// use partiql_value::{BindingsName, Value};
///
/// // Plan for `SELECT VALUE 2*v.a FROM [{'a': 1}, {'a': 2}, {'a': 3}] AS v`
///
/// let mut p: LogicalPlan<BindingsOp> = LogicalPlan::new();
///
/// let from = p.add_operator(BindingsOp::Scan(Scan {
///     expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
///     as_key: "v".to_string(),
///     at_key: None,
/// }));
///
/// let va = ValueExpr::Path(
///     Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
///         "v".into(),
///     ))),
///     vec![PathComponent::Key(BindingsName::CaseInsensitive("a".to_string()))],
/// );
///
/// let select_value = p.add_operator(BindingsOp::ProjectValue(ProjectValue {
///     expr: ValueExpr::BinaryExpr(
///         BinaryOp::Mul,
///         Box::new(va),
///         Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
///     ),
/// }));
///
/// let sink = p.add_operator(BindingsOp::Sink);
///
/// // Define the data flow as SCAN -> PROJECT_VALUE -> SINK
/// p.add_flow(from, select_value);
/// p.add_flow(select_value, sink);
///
/// assert_eq!(3, p.operators().len());
/// assert_eq!(2, p.flows().len());
/// ```
use partiql_value::{BindingsName, Value};
use std::collections::HashMap;

/// Represents a PartiQL logical plan.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct LogicalPlan<T>
where
    T: Default,
{
    nodes: Vec<T>,
    /// Third argument indicates the branch number into the outgoing node.
    edges: Vec<(OpId, OpId, u8)>,
}

impl<T> LogicalPlan<T>
where
    T: Default,
{
    /// Creates a new default logical plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new operator to the plan.
    pub fn add_operator(&mut self, op: T) -> OpId {
        self.nodes.push(op);
        OpId(self.operator_count())
    }

    /// Adds a data flow to the plan.
    #[inline]
    pub fn add_flow(&mut self, src: OpId, dst: OpId) {
        assert!(src.index() <= self.operator_count());
        assert!(dst.index() <= self.operator_count());

        self.edges.push((src, dst, 0));
    }

    /// Adds a data flow with a branch number.
    /// TODO: decide if `branch_num` is necessary within the current implementation. JOINs were
    ///  previously modeled as having separate data flows and the branch number was used to
    ///  distinguish between the LHS and RHS of a JOIN. JOINs have since been refactored to support
    ///  LATERAL JOINs which don't have separate data flows within the logical plan.
    ///  Tracking issue for possible removal: https://github.com/partiql/partiql-lang-rust/issues/237
    #[inline]
    pub fn add_flow_with_branch_num(&mut self, src: OpId, dst: OpId, branch_num: u8) {
        assert!(src.index() <= self.operator_count());
        assert!(dst.index() <= self.operator_count());

        self.edges.push((src, dst, branch_num));
    }

    /// Extends the logical plan with the given data flows.
    /// #Examples:
    /// ```
    /// use partiql_logical::{BindingsOp, LogicalPlan};
    /// let mut p: LogicalPlan<BindingsOp> = LogicalPlan::new();
    ///
    /// let a = p.add_operator(BindingsOp::OrderBy);
    /// let b = p.add_operator(BindingsOp::Sink);
    /// let c = p.add_operator(BindingsOp::Limit);
    /// let d = p.add_operator(BindingsOp::GroupBy);
    /// let e = p.add_operator(BindingsOp::Offset);
    ///
    /// p.add_flow(a, b);
    /// p.add_flow(a, c);
    ///
    /// p.extend_with_flows(&[(c, d), (d, e)]);
    /// assert_eq!(4, p.flows().len());
    /// ```
    #[inline]
    pub fn extend_with_flows(&mut self, flows: &[(OpId, OpId)]) {
        flows.iter().for_each(|&(s, d)| self.add_flow(s, d));
    }

    /// Returns the number of operators in the plan.
    #[inline]
    pub fn operator_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the operators of the plan.
    pub fn operators(&self) -> &Vec<T> {
        &self.nodes
    }

    /// Returns the data flows of the plan.
    pub fn flows(&self) -> &Vec<(OpId, OpId, u8)> {
        &self.edges
    }

    pub fn operator(&self, id: OpId) -> Option<&T> {
        self.nodes.get(id.0 - 1)
    }

    // TODO add DAG validation method.
}

/// Represents an operator identifier in a [`LogicalPlan`]
#[derive(Debug, Clone, Eq, PartialEq, Copy, Hash)]
pub struct OpId(usize);

impl OpId {
    /// Returns operator's index
    pub fn index(&self) -> usize {
        self.0
    }
}

/// Represents PartiQL binding operators; A `BindingOp` is an operator that operates on
/// binding tuples as specified by [PartiQL Specification 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum BindingsOp {
    Scan(Scan),
    Unpivot(Unpivot),
    Filter(Filter),
    OrderBy,
    Offset,
    Limit,
    Join(Join),
    SetOp,
    Project(Project),
    ProjectAll,
    ProjectValue(ProjectValue),
    ExprQuery(ExprQuery),
    Distinct,
    GroupBy,
    #[default]
    Sink,
}

/// [`Scan`] bridges from [`ValueExpr`]s to [`BindingsOp`]s.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scan {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
}

/// [`Unpivot`] bridges from [`ValueExpr`]s to [`BindingsOp`]s.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Unpivot {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
}

/// [`Filter`] represents a filter operator, e.g. `WHERE a = 10` in `SELECT a FROM t WHERE a = 10`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Filter {
    pub expr: ValueExpr,
}

/// ['Join`] represents a join operator, e.g. implicit `CROSS JOIN` specified by comma in `FROM`
/// clause in `SELECT t1.a, t2.b FROM tbl1 AS t1, tbl2 AS t2`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Join {
    pub kind: JoinKind,
    pub left: Box<BindingsOp>,
    pub right: Box<BindingsOp>,
    pub on: Option<ValueExpr>,
}

/// Represents join types.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// Represents a projection, e.g. `SELECT a` in `SELECT a FROM t`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Project {
    pub exprs: HashMap<String, ValueExpr>,
}

/// Represents a value projection (SELECT VALUE) e.g. `SELECT VALUE t.a * 2` in
///`SELECT VALUE t.a * 2 IN tbl AS t`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectValue {
    pub expr: ValueExpr,
}

/// Represents an expression query e.g. `a * 2` in `a * 2` or an expression like `2+2`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExprQuery {
    pub expr: ValueExpr,
}

/// Represents a PartiQL value expression. Evaluation of a [`ValueExpr`] leads to a PartiQL value as
/// specified by [PartiQL Specification 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueExpr {
    UnExpr(UnaryOp, Box<ValueExpr>),
    BinaryExpr(BinaryOp, Box<ValueExpr>, Box<ValueExpr>),
    Lit(Box<Value>),
    DynamicLookup(Box<Vec<ValueExpr>>),
    Path(Box<ValueExpr>, Vec<PathComponent>),
    VarRef(BindingsName),
    TupleExpr(TupleExpr),
    ListExpr(ListExpr),
    BagExpr(BagExpr),
    BetweenExpr(BetweenExpr),
    SubQueryExpr(SubQueryExpr),
    SimpleCase(SimpleCase),
    SearchedCase(SearchedCase),
    IsTypeExpr(IsTypeExpr),
    NullIfExpr(NullIfExpr),
    CoalesceExpr(CoalesceExpr),
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry?
/// Represents logical plan's unary operators.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry?
/// Represents logical plan's binary operators.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BinaryOp {
    And,
    Or,
    Concat,
    Eq,
    Neq,
    Gt,
    Gteq,
    Lt,
    Lteq,

    // Arithmetic ops
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,

    In,
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// Represents a path component in a plan.
pub enum PathComponent {
    /// E.g. `b` in `a.b`
    Key(BindingsName),
    /// E.g. 4 in `a[4]`
    Index(i64),
    KeyExpr(Box<ValueExpr>),
    IndexExpr(Box<ValueExpr>),
}

/// Represents a PartiQL tuple expression, e.g: `{ a.b: a.c * 2, 'count': a.c + 10}`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TupleExpr {
    pub attrs: Vec<ValueExpr>,
    pub values: Vec<ValueExpr>,
}

impl TupleExpr {
    /// Creates a new default [`TupleExpr`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Represents a PartiQL list expression, e.g. `[a.c * 2, 5]`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ListExpr {
    pub elements: Vec<ValueExpr>,
}

impl ListExpr {
    /// Creates a new default [`ListExpr`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Represents a PartiQL bag expression, e.g. `<<a.c * 2, 5>>`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BagExpr {
    pub elements: Vec<ValueExpr>,
}

impl BagExpr {
    /// Creates a new default [`BagExpr`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Represents a PartiQL `BETWEEN` expression, e.g. `BETWEEN 500 AND 600`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BetweenExpr {
    pub value: Box<ValueExpr>,
    pub from: Box<ValueExpr>,
    pub to: Box<ValueExpr>,
}

/// Represents a sub-query expression, e.g. `SELECT v.a*2 AS u FROM t AS v` in
/// `SELECT t.a, s FROM data AS t, (SELECT v.a*2 AS u FROM t AS v) AS s`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SubQueryExpr {
    pub plan: LogicalPlan<BindingsOp>,
}

/// Represents a PartiQL's simple case expressions,
/// e.g.`CASE <expr> [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SimpleCase {
    pub expr: Box<ValueExpr>,
    pub cases: Vec<(Box<ValueExpr>, Box<ValueExpr>)>,
    pub default: Option<Box<ValueExpr>>,
}

/// Represents a PartiQL's searched case expressions,
/// e.g.`CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SearchedCase {
    pub cases: Vec<(Box<ValueExpr>, Box<ValueExpr>)>,
    pub default: Option<Box<ValueExpr>>,
}

/// Represents an `IS` expression, e.g. `IS TRUE`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IsTypeExpr {
    pub not: bool,
    pub expr: Box<ValueExpr>,
    pub is_type: Type,
}

/// Represents a PartiQL Type.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    // TODO CustomType
}

/// Represents a `NULLIF` expression, e.g. `NULLIF(v1, v2)` in `SELECT NULLIF(v1, v2) FROM data`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NullIfExpr {
    pub lhs: Box<ValueExpr>,
    pub rhs: Box<ValueExpr>,
}

/// Represents a `COALESCE` expression, e.g.
/// `COALESCE(NULL, 10)` in `SELECT COALESCE(NULL, 10) FROM data`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CoalesceExpr {
    pub elements: Vec<ValueExpr>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan() {
        let mut p: LogicalPlan<BindingsOp> = LogicalPlan::new();
        let a = p.add_operator(BindingsOp::OrderBy);
        let b = p.add_operator(BindingsOp::Sink);
        let c = p.add_operator(BindingsOp::Limit);
        let d = p.add_operator(BindingsOp::GroupBy);
        let e = p.add_operator(BindingsOp::Offset);
        p.add_flow(a, b);
        p.add_flow(a, c);
        p.add_flow(b, c);
        p.extend_with_flows(&[(c, d), (d, e)]);
        assert_eq!(5, p.operators().len());
        assert_eq!(5, p.flows().len());
    }
}
