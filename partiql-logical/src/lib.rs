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
use std::fmt::{Debug, Display, Formatter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a PartiQL logical plan.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[inline]
    pub fn add_flow_with_branch_num(&mut self, src: OpId, dst: OpId, branch_num: u8) {
        assert!(src.index() <= self.operator_count());
        assert!(dst.index() <= self.operator_count());

        self.edges.push((src, dst, branch_num));
    }

    /// Extends the logical plan with the given data flows.
    /// #Examples:
    /// ```
    /// use partiql_logical::{BindingsOp, GroupBy, GroupingStrategy, LimitOffset, LogicalPlan, OrderBy};
    /// let mut p: LogicalPlan<BindingsOp> = LogicalPlan::new();
    ///
    /// let a = p.add_operator(BindingsOp::OrderBy(OrderBy{specs: vec![]}));
    /// let b = p.add_operator(BindingsOp::Sink);
    /// let c = p.add_operator(BindingsOp::LimitOffset(LimitOffset{limit:None, offset:None}));
    /// let d = p.add_operator(BindingsOp::GroupBy(GroupBy {
    ///     strategy: GroupingStrategy::GroupFull,
    ///     exprs: Default::default(),
    ///     aggregate_exprs: vec![],
    ///     group_as_alias: None,
    /// }));
    ///
    /// p.add_flow(a, b);
    ///
    /// p.extend_with_flows(&[(a,c), (c, d)]);
    /// assert_eq!(3, p.flows().len());
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

    /// Returns the operators of the plan with their `OpId`.
    pub fn operators_by_id(&self) -> impl Iterator<Item = (OpId, &T)> {
        self.nodes.iter().enumerate().map(|(i, n)| (OpId(i + 1), n))
    }

    /// Returns the data flows of the plan.
    pub fn flows(&self) -> &Vec<(OpId, OpId, u8)> {
        &self.edges
    }

    pub fn operator(&self, id: OpId) -> Option<&T> {
        self.nodes.get(id.0 - 1)
    }

    pub fn operator_as_mut(&mut self, id: OpId) -> Option<&mut T> {
        self.nodes.get_mut(id.0 - 1)
    }

    // TODO add DAG validation method.
}

/// Represents an operator identifier in a [`LogicalPlan`]
#[derive(Debug, Clone, Eq, PartialEq, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpId(usize);

impl OpId {
    /// Returns operator's index
    pub fn index(&self) -> usize {
        self.0
    }
}

impl<T> Display for LogicalPlan<T>
where
    T: Default + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let flows = self.flows();
        writeln!(f, "LogicalPlan")?;
        writeln!(f, "---")?;
        for (s, d, _w) in flows {
            let src_node = self.operator(*s).expect("Unable to get the src operator");
            let dst_node = self.operator(*d).expect("Unable to get the dst operator");
            writeln!(f, ">>> [{src_node:?}] -> [{dst_node:?}]")?;
        }
        writeln!(f)
    }
}

/// Represents PartiQL binding operators; A `BindingOp` is an operator that operates on
/// binding tuples as specified by [PartiQL Specification 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BindingsOp {
    Scan(Scan),
    Pivot(Pivot),
    Unpivot(Unpivot),
    Filter(Filter),
    OrderBy(OrderBy),
    LimitOffset(LimitOffset),
    Join(Join),
    BagOp(BagOp),
    Project(Project),
    ProjectAll,
    ProjectValue(ProjectValue),
    ExprQuery(ExprQuery),
    Distinct,
    GroupBy(GroupBy),
    Having(Having),
    #[default]
    Sink,
}

/// [`Scan`] bridges from [`ValueExpr`]s to [`BindingsOp`]s.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scan {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
}

/// [`Pivot`] represents a PIVOT operator, e.g. `PIVOT sp.price AT sp."symbol` in
/// `PIVOT sp.price AT sp."symbol" FROM todaysStockPrices sp`. For `Pivot` operational semantics,
/// see section `6.2` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pivot {
    pub key: ValueExpr,
    pub value: ValueExpr,
}

/// [`Unpivot`] bridges from [`ValueExpr`]s to [`BindingsOp`]s.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Unpivot {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
}

/// [`Filter`] represents a filter operator, e.g. `WHERE a = 10` in `SELECT a FROM t WHERE a = 10`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Filter {
    pub expr: ValueExpr,
}

/// [`Having`] represents the having operator, e.g. `HAVING a = 10` in `SELECT b FROM t GROUP BY a, b HAVING a = 10`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Having {
    pub expr: ValueExpr,
}

/// [`OrderBy`] represents a sort operatyion, e.g. `ORDER BY a DESC NULLS LAST` in `SELECT a FROM t ORDER BY a DESC NULLS LAST`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrderBy {
    pub specs: Vec<SortSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SortSpecOrder {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SortSpecNullOrder {
    First,
    Last,
}

/// Represents a PartiQL sort specification.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortSpec {
    pub expr: ValueExpr,
    pub order: SortSpecOrder,
    pub null_order: SortSpecNullOrder,
}

/// [`LimitOffset`] represents a possible limit and/or offset operator, e.g. `LIMIT 10 OFFSET 5` in `SELECT a FROM t LIMIT 10 OFFSET 5`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LimitOffset {
    pub limit: Option<ValueExpr>,
    pub offset: Option<ValueExpr>,
}

/// [`BagOp`] represents a bag operator, e.g. `UNION ALL` in `SELECT a, b FROM foo UNION ALL SELECT c, d FROM bar`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BagOp {
    pub bag_op: BagOperator,
    pub setq: SetQuantifier,
}

/// Represents the supported bag operator types.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BagOperator {
    Union,
    Except,
    Intersect,
    OuterUnion,
    OuterExcept,
    OuterIntersect,
}

/// ['Join`] represents a join operator, e.g. implicit `CROSS JOIN` specified by comma in `FROM`
/// clause in `SELECT t1.a, t2.b FROM tbl1 AS t1, tbl2 AS t2`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Join {
    pub kind: JoinKind,
    pub left: Box<BindingsOp>,
    pub right: Box<BindingsOp>,
    pub on: Option<ValueExpr>,
}

/// Represents join types.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// An SQL aggregation function call with its arguments
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AggregateExpression {
    pub name: String,
    pub expr: ValueExpr,
    pub func: AggFunc,
    pub setq: SetQuantifier,
}

/// SQL aggregate function
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AggFunc {
    // TODO: modeling of COUNT(*)
    /// Represents SQL's `AVG` aggregation function
    AggAvg,
    /// Represents SQL's `COUNT` aggregation function
    AggCount,
    /// Represents SQL's `MAX` aggregation function
    AggMax,
    /// Represents SQL's `MIN` aggregation function
    AggMin,
    /// Represents SQL's `SUM` aggregation function
    AggSum,
}

/// Represents `GROUP BY` <strategy> <group_key>[, <group_key>] ... \[AS <as_alias>\]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupBy {
    pub strategy: GroupingStrategy,
    pub exprs: HashMap<String, ValueExpr>,
    pub aggregate_exprs: Vec<AggregateExpression>,
    pub group_as_alias: Option<String>,
}

/// Grouping qualifier: ALL or PARTIAL
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GroupingStrategy {
    GroupFull,
    GroupPartial,
}

/// Represents a projection, e.g. `SELECT a` in `SELECT a FROM t`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Project {
    pub exprs: Vec<(String, ValueExpr)>,
}

/// Represents a value projection (SELECT VALUE) e.g. `SELECT VALUE t.a * 2` in
///`SELECT VALUE t.a * 2 IN tbl AS t`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProjectValue {
    pub expr: ValueExpr,
}

/// Represents an expression query e.g. `a * 2` in `a * 2` or an expression like `2+2`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExprQuery {
    pub expr: ValueExpr,
}

/// Represents a PartiQL value expression. Evaluation of a [`ValueExpr`] leads to a PartiQL value as
/// specified by [PartiQL Specification 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    PatternMatchExpr(PatternMatchExpr),
    SubQueryExpr(SubQueryExpr),
    SimpleCase(SimpleCase),
    SearchedCase(SearchedCase),
    IsTypeExpr(IsTypeExpr),
    NullIfExpr(NullIfExpr),
    CoalesceExpr(CoalesceExpr),
    Call(CallExpr),
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry?
/// Represents logical plan's unary operators.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry?
/// Represents logical plan's binary operators.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BetweenExpr {
    pub value: Box<ValueExpr>,
    pub from: Box<ValueExpr>,
    pub to: Box<ValueExpr>,
}

/// Represents a PartiQL Pattern Match expression, e.g. `'foo' LIKE 'foo'`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PatternMatchExpr {
    pub value: Box<ValueExpr>,
    pub pattern: Pattern,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Pattern {
    Like(LikeMatch), // TODO other e.g., SIMILAR_TO, or regex match
    LikeNonStringNonLiteral(LikeNonStringNonLiteralMatch),
}

/// Represents a LIKE expression where both the `pattern` and `escape` are string literals,
/// e.g. `'foo%' ESCAPE '/'`
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LikeMatch {
    pub pattern: String,
    pub escape: String,
}

/// Represents a LIKE expression where one of `pattern` and `escape` is not a string literal,
/// e.g. `some_pattern ESCAPE '/'`
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LikeNonStringNonLiteralMatch {
    pub pattern: Box<ValueExpr>,
    pub escape: Box<ValueExpr>,
}

/// Represents a sub-query expression, e.g. `SELECT v.a*2 AS u FROM t AS v` in
/// `SELECT t.a, s FROM data AS t, (SELECT v.a*2 AS u FROM t AS v) AS s`
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubQueryExpr {
    pub plan: LogicalPlan<BindingsOp>,
}

/// Represents a PartiQL's simple case expressions,
/// e.g.`CASE <expr> [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SimpleCase {
    pub expr: Box<ValueExpr>,
    pub cases: Vec<(Box<ValueExpr>, Box<ValueExpr>)>,
    pub default: Option<Box<ValueExpr>>,
}

/// Represents a PartiQL's searched case expressions,
/// e.g.`CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchedCase {
    pub cases: Vec<(Box<ValueExpr>, Box<ValueExpr>)>,
    pub default: Option<Box<ValueExpr>>,
}

/// Represents an `IS` expression, e.g. `IS TRUE`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IsTypeExpr {
    pub not: bool,
    pub expr: Box<ValueExpr>,
    pub is_type: Type,
}

/// Represents a PartiQL Type.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    // TODO CustomType
}

/// Represents a `NULLIF` expression, e.g. `NULLIF(v1, v2)` in `SELECT NULLIF(v1, v2) FROM data`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NullIfExpr {
    pub lhs: Box<ValueExpr>,
    pub rhs: Box<ValueExpr>,
}

/// Represents a `COALESCE` expression, e.g.
/// `COALESCE(NULL, 10)` in `SELECT COALESCE(NULL, 10) FROM data`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CoalesceExpr {
    pub elements: Vec<ValueExpr>,
}

/// Represents a `CALL` expression (i.e., a function call), e.g. `LOWER("ALL CAPS")`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallExpr {
    pub name: CallName,
    pub arguments: Vec<ValueExpr>,
}

/// Represents a known function.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CallName {
    Lower,
    Upper,
    CharLength,
    OctetLength,
    BitLength,
    LTrim,
    BTrim,
    RTrim,
    Substring,
    Position,
    Overlay,
    Exists,
    Abs,
    Mod,
    Cardinality,
    ExtractYear,
    ExtractMonth,
    ExtractDay,
    ExtractHour,
    ExtractMinute,
    ExtractSecond,
    ExtractTimezoneHour,
    ExtractTimezoneMinute,
    CollAvg(SetQuantifier),
    CollCount(SetQuantifier),
    CollMax(SetQuantifier),
    CollMin(SetQuantifier),
    CollSum(SetQuantifier),
    ByName(String),
}

/// Indicates if a set should be reduced to its distinct elements or not.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SetQuantifier {
    All,
    Distinct,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan() {
        let mut p: LogicalPlan<BindingsOp> = LogicalPlan::new();
        let a = p.add_operator(BindingsOp::OrderBy(OrderBy { specs: vec![] }));
        let b = p.add_operator(BindingsOp::Sink);
        let c = p.add_operator(BindingsOp::LimitOffset(LimitOffset {
            limit: None,
            offset: None,
        }));
        let d = p.add_operator(BindingsOp::GroupBy(GroupBy {
            strategy: GroupingStrategy::GroupFull,
            exprs: Default::default(),
            aggregate_exprs: vec![],
            group_as_alias: None,
        }));
        p.add_flow(a, b);
        p.add_flow(a, c);
        p.extend_with_flows(&[(c, d), (b, c)]);
        assert_eq!(4, p.operators().len());
        assert_eq!(4, p.flows().len());
    }
}
