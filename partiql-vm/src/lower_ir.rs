//! Pass 1 of the compiler: lower a logical plan to a [`Query`], the IR that
//! [`crate::compile`] emits bytecode from.
//!
//! The IR is a physical plan: a tree of operators ([`Op`]) with every decision made. That
//! is the algorithm of each operator (a nested-loop join, a sort-based GROUP BY), which
//! side of a join is the outer loop, which cursor reads each scan and which of a scan's
//! fields it loads, every register the query uses (from one allocator), and every
//! expression, already lowered to register references. Emission only walks the tree.

use crate::arena::SlotId;
use crate::catalog::CompilationContext;
use crate::error::{count_u16, EngineError, Result};
use crate::expr::{
    lit_to_value, AggFunc, Expr, LogicalExprCompiler, LoweredSubqueries, SlotResolver,
};
use crate::field_resolver::{CompileContext, ExprFieldExtractor};
use crate::plan::{
    CursorInfo, EvaluationMode, ObjectId, ScanId, ScanMetadata, SortKeys, SortOrder,
    SorterMetadata, TableFnScanMetadata,
};
use crate::source::{
    DataSourceHandle, ScanLayout, ScanProjection, ScanSource, TableFunctionHandle,
};
use crate::value::{FieldName, FieldShape, PhysicalType, RowShape, Shape, ValueOwned};
use partiql_logical::{
    BindingsOp, CallName, DBRef, GroupBy, JoinKind, LimitOffset, LogicalPlan, OpId, OrderBy,
    PathComponent, Pattern, Project as LogicalProject, Scan, SortSpecNullOrder, SortSpecOrder,
    SubQueryExpr, ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Range;

// ---------------------------------------------------------------------------
// The IR
// ---------------------------------------------------------------------------

/// A query ready for emission, with the metadata its `CompiledPlan` carries.
pub(crate) struct Query {
    pub(crate) root: Root,
    /// Registers the IR uses. Expression temporaries go above them.
    pub(crate) reg_count: u16,
    pub(crate) cursors: Vec<CursorInfo>,
    pub(crate) shape: Shape,
    pub(crate) scan_metadata: HashMap<ScanId, ScanMetadata>,
    pub(crate) inline_scans: HashMap<ScanId, Vec<ValueOwned>>,
    pub(crate) expr_scan_ids: Vec<ScanId>,
    pub(crate) table_fn_scans: HashMap<ScanId, TableFnScanMetadata>,
    pub(crate) sorters: Vec<SorterMetadata>,
    /// Index = the `id` of `Expr::Subquery`.
    pub(crate) subqueries: Vec<Subquery>,
}

pub(crate) enum Root {
    /// A query without FROM: read the one row of each catalog object it names (`cursors`),
    /// then evaluate `expr` into `dst`.
    Expr {
        cursors: Vec<u16>,
        expr: Expr,
        dst: SlotId,
    },
    /// Every row of the operator tree is a result.
    Rows(Op),
}

/// A physical operator. Each produces rows into registers; the operator above it runs
/// once per row. Operators stream (run the one above for each row of their input as it
/// comes) unless noted.
pub(crate) enum Op {
    /// The rows of one cursor.
    Scan(ScanLoop),
    /// A nested-loop join.
    Join(Box<Join>),
    /// The rows of `input` that pass a condition (WHERE, HAVING).
    Filter(Box<Filter>),
    /// The rows of `input`, with the SELECT list evaluated into the output registers.
    Project(Box<Project>),
    /// GROUP BY: one row per group of `input`'s rows. Blocking: it reads all of `input`
    /// before its first row.
    Aggregate(Box<Aggregate>),
    /// ORDER BY: the rows of `input`, sorted. Blocking.
    Sort(Box<Sort>),
    /// The first rows of `input`.
    Limit(Box<Limit>),
}

pub(crate) struct Filter {
    pub(crate) input: Op,
    pub(crate) cond: Cond,
}

pub(crate) struct Project {
    pub(crate) input: Op,
    pub(crate) outputs: Vec<Output>,
}

/// One step of computing a projection's output registers.
pub(crate) enum Output {
    /// Evaluate `expr` into `dst`.
    Eval { dst: SlotId, expr: Expr },
    /// SELECT * of one binding (permissive): make the value in the register a tuple.
    CoerceToTuple(SlotId),
    /// SELECT * of one binding (strict): fail unless the value in the register is a tuple.
    AssertTuple(SlotId),
    /// SELECT * of a join: merge the bindings' tuples into `dst`.
    ConcatTuples { dst: SlotId, srcs: Vec<SlotId> },
}

/// LIMIT `count`: `counter` counts down the rows left.
pub(crate) struct Limit {
    pub(crate) input: Op,
    pub(crate) count: i64,
    pub(crate) counter: SlotId,
}

/// A cursor loop over one scan.
pub(crate) struct ScanLoop {
    pub(crate) cursor: u16,
    pub(crate) open: Open,
    /// The registers the cursor writes each row into.
    pub(crate) window: Range<SlotId>,
    /// The scan loads whole rows (rather than only the fields the query reads).
    pub(crate) whole_row: bool,
}

/// What a scan's cursor needs before `OpenCursor`.
pub(crate) enum Open {
    /// Nothing: a catalog table or literal, bound when the VM starts.
    Bound,
    /// A table function: evaluate each argument into its register, then create the cursor.
    TableFn {
        func_name: String,
        args: Vec<(SlotId, Expr)>,
    },
    /// A collection computed at runtime: evaluate `expr` into `src`, then read from it.
    Expr {
        src: SlotId,
        expr: Expr,
        /// Strict mode: fail unless `src` holds a collection.
        assert_collection: bool,
    },
}

/// A nested-loop join of `left` and `right` on `on` (none for CROSS JOIN).
pub(crate) struct Join {
    pub(crate) kind: JoinType,
    pub(crate) left: Op,
    pub(crate) right: Op,
    pub(crate) on: Option<Cond>,
    /// Outer joins: set when the current row of the preserved side found a match.
    pub(crate) matched: SlotId,
    /// Outer joins: what follows the join runs from more than one place, so it becomes a
    /// subroutine returning through this register.
    pub(crate) ret: SlotId,
}

/// CROSS JOIN is an INNER JOIN without ON.
#[derive(Clone, Copy)]
pub(crate) enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// A condition: `expr` evaluated into `pred`.
pub(crate) struct Cond {
    pub(crate) pred: SlotId,
    pub(crate) expr: Expr,
}

/// A sort-based GROUP BY. Each row of `input` is evaluated into a record (its group keys,
/// then its aggregate inputs) and inserted into `sorter`. The sorted records are then read
/// back, and each run of equal keys is folded into the aggregates: one row per group.
pub(crate) struct Aggregate {
    pub(crate) input: Op,
    pub(crate) sorter: u16,
    /// The record's expressions, evaluated into consecutive registers from `insert`.
    pub(crate) record_exprs: Vec<Expr>,
    pub(crate) insert: SlotId,
    /// Where each sorted record is read back into.
    pub(crate) record: SlotId,
    pub(crate) key_count: u16,
    /// The current group's keys.
    pub(crate) keys: SlotId,
    /// Registers comparing a record's keys with the current group's.
    pub(crate) same_group: SlotId,
    pub(crate) key_eq: SlotId,
    pub(crate) new_group: SlotId,
    pub(crate) aggs: Vec<GroupAgg>,
    /// Every aggregate's accumulator registers, reset for each group.
    pub(crate) accums: Range<SlotId>,
    /// Set once the current group has a record.
    pub(crate) has_data: SlotId,
    /// A group's row is produced from two places (a change of keys, and the last group),
    /// so what follows is a subroutine returning through this register.
    pub(crate) ret: SlotId,
}

/// ORDER BY. Each row of `input` is evaluated into a record (its sort keys, then its
/// payload: every register the operators above can read) and inserted into `sorter`. The
/// sorted records are then read back, each copying its payload back into those registers.
pub(crate) struct Sort {
    pub(crate) input: Op,
    pub(crate) sorter: u16,
    /// The sort keys, evaluated into consecutive registers from `insert`.
    pub(crate) keys: Vec<Expr>,
    /// The registers carried through the sort, copied into the record after the keys.
    pub(crate) payload: Vec<SlotId>,
    pub(crate) insert: SlotId,
    /// Where each sorted record is read back into.
    pub(crate) record: SlotId,
}

/// A subquery, evaluated to the collection of its rows: each row of `input` (its registers
/// `row`, copied into consecutive registers from `record`) is inserted into `sorter`, then the
/// records are collected into `dst`.
pub(crate) struct Subquery {
    pub(crate) input: Op,
    pub(crate) sorter: u16,
    pub(crate) row: Vec<SlotId>,
    pub(crate) record: SlotId,
    /// A struct row's attribute names, one per row register; none for a row of one value.
    pub(crate) fields: Option<Vec<String>>,
    /// The rows are in ORDER BY order, so the collection is a list.
    pub(crate) ordered: bool,
    pub(crate) dst: SlotId,
    /// An uncorrelated subquery (one that reads nothing of the queries around it) runs at most
    /// once: this register is set once it has.
    pub(crate) once: Option<SlotId>,
}

pub(crate) struct GroupAgg {
    /// The record register holding the aggregate's input.
    pub(crate) input: SlotId,
    /// The accumulators it steps for each record (AVG keeps a sum and a count).
    pub(crate) steps: Vec<(AggFunc, SlotId)>,
    /// Finalizes the accumulators in place, from the first.
    pub(crate) finalize: AggFunc,
    pub(crate) accum: SlotId,
}

/// Lower `plan`.
pub(crate) fn lower(
    compilation_context: &CompilationContext,
    mode: EvaluationMode,
    plan: &LogicalPlan<BindingsOp>,
) -> Result<Query> {
    let mut lowerer = Lowerer {
        compilation_context,
        mode,
        next_scan_id: 0,
        next_reg: 0,
        cursors: Vec::new(),
        scans: HashMap::new(),
        scan_metadata: HashMap::new(),
        inline_scans: HashMap::new(),
        sorters: Vec::new(),
        subqueries: Vec::new(),
        scopes: Vec::new(),
    };
    let graph = PlanGraph::new(plan);
    let top = graph.single_input(graph.find_sink()?)?;
    let (root, shape) = lowerer.lower_root(&graph, top)?;

    let mut expr_scan_ids = vec![];
    let table_fn_scans = lowerer
        .scans
        .into_iter()
        .filter_map(|(id, scan)| match scan {
            ScanKind::TableFn(tf) => Some((
                id,
                TableFnScanMetadata {
                    func_name: tf.func_name,
                    layout: tf.layout,
                    arg_slots: tf.arg_slots,
                },
            )),
            ScanKind::Expr { .. } => {
                expr_scan_ids.push(id);
                None
            }
            ScanKind::Bound => None,
        })
        .collect();
    Ok(Query {
        root,
        reg_count: lowerer.next_reg,
        cursors: lowerer.cursors,
        shape,
        scan_metadata: lowerer.scan_metadata,
        inline_scans: lowerer.inline_scans,
        expr_scan_ids,
        table_fn_scans,
        sorters: lowerer.sorters,
        subqueries: lowerer.subqueries,
    })
}

// ---------------------------------------------------------------------------
// Plan graph navigation helpers
// ---------------------------------------------------------------------------

/// Pre-built index for navigating the logical plan graph.
struct PlanGraph<'p> {
    plan: &'p LogicalPlan<BindingsOp>,
    /// For each node, its list of (source_node, branch_number) inputs, sorted by branch.
    inputs: FxHashMap<OpId, Vec<(OpId, u8)>>,
}

impl<'p> PlanGraph<'p> {
    fn new(plan: &'p LogicalPlan<BindingsOp>) -> Self {
        let mut inputs: FxHashMap<OpId, Vec<(OpId, u8)>> = FxHashMap::default();
        for &(src, dst, branch) in plan.flows() {
            inputs.entry(dst).or_default().push((src, branch));
        }
        // Sort inputs by branch number for deterministic ordering
        for v in inputs.values_mut() {
            v.sort_by_key(|(_, b)| *b);
        }
        PlanGraph { plan, inputs }
    }

    fn operator(&self, id: OpId) -> Result<&'p BindingsOp> {
        self.plan
            .operator(id)
            .ok_or_else(|| EngineError::InvalidPlan(format!("missing operator {:?}", id)))
    }

    /// Get the single input to a node (for linear operators like Filter, Project).
    fn single_input(&self, id: OpId) -> Result<OpId> {
        let ins = self.inputs.get(&id);
        match ins.map(|v| v.as_slice()) {
            Some(&[(src, _)]) => Ok(src),
            Some(slice) => Err(EngineError::InvalidPlan(format!(
                "expected 1 input for {:?}, got {}",
                id,
                slice.len()
            ))),
            None => Err(EngineError::InvalidPlan(format!("no inputs for {:?}", id))),
        }
    }

    /// Get the (left, right) inputs of a join.
    fn join_inputs(&self, id: OpId) -> Result<(OpId, OpId)> {
        match self.inputs.get(&id).map(|v| v.as_slice()) {
            Some(&[(left, 0), (right, 1)]) => Ok((left, right)),
            _ => Err(EngineError::InvalidPlan(format!(
                "expected left and right inputs for join {:?}",
                id
            ))),
        }
    }

    /// Find the sink node (the node with no outgoing edges that is a Sink).
    fn find_sink(&self) -> Result<OpId> {
        let mut sink = None;
        for (id, op) in self.plan.operators_by_id() {
            if matches!(op, BindingsOp::Sink) {
                if sink.is_some() {
                    return Err(EngineError::InvalidPlan("multiple sinks".to_string()));
                }
                sink = Some(id);
            }
        }
        sink.ok_or_else(|| EngineError::InvalidPlan("no sink node found".to_string()))
    }

    /// The operator whose rows are a subquery's: the one that flows into no other. A
    /// subquery's plan has no sink.
    fn subquery_top(&self) -> Result<OpId> {
        let mut tops = self
            .plan
            .operators_by_id()
            .map(|(id, _)| id)
            .filter(|&id| self.plan.flows().iter().all(|&(src, _, _)| src != id));
        match (tops.next(), tops.next()) {
            (Some(top), None) => Ok(top),
            _ => Err(EngineError::InvalidPlan(
                "subquery plan without a single top operator".to_string(),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Lowering
// ---------------------------------------------------------------------------

struct Lowerer<'a> {
    compilation_context: &'a CompilationContext,
    mode: EvaluationMode,
    next_scan_id: u64,
    /// The register allocator: the first register not yet assigned.
    next_reg: u16,
    /// Index = cursor_id.
    cursors: Vec<CursorInfo>,
    scans: HashMap<ScanId, ScanKind>,
    scan_metadata: HashMap<ScanId, ScanMetadata>,
    inline_scans: HashMap<ScanId, Vec<ValueOwned>>,
    /// Index = sorter_id.
    sorters: Vec<SorterMetadata>,
    subqueries: Vec<Subquery>,
    /// While lowering a subquery: the scopes of the queries around it, outermost first.
    scopes: Vec<OuterScope>,
}

/// The names of a query around the subquery being lowered.
struct OuterScope {
    resolver: ResolverKind,
    /// Set when the subquery lowered with this scope pushed (or one inside it) reads a name of
    /// this scope or one further out.
    correlated: Cell<bool>,
}

/// A lowered operator, with what the operators above it see of its rows.
struct Lowered {
    op: Op,
    /// The names of the row's bindings.
    resolver: ResolverKind,
    /// The shape of a result row, once a projection has set it.
    shape: Option<Shape>,
    /// The rows are in ORDER BY order, so the result is a list.
    ordered: bool,
}

impl Lowered {
    fn new(op: Op, resolver: ResolverKind, shape: Option<Shape>) -> Self {
        Lowered {
            op,
            resolver,
            shape,
            ordered: false,
        }
    }

    /// A projection of `input`. Its rows have no bindings: nothing above reads names.
    fn projected(input: Op, outputs: Vec<Output>, shape: Shape) -> Self {
        Lowered::new(
            Op::Project(Box::new(Project { input, outputs })),
            ResolverKind::Empty,
            Some(shape),
        )
    }
}

/// Where a scan's rows come from, as far as emission is concerned.
enum ScanKind {
    /// A catalog table or literal, bound when the VM starts.
    Bound,
    TableFn(TableFnScan),
    /// A collection computed at runtime by `expr` into `src`.
    Expr {
        expr: ValueExpr,
        src: SlotId,
    },
}

struct TableFnScan {
    func_name: String,
    layout: ScanLayout,
    arguments: Vec<ValueExpr>,
    arg_slots: Vec<SlotId>,
}

/// A FROM subtree with its cursors and registers assigned, before its expressions are
/// lowered (they may read any binding of the FROM clause).
enum Layout<'p> {
    Scan(ScanSlots),
    Join(Box<JoinLayout<'p>>),
}

struct ScanSlots {
    scan_id: ScanId,
    cursor: u16,
    window: Range<SlotId>,
    whole_row: bool,
}

struct JoinLayout<'p> {
    kind: JoinType,
    on: Option<&'p ValueExpr>,
    left: Layout<'p>,
    right: Layout<'p>,
}

impl<'a> Lowerer<'a> {
    fn alloc(&mut self, count: usize) -> Result<Range<SlotId>> {
        let start = self.next_reg;
        self.next_reg = u16::try_from(count)
            .ok()
            .and_then(|count| start.checked_add(count))
            .ok_or(EngineError::TooLarge("registers"))?;
        Ok(start..self.next_reg)
    }

    fn alloc_reg(&mut self) -> Result<SlotId> {
        Ok(self.alloc(1)?.start)
    }

    fn alloc_scan_id(&mut self) -> ScanId {
        let id = ScanId::new(self.next_scan_id);
        self.next_scan_id += 1;
        id
    }

    /// Register a cursor reading `scan_id` into the registers from `slot_base`.
    fn add_cursor(&mut self, scan_id: ScanId, slot_base: SlotId) -> Result<u16> {
        let cursor_id = count_u16(self.cursors.len(), "cursors")?;
        self.cursors.push(CursorInfo { scan_id, slot_base });
        Ok(cursor_id)
    }

    /// Lower the operators from `top` (just below the sink) down to the query's sources.
    fn lower_root(&mut self, graph: &PlanGraph<'_>, top: OpId) -> Result<(Root, Shape)> {
        if let BindingsOp::ExprQuery(eq) = graph.operator(top)? {
            return self.lower_expr_query(eq);
        }
        let lowered = self.lower_op(graph, top, &mut CompileContext::new())?;
        let shape = lowered
            .shape
            .unwrap_or(Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic)));
        Ok((Root::Rows(lowered.op), shape))
    }

    /// Lower the operator `id` and everything below it. `ctx` collects the fields of FROM
    /// bindings that the operators above read, so that the scans load only those: each
    /// operator adds its own before lowering its input.
    fn lower_op(
        &mut self,
        graph: &PlanGraph<'_>,
        id: OpId,
        ctx: &mut CompileContext,
    ) -> Result<Lowered> {
        let op = graph.operator(id)?;
        if let BindingsOp::Scan(_) | BindingsOp::Join(_) = op {
            let (resolver, layout) = self.layout_from(graph, id, ctx)?;
            let op = self.lower_from(&layout, &resolver)?;
            return Ok(Lowered::new(op, resolver, None));
        }

        // What the operators above read, before this operator adds its own reads
        let read_above = matches!(op, BindingsOp::OrderBy(_)).then(|| ctx.clone());
        extract_fields(op, ctx);
        let Lowered {
            op: input,
            resolver,
            shape,
            ordered,
        } = self.lower_op(graph, graph.single_input(id)?, ctx)?;
        let mut lowered = match op {
            BindingsOp::Filter(filter) => {
                let cond = self.lower_cond(&filter.expr, &resolver)?;
                Lowered::new(
                    Op::Filter(Box::new(Filter { input, cond })),
                    resolver,
                    shape,
                )
            }
            BindingsOp::Having(having) => {
                let cond = self.lower_cond(&having.expr, &resolver)?;
                Lowered::new(
                    Op::Filter(Box::new(Filter { input, cond })),
                    resolver,
                    shape,
                )
            }
            BindingsOp::Project(project) => {
                let (outputs, shape) = self.lower_project(project, &resolver)?;
                Lowered::projected(input, outputs, shape)
            }
            BindingsOp::ProjectValue(pv) => {
                let dst = self.alloc_reg()?;
                let outputs = vec![Output::Eval {
                    dst,
                    expr: self.lower_expr(&pv.expr, &resolver)?,
                }];
                let shape = Shape::Bag(RowShape::Register(dst as usize, PhysicalType::Dynamic));
                Lowered::projected(input, outputs, shape)
            }
            BindingsOp::ProjectAll(_) => {
                let (outputs, dst) = self.lower_project_all(&resolver)?;
                let shape = Shape::Bag(RowShape::Register(dst as usize, PhysicalType::Dynamic));
                Lowered::projected(input, outputs, shape)
            }
            BindingsOp::LimitOffset(lo) => match parse_limit(lo)? {
                Some(count) => {
                    let limit = Limit {
                        input,
                        count: count as i64,
                        counter: self.alloc_reg()?,
                    };
                    Lowered::new(Op::Limit(Box::new(limit)), resolver, shape)
                }
                None => Lowered::new(input, resolver, shape),
            },
            BindingsOp::GroupBy(group_by) => {
                let (aggregate, grouped) = self.lower_group_by(group_by, input, &resolver)?;
                Lowered::new(Op::Aggregate(Box::new(aggregate)), grouped, shape)
            }
            BindingsOp::OrderBy(order_by) => {
                let read_above = read_above.unwrap_or_default();
                let sort = self.lower_order_by(order_by, input, &resolver, &read_above)?;
                let mut sorted = Lowered::new(Op::Sort(Box::new(sort)), resolver, shape);
                sorted.ordered = true;
                sorted
            }
            other => {
                return Err(EngineError::InvalidPlan(format!(
                    "unsupported operator in pipeline: {:?}",
                    std::mem::discriminant(other)
                )))
            }
        };
        // The operators above a sort keep its order
        if ordered {
            lowered.ordered = true;
            lowered.shape = lowered.shape.map(|shape| match shape {
                Shape::Bag(row) => Shape::List(row),
                shape => shape,
            });
        }
        Ok(lowered)
    }

    /// Lay out a FROM subtree: compile its scans, give each a cursor and a window of
    /// registers, and resolve its bindings.
    fn layout_from<'p>(
        &mut self,
        graph: &PlanGraph<'p>,
        id: OpId,
        ctx: &mut CompileContext,
    ) -> Result<(ResolverKind, Layout<'p>)> {
        match graph.operator(id)? {
            BindingsOp::Scan(scan) => {
                let (scan_id, binding, width) = self.compile_scan(scan, ctx)?;
                let window = self.alloc(width)?;
                let binding = binding.shifted(window.start);
                let slots = ScanSlots {
                    scan_id,
                    cursor: self.add_cursor(scan_id, window.start)?,
                    window,
                    whole_row: binding.base_row_slot.is_some(),
                };
                Ok((ResolverKind::Pipeline(binding), Layout::Scan(slots)))
            }
            BindingsOp::Join(join) => {
                let (left_id, right_id) = graph.join_inputs(id)?;
                {
                    let mut extractor = ExprFieldExtractor::new(ctx);
                    if let Some(on) = &join.on {
                        extractor.extract(on);
                    }
                    // A lateral right side (`FROM t, t.arr x`) reads fields of the left side.
                    for expr in from_source_exprs(graph, right_id)? {
                        extractor.extract(expr);
                    }
                }
                let (left_resolver, left) = self.layout_from(graph, left_id, ctx)?;
                let (right_resolver, right) = self.layout_from(graph, right_id, ctx)?;
                let mut bindings = left_resolver.into_bindings()?;
                bindings.extend(right_resolver.into_bindings()?);
                let layout = Layout::Join(Box::new(JoinLayout {
                    kind: match join.kind {
                        JoinKind::Inner | JoinKind::Cross => JoinType::Inner,
                        JoinKind::Left => JoinType::Left,
                        JoinKind::Right => JoinType::Right,
                        JoinKind::Full => JoinType::Full,
                        _ => {
                            return Err(EngineError::InvalidPlan(format!(
                                "unsupported join kind: {:?}",
                                join.kind
                            )))
                        }
                    },
                    on: join.on.as_ref(),
                    left,
                    right,
                }));
                Ok((ResolverKind::Multi(bindings), layout))
            }
            other => Err(EngineError::InvalidPlan(format!(
                "unsupported FROM source: {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    /// Lower a laid-out FROM subtree, resolving names with the whole FROM clause's
    /// bindings.
    fn lower_from(&mut self, layout: &Layout<'_>, resolver: &ResolverKind) -> Result<Op> {
        match layout {
            Layout::Scan(scan) => Ok(Op::Scan(ScanLoop {
                cursor: scan.cursor,
                open: self.lower_open(scan.scan_id, resolver)?,
                window: scan.window.clone(),
                whole_row: scan.whole_row,
            })),
            Layout::Join(join) => Ok(Op::Join(Box::new(Join {
                kind: join.kind,
                left: self.lower_from(&join.left, resolver)?,
                right: self.lower_from(&join.right, resolver)?,
                on: join
                    .on
                    .map(|on| self.lower_cond(on, resolver))
                    .transpose()?,
                matched: self.alloc_reg()?,
                ret: self.alloc_reg()?,
            }))),
        }
    }

    fn lower_open(&mut self, scan_id: ScanId, resolver: &ResolverKind) -> Result<Open> {
        Ok(match &self.scans[&scan_id] {
            ScanKind::Bound => Open::Bound,
            ScanKind::TableFn(tf) => {
                let func_name = tf.func_name.clone();
                let (arg_slots, arguments) = (tf.arg_slots.clone(), tf.arguments.clone());
                let mut args = Vec::with_capacity(arguments.len());
                for (slot, arg) in arg_slots.into_iter().zip(&arguments) {
                    args.push((slot, self.lower_expr(arg, resolver)?));
                }
                Open::TableFn { func_name, args }
            }
            ScanKind::Expr { expr, src } => {
                let (expr, src) = (expr.clone(), *src);
                Open::Expr {
                    src,
                    expr: self.lower_expr(&expr, resolver)?,
                    assert_collection: self.mode == EvaluationMode::Strict,
                }
            }
        })
    }

    fn lower_cond(&mut self, expr: &ValueExpr, resolver: &ResolverKind) -> Result<Cond> {
        Ok(Cond {
            pred: self.alloc_reg()?,
            expr: self.lower_expr(expr, resolver)?,
        })
    }

    /// Lower `expr`, resolving names with `resolver`, then with those of the queries around
    /// this one. Its subqueries are lowered first.
    fn lower_expr(&mut self, expr: &ValueExpr, resolver: &ResolverKind) -> Result<Expr> {
        let mut found = vec![];
        subquery_exprs(expr, &mut found);
        let mut subqueries = LoweredSubqueries::default();
        for subquery in found {
            let lowered = self.lower_subquery(subquery, resolver)?;
            subqueries.insert(std::ptr::from_ref(subquery), lowered);
        }
        let scoped = ScopedResolver {
            inner: resolver,
            outer: &self.scopes,
        };
        LogicalExprCompiler::new(&scoped)
            .with_subqueries(&subqueries)
            .lower_expr(expr)
    }

    /// Lower a subquery of an expression that names things with `resolver`.
    fn lower_subquery(&mut self, subquery: &SubQueryExpr, resolver: &ResolverKind) -> Result<Expr> {
        let graph = PlanGraph::new(&subquery.plan);
        let top = match graph.find_sink() {
            Ok(sink) => graph.single_input(sink)?,
            Err(_) => graph.subquery_top()?,
        };
        if let BindingsOp::ExprQuery(_) = graph.operator(top)? {
            return Err(EngineError::UnsupportedExpr(
                "subquery without FROM".to_string(),
            ));
        }
        self.scopes.push(OuterScope {
            resolver: resolver.clone(),
            correlated: Cell::new(false),
        });
        let lowered = self.lower_op(&graph, top, &mut CompileContext::new());
        let scope = self.scopes.pop().expect("pushed above");
        let lowered = lowered?;

        let (row, fields) = match lowered.shape {
            Some(
                Shape::Bag(RowShape::Register(reg, _)) | Shape::List(RowShape::Register(reg, _)),
            ) => (vec![reg as SlotId], None),
            Some(Shape::Bag(RowShape::Struct(shapes)) | Shape::List(RowShape::Struct(shapes))) => {
                let mut row = Vec::with_capacity(shapes.len());
                let mut fields = Vec::with_capacity(shapes.len());
                for shape in shapes {
                    let (FieldName::Static(name), RowShape::Register(reg, _)) =
                        (shape.name, shape.value)
                    else {
                        return Err(EngineError::InvalidPlan(
                            "subquery row with computed attributes".to_string(),
                        ));
                    };
                    fields.push(name.to_string());
                    row.push(reg as SlotId);
                }
                (row, Some(fields))
            }
            _ => {
                return Err(EngineError::InvalidPlan(
                    "subquery without a projection".to_string(),
                ))
            }
        };
        // Insert the row's registers in place when they are consecutive
        let record = match row.first() {
            Some(&first) if row.windows(2).all(|w| w[1] == w[0] + 1) => first,
            _ => self.alloc(row.len())?.start,
        };
        let sorter = count_u16(self.sorters.len(), "subqueries")?;
        self.sorters.push(SorterMetadata {
            keys: SortKeys::Group(0),
        });
        let dst = self.alloc_reg()?;
        let once = match scope.correlated.get() {
            true => None,
            false => Some(self.alloc_reg()?),
        };
        let id = u32::try_from(self.subqueries.len())
            .map_err(|_| EngineError::TooLarge("subqueries"))?;
        self.subqueries.push(Subquery {
            input: lowered.op,
            sorter,
            row,
            record,
            fields,
            ordered: lowered.ordered,
            dst,
            once,
        });
        Ok(Expr::Subquery { id, dst })
    }

    /// Lower a SELECT list: its outputs and the shape of its rows.
    fn lower_project(
        &mut self,
        project: &LogicalProject,
        resolver: &ResolverKind,
    ) -> Result<(Vec<Output>, Shape)> {
        let dsts = self.alloc(project.exprs.len())?;
        let mut outputs = Vec::with_capacity(project.exprs.len());
        let mut fields = Vec::with_capacity(project.exprs.len());
        for (dst, (name, expr)) in dsts.zip(&project.exprs) {
            outputs.push(Output::Eval {
                dst,
                expr: self.lower_expr(expr, resolver)?,
            });
            fields.push(FieldShape {
                name: FieldName::Static(name.clone()),
                value: RowShape::Register(dst as usize, PhysicalType::Dynamic),
            });
        }
        Ok((outputs, Shape::Bag(RowShape::Struct(fields))))
    }

    /// Lower `SELECT *`: its outputs and its output register.
    fn lower_project_all(&mut self, resolver: &ResolverKind) -> Result<(Vec<Output>, SlotId)> {
        let dst = self.alloc_reg()?;
        let mut outputs = vec![];
        match resolver {
            ResolverKind::Pipeline(binding) => {
                let row = ValueExpr::VarRef(
                    BindingsName::CaseInsensitive(binding.scan_alias.clone().into()),
                    VarRefType::Local,
                );
                outputs.push(Output::Eval {
                    dst,
                    expr: self.lower_expr(&row, resolver)?,
                });
                outputs.push(match self.mode {
                    EvaluationMode::Permissive => Output::CoerceToTuple(dst),
                    EvaluationMode::Strict => Output::AssertTuple(dst),
                });
            }
            ResolverKind::Multi(bindings) => {
                // Concatenate each binding's fields, in FROM order
                let srcs = bindings
                    .iter()
                    .map(|b| {
                        b.base_row_slot.ok_or_else(|| {
                            EngineError::InvalidPlan(format!(
                                "SELECT * without the whole row of {}",
                                b.scan_alias
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                outputs.push(Output::ConcatTuples { dst, srcs });
            }
            ResolverKind::Global(_) | ResolverKind::Empty => {
                return Err(EngineError::InvalidPlan(
                    "SELECT * on expression query".to_string(),
                ))
            }
        }
        Ok((outputs, dst))
    }

    /// Lower GROUP BY over `input`, and the resolver for what follows (group keys and
    /// aggregates by name).
    fn lower_group_by(
        &mut self,
        group_by: &GroupBy,
        input: Op,
        resolver: &ResolverKind,
    ) -> Result<(Aggregate, ResolverKind)> {
        let keys: Vec<(&String, &ValueExpr)> = group_by.exprs.iter().collect();
        // Every index into the record fits a register number once the record is allocated
        let record_len = keys.len() + group_by.aggregate_exprs.len();
        let key_count = count_u16(keys.len(), "GROUP BY keys")?;

        let mut record_exprs = Vec::with_capacity(record_len);
        for (_, expr) in &keys {
            record_exprs.push(self.lower_expr(expr, resolver)?);
        }
        for agg in &group_by.aggregate_exprs {
            record_exprs.push(self.lower_expr(&agg.expr, resolver)?);
        }
        let sorter = count_u16(self.sorters.len(), "GROUP BY clauses")?;
        self.sorters.push(SorterMetadata {
            keys: SortKeys::Group(key_count as usize),
        });
        let insert = self.alloc(record_len)?.start;

        let record = self.alloc(record_len)?.start;
        let group_keys = self.alloc(keys.len())?.start;
        let accums_start = self.next_reg;
        let mut aggs = Vec::with_capacity(group_by.aggregate_exprs.len());
        for (i, agg) in group_by.aggregate_exprs.iter().enumerate() {
            let input = record + key_count + i as u16;
            let accum = self.alloc_reg()?;
            let (steps, finalize) = match &agg.func {
                // AVG keeps a sum and a count
                partiql_logical::AggFunc::AggAvg => (
                    vec![(AggFunc::Sum, accum), (AggFunc::Count, self.alloc_reg()?)],
                    AggFunc::Avg,
                ),
                other => {
                    let func = agg_func(other)?;
                    (vec![(func, accum)], func)
                }
            };
            aggs.push(GroupAgg {
                input,
                steps,
                finalize,
                accum,
            });
        }
        let accums = accums_start..self.next_reg;

        // Group keys and aggregates, by name, for HAVING and the SELECT list
        let mut column_slots = FxHashMap::default();
        for (i, (name, _)) in keys.iter().enumerate() {
            column_slots.insert((*name).clone(), group_keys + i as u16);
        }
        for (agg, lowered) in group_by.aggregate_exprs.iter().zip(&aggs) {
            column_slots.insert(agg.name.clone(), lowered.accum);
        }
        let grouped = ResolverKind::Pipeline(PipelineSlotResolver {
            base_row_slot: None,
            scan_alias: String::new(),
            table_name: None,
            column_slots,
            group_output: true,
        });

        let aggregate = Aggregate {
            input,
            sorter,
            record_exprs,
            insert,
            record,
            key_count,
            keys: group_keys,
            same_group: self.alloc_reg()?,
            key_eq: self.alloc_reg()?,
            new_group: self.alloc_reg()?,
            aggs,
            accums,
            has_data: self.alloc_reg()?,
            ret: self.alloc_reg()?,
        };
        Ok((aggregate, grouped))
    }

    /// Lower ORDER BY over `input`. Its rows keep `input`'s bindings: the payload is every
    /// register of them that the operators above read (`read_above`).
    fn lower_order_by(
        &mut self,
        order_by: &OrderBy,
        input: Op,
        resolver: &ResolverKind,
        read_above: &CompileContext,
    ) -> Result<Sort> {
        let mut keys = Vec::with_capacity(order_by.specs.len());
        for spec in &order_by.specs {
            keys.push(self.lower_expr(&spec.expr, resolver)?);
        }
        let orders = order_by
            .specs
            .iter()
            .map(|spec| SortOrder {
                desc: matches!(spec.order, SortSpecOrder::Desc),
                nulls_first: matches!(spec.null_order, SortSpecNullOrder::First),
            })
            .collect();
        let payload = resolver.registers_read(read_above);
        // Every index into the record fits a register number once the record is allocated
        let record_len = keys.len() + payload.len();
        let sorter = count_u16(self.sorters.len(), "ORDER BY clauses")?;
        self.sorters.push(SorterMetadata {
            keys: SortKeys::Order(orders),
        });
        Ok(Sort {
            input,
            sorter,
            keys,
            payload,
            insert: self.alloc(record_len)?.start,
            record: self.alloc(record_len)?.start,
        })
    }

    /// Lower a query without FROM. Catalog objects the expression names are read through
    /// implicit scans, one register each.
    fn lower_expr_query(
        &mut self,
        expr_query: &partiql_logical::ExprQuery,
    ) -> Result<(Root, Shape)> {
        let mut slots = FxHashMap::default();
        let mut cursors = vec![];
        for db_ref in collect_db_refs(&expr_query.expr) {
            let Some(name) = db_ref.path.first().and_then(binding_name_string) else {
                continue;
            };
            if slots.contains_key(&name) {
                continue;
            }
            let Ok((catalog_id, reader_factory)) = self.resolve_catalog_table(db_ref) else {
                continue;
            };

            let scan_id = self.alloc_scan_id();
            let slot = self.alloc_reg()?;
            let layout = ScanLayout {
                projections: vec![ScanProjection {
                    source: ScanSource::whole_value(),
                    target_slot: 0,
                }],
            };
            let object_id = ObjectId::new(catalog_id, reader_factory.entry_id);
            self.scan_metadata
                .insert(scan_id, ScanMetadata { layout, object_id });
            self.scans.insert(scan_id, ScanKind::Bound);
            cursors.push(self.add_cursor(scan_id, slot)?);
            slots.insert(name, slot);
        }

        let resolver = if slots.is_empty() {
            ResolverKind::Empty
        } else {
            ResolverKind::Global(GlobalSlotResolver { slots })
        };
        let expr = self.lower_expr(&expr_query.expr, &resolver)?;
        let dst = self.alloc_reg()?;
        let shape = Shape::Single(RowShape::Register(dst as usize, PhysicalType::Dynamic));
        Ok((Root::Expr { cursors, expr, dst }, shape))
    }

    // -----------------------------------------------------------------------
    // Scans
    // -----------------------------------------------------------------------

    /// Compile a scan: what it reads and which fields it loads, into registers numbered
    /// from 0. Returns its id, its binding and how many registers it fills.
    fn compile_scan(
        &mut self,
        scan: &Scan,
        ctx: &CompileContext,
    ) -> Result<(ScanId, PipelineSlotResolver, usize)> {
        // Literal collections are inlined at compile time
        if let Some(values) = try_expr_to_inline_values(&scan.expr) {
            if self.mode == EvaluationMode::Strict && !is_collection_expr(&scan.expr) {
                return Err(EngineError::StrictModeViolation(
                    "FROM clause requires collection value".to_string(),
                ));
            }
            let scan_id = self.whole_value_scan(ScanKind::Bound);
            self.inline_scans.insert(scan_id, values);
            return Ok((scan_id, whole_row_binding(scan), 1));
        }

        // Table functions
        if let ValueExpr::Call(call) = &scan.expr {
            if let CallName::ByName(func_name) = &call.name {
                if let Some(handle) = self.resolve_table_function(func_name) {
                    let table_name = Some(func_name.clone());
                    let (binding, layout) =
                        pushdown(scan, table_name, ctx, |field| handle.resolve(field));
                    let width = layout.projections.len();
                    let tf = TableFnScan {
                        func_name: func_name.clone(),
                        layout: layout.clone(),
                        arguments: call.arguments.clone(),
                        arg_slots: self.alloc(call.arguments.len())?.collect(),
                    };
                    let scan_id = self.alloc_scan_id();
                    self.scan_metadata.insert(
                        scan_id,
                        ScanMetadata {
                            layout,
                            object_id: unbound_object_id(),
                        },
                    );
                    self.scans.insert(scan_id, ScanKind::TableFn(tf));
                    return Ok((scan_id, binding, width));
                }
            }
        }

        // Catalog tables
        let Ok((catalog_id, reader_factory)) = self.resolve_reader_factory(scan) else {
            // Anything else is evaluated at runtime and scanned from its value
            let src = self.alloc_reg()?;
            let scan_id = self.whole_value_scan(ScanKind::Expr {
                expr: scan.expr.clone(),
                src,
            });
            return Ok((scan_id, whole_row_binding(scan), 1));
        };
        let table_name = extract_table_name(&scan.expr);
        let (binding, layout) =
            pushdown(scan, table_name, ctx, |field| reader_factory.resolve(field));
        let width = layout.projections.len();
        let scan_id = self.alloc_scan_id();
        let object_id = ObjectId::new(catalog_id, reader_factory.entry_id);
        self.scan_metadata
            .insert(scan_id, ScanMetadata { layout, object_id });
        self.scans.insert(scan_id, ScanKind::Bound);
        Ok((scan_id, binding, width))
    }

    /// Register a scan that loads whole rows into one register.
    fn whole_value_scan(&mut self, kind: ScanKind) -> ScanId {
        let scan_id = self.alloc_scan_id();
        self.scan_metadata.insert(
            scan_id,
            ScanMetadata {
                layout: whole_value_layout(),
                object_id: unbound_object_id(),
            },
        );
        self.scans.insert(scan_id, kind);
        scan_id
    }

    /// Search all catalogs for a table function by name.
    fn resolve_table_function(&self, name: &str) -> Option<TableFunctionHandle> {
        // Only the "default" catalog is searched for now.
        let (_id, catalog) = self.compilation_context.get_catalog("default")?;
        catalog.get_table_function(name)
    }

    fn resolve_reader_factory(
        &self,
        scan: &Scan,
    ) -> Result<(partiql_common::catalog::CatalogId, DataSourceHandle)> {
        match &scan.expr {
            ValueExpr::DBRef(db_ref) => self.resolve_catalog_table(db_ref),
            ValueExpr::VarRef(table_name, _) => {
                let db_ref = DBRef {
                    catalog: "default".to_string(),
                    path: vec![table_name.clone()],
                };
                self.resolve_catalog_table(&db_ref)
            }
            _ => Err(EngineError::InvalidPlan(
                "unsupported scan expression type".to_string(),
            )),
        }
    }

    fn resolve_catalog_table(
        &self,
        db_ref: &DBRef,
    ) -> Result<(partiql_common::catalog::CatalogId, DataSourceHandle)> {
        let (catalog_id, catalog) = self
            .compilation_context
            .get_catalog(&db_ref.catalog)
            .ok_or_else(|| {
                EngineError::InvalidPlan(format!("catalog '{}' not found", db_ref.catalog))
            })?;

        let handle = catalog.get_table(&db_ref.path).ok_or_else(|| {
            let path_str = db_ref
                .path
                .iter()
                .map(|c| match c {
                    BindingsName::CaseSensitive(s) => format!("\"{}\"", s),
                    BindingsName::CaseInsensitive(s) => s.to_string(),
                    other => format!("{other:?}"),
                })
                .collect::<Vec<_>>()
                .join(".");
            EngineError::InvalidPlan(format!(
                "table '{}' not found in catalog '{}'",
                path_str, db_ref.catalog
            ))
        })?;

        Ok((catalog_id, handle))
    }
}

/// Projection pushdown: load only the fields the query reads from the scan, each into its
/// own register, when the source can `resolve` every one of them; else the whole row.
fn pushdown(
    scan: &Scan,
    table_name: Option<String>,
    ctx: &CompileContext,
    resolve: impl Fn(&str) -> Option<ScanSource>,
) -> (PipelineSlotResolver, ScanLayout) {
    let requests = ctx.requests_for_alias(&scan.as_key, table_name.as_deref());
    let whole_value =
        requests.is_empty() || ctx.needs_whole_value(&scan.as_key, table_name.as_deref());
    let mut binding = PipelineSlotResolver {
        base_row_slot: None,
        scan_alias: scan.as_key.clone(),
        table_name,
        column_slots: FxHashMap::default(),
        group_output: false,
    };
    if !whole_value {
        let mut projections = Vec::with_capacity(requests.len());
        for req in &requests {
            let Some(source) = resolve(&req.field_name) else {
                break;
            };
            let slot = projections.len() as SlotId;
            projections.push(ScanProjection {
                source,
                target_slot: slot,
            });
            binding.column_slots.insert(req.field_name.clone(), slot);
        }
        if projections.len() == requests.len() {
            return (binding, ScanLayout { projections });
        }
        binding.column_slots.clear();
    }
    binding.base_row_slot = Some(0);
    (binding, whole_value_layout())
}

fn whole_row_binding(scan: &Scan) -> PipelineSlotResolver {
    PipelineSlotResolver {
        base_row_slot: Some(0),
        scan_alias: scan.as_key.clone(),
        table_name: None,
        column_slots: FxHashMap::default(),
        group_output: false,
    }
}

fn whole_value_layout() -> ScanLayout {
    ScanLayout {
        projections: vec![ScanProjection {
            source: ScanSource::whole_value(),
            target_slot: 0,
        }],
    }
}

/// The object id of a scan not read from a catalog table (literals, table functions,
/// runtime collections).
fn unbound_object_id() -> ObjectId {
    ObjectId::from((
        partiql_common::catalog::CatalogId::from(u64::MAX),
        partiql_common::catalog::EntryId::from(u64::MAX),
    ))
}

/// Record the fields of FROM bindings that `op` reads.
fn extract_fields(op: &BindingsOp, ctx: &mut CompileContext) {
    if let BindingsOp::ProjectAll(_) = op {
        ctx.request_all_whole_values();
        return;
    }
    let mut extractor = ExprFieldExtractor::new(ctx);
    match op {
        BindingsOp::Filter(filter) => extractor.extract(&filter.expr),
        BindingsOp::Having(having) => extractor.extract(&having.expr),
        BindingsOp::Project(project) => {
            for (_, expr) in &project.exprs {
                extractor.extract(expr);
            }
        }
        BindingsOp::ProjectValue(pv) => extractor.extract(&pv.expr),
        BindingsOp::GroupBy(group_by) => {
            for expr in group_by.exprs.values() {
                extractor.extract(expr);
            }
            for agg in &group_by.aggregate_exprs {
                extractor.extract(&agg.expr);
            }
        }
        BindingsOp::OrderBy(order_by) => {
            for spec in &order_by.specs {
                extractor.extract(&spec.expr);
            }
        }
        _ => {}
    }
}

/// The subqueries of `expr`, outermost first; not those inside them.
fn subquery_exprs<'e>(expr: &'e ValueExpr, out: &mut Vec<&'e SubQueryExpr>) {
    let mut walk_all = |exprs: &mut dyn Iterator<Item = &'e ValueExpr>| {
        exprs.for_each(|e| subquery_exprs(e, out));
    };
    match expr {
        ValueExpr::SubQueryExpr(subquery) => out.push(subquery),
        ValueExpr::Lit(_) | ValueExpr::VarRef(..) | ValueExpr::DBRef(_) => {}
        ValueExpr::Path(base, steps) => {
            let step_exprs = steps.iter().filter_map(|step| match step {
                PathComponent::KeyExpr(e) | PathComponent::IndexExpr(e) => Some(&**e),
                _ => None,
            });
            walk_all(&mut std::iter::once(&**base).chain(step_exprs));
        }
        ValueExpr::UnExpr(_, e) => walk_all(&mut std::iter::once(&**e)),
        ValueExpr::BinaryExpr(_, lhs, rhs) => walk_all(&mut [&**lhs, &**rhs].into_iter()),
        ValueExpr::DynamicLookup(lookups) => walk_all(&mut lookups.iter()),
        ValueExpr::TupleExpr(t) => walk_all(&mut t.attrs.iter().chain(&t.values)),
        ValueExpr::ListExpr(l) => walk_all(&mut l.elements.iter()),
        ValueExpr::BagExpr(b) => walk_all(&mut b.elements.iter()),
        ValueExpr::BetweenExpr(b) => walk_all(&mut [&*b.value, &*b.from, &*b.to].into_iter()),
        ValueExpr::PatternMatchExpr(m) => {
            walk_all(&mut std::iter::once(&*m.value));
            if let Pattern::LikeNonStringNonLiteral(p) = &m.pattern {
                walk_all(&mut [&*p.pattern, &*p.escape].into_iter());
            }
        }
        ValueExpr::SimpleCase(c) => {
            let cases = c.cases.iter().flat_map(|(w, t)| [&**w, &**t]);
            walk_all(
                &mut std::iter::once(&*c.expr)
                    .chain(cases)
                    .chain(c.default.as_deref()),
            );
        }
        ValueExpr::SearchedCase(c) => {
            let cases = c.cases.iter().flat_map(|(w, t)| [&**w, &**t]);
            walk_all(&mut cases.chain(c.default.as_deref()));
        }
        ValueExpr::IsTypeExpr(t) => walk_all(&mut std::iter::once(&*t.expr)),
        ValueExpr::NullIfExpr(n) => walk_all(&mut [&*n.lhs, &*n.rhs].into_iter()),
        ValueExpr::CoalesceExpr(c) => walk_all(&mut c.elements.iter()),
        ValueExpr::Call(c) => walk_all(&mut c.arguments.iter()),
        // Expression lowering rejects the rest
        _ => {}
    }
}

fn agg_func(func: &partiql_logical::AggFunc) -> Result<AggFunc> {
    Ok(match func {
        partiql_logical::AggFunc::AggSum => AggFunc::Sum,
        partiql_logical::AggFunc::AggCount => AggFunc::Count,
        partiql_logical::AggFunc::AggAvg => AggFunc::Avg,
        partiql_logical::AggFunc::AggMin => AggFunc::Min,
        partiql_logical::AggFunc::AggMax => AggFunc::Max,
        partiql_logical::AggFunc::AggAny => AggFunc::Any,
        partiql_logical::AggFunc::AggEvery => AggFunc::Every,
        other => {
            return Err(EngineError::UnsupportedExpr(format!(
                "aggregate function {other:?}"
            )))
        }
    })
}

// ---------------------------------------------------------------------------
// Slot resolvers
// ---------------------------------------------------------------------------

/// How a scope's names resolve to registers.
#[derive(Clone)]
enum ResolverKind {
    /// Single-scan resolver (maps alias and column names to slots)
    Pipeline(PipelineSlotResolver),
    /// One resolver per FROM binding of a join, in FROM order
    Multi(Vec<PipelineSlotResolver>),
    /// Global resolver for expression queries that reference catalog objects
    Global(GlobalSlotResolver),
    /// Empty resolver for expression queries with no variable references
    Empty,
}

impl SlotResolver for ResolverKind {
    fn resolve_var(&self, name: &BindingsName<'_>, scope: VarRefType) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_var(name, scope),
            // A global (catalog) name never means a FROM binding of the same name
            ResolverKind::Multi(_) if scope == VarRefType::Global => None,
            ResolverKind::Multi(rs) => binding_for(rs, name).and_then(|r| r.base_row_slot),
            ResolverKind::Global(r) => r.resolve_var(name, scope),
            ResolverKind::Empty => None,
        }
    }
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_alias(name),
            ResolverKind::Multi(rs) => binding_for(rs, name).and_then(|r| r.base_row_slot),
            ResolverKind::Global(r) => r.resolve_alias(name),
            ResolverKind::Empty => None,
        }
    }
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_field(name),
            // An unqualified field is ambiguous across join bindings; the planner qualifies it.
            ResolverKind::Multi(_) => None,
            ResolverKind::Global(r) => r.resolve_field(name),
            ResolverKind::Empty => None,
        }
    }
    fn is_alias(&self, name: &BindingsName<'_>) -> bool {
        match self {
            ResolverKind::Pipeline(r) => r.is_alias(name),
            ResolverKind::Multi(rs) => binding_for(rs, name).is_some(),
            ResolverKind::Global(_) => false,
            ResolverKind::Empty => false,
        }
    }
    fn resolve_qualified(
        &self,
        alias: &BindingsName<'_>,
        field: &BindingsName<'_>,
    ) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_qualified(alias, field),
            ResolverKind::Multi(rs) => binding_for(rs, alias)?.resolve_qualified(alias, field),
            ResolverKind::Global(_) | ResolverKind::Empty => None,
        }
    }
}

impl ResolverKind {
    /// The registers that names read in `reads` resolve to, in order.
    fn registers_read(&self, reads: &CompileContext) -> Vec<SlotId> {
        let mut regs: Vec<SlotId> = match self {
            ResolverKind::Pipeline(r) => r.registers_read(reads),
            ResolverKind::Multi(rs) => rs.iter().flat_map(|r| r.registers_read(reads)).collect(),
            ResolverKind::Global(r) => r.slots.values().copied().collect(),
            ResolverKind::Empty => vec![],
        };
        regs.sort_unstable();
        regs.dedup();
        regs
    }

    /// The per-binding resolvers of a FROM subtree.
    fn into_bindings(self) -> Result<Vec<PipelineSlotResolver>> {
        match self {
            ResolverKind::Pipeline(r) => Ok(vec![r]),
            ResolverKind::Multi(rs) => Ok(rs),
            ResolverKind::Global(_) | ResolverKind::Empty => Err(EngineError::InvalidPlan(
                "join input is not a FROM source".to_string(),
            )),
        }
    }
}

/// The join binding `name` refers to: an alias match first, then a table name. `None` when
/// no binding or more than one matches (`FROM t1 AS x, t2 AS X` makes `x` ambiguous).
fn binding_for<'r>(
    bindings: &'r [PipelineSlotResolver],
    name: &BindingsName<'_>,
) -> Option<&'r PipelineSlotResolver> {
    let first_two = |pred: &dyn Fn(&PipelineSlotResolver) -> bool| {
        let mut matches = bindings.iter().filter(move |r| pred(r));
        (matches.next(), matches.next())
    };
    match first_two(&|r| bindings_name_matches(name, &r.scan_alias)) {
        (Some(r), None) => Some(r),
        (Some(_), Some(_)) => None,
        (None, _) => match first_two(&|r| r.matches_alias_or_table(name)) {
            (Some(r), None) => Some(r),
            _ => None,
        },
    }
}

/// The names of a query: its own (`inner`), then those of the queries around it, innermost
/// first. Resolving a name of an outer query marks the subqueries in between as correlated.
/// Unqualified fields, and bindings whose pushed-down fields a path may read, are the query's
/// own only: an outer binding is read whole, so an inner field `a` never answers `e.a`.
struct ScopedResolver<'r> {
    inner: &'r ResolverKind,
    outer: &'r [OuterScope],
}

impl ScopedResolver<'_> {
    fn outward(&self, resolve: impl Fn(&ResolverKind) -> Option<SlotId>) -> Option<SlotId> {
        self.outer
            .iter()
            .enumerate()
            .rev()
            .find_map(|(depth, scope)| {
                let slot = resolve(&scope.resolver)?;
                for scope in &self.outer[depth..] {
                    scope.correlated.set(true);
                }
                Some(slot)
            })
    }
}

impl SlotResolver for ScopedResolver<'_> {
    fn resolve_var(&self, name: &BindingsName<'_>, scope: VarRefType) -> Option<SlotId> {
        self.inner
            .resolve_var(name, scope.clone())
            .or_else(|| self.outward(|r| r.resolve_var(name, scope.clone())))
    }
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        self.inner
            .resolve_alias(name)
            .or_else(|| self.outward(|r| r.resolve_alias(name)))
    }
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        self.inner.resolve_field(name)
    }
    fn is_alias(&self, name: &BindingsName<'_>) -> bool {
        self.inner.is_alias(name)
    }
    fn resolve_qualified(
        &self,
        alias: &BindingsName<'_>,
        field: &BindingsName<'_>,
    ) -> Option<SlotId> {
        self.inner
            .resolve_qualified(alias, field)
            .or_else(|| self.outward(|r| r.resolve_qualified(alias, field)))
    }
}

/// Resolver for global DB object references in expression queries.
#[derive(Clone)]
struct GlobalSlotResolver {
    slots: FxHashMap<String, SlotId>,
}

impl SlotResolver for GlobalSlotResolver {
    fn resolve_var(&self, name: &BindingsName<'_>, _scope: VarRefType) -> Option<SlotId> {
        let key = match name {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
            _ => return None,
        };
        self.slots
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| *v)
    }
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        self.resolve_var(name, VarRefType::Global)
    }
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        self.resolve_var(name, VarRefType::Global)
    }
    fn is_alias(&self, _name: &BindingsName<'_>) -> bool {
        false
    }
    /// Catalog objects are loaded whole; none has pushed-down fields.
    fn resolve_qualified(
        &self,
        _alias: &BindingsName<'_>,
        _field: &BindingsName<'_>,
    ) -> Option<SlotId> {
        None
    }
}

/// Resolves the names of one FROM binding: its alias (or table name) to the register
/// holding its whole row, and its pushed-down fields to theirs.
#[derive(Clone)]
struct PipelineSlotResolver {
    base_row_slot: Option<SlotId>,
    scan_alias: String,
    table_name: Option<String>,
    column_slots: FxHashMap<String, SlotId>,
    /// GROUP BY's output instead: its keys and aggregates by name, with no alias.
    group_output: bool,
}

impl PipelineSlotResolver {
    fn matches_alias_or_table(&self, name: &BindingsName<'_>) -> bool {
        if bindings_name_matches(name, &self.scan_alias) {
            return true;
        }
        if let Some(ref table) = self.table_name {
            if bindings_name_matches(name, table) {
                return true;
            }
        }
        false
    }

    /// The binding's registers that names read in `reads` resolve to: the requested
    /// fields, and the whole row when it is read (or a field is read through it).
    fn registers_read(&self, reads: &CompileContext) -> Vec<SlotId> {
        // Field requests name FROM bindings, never GROUP BY's keys and aggregates
        if self.group_output {
            return self.column_slots.values().copied().collect();
        }
        let table = self.table_name.as_deref();
        let mut whole_row = reads.needs_whole_value(&self.scan_alias, table);
        let mut regs = vec![];
        for request in reads.requests_for_alias(&self.scan_alias, table) {
            match self.resolve_field(&BindingsName::CaseInsensitive(
                request.field_name.as_str().into(),
            )) {
                Some(slot) => regs.push(slot),
                None => whole_row = true,
            }
        }
        if whole_row {
            regs.extend(self.base_row_slot);
        }
        regs
    }

    /// The same binding with its registers moved up by `offset`.
    fn shifted(mut self, offset: SlotId) -> Self {
        self.base_row_slot = self.base_row_slot.map(|s| s + offset);
        for slot in self.column_slots.values_mut() {
            *slot += offset;
        }
        self
    }
}

impl SlotResolver for PipelineSlotResolver {
    fn resolve_qualified(
        &self,
        alias: &BindingsName<'_>,
        field: &BindingsName<'_>,
    ) -> Option<SlotId> {
        if self.matches_alias_or_table(alias) {
            self.resolve_field(field)
        } else {
            None
        }
    }

    fn resolve_var(&self, name: &BindingsName<'_>, _scope: VarRefType) -> Option<SlotId> {
        if self.matches_alias_or_table(name) {
            return self.base_row_slot;
        }
        self.resolve_field(name)
    }

    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        if self.matches_alias_or_table(name) {
            self.base_row_slot
        } else {
            None
        }
    }

    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        let key = match name {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
            _ => return None,
        };
        self.column_slots
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| *v)
    }

    fn is_alias(&self, name: &BindingsName<'_>) -> bool {
        self.matches_alias_or_table(name)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The expressions a FROM subtree evaluates per row of what is to its left: scan sources and
/// table function arguments.
fn from_source_exprs<'p>(graph: &PlanGraph<'p>, id: OpId) -> Result<Vec<&'p ValueExpr>> {
    match graph.operator(id)? {
        BindingsOp::Scan(scan) => Ok(match &scan.expr {
            ValueExpr::Call(call) => call.arguments.iter().collect(),
            expr => vec![expr],
        }),
        BindingsOp::Join(join) => {
            let (left, right) = graph.join_inputs(id)?;
            let mut exprs = from_source_exprs(graph, left)?;
            exprs.extend(from_source_exprs(graph, right)?);
            exprs.extend(join.on.as_ref());
            Ok(exprs)
        }
        _ => Ok(vec![]),
    }
}

fn extract_table_name(expr: &ValueExpr) -> Option<String> {
    match expr {
        ValueExpr::VarRef(name, _) => binding_name_string(name),
        ValueExpr::DBRef(db_ref) => db_ref.path.last().and_then(binding_name_string),
        _ => None,
    }
}

/// Try to convert a scan expression to inline values.
/// Returns Some(values) for literal lists/bags/structs, None otherwise.
fn try_expr_to_inline_values(expr: &ValueExpr) -> Option<Vec<ValueOwned>> {
    match expr {
        ValueExpr::Lit(lit) => match lit.as_ref() {
            partiql_logical::Lit::List(elements) | partiql_logical::Lit::Bag(elements) => {
                let values: Vec<ValueOwned> = elements
                    .iter()
                    .filter_map(|e| lit_to_value(e).ok())
                    .collect();
                if values.len() == elements.len() {
                    Some(values)
                } else {
                    None
                }
            }
            partiql_logical::Lit::Struct(_) => lit_to_value(lit).ok().map(|val| vec![val]),
            _ => lit_to_value(lit).ok().map(|val| vec![val]),
        },
        ValueExpr::ListExpr(list) => {
            let values: Vec<ValueOwned> = list
                .elements
                .iter()
                .filter_map(try_expr_to_single_value)
                .collect();
            if values.len() == list.elements.len() {
                Some(values)
            } else {
                None
            }
        }
        ValueExpr::BagExpr(bag) => {
            let values: Vec<ValueOwned> = bag
                .elements
                .iter()
                .filter_map(try_expr_to_single_value)
                .collect();
            if values.len() == bag.elements.len() {
                Some(values)
            } else {
                None
            }
        }
        ValueExpr::TupleExpr(_) => {
            // A tuple expression in FROM — treat as a single row
            try_expr_to_single_value(expr).map(|val| vec![val])
        }
        _ => None,
    }
}

/// Whether a literal FROM source is a collection (rather than a value scanned as `<<value>>`).
fn is_collection_expr(expr: &ValueExpr) -> bool {
    match expr {
        ValueExpr::Lit(lit) => matches!(
            lit.as_ref(),
            partiql_logical::Lit::List(_) | partiql_logical::Lit::Bag(_)
        ),
        ValueExpr::ListExpr(_) | ValueExpr::BagExpr(_) => true,
        _ => false,
    }
}

/// Try to convert a single ValueExpr to a ValueOwned (for literal expressions).
fn try_expr_to_single_value(expr: &ValueExpr) -> Option<ValueOwned> {
    match expr {
        ValueExpr::Lit(lit) => lit_to_value(lit).ok(),
        ValueExpr::TupleExpr(tuple) => {
            let mut fields = Vec::with_capacity(tuple.attrs.len());
            for (attr, val) in tuple.attrs.iter().zip(tuple.values.iter()) {
                let name = match attr {
                    ValueExpr::Lit(lit) => match lit.as_ref() {
                        partiql_logical::Lit::String(s) => s.clone(),
                        _ => return None,
                    },
                    _ => return None,
                };
                let value = try_expr_to_single_value(val)?;
                fields.push(crate::value::TupleFieldOwned { name, value });
            }
            Some(ValueOwned::Tuple(crate::value::TupleOwned { fields }))
        }
        ValueExpr::ListExpr(list) => {
            let items: Vec<ValueOwned> = list
                .elements
                .iter()
                .filter_map(try_expr_to_single_value)
                .collect();
            if items.len() == list.elements.len() {
                Some(ValueOwned::List(items))
            } else {
                None
            }
        }
        ValueExpr::BagExpr(bag) => {
            let items: Vec<ValueOwned> = bag
                .elements
                .iter()
                .filter_map(try_expr_to_single_value)
                .collect();
            if items.len() == bag.elements.len() {
                Some(ValueOwned::Bag(items))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Recursively collect all DBRef nodes from a ValueExpr tree.
fn collect_db_refs(expr: &ValueExpr) -> Vec<&DBRef> {
    let mut refs = Vec::new();
    collect_db_refs_inner(expr, &mut refs);
    refs
}

fn collect_db_refs_inner<'a>(expr: &'a ValueExpr, out: &mut Vec<&'a DBRef>) {
    match expr {
        ValueExpr::DBRef(db_ref) => out.push(db_ref),
        ValueExpr::UnExpr(_, inner) => collect_db_refs_inner(inner, out),
        ValueExpr::BinaryExpr(_, lhs, rhs) => {
            collect_db_refs_inner(lhs, out);
            collect_db_refs_inner(rhs, out);
        }
        ValueExpr::Call(call) => {
            for arg in &call.arguments {
                collect_db_refs_inner(arg, out);
            }
        }
        ValueExpr::ListExpr(list) => {
            for elem in &list.elements {
                collect_db_refs_inner(elem, out);
            }
        }
        ValueExpr::BagExpr(bag) => {
            for elem in &bag.elements {
                collect_db_refs_inner(elem, out);
            }
        }
        ValueExpr::TupleExpr(tuple) => {
            for attr in &tuple.attrs {
                collect_db_refs_inner(attr, out);
            }
            for val in &tuple.values {
                collect_db_refs_inner(val, out);
            }
        }
        ValueExpr::Path(base, _steps) => {
            collect_db_refs_inner(base.as_ref(), out);
        }
        ValueExpr::DynamicLookup(lookups) => {
            for lookup in lookups.iter() {
                collect_db_refs_inner(lookup, out);
            }
        }
        _ => {}
    }
}

fn parse_limit(lo: &LimitOffset) -> Result<Option<usize>> {
    if lo.offset.is_some() {
        return Err(EngineError::InvalidPlan("offset not supported".to_string()));
    }
    let expr = match &lo.limit {
        Some(expr) => expr,
        None => return Ok(None),
    };
    match expr {
        ValueExpr::Lit(lit) => match &**lit {
            partiql_logical::Lit::Int8(v) => limit_to_usize(*v as i64),
            partiql_logical::Lit::Int16(v) => limit_to_usize(*v as i64),
            partiql_logical::Lit::Int32(v) => limit_to_usize(*v as i64),
            partiql_logical::Lit::Int64(v) => limit_to_usize(*v),
            _ => Err(EngineError::InvalidPlan(
                "limit must be integer literal".to_string(),
            )),
        },
        _ => Err(EngineError::InvalidPlan(
            "limit must be a literal".to_string(),
        )),
    }
}

fn limit_to_usize(value: i64) -> Result<Option<usize>> {
    if value < 0 {
        return Err(EngineError::InvalidPlan(
            "limit must be non-negative".to_string(),
        ));
    }
    Ok(Some(value as usize))
}

fn binding_name_string(name: &BindingsName<'_>) -> Option<String> {
    match name {
        BindingsName::CaseSensitive(s) => Some(s.as_ref().to_string()),
        BindingsName::CaseInsensitive(s) => Some(s.as_ref().to_string()),
        _ => None,
    }
}

fn bindings_name_matches(name: &BindingsName<'_>, target: &str) -> bool {
    match name {
        BindingsName::CaseSensitive(s) => s.as_ref() == target,
        BindingsName::CaseInsensitive(s) => s.as_ref().eq_ignore_ascii_case(target),
        _ => false,
    }
}
