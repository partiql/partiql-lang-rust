use crate::engine::arena::SlotId;
use crate::engine::catalog::CompilationContext;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::LogicalExprCompiler;
use crate::engine::field_resolver::{CompileContext, ExprFieldExtractor};
use crate::engine::plan::{
    CompiledPlan, ExprQuerySpec, NestedLoopJoinSpec, ObjectId, PipelineSpec, RelOpSpec, ScanId,
    ScanMetadata, StepSpec,
};
use crate::engine::source::{DataSourceHandle, ScanLayout, ScanProjection, ScanSource};
use crate::engine::value::{FieldName, FieldShape, PhysicalType, RowShape, Shape};
use crate::engine::SlotResolver;
use partiql_logical::{
    BindingsOp, DBRef, Join, LimitOffset, LogicalPlan, OpId, Project, ProjectAllMode, ProjectValue,
    Scan, ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Plan graph navigation helpers
// ---------------------------------------------------------------------------

/// Pre-built index for navigating the logical plan graph.
///
/// The logical plan stores edges as `(src, dst, branch)` triples.
/// This struct inverts that into per-node input lists for O(1) lookup.
struct PlanGraph<'p> {
    plan: &'p LogicalPlan<BindingsOp>,
    /// For each node, its list of (source_node, branch_number) inputs, sorted by branch.
    inputs: FxHashMap<OpId, Vec<(OpId, u8)>>,
    /// For each node, its single output (destination) node. Only branch-0 outgoing tracked.
    #[allow(dead_code)]
    output: FxHashMap<OpId, OpId>,
}

impl<'p> PlanGraph<'p> {
    fn new(plan: &'p LogicalPlan<BindingsOp>) -> Self {
        let mut inputs: FxHashMap<OpId, Vec<(OpId, u8)>> = FxHashMap::default();
        let mut output: FxHashMap<OpId, OpId> = FxHashMap::default();
        for &(src, dst, branch) in plan.flows() {
            inputs.entry(dst).or_default().push((src, branch));
            // Track outgoing from src — for nodes with a single output chain
            output.entry(src).or_insert(dst);
        }
        // Sort inputs by branch number for deterministic ordering
        for v in inputs.values_mut() {
            v.sort_by_key(|(_, b)| *b);
        }
        PlanGraph {
            plan,
            inputs,
            output,
        }
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

    /// Get the two inputs to a join node: (branch 0 = left, branch 1 = right).
    #[allow(dead_code)]
    fn join_inputs(&self, id: OpId) -> Result<(OpId, OpId)> {
        let ins = self.inputs.get(&id);
        match ins.map(|v| v.as_slice()) {
            Some(slice) if slice.len() == 2 => {
                let left = slice
                    .iter()
                    .find(|(_, b)| *b == 0)
                    .map(|(id, _)| *id)
                    .ok_or_else(|| {
                        EngineError::InvalidPlan("join missing branch 0 (left)".to_string())
                    })?;
                let right = slice
                    .iter()
                    .find(|(_, b)| *b == 1)
                    .map(|(id, _)| *id)
                    .ok_or_else(|| {
                        EngineError::InvalidPlan("join missing branch 1 (right)".to_string())
                    })?;
                Ok((left, right))
            }
            Some(slice) => Err(EngineError::InvalidPlan(format!(
                "expected 2 inputs for join {:?}, got {}",
                id,
                slice.len()
            ))),
            None => Err(EngineError::InvalidPlan(format!(
                "no inputs for join {:?}",
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
/// - The operator spec being built
/// - Accumulated scan metadata
/// - Slot allocation state
struct SubtreeResult {
    /// How to resolve variable references in expressions above this node
    resolver: ResolverKind,
    /// The physical operator spec for this subtree
    op: OpKind,
    /// Scan metadata accumulated from this subtree
    scan_metadata: HashMap<ScanId, ScanMetadata>,
    /// Post-scan steps accumulated (filters, projections, limits)
    steps: Vec<StepSpec>,
    /// Current slot allocation high-water mark
    slot_count: usize,
    /// Max expression registers needed
    max_registers: usize,
    /// Output shape (set by projection nodes)
    shape: Option<Shape>,
}

/// The kind of physical operator produced by a subtree.
enum OpKind {
    /// Single-table pipeline scan
    Pipeline { scan_id: ScanId },
    /// Nested-loop join (the NestedLoopJoinSpec is built at finalization)
    Join(NestedLoopJoinSpec),
    /// Expression-only query (no scan)
    ExprQuery(ExprQuerySpec),
}

/// The kind of slot resolver produced by a subtree.
enum ResolverKind {
    /// Single-scan resolver (maps alias and column names to slots)
    Pipeline(PipelineSlotResolver),
    /// Join resolver (maps left and right aliases to their slots)
    Join(JoinSlotResolver),
    /// Empty resolver for expression queries
    Empty,
}

impl SlotResolver for ResolverKind {
    fn resolve_var(&self, name: &BindingsName<'_>, scope: VarRefType) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_var(name, scope),
            ResolverKind::Join(r) => r.resolve_var(name, scope),
            ResolverKind::Empty => None,
        }
    }
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_alias(name),
            ResolverKind::Join(r) => r.resolve_alias(name),
            ResolverKind::Empty => None,
        }
    }
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        match self {
            ResolverKind::Pipeline(r) => r.resolve_field(name),
            ResolverKind::Join(r) => r.resolve_field(name),
            ResolverKind::Empty => None,
        }
    }
    fn is_alias(&self, name: &BindingsName<'_>) -> bool {
        match self {
            ResolverKind::Pipeline(r) => r.is_alias(name),
            ResolverKind::Join(r) => r.is_alias(name),
            ResolverKind::Empty => false,
        }
    }
}

// ---------------------------------------------------------------------------
// PlanCompiler — recursive tree-walk compilation
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

    /// Compile a logical plan into a physical execution plan.
    ///
    /// Walks the plan graph recursively from the sink node downward,
    /// compiling each node based on its type and its children.
    pub fn compile(&mut self, plan: &LogicalPlan<BindingsOp>) -> Result<CompiledPlan> {
        let graph = PlanGraph::new(plan);
        let sink_id = graph.find_sink()?;
        let mut ctx = CompileContext::new();
        let result = self.compile_node(&graph, sink_id, &mut ctx)?;

        // Package the result into a CompiledPlan
        let shape = result
            .shape
            .unwrap_or(Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic)));

        let (nodes, root) = match result.op {
            OpKind::Pipeline { scan_id } => {
                let spec = PipelineSpec {
                    scan_id,
                    steps: result.steps,
                };
                (vec![RelOpSpec::Pipeline(spec)], 0)
            }
            OpKind::Join(mut join_spec) => {
                join_spec.steps = result.steps;
                (vec![RelOpSpec::NestedLoopJoin(join_spec)], 0)
            }
            OpKind::ExprQuery(spec) => (vec![RelOpSpec::ExprQuery(spec)], 0),
        };

        Ok(CompiledPlan {
            nodes,
            root,
            shape,
            slot_count: result.slot_count,
            max_registers: result.max_registers,
            scan_metadata: result.scan_metadata,
        })
    }

    /// Recursively compile a single node and its inputs.
    ///
    /// `ctx` carries field requests accumulated from ancestor nodes (Project, Filter).
    /// These are passed down so that Scan nodes can resolve them into column projections.
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
            BindingsOp::Join(join) => self.compile_join(graph, id, join, ctx),
            BindingsOp::Filter(filter) => {
                let input_id = graph.single_input(id)?;
                // Extract field requests from filter expression BEFORE recursing
                let filter_expr = filter.expr.clone();
                let mut extractor = ExprFieldExtractor::new(ctx);
                extractor.extract(&filter_expr);
                // Recurse with accumulated context
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_filter(&mut result, &filter_expr)?;
                Ok(result)
            }
            BindingsOp::Project(project) => {
                let input_id = graph.single_input(id)?;
                // Extract field requests from all project expressions BEFORE recursing
                let project = project.clone();
                {
                    let mut extractor = ExprFieldExtractor::new(ctx);
                    for (_name, expr) in &project.exprs {
                        extractor.extract(expr);
                    }
                }
                // Recurse with accumulated context
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project(&mut result, &project)?;
                Ok(result)
            }
            BindingsOp::ProjectValue(pv) => {
                let input_id = graph.single_input(id)?;
                // For ProjectValue, extract field requests from the expression
                let pv = pv.clone();
                {
                    let mut extractor = ExprFieldExtractor::new(ctx);
                    extractor.extract(&pv.expr);
                }
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project_value(&mut result, &pv)?;
                Ok(result)
            }
            BindingsOp::ProjectAll(mode) => {
                let input_id = graph.single_input(id)?;
                // SELECT * — no specific fields to request, scan will use WholeValue
                let mode = mode.clone();
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_project_all(&mut result, &mode)?;
                Ok(result)
            }
            BindingsOp::LimitOffset(lo) => {
                let input_id = graph.single_input(id)?;
                let lo = lo.clone();
                let mut result = self.compile_node(graph, input_id, ctx)?;
                self.apply_limit(&mut result, &lo)?;
                Ok(result)
            }
            BindingsOp::ExprQuery(eq) => self.compile_expr_query(eq),
            other => Err(EngineError::InvalidPlan(format!(
                "unsupported operator: {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // Leaf node compilation
    // -----------------------------------------------------------------------

    /// Compile a Scan node into a pipeline operator.
    ///
    /// Uses the `CompileContext` to determine whether to use column projection
    /// or whole-value mode. If ancestor nodes requested specific fields and the
    /// data source can resolve them, we use column projection. Otherwise we
    /// fall back to whole-value mode.
    fn compile_scan(&mut self, scan: &Scan, ctx: &CompileContext) -> Result<SubtreeResult> {
        let (catalog_id, reader_factory) = self.resolve_reader_factory(scan)?;
        let scan_id = self.alloc_scan_id();
        let table_name = extract_table_name(&scan.expr);

        // Find field requests targeting this scan (by alias or table name)
        let my_requests = ctx.requests_for_alias(&scan.as_key, table_name.as_deref());

        let mut projections = Vec::new();
        let mut column_slots = FxHashMap::default();
        let mut base_row_slot: Option<SlotId> = None;
        let mut next_slot: SlotId = 0;
        let mut needs_whole_value = my_requests.is_empty();

        if !needs_whole_value {
            // Try to resolve each requested field via the data source metadata
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
                    // Data source can't resolve this field → fall back to whole-value
                    needs_whole_value = true;
                    break;
                }
            }
        }

        if needs_whole_value {
            // Fall back to whole-value mode
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
            op: OpKind::Pipeline { scan_id },
            scan_metadata,
            steps: Vec::new(),
            slot_count: next_slot as usize,
            max_registers: 0,
            shape: None,
        })
    }

    /// Compile a Join node by recursively compiling left and right children.
    ///
    /// Uses `ctx` to determine column projections for each side:
    /// - Extracts field requests from the ON clause
    /// - Resolves requests per side via `ctx.requests_for_alias()`
    /// - Falls back to WholeValue if any field can't be resolved
    fn compile_join(
        &mut self,
        _graph: &PlanGraph<'_>,
        _join_id: OpId,
        join: &Join,
        ctx: &mut CompileContext,
    ) -> Result<SubtreeResult> {
        let left_scan = extract_scan(&join.left)?;
        let right_scan = extract_scan(&join.right)?;

        // 1. Extract field requests from the ON clause BEFORE resolving scans
        if let Some(on_expr) = &join.on {
            let mut extractor = ExprFieldExtractor::new(ctx);
            extractor.extract(on_expr);
        }

        // 2. Resolve scan factories
        let (left_catalog_id, left_factory) = self.resolve_reader_factory(left_scan)?;
        let (right_catalog_id, right_factory) = self.resolve_reader_factory(right_scan)?;

        let left_scan_id = self.alloc_scan_id();
        let right_scan_id = self.alloc_scan_id();

        let left_table_name = extract_table_name(&left_scan.expr);
        let right_table_name = extract_table_name(&right_scan.expr);

        // 3. Try column projection for LEFT scan
        let left_requests = ctx.requests_for_alias(&left_scan.as_key, left_table_name.as_deref());
        let (left_projections, left_column_slots, left_base_slot, left_slot_count) =
            resolve_scan_projections(&left_requests, &left_factory, 0)?;

        // 4. Try column projection for RIGHT scan (slots start after left)
        let right_start_slot = left_slot_count;
        let right_requests =
            ctx.requests_for_alias(&right_scan.as_key, right_table_name.as_deref());
        let (right_projections, right_column_slots, right_base_slot, right_slot_count) =
            resolve_scan_projections(&right_requests, &right_factory, right_start_slot)?;

        let mut slot_count = (right_start_slot + right_slot_count) as usize;

        // 5. Build resolver for compiling the ON clause and post-join expressions
        let resolver = JoinSlotResolver {
            left_alias: left_scan.as_key.clone(),
            left_base_slot,
            left_column_slots: left_column_slots.clone(),
            right_alias: right_scan.as_key.clone(),
            right_base_slot,
            right_column_slots: right_column_slots.clone(),
        };

        // 6. Compile ON clause condition
        let (condition_slot, condition) = if let Some(on_expr) = &join.on {
            let cond_slot = slot_count as SlotId;
            slot_count += 1;

            let expr_compiler = LogicalExprCompiler::new(&resolver);
            let program =
                expr_compiler.compile_to_program(on_expr, cond_slot, slot_count as u16)?;
            (Some(cond_slot), Some(program))
        } else {
            (None, None)
        };

        let max_registers = condition
            .as_ref()
            .map(|p| p.reg_count as usize)
            .unwrap_or(0);

        // 7. Build scan layouts and metadata
        let left_layout = ScanLayout {
            projections: left_projections,
        };
        let right_layout = ScanLayout {
            projections: right_projections,
        };

        let mut scan_metadata = HashMap::new();
        scan_metadata.insert(
            left_scan_id,
            ScanMetadata {
                layout: left_layout,
                object_id: ObjectId::new(left_catalog_id, left_factory.entry_id),
            },
        );
        scan_metadata.insert(
            right_scan_id,
            ScanMetadata {
                layout: right_layout,
                object_id: ObjectId::new(right_catalog_id, right_factory.entry_id),
            },
        );

        // 8. Build child operator specs
        let left_child = Box::new(RelOpSpec::Pipeline(PipelineSpec {
            scan_id: left_scan_id,
            steps: Vec::new(),
        }));
        let right_child = Box::new(RelOpSpec::Pipeline(PipelineSpec {
            scan_id: right_scan_id,
            steps: Vec::new(),
        }));

        let right_base_start = right_start_slot as usize;
        let join_spec = NestedLoopJoinSpec {
            kind: join.kind.clone(),
            left: left_child,
            right: right_child,
            condition,
            condition_slot,
            right_input_start: right_base_start,
            right_input_count: right_slot_count as usize,
            steps: Vec::new(), // filled in by parent
        };

        Ok(SubtreeResult {
            resolver: ResolverKind::Join(resolver),
            op: OpKind::Join(join_spec),
            scan_metadata,
            steps: Vec::new(),
            slot_count,
            max_registers,
            shape: None,
        })
    }

    /// Compile an ExprQuery (expression-only query without a scan).
    fn compile_expr_query(
        &mut self,
        expr_query: &partiql_logical::ExprQuery,
    ) -> Result<SubtreeResult> {
        let resolver = EmptySlotResolver;
        let expr_compiler = LogicalExprCompiler::new(&resolver);
        let program = expr_compiler.compile_to_program(&expr_query.expr, 0, 1)?;
        let max_registers = program.reg_count as usize;

        Ok(SubtreeResult {
            resolver: ResolverKind::Empty,
            op: OpKind::ExprQuery(ExprQuerySpec { program }),
            scan_metadata: HashMap::new(),
            steps: Vec::new(),
            slot_count: 1,
            max_registers,
            shape: Some(Shape::Single(RowShape::Register(0, PhysicalType::Dynamic))),
        })
    }

    // -----------------------------------------------------------------------
    // Step application (Filter, Project, Limit) — applied on top of any child
    // -----------------------------------------------------------------------

    fn apply_filter(&self, result: &mut SubtreeResult, expr: &ValueExpr) -> Result<()> {
        let pred_slot = result.slot_count as SlotId;
        result.slot_count += 1;

        let expr_compiler = LogicalExprCompiler::new(&result.resolver);
        let program =
            expr_compiler.compile_to_program(expr, pred_slot, result.slot_count as u16)?;
        result.max_registers = result.max_registers.max(program.reg_count as usize);
        result.steps.push(StepSpec::Filter {
            program,
            predicate_slot: pred_slot,
        });
        Ok(())
    }

    fn apply_project(&self, result: &mut SubtreeResult, project: &Project) -> Result<()> {
        let output_start = result.slot_count;
        let num_outputs = project.exprs.len();
        result.slot_count += num_outputs;

        let expr_compiler = LogicalExprCompiler::new(&result.resolver);

        let mut exprs = Vec::with_capacity(num_outputs);
        let mut fields = Vec::with_capacity(num_outputs);
        for (idx, (name, expr)) in project.exprs.iter().enumerate() {
            let target_slot = (output_start + idx) as SlotId;
            exprs.push((target_slot, expr.clone()));
            fields.push(FieldShape {
                name: FieldName::Static(name.clone()),
                value: RowShape::Register(output_start + idx, PhysicalType::Dynamic),
            });
        }

        let program = expr_compiler.compile_to_program_multi(&exprs, result.slot_count as u16)?;
        result.max_registers = result.max_registers.max(program.reg_count as usize);
        result.steps.push(StepSpec::Project { program });
        result.shape = Some(Shape::Bag(RowShape::Struct(fields)));
        Ok(())
    }

    fn apply_project_value(&self, result: &mut SubtreeResult, pv: &ProjectValue) -> Result<()> {
        let output_slot = result.slot_count as SlotId;
        result.slot_count += 1;

        let expr_compiler = LogicalExprCompiler::new(&result.resolver);
        let program =
            expr_compiler.compile_to_program(&pv.expr, output_slot, result.slot_count as u16)?;
        result.max_registers = result.max_registers.max(program.reg_count as usize);
        result.steps.push(StepSpec::Project { program });
        result.shape = Some(Shape::Bag(RowShape::Register(
            output_slot as usize,
            PhysicalType::Dynamic,
        )));
        Ok(())
    }

    fn apply_project_all(&self, result: &mut SubtreeResult, _mode: &ProjectAllMode) -> Result<()> {
        // SELECT * — pass through the whole-value slot(s) from the child.
        // For a single scan, copy the base row slot to an output slot.
        // For a join, we'd need to merge both sides (TODO: proper merge).
        let output_slot = result.slot_count as SlotId;
        result.slot_count += 1;

        // Build a VarRef to the first alias to copy its whole-value
        let alias = match &result.resolver {
            ResolverKind::Pipeline(r) => r.scan_alias.clone(),
            ResolverKind::Join(r) => r.left_alias.clone(),
            ResolverKind::Empty => {
                return Err(EngineError::InvalidPlan(
                    "SELECT * on expression query".to_string(),
                ))
            }
        };

        let copy_expr = ValueExpr::VarRef(
            BindingsName::CaseInsensitive(alias.into()),
            VarRefType::Local,
        );
        let expr_compiler = LogicalExprCompiler::new(&result.resolver);
        let program =
            expr_compiler.compile_to_program(&copy_expr, output_slot, result.slot_count as u16)?;
        result.max_registers = result.max_registers.max(program.reg_count as usize);
        result.steps.push(StepSpec::Project { program });
        result.shape = Some(Shape::Bag(RowShape::Register(
            output_slot as usize,
            PhysicalType::Dynamic,
        )));
        Ok(())
    }

    fn apply_limit(&self, result: &mut SubtreeResult, lo: &LimitOffset) -> Result<()> {
        if let Some(limit) = parse_limit(lo)? {
            result.steps.push(StepSpec::Limit { limit });
        }
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

/// Extract the table name from a scan expression.
///
/// The logical planner generates scan expressions like:
/// - `VarRef(CaseInsensitive("data"), Local)` → table name is "data"  
/// - `DBRef { path: [CaseInsensitive("data")] }` → table name is "data"
///
/// This is needed because the logical planner auto-generates aliases (e.g. "_1")
/// in `scan.as_key`, but projection expressions reference the original table name.
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

fn extract_scan(op: &BindingsOp) -> Result<&Scan> {
    match op {
        BindingsOp::Scan(scan) => Ok(scan),
        other => Err(EngineError::InvalidPlan(format!(
            "expected Scan, got: {:?}",
            std::mem::discriminant(other)
        ))),
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

/// Shared logic for resolving scan projections from field requests.
///
/// Given a set of field requests and a data source handle, tries to resolve
/// each field to a column projection. If any field can't be resolved, falls
/// back to whole-value mode.
///
/// Returns `(projections, column_slots, base_slot, slot_count)`:
///
/// - `base_slot`: `Some(slot)` in whole-value mode, `None` in column mode
/// - `slot_count`: number of slots consumed
///
/// Result of resolving scan projections.
type ScanProjectionResult = (
    Vec<ScanProjection>,
    FxHashMap<String, SlotId>,
    Option<SlotId>,
    SlotId,
);

fn resolve_scan_projections(
    requests: &[&crate::engine::field_resolver::FieldRequest],
    handle: &DataSourceHandle,
    start_slot: SlotId,
) -> Result<ScanProjectionResult> {
    let mut projections = Vec::new();
    let mut column_slots = FxHashMap::default();
    let mut next_slot = start_slot;
    let mut needs_whole_value = requests.is_empty();

    if !needs_whole_value {
        for req in requests {
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
        let slot = start_slot;
        projections.push(ScanProjection {
            source: ScanSource::whole_value(),
            target_slot: slot,
        });
        return Ok((projections, column_slots, Some(slot), 1));
    }

    let slot_count = next_slot - start_slot;
    Ok((projections, column_slots, None, slot_count))
}

// ---------------------------------------------------------------------------
// Slot resolvers
// ---------------------------------------------------------------------------

/// Resolves variable references for a single-scan pipeline.
struct PipelineSlotResolver {
    base_row_slot: Option<SlotId>,
    scan_alias: String,
    /// The original table name (e.g. "data") in addition to the auto-generated alias (e.g. "_1")
    table_name: Option<String>,
    column_slots: FxHashMap<String, SlotId>,
}

impl PipelineSlotResolver {
    /// Check if a name matches either the scan alias or the table name.
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

/// Resolves variable references for a two-way join.
///
/// Supports both whole-value mode (base_slot is Some) and column projection
/// mode (column_slots populated, base_slot is None).
struct JoinSlotResolver {
    left_alias: String,
    left_base_slot: Option<SlotId>,
    left_column_slots: FxHashMap<String, SlotId>,
    right_alias: String,
    right_base_slot: Option<SlotId>,
    right_column_slots: FxHashMap<String, SlotId>,
}

impl JoinSlotResolver {
    fn resolve_column_slot(
        column_slots: &FxHashMap<String, SlotId>,
        name: &BindingsName<'_>,
    ) -> Option<SlotId> {
        let key = match name {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
        };
        column_slots
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| *v)
    }
}

impl SlotResolver for JoinSlotResolver {
    fn resolve_var(&self, name: &BindingsName<'_>, _scope: VarRefType) -> Option<SlotId> {
        if bindings_name_matches(name, &self.left_alias) {
            return self.left_base_slot;
        }
        if bindings_name_matches(name, &self.right_alias) {
            return self.right_base_slot;
        }
        // Try unqualified field lookup across both sides
        if let Some(slot) = Self::resolve_column_slot(&self.left_column_slots, name) {
            return Some(slot);
        }
        if let Some(slot) = Self::resolve_column_slot(&self.right_column_slots, name) {
            return Some(slot);
        }
        None
    }

    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        if bindings_name_matches(name, &self.left_alias) {
            self.left_base_slot
        } else if bindings_name_matches(name, &self.right_alias) {
            self.right_base_slot
        } else {
            None
        }
    }

    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        // Try both sides for unqualified field access
        if let Some(slot) = Self::resolve_column_slot(&self.left_column_slots, name) {
            return Some(slot);
        }
        Self::resolve_column_slot(&self.right_column_slots, name)
    }

    fn is_alias(&self, name: &BindingsName<'_>) -> bool {
        bindings_name_matches(name, &self.left_alias)
            || bindings_name_matches(name, &self.right_alias)
    }
}

/// Empty slot resolver for ExprQuery expressions that don't reference variables.
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
