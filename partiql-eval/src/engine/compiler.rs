use crate::engine::arena::SlotId;
use crate::engine::catalog::CompilationContext;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::{Inst, LogicalExprCompiler, ProgramBuilder};
use crate::engine::field_resolver::{CompileContext, ExprFieldExtractor};
use crate::engine::plan::{CompiledPlan, CursorInfo, ObjectId, ScanId, ScanMetadata};
use crate::engine::source::{DataSourceHandle, ScanLayout, ScanProjection, ScanSource};
use crate::engine::value::{FieldName, FieldShape, PhysicalType, RowShape, Shape};
use crate::engine::SlotResolver;
use partiql_logical::{
    BindingsOp, DBRef, LimitOffset, LogicalPlan, OpId, Project, ProjectAllMode, ProjectValue, Scan,
    ValueExpr, VarRefType,
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
    /// Global resolver for expression queries that reference catalog objects
    Global(GlobalSlotResolver),
    /// Empty resolver for expression queries with no variable references
    Empty,
}

impl SlotResolver for ResolverKind {
    fn resolve_var(&self, name: &BindingsName<'_>, scope: VarRefType) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_var(name, scope),
            ResolverKind::Global(r) => r.resolve_var(name, scope),
            ResolverKind::Empty => None,
        }
    }
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_alias(name),
            ResolverKind::Global(r) => r.resolve_alias(name),
            ResolverKind::Empty => None,
        }
    }
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_field(name),
            ResolverKind::Global(r) => r.resolve_field(name),
            ResolverKind::Empty => None,
        }
    }
    fn is_alias(&self, name: &BindingsName<'_>) -> bool {
        match self {
            ResolverKind::Pipeline(r) => r.is_alias(name),
            ResolverKind::Global(_) => false,
            ResolverKind::Empty => false,
        }
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
}

// ---------------------------------------------------------------------------
// PlanCompiler — recursive tree-walk compilation emitting flat bytecode
// ---------------------------------------------------------------------------

pub struct PlanCompiler<'a> {
    compilation_context: &'a CompilationContext,
    next_scan_id: u64,
}

impl<'a> PlanCompiler<'a> {
    pub fn new(compilation_context: &'a CompilationContext) -> Self {
        PlanCompiler {
            compilation_context,
            next_scan_id: 0,
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

        Ok(CompiledPlan {
            program,
            cursors: cursor_infos,
            shape,
            slot_count: result.slot_count,
            scan_metadata: result.scan_metadata,
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
    ) -> Result<(crate::engine::expr::Program, Vec<CursorInfo>)> {
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
                cursor_infos.push(CursorInfo { scan_id });

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
                BindingsOp::Scan(_) | BindingsOp::ExprQuery(_) => break,
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
        // First element must be a Scan
        let scan = match chain.first() {
            Some((_, BindingsOp::Scan(scan))) => scan,
            _ => {
                return Err(EngineError::InvalidPlan(
                    "pipeline must start with Scan".to_string(),
                ))
            }
        };

        let scan_id = self.find_scan_id_for(scan, result)?;
        let cursor_id = cursor_infos.len() as u16;
        cursor_infos.push(CursorInfo { scan_id });

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

        // Emit: OpenCursor
        builder.emit_open_cursor(cursor_id);

        // Emit: NextRow (loop head)
        let loop_head = builder.current_offset() as usize;
        let next_row_idx = builder.emit_next_row(cursor_id);

        // Track slot allocation incrementally (mirroring the analysis pass).
        // The scan occupies slots 0..scan_slots. Each subsequent operator allocates
        // from current_slot onward.
        let scan_slots = result
            .scan_metadata
            .get(&scan_id)
            .map(|m| m.layout.projections.len())
            .unwrap_or(1);
        let mut current_slot = scan_slots;

        for (_, op) in chain.iter().skip(1) {
            match op {
                BindingsOp::Filter(filter) => {
                    let pred_slot = current_slot as SlotId;
                    current_slot += 1;
                    let expr_compiler = LogicalExprCompiler::new(&result.resolver);
                    let filter_program = expr_compiler.compile_to_program(
                        &filter.expr,
                        pred_slot,
                        current_slot as u16,
                    )?;

                    // Inline the filter program's instructions
                    self.inline_program(&filter_program, builder);

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
            let const_idx =
                builder.push_const_pub(crate::engine::value::ValueOwned::I64(limit as i64));
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

        // Patch NextRow's eof_target to point here
        builder.patch_target(next_row_idx, after_loop);

        // Patch DecrOrJump's target to point here (skip to CloseCursor when limit reached)
        if let Some(idx) = decr_jump_idx {
            builder.patch_target(idx, after_loop);
        }

        // Emit: CloseCursor
        builder.emit_close_cursor(cursor_id);

        // Emit: Halt
        builder.emit_halt();

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
            cursor_infos.push(CursorInfo { scan_id });
            builder.emit_open_cursor(cursor_id);
            let next_row_idx = builder.emit_next_row(cursor_id);
            next_row_indices.push(next_row_idx);
        }

        let resolver = &result.resolver;
        let expr_compiler = LogicalExprCompiler::new(resolver);
        let program = expr_compiler.compile_to_program(&eq.expr, 0, result.slot_count as u16)?;
        self.inline_program(&program, builder);
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

        let program =
            expr_compiler.compile_to_program_multi(&exprs, (output_start + num_outputs) as u16)?;
        self.inline_program(&program, builder);
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
        let program = expr_compiler.compile_to_program(
            &pv.expr,
            output_slot as SlotId,
            (output_slot + 1) as u16,
        )?;
        self.inline_program(&program, builder);
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
        let program = expr_compiler.compile_to_program(&copy_expr, output_slot, output_slot + 1)?;
        self.inline_program(&program, builder);
        Ok(())
    }

    /// Inline a compiled scalar program's instructions into the builder.
    ///
    /// This copies all instructions from a sub-program into the main program builder.
    /// Constants and keys are merged, and register/const/key indices are remapped.
    fn inline_program(
        &self,
        sub_program: &crate::engine::expr::Program,
        builder: &mut ProgramBuilder,
    ) {
        // For now, simply append instructions directly.
        // This works because the sub-program was compiled with the same slot_count
        // and registers start from slot_count, which matches the main builder.
        //
        // TODO: If sub-programs use different const/key pools, we'd need to remap.
        // Currently LogicalExprCompiler uses its own ProgramBuilder, so we need to
        // transfer constants and keys.

        // Remap const indices
        let const_offset = builder.consts_len() as u16;
        let key_offset = builder.keys_len() as u16;

        // Copy constants
        for c in sub_program.consts.iter() {
            builder.push_const_pub(c.clone());
        }

        // Copy keys
        for k in sub_program.keys.iter() {
            builder.push_key_pub(k.clone());
        }

        // Copy instructions with remapped indices
        for inst in &sub_program.insts {
            let remapped = remap_inst(inst, const_offset, key_offset);
            builder.insts.push(remapped);
        }

        // Update next_reg high-water mark
        builder.update_next_reg(sub_program.reg_count);
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
            BindingsOp::Scan(scan) => self.compile_scan(scan, ctx),
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
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project_all_metadata(&mut result, &mode)?;
                Ok(result)
            }
            BindingsOp::LimitOffset(lo) => {
                let input_id = graph.single_input(id)?;
                let _lo = lo.clone();
                self.compile_node(graph, input_id, ctx)
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
        let (catalog_id, reader_factory) = self.resolve_reader_factory(scan)?;
        let scan_id = self.alloc_scan_id();
        let table_name = extract_table_name(&scan.expr);

        let my_requests = ctx.requests_for_alias(&scan.as_key, table_name.as_deref());

        let mut projections = Vec::new();
        let mut column_slots = FxHashMap::default();
        let mut base_row_slot: Option<SlotId> = None;
        let mut next_slot: SlotId = 0;
        let mut needs_whole_value = my_requests.is_empty();

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
                None => continue,
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

            let layout = ScanLayout {
                projections: vec![ScanProjection {
                    source: ScanSource::whole_value(),
                    target_slot: slot,
                }],
            };
            let object_id = ObjectId::new(catalog_id, reader_factory.entry_id);
            scan_metadata.insert(scan_id, ScanMetadata { layout, object_id });
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

fn extract_table_name(expr: &ValueExpr) -> Option<String> {
    match expr {
        ValueExpr::VarRef(name, _) => Some(match name {
            BindingsName::CaseSensitive(s) => s.as_ref().to_string(),
            BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
        }),
        ValueExpr::DBRef(db_ref) => db_ref.path.last().map(|name| match name {
            BindingsName::CaseSensitive(s) => s.as_ref().to_string(),
            BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
        }),
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

fn bindings_name_matches(name: &BindingsName<'_>, target: &str) -> bool {
    match name {
        BindingsName::CaseSensitive(s) => s.as_ref() == target,
        BindingsName::CaseInsensitive(s) => s.as_ref().eq_ignore_ascii_case(target),
    }
}

/// Remap constant and key indices in an instruction.
fn remap_inst(inst: &Inst, const_offset: u16, key_offset: u16) -> Inst {
    match inst {
        Inst::LoadConst { dst, const_idx } => Inst::LoadConst {
            dst: *dst,
            const_idx: *const_idx + const_offset,
        },
        Inst::GetField { dst, base, key_idx } => Inst::GetField {
            dst: *dst,
            base: *base,
            key_idx: *key_idx + key_offset,
        },
        Inst::CallUdf {
            dst,
            func_idx,
            args,
        } => Inst::CallUdf {
            dst: *dst,
            func_idx: *func_idx + key_offset,
            args: args.clone(),
        },
        // All other instructions don't reference const/key pools
        other => other.clone(),
    }
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
}
