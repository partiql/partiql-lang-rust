use crate::arena::SlotId;
use crate::catalog::CompilationContext;
use crate::error::{EngineError, Result};
use crate::expr::AggFunc;
use crate::expr::SlotResolver;
use crate::expr::{lit_to_value, Inst, LogicalExprCompiler, ProgramBuilder};
use crate::field_resolver::{CompileContext, ExprFieldExtractor};
use crate::plan::EvaluationMode;
use crate::plan::{CompiledPlan, CursorInfo, ObjectId, ScanId, ScanMetadata};
use crate::source::{
    DataSourceHandle, ScanLayout, ScanProjection, ScanSource, TableFunctionHandle,
};
use crate::value::{FieldName, FieldShape, PhysicalType, RowShape, Shape, ValueOwned};
use partiql_logical::{
    BindingsOp, CallName, DBRef, GroupBy, JoinKind, LimitOffset, LogicalPlan, OpId, PathComponent,
    Project, ProjectAllMode, ProjectValue, Scan, ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

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
}

// ---------------------------------------------------------------------------
// Compilation context passed up from child → parent during tree walk
// ---------------------------------------------------------------------------

/// The result of compiling a subtree of the logical plan.
///
/// Each `compile_node()` call returns this, providing the parent with:
/// - A slot resolver for compiling expressions that reference this subtree's bindings
/// - Cursor info for the scan (if this is a scan node)
/// - Accumulated scan metadata
/// - Slot allocation state
struct SubtreeResult {
    /// How to resolve variable references in expressions above this node
    resolver: ResolverKind,
    /// If this subtree is a scan, the cursor_id assigned to it
    #[allow(dead_code)]
    cursor_id: Option<u16>,
    /// Scan metadata accumulated from this subtree
    scan_metadata: HashMap<ScanId, ScanMetadata>,
    /// Current slot allocation high-water mark
    slot_count: usize,
    /// Output shape (set by projection nodes)
    shape: Option<Shape>,
}

/// The kind of slot resolver produced by a subtree.
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
    /// The per-binding resolvers of a FROM subtree, shifted to start at register `offset`.
    fn into_bindings(self, offset: SlotId) -> Result<Vec<PipelineSlotResolver>> {
        let mut bindings = match self {
            ResolverKind::Pipeline(r) => vec![r],
            ResolverKind::Multi(rs) => rs,
            ResolverKind::Global(_) | ResolverKind::Empty => {
                return Err(EngineError::InvalidPlan(
                    "join input is not a FROM source".to_string(),
                ))
            }
        };
        for r in &mut bindings {
            r.base_row_slot = r.base_row_slot.map(|s| s + offset);
            for slot in r.column_slots.values_mut() {
                *slot += offset;
            }
        }
        Ok(bindings)
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

/// Resolver for global DB object references in expression queries.
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

// ---------------------------------------------------------------------------
// PlanCompiler — recursive tree-walk compilation emitting flat bytecode
// ---------------------------------------------------------------------------

pub struct PlanCompiler<'a> {
    compilation_context: &'a CompilationContext,
    mode: EvaluationMode,
    next_scan_id: u64,
    inline_scans: HashMap<ScanId, Vec<ValueOwned>>,
    expr_scans: HashMap<ScanId, ValueExpr>,
    table_fn_scans: HashMap<ScanId, TableFnScanInfo>,
    table_fn_arg_slots: HashMap<ScanId, Vec<SlotId>>,
    next_sorter_id: usize,
    sorter_infos: Vec<SorterCompileInfo>,
    /// The scan compiled for each `Scan` operator.
    scan_ids: HashMap<OpId, ScanId>,
    /// First register of each scan's cursor window.
    scan_slot_bases: HashMap<ScanId, SlotId>,
    /// Registers taken by the rows of each FROM subtree (a `Scan` or a `Join`).
    from_slots: HashMap<OpId, usize>,
    /// Registers in each scan's cursor window.
    scan_widths: HashMap<ScanId, u16>,
    /// Scans that load whole rows (rather than projected fields).
    whole_row_scans: std::collections::HashSet<ScanId>,
}

/// The loops of a FROM subtree, as emitted by `emit_from`.
struct LoopNest {
    /// The innermost `NextRow`: where the row body jumps back to for the next row.
    head: u32,
    /// `NextRow`s whose end-of-input leaves the whole nest, patched by the caller.
    exits: Vec<usize>,
    /// The outermost cursor, still open when the nest ends (inner cursors close themselves).
    outer_cursor: u16,
    /// Every cursor of the nest.
    cursors: Vec<u16>,
}

struct SorterCompileInfo {
    key_count: usize,
}

#[derive(Clone)]
struct TableFnScanInfo {
    func_name: String,
    layout: ScanLayout,
    arguments: Vec<ValueExpr>,
}

impl<'a> PlanCompiler<'a> {
    pub fn new(compilation_context: &'a CompilationContext, mode: EvaluationMode) -> Self {
        PlanCompiler {
            compilation_context,
            mode,
            next_scan_id: 0,
            inline_scans: HashMap::new(),
            expr_scans: HashMap::new(),
            table_fn_scans: HashMap::new(),
            table_fn_arg_slots: HashMap::new(),
            next_sorter_id: 0,
            sorter_infos: Vec::new(),
            scan_ids: HashMap::new(),
            scan_slot_bases: HashMap::new(),
            from_slots: HashMap::new(),
            scan_widths: HashMap::new(),
            whole_row_scans: Default::default(),
        }
    }

    fn alloc_scan_id(&mut self) -> ScanId {
        let id = ScanId::new(self.next_scan_id);
        self.next_scan_id += 1;
        id
    }

    /// Compile a logical plan into a flat bytecode `CompiledPlan`.
    ///
    /// Walks the plan graph recursively from the sink node downward,
    /// gathering scan metadata and slot layout. Then emits a single flat
    /// instruction stream: OpenCursor → NextRow → [filter] → [project] → EmitRow → Jump → CloseCursor → Halt
    pub fn compile(&mut self, plan: &LogicalPlan<BindingsOp>) -> Result<CompiledPlan> {
        let graph = PlanGraph::new(plan);
        let sink_id = graph.find_sink()?;
        let mut ctx = CompileContext::new();

        // Phase 1: Gather metadata (scan layout, slot assignments, shapes)
        let result = self.compile_node(&graph, sink_id, &mut ctx)?;

        // Phase 2: Emit flat bytecode
        let (program, cursor_infos) =
            self.emit_bytecode(&graph, sink_id, &result, &mut CompileContext::new())?;

        let shape = result
            .shape
            .unwrap_or(Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic)));

        let table_fn_scans = self
            .table_fn_scans
            .drain()
            .map(|(id, info)| {
                let arg_slots = self.table_fn_arg_slots.remove(&id).unwrap_or_default();
                (
                    id,
                    crate::plan::TableFnScanMetadata {
                        func_name: info.func_name,
                        layout: info.layout,
                        arg_slots,
                    },
                )
            })
            .collect();

        let sorter_metadata = self
            .sorter_infos
            .iter()
            .map(|s| crate::plan::SorterMetadata {
                key_count: s.key_count,
            })
            .collect();

        Ok(CompiledPlan {
            program,
            cursors: cursor_infos,
            shape,
            slot_count: result.slot_count,
            scan_metadata: result.scan_metadata,
            inline_scans: std::mem::take(&mut self.inline_scans),
            expr_scan_ids: self.expr_scans.keys().copied().collect(),
            table_fn_scans,
            sorter_metadata,
        })
    }

    /// Emit the flat bytecode program for the entire query.
    ///
    /// Layout for a simple SFW query:
    /// ```text
    ///   0: OpenCursor(0)
    ///   1: NextRow(0, eof_target=N)    // loop_head
    ///   2: ... filter instructions ...
    ///   3: JumpIfNotTrue(pred, loop_head)
    ///   4: ... project instructions ...
    ///   5: [DecrOrJump(counter, done)]  // if LIMIT
    ///   6: EmitRow
    ///   7: Jump(loop_head)             // back to NextRow
    ///   N: CloseCursor(0)
    /// N+1: Halt
    /// ```
    ///
    /// For expression-only queries (no scan):
    /// ```text
    ///   0: ... expr instructions ...
    ///   1: EmitRow
    ///   2: Halt
    /// ```
    fn emit_bytecode(
        &mut self,
        graph: &PlanGraph<'_>,
        sink_id: OpId,
        result: &SubtreeResult,
        ctx: &mut CompileContext,
    ) -> Result<(crate::expr::Program, Vec<CursorInfo>)> {
        let mut builder = ProgramBuilder::new(result.slot_count as u16);
        let mut cursor_infos: Vec<CursorInfo> = Vec::new();

        // Walk down to find the structure
        let input_id = graph.single_input(sink_id)?;
        self.emit_node(
            graph,
            input_id,
            result,
            ctx,
            &mut builder,
            &mut cursor_infos,
        )?;

        let program = builder.build();
        Ok((program, cursor_infos))
    }

    /// Recursively emit bytecode for a node and its descendants.
    fn emit_node(
        &mut self,
        graph: &PlanGraph<'_>,
        id: OpId,
        result: &SubtreeResult,
        _ctx: &mut CompileContext,
        builder: &mut ProgramBuilder,
        cursor_infos: &mut Vec<CursorInfo>,
    ) -> Result<()> {
        let op = graph.operator(id)?;
        match op {
            BindingsOp::Sink => {
                let input_id = graph.single_input(id)?;
                self.emit_node(graph, input_id, result, _ctx, builder, cursor_infos)
            }
            BindingsOp::Scan(scan) => {
                // Emit: OpenCursor → NextRow (with eof_target placeholder)
                let scan_id = self.find_scan_id_for(scan, result)?;
                let cursor_id = cursor_infos.len() as u16;
                cursor_infos.push(CursorInfo {
                    scan_id,
                    slot_base: 0,
                    width: 1,
                    spool: None,
                });

                builder.emit_open_cursor(cursor_id);
                let _next_row_idx = builder.emit_next_row(cursor_id);

                // The caller (project/filter/limit) will continue emitting after this.
                // We need to store the loop state for patching.
                // Store the info we need in a way the caller can access it.
                // Actually, for a linear pipeline we process bottom-up:
                // Scan → Filter → Project → LimitOffset → Sink
                // But we're called top-down from Sink. So let's restructure.
                //
                // The simplest approach: do a second pass where we gather the
                // "pipeline" (linear chain of operators) and emit in order.

                // This method shouldn't be called directly for Scan in the recursive
                // top-down approach. Instead, emit_pipeline handles the full chain.
                // Let's keep this as unreachable for now.
                unreachable!("Scan should be handled by emit_pipeline");
                #[allow(unreachable_code)]
                {
                    let _ = (_next_row_idx, cursor_id);
                    Ok(())
                }
            }
            _ => {
                // For the top-down approach, we collect the linear pipeline and emit it all at once
                self.emit_pipeline(graph, id, result, builder, cursor_infos)
            }
        }
    }

    /// Emit bytecode for a linear pipeline (Scan → Filter* → Project → Limit? → Sink).
    ///
    /// Collects the chain of operators, then emits them in the correct order.
    fn emit_pipeline(
        &mut self,
        graph: &PlanGraph<'_>,
        top_id: OpId,
        result: &SubtreeResult,
        builder: &mut ProgramBuilder,
        cursor_infos: &mut Vec<CursorInfo>,
    ) -> Result<()> {
        // Collect the linear chain from top (just below Sink) down to Scan
        let mut chain = Vec::new();
        let mut current_id = top_id;
        loop {
            let op = graph.operator(current_id)?;
            chain.push((current_id, op.clone()));
            match op {
                BindingsOp::Scan(_) | BindingsOp::Join(_) | BindingsOp::ExprQuery(_) => break,
                _ => {
                    current_id = graph.single_input(current_id)?;
                }
            }
        }
        // chain is [top, ..., scan] — reverse to get [scan, ..., top]
        chain.reverse();

        // Determine if this is an expression-only query
        if let Some((_, BindingsOp::ExprQuery(eq))) = chain.first() {
            return self.emit_expr_query(eq, result, builder, cursor_infos);
        }

        // --- Scan-based pipeline ---
        // First element must be a FROM source (a Scan or a Join of scans)
        let from_id = match chain.first() {
            Some((id, BindingsOp::Scan(_) | BindingsOp::Join(_))) => *id,
            _ => {
                return Err(EngineError::InvalidPlan(
                    "pipeline must start with Scan".to_string(),
                ))
            }
        };

        // Pre-scan chain for LIMIT value.
        let mut limit_value: Option<usize> = None;
        for (_, op) in chain.iter().skip(1) {
            if let BindingsOp::LimitOffset(lo) = op {
                limit_value = parse_limit(lo)?;
            }
        }

        // Reserve a placeholder slot for the LIMIT LoadConst (emitted later once
        // we know the counter register won't conflict with sub-program temps).
        let limit_loadconst_idx = if limit_value.is_some() {
            let idx = builder.current_offset();
            builder.insts.push(Inst::Halt); // placeholder — will be replaced
            Some(idx)
        } else {
            None
        };

        // Emit the scan loops: OpenCursor → NextRow (loop head), nested for joins
        let nest = self.emit_from(graph, from_id, result, builder, cursor_infos)?;
        let loop_head = nest.head as usize;

        // Track slot allocation incrementally (mirroring the analysis pass).
        // The FROM rows occupy slots 0..from_slots. Each subsequent operator allocates
        // from current_slot onward.
        let mut current_slot = self.from_slots.get(&from_id).copied().unwrap_or(1);

        for (_, op) in chain.iter().skip(1) {
            match op {
                BindingsOp::Filter(filter) => {
                    // The predicate and its temps go above every allocated register: the
                    // slots past the FROM rows may hold join state (e.g. LEFT JOIN flags).
                    current_slot += 1;
                    let pred_slot = builder.alloc_reg_pub();
                    let expr_compiler = LogicalExprCompiler::new(&result.resolver);
                    let filter_program =
                        expr_compiler.compile_to_program(&filter.expr, pred_slot, pred_slot + 1)?;

                    // Inline the filter program's instructions
                    self.inline_program(&filter_program, builder)?;

                    // Emit: JumpIfNotTrue → back to loop_head (skip this row)
                    let jump_idx = builder.emit_jump_if_not_true(pred_slot);
                    builder.patch_target(jump_idx, loop_head as u32);
                }
                BindingsOp::Project(project) => {
                    self.emit_project_at(project, result, current_slot, builder)?;
                    current_slot += project.exprs.len();
                }
                BindingsOp::ProjectValue(pv) => {
                    self.emit_project_value_at(pv, result, current_slot, builder)?;
                    current_slot += 1;
                }
                BindingsOp::ProjectAll(_mode) => {
                    self.emit_project_all_at(result, current_slot, builder)?;
                    current_slot += 1;
                }
                BindingsOp::LimitOffset(_) | BindingsOp::Scan(_) => {}
                BindingsOp::GroupBy(group_by) => {
                    return self.emit_group_by_pipeline(
                        &chain,
                        group_by,
                        &nest,
                        current_slot,
                        result,
                        builder,
                        cursor_infos,
                    );
                }
                other => {
                    return Err(EngineError::InvalidPlan(format!(
                        "unsupported operator in pipeline: {:?}",
                        std::mem::discriminant(other)
                    )));
                }
            }
        }

        // Emit: DecrOrJump (if LIMIT) — jumps past the loop when counter hits 0.
        // Allocate the counter register NOW (after all sub-programs have been inlined)
        // so it doesn't conflict with any temp registers used by filter/project code.
        let decr_jump_idx = if let Some(limit) = limit_value {
            let counter_reg = builder.alloc_reg_pub();
            let const_idx = builder.push_const_pub(crate::value::ValueOwned::I64(limit as i64));
            // Patch the placeholder LoadConst at the beginning
            builder.insts[limit_loadconst_idx.unwrap() as usize] = Inst::LoadConst {
                dst: counter_reg,
                const_idx,
            };
            Some(builder.emit_decr_or_jump(counter_reg))
        } else {
            None
        };

        // Emit: EmitRow
        builder.emit_emit_row();

        // Emit: Jump back to loop_head (NextRow)
        let back_jump = builder.emit_jump();
        builder.patch_target(back_jump, loop_head as u32);

        // --- After loop ---
        let after_loop = builder.current_offset();

        // Patch the outermost NextRow's eof_target to point here
        for idx in &nest.exits {
            builder.patch_target(*idx, after_loop);
        }

        // Emit: CloseCursor (inner join cursors closed themselves at their end)
        builder.emit_close_cursor(nest.outer_cursor);

        // Emit: Halt
        builder.emit_halt();

        // LIMIT reached mid-row: every cursor of the nest is still open
        if let Some(idx) = decr_jump_idx {
            builder.patch_target(idx, builder.current_offset());
            for cursor_id in &nest.cursors {
                builder.emit_close_cursor(*cursor_id);
            }
            builder.emit_halt();
        }

        Ok(())
    }

    /// Emit the loops of a FROM subtree: one cursor loop per scan, nested for joins.
    ///
    /// A join reopens its right side for every left row, so a lateral right side
    /// (`FROM t, t.arr x`) sees the current left row. A right side that does not read the
    /// left side is spooled instead: read once per entry into the join, then replayed.
    ///
    /// TODO: the spool replays every row; for RIGHT/FULL the left side is still rescanned
    /// per right row. Memoize a side into a B-tree keyed on the ON equality (a `CreateBTree`
    /// / probe instruction pair) so each row probes instead of scanning.
    /// ```text
    ///   <left loops>                       // innermost left NextRow = L.head
    ///   Jump open_right
    ///   right_done: CloseCursor(R); Jump L.head
    ///   open_right: <open R>
    ///   R.head: NextRow(R, eof=right_done)
    ///   [ON predicate; JumpIfNotTrue R.head]
    ///   <row body>                         // jumps back to the nest's head
    /// ```
    ///
    /// LEFT and FULL track whether a right row matched; when none did, `right_done` sets
    /// the right side's registers to NULL and runs the body once with `padding` set.
    /// RIGHT and FULL add a second pass once the left side ends: for each right row, scan
    /// the left side again, and if no left row matches, run the body with the left side's
    /// registers NULL. The body of a padded or pass-2 row returns to a `cont` stub that
    /// picks the next row.
    fn emit_from(
        &mut self,
        graph: &PlanGraph<'_>,
        id: OpId,
        result: &SubtreeResult,
        builder: &mut ProgramBuilder,
        cursor_infos: &mut Vec<CursorInfo>,
    ) -> Result<LoopNest> {
        let join = match graph.operator(id)? {
            BindingsOp::Scan(_) => {
                let scan_id = self.scan_id(id)?;
                let cursor_id = self.emit_open_scan(scan_id, result, builder, cursor_infos)?;
                let head = builder.current_offset();
                let next_row_idx = builder.emit_next_row(cursor_id);
                return Ok(LoopNest {
                    head,
                    exits: vec![next_row_idx],
                    outer_cursor: cursor_id,
                    cursors: vec![cursor_id],
                });
            }
            BindingsOp::Join(join) => join,
            other => {
                return Err(EngineError::InvalidPlan(format!(
                    "unsupported FROM source: {:?}",
                    std::mem::discriminant(other)
                )))
            }
        };

        let (left_id, right_id) = graph.join_inputs(id)?;
        let pad_right = matches!(join.kind, JoinKind::Left | JoinKind::Full);
        let unmatched_right = matches!(join.kind, JoinKind::Right | JoinKind::Full);
        let right_scan = match graph.operator(right_id)? {
            BindingsOp::Scan(_) => Some(self.scan_id(right_id)?),
            _ => None,
        };
        if unmatched_right && right_scan.is_none() {
            return Err(EngineError::UnsupportedExpr(format!(
                "{:?} JOIN with a join on its right side",
                join.kind
            )));
        }
        // RIGHT and FULL never see the left side; the planner scopes them that way.
        let spool = right_scan.is_some()
            && (unmatched_right || !reads_any(graph, right_id, &from_aliases(graph, left_id)?)?);

        let f = builder.push_const_pub(ValueOwned::Bool(false));
        let t = builder.push_const_pub(ValueOwned::Bool(true));
        let null = builder.push_const_pub(ValueOwned::Null);
        let load = |builder: &mut ProgramBuilder, dst: SlotId, const_idx: u16| {
            builder.insts.push(Inst::LoadConst { dst, const_idx })
        };
        // A fresh register is MISSING, which reads as false.
        let flag = |builder: &mut ProgramBuilder, used: bool| {
            if used {
                builder.alloc_reg_pub()
            } else {
                0
            }
        };
        let matched = flag(builder, pad_right);
        let padding = flag(builder, pad_right);
        let pass2 = flag(builder, unmatched_right);
        let left_matched = flag(builder, unmatched_right);

        // The right side's cursor comes after the left side's, and for RIGHT/FULL after
        // pass 2's second scan of the left side.
        let left_scans = count_scans(graph, left_id)?;
        let passes = if unmatched_right { 2 } else { 1 };
        let right_cursor = (cursor_infos.len() + left_scans * passes) as u16;
        if unmatched_right {
            load(builder, pass2, f);
        }
        let left = self.emit_from(graph, left_id, result, builder, cursor_infos)?;
        if pad_right {
            load(builder, matched, f);
        }
        let skip_done = builder.emit_jump();

        // The right side ended for this left row.
        let right_done = builder.current_offset();
        builder.emit_close_cursor(right_cursor);
        let mut to_body = vec![];
        if pad_right {
            // No right row matched: emit the left row once with the right side NULL.
            let pad = builder.emit_jump_if_not_true(matched);
            let next_left = builder.emit_jump();
            builder.patch_target(next_left, left.head);
            builder.patch_target(pad, builder.current_offset());
            self.emit_null_window(graph, right_id, null, builder)?;
            load(builder, padding, t);
            to_body.push(builder.emit_jump());
        } else {
            let next_left = builder.emit_jump();
            builder.patch_target(next_left, left.head);
        }

        // Pass 2: the left side ended. For each right row (replayed from the spool), scan
        // the left side again; emit the right row if no left row matches it.
        let mut exits = left.exits.clone();
        let mut cursors = left.cursors.clone();
        let mut pass2_head = 0;
        if let (true, Some(scan_id)) = (unmatched_right, right_scan) {
            for idx in &left.exits {
                builder.patch_target(*idx, builder.current_offset());
            }
            load(builder, pass2, t);
            self.emit_create_scan(scan_id, right_cursor, result, builder)?;
            builder.emit_open_cursor(right_cursor);
            pass2_head = builder.current_offset();
            let next_right = builder.emit_next_row(right_cursor);
            load(builder, left_matched, f);
            let left2 = self.emit_from(graph, left_id, result, builder, cursor_infos)?;
            if let Some(on) = &join.on {
                let pred_reg = builder.alloc_reg_pub();
                let expr_compiler = LogicalExprCompiler::new(&result.resolver);
                let prog = expr_compiler.compile_to_program(on, pred_reg, pred_reg + 1)?;
                self.inline_program(&prog, builder)?;
                let skip = builder.emit_jump_if_not_true(pred_reg);
                builder.patch_target(skip, left2.head);
            }
            load(builder, left_matched, t);
            let next_left2 = builder.emit_jump();
            builder.patch_target(next_left2, left2.head);
            for idx in &left2.exits {
                builder.patch_target(*idx, builder.current_offset());
            }
            builder.emit_close_cursor(left2.outer_cursor);
            let pad = builder.emit_jump_if_not_true(left_matched);
            let next = builder.emit_jump();
            builder.patch_target(next, pass2_head);
            builder.patch_target(pad, builder.current_offset());
            self.emit_null_window(graph, left_id, null, builder)?;
            to_body.push(builder.emit_jump());
            builder.patch_target(next_right, builder.current_offset());
            builder.emit_close_cursor(right_cursor);
            load(builder, pass2, f);
            exits = vec![builder.emit_jump()];
            cursors.extend(left2.cursors);
        }

        // Where the row body continues for this join: the next right row, or after a
        // padded row the next left row, or in pass 2 the next right row of pass 2.
        let cont = builder.current_offset();
        let mut to_right_head = vec![];
        if unmatched_right {
            let not_pass2 = builder.emit_jump_if_not_true(pass2);
            let next = builder.emit_jump();
            builder.patch_target(next, pass2_head);
            builder.patch_target(not_pass2, builder.current_offset());
        }
        if pad_right {
            to_right_head.push(builder.emit_jump_if_not_true(padding));
            load(builder, padding, f);
            let next_left = builder.emit_jump();
            builder.patch_target(next_left, left.head);
        } else if unmatched_right {
            to_right_head.push(builder.emit_jump());
        }
        builder.patch_target(skip_done, builder.current_offset());

        let right = self.emit_from(graph, right_id, result, builder, cursor_infos)?;
        debug_assert_eq!(right.outer_cursor, right_cursor);
        if spool {
            cursor_infos[right_cursor as usize].spool = Some(left.outer_cursor);
        }
        for idx in &right.exits {
            builder.patch_target(*idx, right_done);
        }

        if let Some(on) = &join.on {
            let pred_reg = builder.alloc_reg_pub();
            let expr_compiler = LogicalExprCompiler::new(&result.resolver);
            let prog = expr_compiler.compile_to_program(on, pred_reg, pred_reg + 1)?;
            self.inline_program(&prog, builder)?;
            let skip = builder.emit_jump_if_not_true(pred_reg);
            builder.patch_target(skip, right.head);
        }
        if pad_right {
            load(builder, matched, t);
        }
        for idx in to_body {
            builder.patch_target(idx, builder.current_offset());
        }
        for idx in to_right_head {
            builder.patch_target(idx, right.head);
        }

        cursors.extend(right.cursors);
        Ok(LoopNest {
            head: if pad_right || unmatched_right {
                cont
            } else {
                right.head
            },
            exits,
            outer_cursor: left.outer_cursor,
            cursors,
        })
    }

    fn scan_id(&self, id: OpId) -> Result<ScanId> {
        self.scan_ids
            .get(&id)
            .copied()
            .ok_or_else(|| EngineError::InvalidPlan(format!("scan {:?} was not compiled", id)))
    }

    /// Outer join padding: bind each variable of a FROM subtree to NULL. A scan that
    /// projected fields gets MISSING in them instead, as `NULL.field` is MISSING.
    fn emit_null_window(
        &self,
        graph: &PlanGraph<'_>,
        id: OpId,
        null: u16,
        builder: &mut ProgramBuilder,
    ) -> Result<()> {
        let missing = builder.push_const_pub(ValueOwned::Missing);
        for scan in from_scans(graph, id)? {
            let scan_id = self.scan_id(scan)?;
            let base = self.scan_slot_bases[&scan_id];
            let width = self.scan_widths[&scan_id];
            let const_idx = if self.whole_row_scans.contains(&scan_id) {
                null
            } else {
                missing
            };
            for dst in base..base + width {
                builder.insts.push(Inst::LoadConst { dst, const_idx });
            }
        }
        Ok(())
    }

    /// Emit the instructions that bind and open a scan's cursor; returns the cursor id.
    fn emit_open_scan(
        &mut self,
        scan_id: ScanId,
        result: &SubtreeResult,
        builder: &mut ProgramBuilder,
        cursor_infos: &mut Vec<CursorInfo>,
    ) -> Result<u16> {
        let cursor_id = cursor_infos.len() as u16;
        cursor_infos.push(CursorInfo {
            scan_id,
            slot_base: self.scan_slot_bases.get(&scan_id).copied().unwrap_or(0),
            width: self.scan_widths.get(&scan_id).copied().unwrap_or(1),
            spool: None,
        });
        self.emit_create_scan(scan_id, cursor_id, result, builder)?;
        builder.emit_open_cursor(cursor_id);
        Ok(cursor_id)
    }

    /// Emit the instructions that (re)build a table function or expression scan's data
    /// source; a catalog scan needs none.
    fn emit_create_scan(
        &mut self,
        scan_id: ScanId,
        cursor_id: u16,
        result: &SubtreeResult,
        builder: &mut ProgramBuilder,
    ) -> Result<()> {
        if let Some(tf_info) = self.table_fn_scans.get(&scan_id).cloned() {
            // Compile each argument expression into its own register. A scan built twice
            // (RIGHT/FULL pass 2) reuses the registers its CompiledPlan entry names.
            let arg_slots = match self.table_fn_arg_slots.get(&scan_id) {
                Some(slots) => slots.clone(),
                None => tf_info
                    .arguments
                    .iter()
                    .map(|_| builder.alloc_reg_pub())
                    .collect(),
            };
            let expr_compiler = LogicalExprCompiler::new(&result.resolver);
            for (arg_expr, &target) in tf_info.arguments.iter().zip(&arg_slots) {
                let temps = (target + 1).max(builder.next_reg());
                let prog = expr_compiler.compile_to_program(arg_expr, target, temps)?;
                self.inline_program(&prog, builder)?;
            }

            // Store resolved arg_slots for the CompiledPlan
            self.table_fn_arg_slots.insert(scan_id, arg_slots);

            let func_name_idx = builder.push_key_pub(tf_info.func_name.clone());
            builder.insts.push(Inst::CreateTableFnCursor {
                cursor_id,
                func_name_idx,
            });
        } else if let Some(expr) = self.expr_scans.get(&scan_id).cloned() {
            // For expression-based scans, compile the expression and emit MaterializeCursor
            let expr_compiler = LogicalExprCompiler::new(&result.resolver);
            // A temp register: register 0 may hold an outer (left) join row.
            let src = builder.alloc_reg_pub();
            let expr_program = expr_compiler.compile_to_program(&expr, src, src + 1)?;
            self.inline_program(&expr_program, builder)?;
            if self.mode == EvaluationMode::Strict {
                builder.insts.push(Inst::AssertCollection { src });
            }
            builder
                .insts
                .push(Inst::MaterializeCursor { src, cursor_id });
        }
        Ok(())
    }

    /// Emit bytecode for a GROUP BY pipeline (two-phase: scan+insert → sort+iterate+emit).
    ///
    /// Called when `emit_pipeline` encounters a `GroupBy` operator in the chain.
    /// At this point, the input scan is already open and the NextRow loop head has been emitted.
    /// Any filters before the GroupBy have also been emitted.
    #[allow(clippy::too_many_arguments)]
    fn emit_group_by_pipeline(
        &mut self,
        chain: &[(OpId, BindingsOp)],
        group_by: &GroupBy,
        nest: &LoopNest,
        current_slot: usize,
        result: &SubtreeResult,
        builder: &mut ProgramBuilder,
        cursor_infos: &mut Vec<CursorInfo>,
    ) -> Result<()> {
        let _ = cursor_infos;
        let loop_head = nest.head;
        let expr_compiler = LogicalExprCompiler::new(&result.resolver);

        // --- Determine what to insert into the sorter ---
        // Group key expressions + aggregate input expressions
        let key_exprs: Vec<(&String, &ValueExpr)> = group_by.exprs.iter().collect();
        let key_count = key_exprs.len();
        let agg_count = group_by.aggregate_exprs.len();
        let record_count = key_count + agg_count; // fields per sorter record

        // Allocate registers for computing record fields before SorterInsert, above any
        // register already allocated (join state lives there)
        let record_base = (current_slot as u16).max(builder.next_reg());
        let mut temp_slot = record_base + record_count as u16;
        builder.update_next_reg(temp_slot);

        // Compile key expressions into record_base..record_base+key_count
        for (i, (_name, expr)) in key_exprs.iter().enumerate() {
            let dst = record_base + i as u16;
            let prog = expr_compiler.compile_to_program(expr, dst, temp_slot)?;
            self.inline_program(&prog, builder)?;
            temp_slot = temp_slot.max(dst + 1 + prog.reg_count);
        }

        // Compile aggregate input expressions into record_base+key_count..
        for (i, agg) in group_by.aggregate_exprs.iter().enumerate() {
            let dst = record_base + key_count as u16 + i as u16;
            let prog = expr_compiler.compile_to_program(&agg.expr, dst, temp_slot)?;
            self.inline_program(&prog, builder)?;
            temp_slot = temp_slot.max(dst + 1 + prog.reg_count);
        }

        // Allocate sorter (compiler tracks the ID; VM will have it pre-allocated)
        let sorter_id = self.next_sorter_id;
        self.next_sorter_id += 1;

        // Emit: SorterInsert
        builder.insts.push(Inst::SorterInsert {
            sorter_id: sorter_id as u16,
            src_reg: record_base,
            reg_count: record_count as u16,
        });

        // Emit: Jump back to NextRow (continue input scan loop)
        let back_jump = builder.emit_jump();
        builder.patch_target(back_jump, loop_head);

        // --- After input loop ---
        let after_input = builder.current_offset();
        for idx in &nest.exits {
            builder.patch_target(*idx, after_input);
        }

        // Emit: CloseCursor (done reading input)
        builder.emit_close_cursor(nest.outer_cursor);

        // === PHASE 2: Sort + Iterate ===

        // Emit: SorterSort (jump to halt if empty)
        let sorter_sort_idx = builder.current_offset();
        builder.insts.push(Inst::SorterSort {
            sorter_id: sorter_id as u16,
            eof_target: 0, // patched later
        });

        // Allocate registers for the iterate phase:
        // - data regs: where SorterData writes the current record
        // - prev_key reg: holds previous group key for boundary detection
        // - accum regs: one per aggregate
        // - has_data flag
        // - gosub return address
        let data_reg = builder.alloc_reg_pub();
        // Allocate additional data regs for the full record
        for _ in 1..record_count {
            builder.alloc_reg_pub();
        }
        let prev_key_reg = builder.alloc_reg_pub();
        for _ in 1..key_count {
            builder.alloc_reg_pub();
        }
        // Allocate accumulator registers. AVG uses accum_reg for sum and
        // accum_reg+1 for count (2 registers). Others use 1 register.
        let accum_base = if agg_count > 0 {
            builder.alloc_reg_pub()
        } else {
            0
        };
        let mut accum_offsets: Vec<u16> = Vec::with_capacity(agg_count);
        let mut next_offset: u16 = 0;
        for agg in group_by.aggregate_exprs.iter() {
            accum_offsets.push(next_offset);
            let regs_needed = if matches!(agg.func, partiql_logical::AggFunc::AggAvg) {
                2
            } else {
                1
            };
            next_offset += regs_needed;
        }
        let total_accum_regs = next_offset;
        // Allocate remaining accum registers (first one already allocated above)
        for _ in 1..total_accum_regs {
            builder.alloc_reg_pub();
        }
        let has_data_reg = builder.alloc_reg_pub();
        let ret_reg = builder.alloc_reg_pub();

        // Emit: SorterData (iterate loop head)
        let iterate_head = builder.current_offset();
        builder.insts.push(Inst::SorterData {
            sorter_id: sorter_id as u16,
            dst_reg: data_reg,
        });

        // Compare ALL key fields: prev_key[i] == current[i] for each key
        // Result: all_eq = key0_eq AND key1_eq AND ...
        let cmp_result_reg = builder.alloc_reg_pub();
        // Start by comparing first key
        builder.insts.push(Inst::CompareEq {
            lhs_reg: prev_key_reg,
            rhs_reg: data_reg,
            dst_reg: cmp_result_reg,
        });
        // AND with subsequent keys
        for k in 1..key_count {
            let key_cmp_reg = builder.alloc_reg_pub();
            builder.insts.push(Inst::CompareEq {
                lhs_reg: prev_key_reg + k as u16,
                rhs_reg: data_reg + k as u16,
                dst_reg: key_cmp_reg,
            });
            builder.insts.push(Inst::AndBool {
                dst: cmp_result_reg,
                a: cmp_result_reg,
                b: key_cmp_reg,
            });
        }

        // If all keys equal (same group), skip emit. Otherwise fall through to emit.
        let not_cmp_reg = builder.alloc_reg_pub();
        builder.insts.push(Inst::NotBool {
            dst: not_cmp_reg,
            src: cmp_result_reg,
        });
        let skip_emit_jump = builder.emit_jump_if_not_true(not_cmp_reg);

        // --- GROUP BOUNDARY: emit previous group, then reset ---
        // Gosub to emit subroutine
        let gosub_emit_idx = builder.current_offset();
        builder.insts.push(Inst::Gosub {
            ret_reg,
            target: 0, // patched later
        });

        // AggReset all accumulator registers
        builder.insts.push(Inst::AggReset {
            accum_start: accum_base,
            count: total_accum_regs,
        });

        // --- ACCUMULATE ---
        let accumulate_target = builder.current_offset();
        // Patch the skip-emit jump to here
        builder.patch_target(skip_emit_jump, accumulate_target);

        // Move ALL current keys to prev_key registers
        for k in 0..key_count {
            builder.insts.push(Inst::Copy {
                dst: prev_key_reg + k as u16,
                src: data_reg + k as u16,
            });
        }

        // AggStep for each aggregate
        for (i, agg) in group_by.aggregate_exprs.iter().enumerate() {
            let input_reg = data_reg + key_count as u16 + i as u16;
            let accum_reg = accum_base + accum_offsets[i];
            match &agg.func {
                partiql_logical::AggFunc::AggAvg => {
                    // AVG: emit Sum into accum_reg, Count into accum_reg+1
                    builder.insts.push(Inst::AggStep {
                        func: AggFunc::Sum,
                        accum_reg,
                        input_reg,
                    });
                    builder.insts.push(Inst::AggStep {
                        func: AggFunc::Count,
                        accum_reg: accum_reg + 1,
                        input_reg,
                    });
                }
                other => {
                    let func = match other {
                        partiql_logical::AggFunc::AggSum => AggFunc::Sum,
                        partiql_logical::AggFunc::AggCount => AggFunc::Count,
                        partiql_logical::AggFunc::AggMin => AggFunc::Min,
                        partiql_logical::AggFunc::AggMax => AggFunc::Max,
                        partiql_logical::AggFunc::AggAny => AggFunc::Any,
                        partiql_logical::AggFunc::AggEvery => AggFunc::Every,
                        _ => unreachable!(),
                    };
                    builder.insts.push(Inst::AggStep {
                        func,
                        accum_reg,
                        input_reg,
                    });
                }
            }
        }

        // Set has_data flag
        let const_one = builder.push_const_pub(ValueOwned::Bool(true));
        builder.insts.push(Inst::LoadConst {
            dst: has_data_reg,
            const_idx: const_one,
        });

        // Emit: SorterNext → jump back to iterate_head
        builder.insts.push(Inst::SorterNext {
            sorter_id: sorter_id as u16,
            target: iterate_head,
        });

        // --- After iterate loop: emit final group ---
        let gosub_final_idx = builder.current_offset();
        builder.insts.push(Inst::Gosub {
            ret_reg,
            target: 0, // patched later (same emit subroutine)
        });
        let jump_to_halt = builder.emit_jump(); // jump to halt

        // === EMIT SUBROUTINE ===
        let emit_sub_addr = builder.current_offset();
        // Patch both gosub targets
        if let Inst::Gosub { target, .. } = &mut builder.insts[gosub_emit_idx as usize] {
            *target = emit_sub_addr;
        }
        if let Inst::Gosub { target, .. } = &mut builder.insts[gosub_final_idx as usize] {
            *target = emit_sub_addr;
        }

        // Check has_data flag — if missing/0, skip emit
        let emit_skip_jump = builder.emit_jump_if_not_true(has_data_reg);

        // AggFinal for each aggregate
        for (i, agg) in group_by.aggregate_exprs.iter().enumerate() {
            let accum_reg = accum_base + accum_offsets[i];
            let func = match &agg.func {
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
            };
            builder.insts.push(Inst::AggFinal {
                func,
                accum_reg,
                dst_reg: accum_reg, // finalize in-place
            });
        }

        // Build a resolver for the grouped output phase.
        // Maps group key aliases and $__agg_N names to their registers.
        let mut grouped_column_slots = FxHashMap::default();

        // Group key aliases → prev_key_reg (for single key) or data_reg + offset
        // Since we copy the key to prev_key_reg during accumulate, and the emit
        // subroutine fires with the previous group's key still in prev_key_reg:
        for (i, (name, _expr)) in key_exprs.iter().enumerate() {
            // For multi-key: we'd need multiple prev_key registers.
            // For now, use data_reg + i since during emit the last SorterData
            // still has the boundary row, and prev_key holds the group's key.
            // Actually the key we want is in prev_key_reg for the first key.
            // TODO: multi-key support — for now just map first key to prev_key_reg
            if i == 0 {
                grouped_column_slots.insert((*name).clone(), prev_key_reg);
            } else {
                // Multi-key: allocate additional prev_key registers
                // For now, skip — this limits us to single-key GROUP BY
                grouped_column_slots.insert((*name).clone(), prev_key_reg + i as u16);
            }
        }

        // Aggregate names ($__agg_1, etc.) → accum registers (where AggFinal writes)
        for (i, agg) in group_by.aggregate_exprs.iter().enumerate() {
            grouped_column_slots.insert(agg.name.clone(), accum_base + accum_offsets[i]);
        }

        let grouped_resolver = ResolverKind::Pipeline(PipelineSlotResolver {
            base_row_slot: None,
            scan_alias: String::new(),
            table_name: None,
            column_slots: grouped_column_slots,
        });
        let grouped_result = SubtreeResult {
            resolver: grouped_resolver,
            cursor_id: None,
            slot_count: result.slot_count,
            scan_metadata: HashMap::new(),
            shape: None,
        };

        let grouped_expr_compiler = LogicalExprCompiler::new(&grouped_result.resolver);

        // Now handle any operators AFTER the GroupBy in the chain (Having, Project)
        let group_by_idx = chain
            .iter()
            .position(|(_, op)| matches!(op, BindingsOp::GroupBy(_)))
            .unwrap();

        let mut having_skip_idx: Option<usize> = None;

        // Output slots mirror the analysis pass, which reserves one slot per HAVING
        let mut output_slot = 0;
        for (_, op) in chain.iter().skip(group_by_idx + 1) {
            match op {
                BindingsOp::Having(having) => {
                    output_slot += 1;
                    let having_pred_reg = builder.alloc_reg_pub();
                    let having_temp = having_pred_reg + 1;
                    let prog = grouped_expr_compiler.compile_to_program(
                        &having.expr,
                        having_pred_reg,
                        having_temp,
                    )?;
                    self.inline_program(&prog, builder)?;
                    having_skip_idx = Some(builder.emit_jump_if_not_true(having_pred_reg));
                }
                BindingsOp::Project(project) => {
                    self.emit_project_at(project, &grouped_result, output_slot, builder)?;
                }
                BindingsOp::ProjectValue(pv) => {
                    self.emit_project_value_at(pv, &grouped_result, output_slot, builder)?;
                }
                BindingsOp::ProjectAll(_mode) => {
                    self.emit_project_all_at(&grouped_result, output_slot, builder)?;
                }
                _ => {}
            }
        }

        // EmitRow
        builder.emit_emit_row();

        // Clear has_data flag
        let const_zero = builder.push_const_pub(ValueOwned::Bool(false));
        builder.insts.push(Inst::LoadConst {
            dst: has_data_reg,
            const_idx: const_zero,
        });

        // Return from subroutine
        let return_addr = builder.current_offset();
        builder.insts.push(Inst::Return { ret_reg });

        // Patch emit_skip_jump to the Return
        builder.patch_target(emit_skip_jump, return_addr);

        // Patch HAVING skip jump to the Return (skip EmitRow, go straight to Return)
        if let Some(idx) = having_skip_idx {
            builder.patch_target(idx, return_addr);
        }

        // Patch jump_to_halt
        let halt_addr = builder.current_offset();
        builder.patch_target(jump_to_halt, halt_addr);

        // Patch SorterSort eof_target to halt
        if let Inst::SorterSort { eof_target, .. } = &mut builder.insts[sorter_sort_idx as usize] {
            *eof_target = halt_addr;
        }

        // Emit: Halt
        builder.emit_halt();

        // Record the sorter info for the compiled plan
        self.sorter_infos.push(SorterCompileInfo { key_count });

        Ok(())
    }

    /// Emit bytecode for an expression-only query (no scan).
    fn emit_expr_query(
        &self,
        eq: &partiql_logical::ExprQuery,
        result: &SubtreeResult,
        builder: &mut ProgramBuilder,
        cursor_infos: &mut Vec<CursorInfo>,
    ) -> Result<()> {
        // Emit OpenCursor + NextRow for each implicit scan (global DB refs).
        // Collect NextRow instruction indices so we can patch eof_target to Halt.
        let mut next_row_indices = Vec::new();
        for &scan_id in result.scan_metadata.keys() {
            let cursor_id = cursor_infos.len() as u16;
            cursor_infos.push(CursorInfo {
                scan_id,
                slot_base: self.scan_slot_bases.get(&scan_id).copied().unwrap_or(0),
                width: 1,
                spool: None,
            });
            builder.emit_open_cursor(cursor_id);
            let next_row_idx = builder.emit_next_row(cursor_id);
            next_row_indices.push(next_row_idx);
        }

        let resolver = &result.resolver;
        let expr_compiler = LogicalExprCompiler::new(resolver);
        let program = expr_compiler.compile_to_program(&eq.expr, 0, result.slot_count as u16)?;
        self.inline_program(&program, builder)?;
        builder.emit_emit_row();
        builder.emit_halt();

        // Patch all NextRow eof_targets to point to the Halt instruction
        let halt_offset = builder.current_offset() - 1;
        for idx in next_row_indices {
            builder.patch_target(idx, halt_offset);
        }

        Ok(())
    }

    /// Emit project (named columns) inline at the given slot offset.
    fn emit_project_at(
        &self,
        project: &Project,
        result: &SubtreeResult,
        output_start: usize,
        builder: &mut ProgramBuilder,
    ) -> Result<()> {
        let expr_compiler = LogicalExprCompiler::new(&result.resolver);
        let num_outputs = project.exprs.len();

        let mut exprs = Vec::with_capacity(num_outputs);
        for (idx, (_name, expr)) in project.exprs.iter().enumerate() {
            let target_slot = (output_start + idx) as SlotId;
            exprs.push((target_slot, expr.clone()));
        }

        // Scratch registers start above both the outputs and every register already
        // in use (e.g. the GROUP BY phase's record and key registers).
        let temp_base = ((output_start + num_outputs) as u16).max(builder.next_reg());
        let program = expr_compiler.compile_to_program_multi(&exprs, temp_base)?;
        self.inline_program(&program, builder)?;
        Ok(())
    }

    /// Emit ProjectValue inline at the given slot offset.
    fn emit_project_value_at(
        &self,
        pv: &ProjectValue,
        result: &SubtreeResult,
        output_slot: usize,
        builder: &mut ProgramBuilder,
    ) -> Result<()> {
        let expr_compiler = LogicalExprCompiler::new(&result.resolver);
        let temp_base = ((output_slot + 1) as u16).max(builder.next_reg());
        let program =
            expr_compiler.compile_to_program(&pv.expr, output_slot as SlotId, temp_base)?;
        self.inline_program(&program, builder)?;
        Ok(())
    }

    /// Emit ProjectAll (SELECT *) inline at the given slot offset.
    fn emit_project_all_at(
        &self,
        result: &SubtreeResult,
        output_slot: usize,
        builder: &mut ProgramBuilder,
    ) -> Result<()> {
        let alias = match &result.resolver {
            ResolverKind::Pipeline(r) => r.scan_alias.clone(),
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
                builder.insts.push(Inst::ConcatTuples {
                    dst: output_slot as SlotId,
                    srcs,
                });
                return Ok(());
            }
            ResolverKind::Global(_) | ResolverKind::Empty => {
                return Err(EngineError::InvalidPlan(
                    "SELECT * on expression query".to_string(),
                ))
            }
        };

        let output_slot = output_slot as SlotId;
        let copy_expr = ValueExpr::VarRef(
            BindingsName::CaseInsensitive(alias.into()),
            VarRefType::Local,
        );
        let expr_compiler = LogicalExprCompiler::new(&result.resolver);
        let temp_base = (output_slot + 1).max(builder.next_reg());
        let program = expr_compiler.compile_to_program(&copy_expr, output_slot, temp_base)?;
        self.inline_program(&program, builder)?;

        match self.mode {
            EvaluationMode::Permissive => {
                let const_idx = builder.push_const_pub(ValueOwned::String("_1".to_string()));
                builder.insts.push(Inst::CoerceToTuple {
                    dst: output_slot,
                    src: output_slot,
                    const_idx,
                });
            }
            EvaluationMode::Strict => {
                builder.insts.push(Inst::AssertTuple { src: output_slot });
            }
        }
        Ok(())
    }

    /// Inline a compiled scalar program's instructions into the builder.
    ///
    /// This copies all instructions from a sub-program into the main program builder.
    /// Constants and keys are merged, and register/const/key indices are remapped.
    fn inline_program(
        &self,
        sub_program: &crate::expr::Program,
        builder: &mut ProgramBuilder,
    ) -> Result<()> {
        // For now, simply append instructions directly.
        // This works because the sub-program was compiled with the same slot_count
        // and registers start from slot_count, which matches the main builder.
        //
        // TODO: If sub-programs use different const/key pools, we'd need to remap.
        // Currently LogicalExprCompiler uses its own ProgramBuilder, so we need to
        // transfer constants and keys.

        // Remap const indices
        let const_offset = builder.consts_len() as u16;

        // Copy constants
        for c in sub_program.consts.iter() {
            builder.push_const_pub(c.clone());
        }

        // Copy keys. Keys are interned (deduplicated), so a key already in the
        // pool keeps its existing index; remap per key rather than by offset.
        let key_map: Vec<u16> = sub_program
            .keys
            .iter()
            .map(|k| builder.push_key_pub(k.clone()))
            .collect();

        // Copy instructions with remapped indices
        for inst in &sub_program.insts {
            let remapped = remap_inst(inst, const_offset, &key_map)?;
            builder.insts.push(remapped);
        }

        // Update next_reg high-water mark
        builder.update_next_reg(sub_program.reg_count);
        Ok(())
    }

    /// Find the ScanId for a given scan in the result's metadata.
    fn find_scan_id_for(&self, scan: &Scan, result: &SubtreeResult) -> Result<ScanId> {
        // The scan metadata has all scan IDs. For a single-scan pipeline,
        // there's typically only one. Match by checking all of them.
        // In a multi-scan scenario we'd match by table/alias, but for now
        // a single-scan plan just has one entry.
        if result.scan_metadata.len() == 1 {
            return Ok(*result.scan_metadata.keys().next().unwrap());
        }

        // Multiple scans — match by object_id / table name
        let _table_name = extract_table_name(&scan.expr);
        if let Some((scan_id, _meta)) = result.scan_metadata.iter().next() {
            return Ok(*scan_id);
        }

        Err(EngineError::InvalidPlan(
            "could not find scan_id for scan".to_string(),
        ))
    }

    // -----------------------------------------------------------------------
    // Phase 1: Metadata gathering (same as before, but simplified)
    // -----------------------------------------------------------------------

    /// Recursively compile a single node to gather metadata (slots, shapes, scan layout).
    fn compile_node(
        &mut self,
        graph: &PlanGraph<'_>,
        id: OpId,
        ctx: &mut CompileContext,
    ) -> Result<SubtreeResult> {
        let op = graph.operator(id)?;
        match op {
            BindingsOp::Sink => {
                let input_id = graph.single_input(id)?;
                self.compile_node(graph, input_id, ctx)
            }
            BindingsOp::Scan(scan) => {
                let result = self.compile_scan(scan, ctx)?;
                let scan_id = *result.scan_metadata.keys().next().ok_or_else(|| {
                    EngineError::InvalidPlan("scan produced no metadata".to_string())
                })?;
                self.scan_ids.insert(id, scan_id);
                self.scan_slot_bases.insert(scan_id, 0);
                self.scan_widths.insert(scan_id, result.slot_count as u16);
                if matches!(&result.resolver, ResolverKind::Pipeline(r) if r.base_row_slot.is_some())
                {
                    self.whole_row_scans.insert(scan_id);
                }
                self.from_slots.insert(id, result.slot_count);
                Ok(result)
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
                let left = self.compile_node(graph, left_id, ctx)?;
                let right = self.compile_node(graph, right_id, ctx)?;

                // The right side's registers follow the left side's.
                let offset = left.slot_count as SlotId;
                for scan_id in right.scan_metadata.keys() {
                    if let Some(base) = self.scan_slot_bases.get_mut(scan_id) {
                        *base += offset;
                    }
                }
                let mut bindings = left.resolver.into_bindings(0)?;
                bindings.extend(right.resolver.into_bindings(offset)?);
                let mut scan_metadata = left.scan_metadata;
                scan_metadata.extend(right.scan_metadata);
                let slot_count = left.slot_count + right.slot_count;
                self.from_slots.insert(id, slot_count);
                Ok(SubtreeResult {
                    resolver: ResolverKind::Multi(bindings),
                    cursor_id: None,
                    scan_metadata,
                    slot_count,
                    shape: None,
                })
            }
            BindingsOp::Filter(filter) => {
                let input_id = graph.single_input(id)?;
                let filter_expr = filter.expr.clone();
                let mut extractor = ExprFieldExtractor::new(ctx);
                extractor.extract(&filter_expr);
                let mut result = self.compile_node(graph, input_id, ctx)?;
                // Reserve a slot for the predicate result
                result.slot_count += 1;
                Ok(result)
            }
            BindingsOp::Project(project) => {
                let input_id = graph.single_input(id)?;
                let project = project.clone();
                {
                    let mut extractor = ExprFieldExtractor::new(ctx);
                    for (_name, expr) in &project.exprs {
                        extractor.extract(expr);
                    }
                }
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project_metadata(&mut result, &project)?;
                Ok(result)
            }
            BindingsOp::ProjectValue(pv) => {
                let input_id = graph.single_input(id)?;
                let pv = pv.clone();
                {
                    let mut extractor = ExprFieldExtractor::new(ctx);
                    extractor.extract(&pv.expr);
                }
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project_value_metadata(&mut result, &pv)?;
                Ok(result)
            }
            BindingsOp::ProjectAll(mode) => {
                let input_id = graph.single_input(id)?;
                let mode = mode.clone();
                ctx.request_all_whole_values();
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project_all_metadata(&mut result, &mode)?;
                Ok(result)
            }
            BindingsOp::LimitOffset(lo) => {
                let input_id = graph.single_input(id)?;
                let _lo = lo.clone();
                self.compile_node(graph, input_id, ctx)
            }
            BindingsOp::GroupBy(group_by) => {
                let input_id = graph.single_input(id)?;
                let group_by = group_by.clone();
                // Extract field references from group key exprs and aggregate input exprs
                {
                    let mut extractor = ExprFieldExtractor::new(ctx);
                    for expr in group_by.exprs.values() {
                        extractor.extract(expr);
                    }
                    for agg in &group_by.aggregate_exprs {
                        extractor.extract(&agg.expr);
                    }
                }
                let mut result = self.compile_node(graph, input_id, ctx)?;
                // GROUP BY creates a new output scope. Reset slot_count to 0 so
                // operators above (Project, Having) allocate output from the start.
                result.slot_count = 0;
                Ok(result)
            }
            BindingsOp::Having(having) => {
                let input_id = graph.single_input(id)?;
                let having_expr = having.expr.clone();
                let mut extractor = ExprFieldExtractor::new(ctx);
                extractor.extract(&having_expr);
                let mut result = self.compile_node(graph, input_id, ctx)?;
                result.slot_count += 1;
                Ok(result)
            }
            BindingsOp::ExprQuery(eq) => self.compile_expr_query(eq),
            other => Err(EngineError::InvalidPlan(format!(
                "unsupported operator: {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    /// Compile a Scan node — gather metadata only.
    fn compile_scan(&mut self, scan: &Scan, ctx: &CompileContext) -> Result<SubtreeResult> {
        // Try inline scan for literal expressions first (compile-time constants)
        if let Some(values) = try_expr_to_inline_values(&scan.expr) {
            if self.mode == EvaluationMode::Strict && !is_collection_expr(&scan.expr) {
                return Err(EngineError::StrictModeViolation(
                    "FROM clause requires collection value".to_string(),
                ));
            }
            return self.compile_inline_scan(scan, values);
        }
        // Try table function resolution (Call expressions)
        if let Some(result) = self.try_compile_table_fn_scan(scan, ctx)? {
            return Ok(result);
        }
        // Try catalog resolution (DBRef / VarRef)
        let catalog_result = self.resolve_reader_factory(scan);
        if catalog_result.is_err() {
            // Expression-based scan: will be compiled and materialized at runtime
            return self.compile_expr_scan(scan);
        }
        let (catalog_id, reader_factory) = catalog_result.unwrap();
        let scan_id = self.alloc_scan_id();
        let table_name = extract_table_name(&scan.expr);

        let my_requests = ctx.requests_for_alias(&scan.as_key, table_name.as_deref());

        let mut projections = Vec::new();
        let mut column_slots = FxHashMap::default();
        let mut base_row_slot: Option<SlotId> = None;
        let mut next_slot: SlotId = 0;
        let mut needs_whole_value =
            my_requests.is_empty() || ctx.needs_whole_value(&scan.as_key, table_name.as_deref());

        if !needs_whole_value {
            for req in &my_requests {
                if let Some(scan_source) = reader_factory.resolve(&req.field_name) {
                    let slot = next_slot;
                    next_slot += 1;
                    projections.push(ScanProjection {
                        source: scan_source,
                        target_slot: slot,
                    });
                    column_slots.insert(req.field_name.clone(), slot);
                } else {
                    needs_whole_value = true;
                    break;
                }
            }
        }

        if needs_whole_value {
            projections.clear();
            column_slots.clear();
            let slot = 0;
            base_row_slot = Some(slot);
            projections.push(ScanProjection {
                source: ScanSource::whole_value(),
                target_slot: slot,
            });
            next_slot = 1;
        }

        let layout = ScanLayout { projections };
        let object_id = ObjectId::new(catalog_id, reader_factory.entry_id);
        let mut scan_metadata = HashMap::new();
        scan_metadata.insert(scan_id, ScanMetadata { layout, object_id });

        let resolver = PipelineSlotResolver {
            base_row_slot,
            scan_alias: scan.as_key.clone(),
            table_name,
            column_slots,
        };

        Ok(SubtreeResult {
            resolver: ResolverKind::Pipeline(resolver),
            cursor_id: None,
            scan_metadata,
            slot_count: next_slot as usize,
            shape: None,
        })
    }

    /// Compile an ExprQuery (expression-only query without a scan).
    ///
    /// If the expression references catalog objects (DBRef), we create implicit
    /// scans so their values are loaded into registers before the expression runs.
    fn compile_expr_query(
        &mut self,
        expr_query: &partiql_logical::ExprQuery,
    ) -> Result<SubtreeResult> {
        let db_refs = collect_db_refs(&expr_query.expr);

        if db_refs.is_empty() {
            return Ok(SubtreeResult {
                resolver: ResolverKind::Empty,
                cursor_id: None,
                scan_metadata: HashMap::new(),
                slot_count: 1,
                shape: Some(Shape::Single(RowShape::Register(0, PhysicalType::Dynamic))),
            });
        }

        let mut slots = FxHashMap::default();
        let mut scan_metadata = HashMap::new();
        let mut next_slot: SlotId = 0;

        for db_ref in &db_refs {
            let name = match db_ref.path.first() {
                Some(BindingsName::CaseSensitive(s)) => s.as_ref().to_string(),
                Some(BindingsName::CaseInsensitive(s)) => s.as_ref().to_string(),
                Some(_) | None => continue,
            };

            if slots.contains_key(&name) {
                continue;
            }

            let (catalog_id, reader_factory) = match self.resolve_catalog_table(db_ref) {
                Ok(result) => result,
                Err(_) => continue,
            };

            let scan_id = self.alloc_scan_id();
            let slot = next_slot;
            next_slot += 1;

            // Each implicit scan writes its window's first register, `slot`.
            let layout = ScanLayout {
                projections: vec![ScanProjection {
                    source: ScanSource::whole_value(),
                    target_slot: 0,
                }],
            };
            let object_id = ObjectId::new(catalog_id, reader_factory.entry_id);
            scan_metadata.insert(scan_id, ScanMetadata { layout, object_id });
            self.scan_slot_bases.insert(scan_id, slot);
            slots.insert(name, slot);
        }

        let resolver = if slots.is_empty() {
            ResolverKind::Empty
        } else {
            ResolverKind::Global(GlobalSlotResolver { slots })
        };

        Ok(SubtreeResult {
            resolver,
            cursor_id: None,
            scan_metadata,
            slot_count: std::cmp::max(next_slot as usize, 1),
            shape: Some(Shape::Single(RowShape::Register(0, PhysicalType::Dynamic))),
        })
    }

    fn apply_project_metadata(&self, result: &mut SubtreeResult, project: &Project) -> Result<()> {
        let output_start = result.slot_count;
        let num_outputs = project.exprs.len();
        result.slot_count += num_outputs;

        let mut fields = Vec::with_capacity(num_outputs);
        for (idx, (name, _expr)) in project.exprs.iter().enumerate() {
            fields.push(FieldShape {
                name: FieldName::Static(name.clone()),
                value: RowShape::Register(output_start + idx, PhysicalType::Dynamic),
            });
        }
        result.shape = Some(Shape::Bag(RowShape::Struct(fields)));
        Ok(())
    }

    fn apply_project_value_metadata(
        &self,
        result: &mut SubtreeResult,
        _pv: &ProjectValue,
    ) -> Result<()> {
        let output_slot = result.slot_count as SlotId;
        result.slot_count += 1;
        result.shape = Some(Shape::Bag(RowShape::Register(
            output_slot as usize,
            PhysicalType::Dynamic,
        )));
        Ok(())
    }

    fn apply_project_all_metadata(
        &self,
        result: &mut SubtreeResult,
        _mode: &ProjectAllMode,
    ) -> Result<()> {
        let output_slot = result.slot_count as SlotId;
        result.slot_count += 1;
        result.shape = Some(Shape::Bag(RowShape::Register(
            output_slot as usize,
            PhysicalType::Dynamic,
        )));
        Ok(())
    }

    /// Compile an inline scan (literal collection in FROM clause).
    fn compile_inline_scan(
        &mut self,
        scan: &Scan,
        values: Vec<ValueOwned>,
    ) -> Result<SubtreeResult> {
        let scan_id = self.alloc_scan_id();
        self.inline_scans.insert(scan_id, values);

        let layout = ScanLayout {
            projections: vec![ScanProjection {
                source: ScanSource::whole_value(),
                target_slot: 0,
            }],
        };

        let resolver = PipelineSlotResolver {
            base_row_slot: Some(0),
            scan_alias: scan.as_key.clone(),
            table_name: None,
            column_slots: FxHashMap::default(),
        };

        // Store scan metadata so find_scan_id_for can locate this scan.
        // Use a dummy ObjectId since inline scans bypass catalog lookup.
        let dummy_object_id = ObjectId::from((
            partiql_common::catalog::CatalogId::from(u64::MAX),
            partiql_common::catalog::EntryId::from(u64::MAX),
        ));
        let mut scan_metadata = HashMap::new();
        scan_metadata.insert(
            scan_id,
            ScanMetadata {
                layout,
                object_id: dummy_object_id,
            },
        );

        Ok(SubtreeResult {
            resolver: ResolverKind::Pipeline(resolver),
            cursor_id: None,
            scan_metadata,
            slot_count: 1,
            shape: None,
        })
    }

    /// Compile an expression-based scan (runtime-evaluated collection in FROM).
    /// The expression will be compiled and materialized at emit time.
    fn compile_expr_scan(&mut self, scan: &Scan) -> Result<SubtreeResult> {
        let scan_id = self.alloc_scan_id();
        self.expr_scans.insert(scan_id, scan.expr.clone());

        let resolver = PipelineSlotResolver {
            base_row_slot: Some(0),
            scan_alias: scan.as_key.clone(),
            table_name: None,
            column_slots: FxHashMap::default(),
        };

        let dummy_object_id = ObjectId::from((
            partiql_common::catalog::CatalogId::from(u64::MAX),
            partiql_common::catalog::EntryId::from(u64::MAX),
        ));
        let mut scan_metadata = HashMap::new();
        scan_metadata.insert(
            scan_id,
            ScanMetadata {
                layout: ScanLayout {
                    projections: vec![ScanProjection {
                        source: ScanSource::whole_value(),
                        target_slot: 0,
                    }],
                },
                object_id: dummy_object_id,
            },
        );

        Ok(SubtreeResult {
            resolver: ResolverKind::Pipeline(resolver),
            cursor_id: None,
            scan_metadata,
            slot_count: 1,
            shape: None,
        })
    }

    // -----------------------------------------------------------------------
    // Table function compilation
    // -----------------------------------------------------------------------

    /// Try to compile a scan as a table function call.
    /// Returns `Ok(Some(result))` if the scan expr is a Call to a known table function,
    /// `Ok(None)` if it's not a Call or the function isn't registered.
    fn try_compile_table_fn_scan(
        &mut self,
        scan: &Scan,
        ctx: &CompileContext,
    ) -> Result<Option<SubtreeResult>> {
        let call_expr = match &scan.expr {
            ValueExpr::Call(call) => call,
            _ => return Ok(None),
        };

        let func_name = match &call_expr.name {
            CallName::ByName(name) => name.clone(),
            _ => return Ok(None),
        };

        let handle = match self.resolve_table_function(&func_name) {
            Some(h) => h,
            None => return Ok(None),
        };

        let scan_id = self.alloc_scan_id();

        // Determine output layout from field requests
        let table_name = Some(func_name.clone());
        let my_requests = ctx.requests_for_alias(&scan.as_key, table_name.as_deref());

        let mut projections = Vec::new();
        let mut column_slots = FxHashMap::default();
        let mut base_row_slot: Option<SlotId> = None;
        let mut next_slot: SlotId = 0;
        let mut needs_whole_value =
            my_requests.is_empty() || ctx.needs_whole_value(&scan.as_key, table_name.as_deref());

        if !needs_whole_value {
            for req in &my_requests {
                if let Some(scan_source) = handle.resolve(&req.field_name) {
                    let slot = next_slot;
                    next_slot += 1;
                    projections.push(ScanProjection {
                        source: scan_source,
                        target_slot: slot,
                    });
                    column_slots.insert(req.field_name.clone(), slot);
                } else {
                    needs_whole_value = true;
                    break;
                }
            }
        }

        if needs_whole_value {
            projections.clear();
            column_slots.clear();
            base_row_slot = Some(0);
            projections.push(ScanProjection {
                source: ScanSource::whole_value(),
                target_slot: 0,
            });
            next_slot = 1;
        }

        let layout = ScanLayout { projections };

        // Store table function scan info for bytecode emission
        self.table_fn_scans.insert(
            scan_id,
            TableFnScanInfo {
                func_name,
                layout: layout.clone(),
                arguments: call_expr.arguments.clone(),
            },
        );

        let resolver = PipelineSlotResolver {
            base_row_slot,
            scan_alias: scan.as_key.clone(),
            table_name,
            column_slots,
        };

        // Store scan metadata so find_scan_id_for can locate this scan.
        let dummy_object_id = ObjectId::from((
            partiql_common::catalog::CatalogId::from(u64::MAX),
            partiql_common::catalog::EntryId::from(u64::MAX),
        ));
        let mut scan_metadata = HashMap::new();
        scan_metadata.insert(
            scan_id,
            ScanMetadata {
                layout,
                object_id: dummy_object_id,
            },
        );

        Ok(Some(SubtreeResult {
            resolver: ResolverKind::Pipeline(resolver),
            cursor_id: None,
            scan_metadata,
            slot_count: next_slot as usize,
            shape: None,
        }))
    }

    /// Search all catalogs for a table function by name.
    fn resolve_table_function(&self, name: &str) -> Option<TableFunctionHandle> {
        // Only the "default" catalog is searched for now.
        let (_id, catalog) = self.compilation_context.get_catalog("default")?;
        catalog.get_table_function(name)
    }

    // -----------------------------------------------------------------------
    // Catalog / table resolution
    // -----------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The `Scan` operators of a FROM subtree, in FROM order.
fn from_scans(graph: &PlanGraph<'_>, id: OpId) -> Result<Vec<OpId>> {
    match graph.operator(id)? {
        BindingsOp::Scan(_) => Ok(vec![id]),
        BindingsOp::Join(_) => {
            let (left, right) = graph.join_inputs(id)?;
            let mut scans = from_scans(graph, left)?;
            scans.extend(from_scans(graph, right)?);
            Ok(scans)
        }
        _ => Ok(vec![]),
    }
}

fn count_scans(graph: &PlanGraph<'_>, id: OpId) -> Result<usize> {
    Ok(from_scans(graph, id)?.len())
}

/// The names a FROM subtree binds (`AS` and `AT` aliases).
fn from_aliases(graph: &PlanGraph<'_>, id: OpId) -> Result<Vec<String>> {
    let mut names = vec![];
    for scan in from_scans(graph, id)? {
        if let BindingsOp::Scan(scan) = graph.operator(scan)? {
            names.push(scan.as_key.clone());
            names.extend(scan.at_key.clone());
        }
    }
    Ok(names)
}

/// Whether a FROM subtree's source expressions may read any of `names`.
fn reads_any(graph: &PlanGraph<'_>, id: OpId, names: &[String]) -> Result<bool> {
    Ok(from_source_exprs(graph, id)?
        .into_iter()
        .any(|expr| expr_reads_any(expr, names)))
}

/// Whether `expr` may reference a variable in `names`. Conservative: an expression kind
/// it does not look into counts as a reference.
fn expr_reads_any(expr: &ValueExpr, names: &[String]) -> bool {
    let any = |exprs: &mut dyn Iterator<Item = &ValueExpr>| {
        for e in exprs {
            if expr_reads_any(e, names) {
                return true;
            }
        }
        false
    };
    match expr {
        ValueExpr::VarRef(name, _) => names.iter().any(|n| bindings_name_matches(name, n)),
        ValueExpr::Lit(_) | ValueExpr::DBRef(_) => false,
        ValueExpr::UnExpr(_, e) => expr_reads_any(e, names),
        ValueExpr::BinaryExpr(_, l, r) => expr_reads_any(l, names) || expr_reads_any(r, names),
        ValueExpr::DynamicLookup(lookups) => any(&mut lookups.iter()),
        ValueExpr::Path(base, steps) => {
            expr_reads_any(base, names)
                || steps.iter().any(|step| match step {
                    PathComponent::KeyExpr(e) | PathComponent::IndexExpr(e) => {
                        expr_reads_any(e, names)
                    }
                    PathComponent::Key(_) | PathComponent::Index(_) => false,
                    _ => true,
                })
        }
        ValueExpr::TupleExpr(t) => any(&mut t.attrs.iter().chain(&t.values)),
        ValueExpr::ListExpr(l) => any(&mut l.elements.iter()),
        ValueExpr::BagExpr(b) => any(&mut b.elements.iter()),
        ValueExpr::Call(call) => any(&mut call.arguments.iter()),
        _ => true,
    }
}

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

/// Try to convert a single ValueExpr to a ValueOwned (for literal expressions).
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

/// Remap constant and key indices in an instruction of an inlined sub-program.
///
/// The match is exhaustive on purpose: a new instruction must say whether it indexes
/// the constant or key pool, since an unmapped index silently reads a different entry
/// after inlining.
fn remap_inst(inst: &Inst, const_offset: u16, key_map: &[u16]) -> Result<Inst> {
    let key = |idx: u16| -> Result<u16> {
        key_map.get(idx as usize).copied().ok_or_else(|| {
            EngineError::IllegalState(format!(
                "inlined sub-program references key index {idx} outside its key pool (len {})",
                key_map.len()
            ))
        })
    };
    Ok(match inst {
        Inst::LoadConst { dst, const_idx } => Inst::LoadConst {
            dst: *dst,
            const_idx: *const_idx + const_offset,
        },
        Inst::GetField { dst, base, key_idx } => Inst::GetField {
            dst: *dst,
            base: *base,
            key_idx: key(*key_idx)?,
        },
        Inst::CallUdf {
            dst,
            func_idx,
            args,
        } => Inst::CallUdf {
            dst: *dst,
            func_idx: key(*func_idx)?,
            args: args.clone(),
        },
        Inst::LikeMatch {
            dst,
            value,
            pattern_idx,
        } => Inst::LikeMatch {
            dst: *dst,
            value: *value,
            pattern_idx: key(*pattern_idx)?,
        },
        Inst::CreateTableFnCursor {
            cursor_id,
            func_name_idx,
        } => Inst::CreateTableFnCursor {
            cursor_id: *cursor_id,
            func_name_idx: key(*func_name_idx)?,
        },
        Inst::CoerceToTuple {
            dst,
            src,
            const_idx,
        } => Inst::CoerceToTuple {
            dst: *dst,
            src: *src,
            const_idx: *const_idx + const_offset,
        },
        Inst::ConcatTuples { dst, srcs } => Inst::ConcatTuples {
            dst: *dst,
            srcs: srcs.clone(),
        },

        // Absolute jump targets would need relocating; scalar sub-programs never emit them.
        Inst::NextRow { .. }
        | Inst::Jump { .. }
        | Inst::JumpIfNotTrue { .. }
        | Inst::DecrOrJump { .. }
        | Inst::SorterSort { .. }
        | Inst::SorterNext { .. }
        | Inst::Gosub { .. } => {
            return Err(EngineError::IllegalState(format!(
                "cannot inline a sub-program with a jump: {inst:?}"
            )))
        }

        // These reference only registers, slots, cursors, sorters or inline operands.
        Inst::Cast { .. }
        | Inst::AddI64 { .. }
        | Inst::AddF64 { .. }
        | Inst::AddDecimal { .. }
        | Inst::AddDynamic { .. }
        | Inst::SubI64 { .. }
        | Inst::SubF64 { .. }
        | Inst::SubDecimal { .. }
        | Inst::SubDynamic { .. }
        | Inst::ModI64 { .. }
        | Inst::ModF64 { .. }
        | Inst::ModDecimal { .. }
        | Inst::ModDynamic { .. }
        | Inst::EqI64 { .. }
        | Inst::EqF64 { .. }
        | Inst::EqDecimal { .. }
        | Inst::EqBool { .. }
        | Inst::EqStr { .. }
        | Inst::EqDynamic { .. }
        | Inst::MulI64 { .. }
        | Inst::MulF64 { .. }
        | Inst::MulDecimal { .. }
        | Inst::MulDynamic { .. }
        | Inst::DivI64 { .. }
        | Inst::DivF64 { .. }
        | Inst::DivDecimal { .. }
        | Inst::DivDynamic { .. }
        | Inst::ExpF64 { .. }
        | Inst::ExpDynamic { .. }
        | Inst::GtI64 { .. }
        | Inst::GtF64 { .. }
        | Inst::GtDecimal { .. }
        | Inst::GtStr { .. }
        | Inst::GtDynamic { .. }
        | Inst::LtI64 { .. }
        | Inst::LtF64 { .. }
        | Inst::LtDecimal { .. }
        | Inst::LtStr { .. }
        | Inst::LtDynamic { .. }
        | Inst::ConcatStr { .. }
        | Inst::ConcatDynamic { .. }
        | Inst::InDynamic { .. }
        | Inst::AndBool { .. }
        | Inst::OrBool { .. }
        | Inst::NotBool { .. }
        | Inst::NegNum { .. }
        | Inst::LikeDynamicMatch { .. }
        | Inst::MaterializeCursor { .. }
        | Inst::StoreSlot { .. }
        | Inst::MakeTuple { .. }
        | Inst::MakeList { .. }
        | Inst::MakeBag { .. }
        | Inst::AssertTuple { .. }
        | Inst::AssertCollection { .. }
        | Inst::OpenCursor { .. }
        | Inst::CloseCursor { .. }
        | Inst::EmitRow
        | Inst::Halt
        | Inst::SorterInsert { .. }
        | Inst::SorterData { .. }
        | Inst::AggStep { .. }
        | Inst::AggFinal { .. }
        | Inst::AggReset { .. }
        | Inst::Return { .. }
        | Inst::Copy { .. }
        | Inst::CompareEq { .. } => inst.clone(),
    })
}

// ---------------------------------------------------------------------------
// Slot resolvers
// ---------------------------------------------------------------------------

/// Resolves variable references for a single-scan pipeline.
struct PipelineSlotResolver {
    base_row_slot: Option<SlotId>,
    scan_alias: String,
    table_name: Option<String>,
    column_slots: FxHashMap<String, SlotId>,
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

/// Empty slot resolver for ExprQuery expressions that don't reference variables.
#[allow(dead_code)]
struct EmptySlotResolver;

impl SlotResolver for EmptySlotResolver {
    fn resolve_var(&self, _name: &BindingsName<'_>, _scope: VarRefType) -> Option<SlotId> {
        None
    }
    fn resolve_alias(&self, _name: &BindingsName<'_>) -> Option<SlotId> {
        None
    }
    fn resolve_field(&self, _name: &BindingsName<'_>) -> Option<SlotId> {
        None
    }
    fn is_alias(&self, _name: &BindingsName<'_>) -> bool {
        false
    }
    fn resolve_qualified(
        &self,
        _alias: &BindingsName<'_>,
        _field: &BindingsName<'_>,
    ) -> Option<SlotId> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remap_inst_maps_pool_indices() {
        let key_map = [7, 3];
        let inst = Inst::GetField {
            dst: 1,
            base: 0,
            key_idx: 1,
        };
        assert!(matches!(
            remap_inst(&inst, 5, &key_map),
            Ok(Inst::GetField { key_idx: 3, .. })
        ));
        let inst = Inst::LoadConst {
            dst: 1,
            const_idx: 2,
        };
        assert!(matches!(
            remap_inst(&inst, 5, &key_map),
            Ok(Inst::LoadConst { const_idx: 7, .. })
        ));
        // A key index outside the sub-program's pool is an error, not a panic.
        let inst = Inst::GetField {
            dst: 1,
            base: 0,
            key_idx: 2,
        };
        assert!(remap_inst(&inst, 5, &key_map).is_err());
    }

    #[test]
    fn remap_inst_rejects_jumps() {
        assert!(remap_inst(&Inst::Jump { target: 4 }, 0, &[]).is_err());
    }
}
