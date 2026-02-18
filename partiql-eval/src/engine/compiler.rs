use crate::engine::arena::SlotId;
use crate::engine::catalog::CompilationContext;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::LogicalExprCompiler;
use crate::engine::plan::{
    CompiledPlan, ExprQuerySpec, ObjectId, PipelineSpec, RelOpSpec, ScanId, ScanMetadata, StepSpec,
};
use crate::engine::source::{DataSourceHandle, ScanLayout, ScanProjection, ScanSource};
use crate::engine::value::{FieldName, FieldShape, PhysicalType, RowShape, Shape};
use crate::engine::SlotResolver;
use partiql_logical::{
    BindingsOp, DBRef, LimitOffset, LogicalPlan, OpId, PathComponent, Project, ProjectAllMode,
    ProjectValue, Scan, ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::HashMap;

pub struct PlanCompiler<'a> {
    compilation_context: &'a CompilationContext,
    next_scan_id: u64,
}

impl<'a> PlanCompiler<'a> {
    /// Create a new PlanCompiler with a compilation context.
    ///
    /// The compilation context provides access to catalogs for resolving table references.
    pub fn new(compilation_context: &'a CompilationContext) -> Self {
        PlanCompiler {
            compilation_context,
            next_scan_id: 0,
        }
    }

    /// Allocate a new unique ScanId
    fn alloc_scan_id(&mut self) -> ScanId {
        let id = ScanId::new(self.next_scan_id);
        self.next_scan_id += 1;
        id
    }

    pub fn compile(&mut self, plan: &LogicalPlan<BindingsOp>) -> Result<CompiledPlan> {
        let order = linearize(plan)?;

        // Check if this is an ExprQuery
        if let Some(BindingsOp::ExprQuery(expr_query)) = order.first() {
            if order.len() > 2 || (order.len() == 2 && !matches!(order[1], BindingsOp::Sink)) {
                return Err(EngineError::InvalidPlan(
                    "ExprQuery must be followed only by Sink".to_string(),
                ));
            }
            return self.compile_expr_query(expr_query);
        }

        let mut scan: Option<&Scan> = None;
        let mut filters: Vec<&ValueExpr> = Vec::new();
        let mut project: Option<&Project> = None;
        let mut project_value: Option<&ProjectValue> = None;
        let mut project_all: Option<&ProjectAllMode> = None;
        let mut limit: Option<usize> = None;

        for op in order {
            match op {
                BindingsOp::Scan(scan_op) => {
                    if scan.is_some() {
                        return Err(EngineError::InvalidPlan("multiple scans".to_string()));
                    }
                    scan = Some(scan_op);
                }
                BindingsOp::Filter(filter) => {
                    filters.push(&filter.expr);
                }
                BindingsOp::Project(project_op) => {
                    if project.is_some() || project_value.is_some() || project_all.is_some() {
                        return Err(EngineError::InvalidPlan("multiple projections".to_string()));
                    }
                    project = Some(project_op);
                }
                BindingsOp::ProjectValue(pv) => {
                    if project.is_some() || project_value.is_some() || project_all.is_some() {
                        return Err(EngineError::InvalidPlan("multiple projections".to_string()));
                    }
                    project_value = Some(pv);
                }
                BindingsOp::ProjectAll(mode) => {
                    if project.is_some() || project_value.is_some() || project_all.is_some() {
                        return Err(EngineError::InvalidPlan("multiple projections".to_string()));
                    }
                    project_all = Some(mode);
                }
                BindingsOp::LimitOffset(limit_op) => {
                    if limit.is_some() {
                        return Err(EngineError::InvalidPlan("multiple limits".to_string()));
                    }
                    limit = parse_limit(limit_op)?;
                }
                BindingsOp::Sink => {}
                BindingsOp::Pivot(_)
                | BindingsOp::Unpivot(_)
                | BindingsOp::OrderBy(_)
                | BindingsOp::Join(_)
                | BindingsOp::BagOp(_)
                | BindingsOp::ExprQuery(_)
                | BindingsOp::Distinct
                | BindingsOp::GroupBy(_)
                | BindingsOp::Having(_) => {
                    return Err(EngineError::InvalidPlan(format!(
                        "unsupported operator in streaming pipeline: {:?}",
                        op
                    )));
                }
            }
        }

        let scan = scan.ok_or_else(|| EngineError::InvalidPlan("missing scan".to_string()))?;
        let (catalog_id, reader_factory) = self.resolve_reader_factory(scan)?;

        // Determine output slot count based on projection type
        let output_count = if let Some(proj) = project {
            proj.exprs.len()
        } else {
            // ProjectValue, ProjectAll, and no-projection all use 1 output slot
            1
        };

        let required_columns =
            collect_column_requirements_extended(&filters, project, project_value, &scan.as_key)?;
        let mut columns: Vec<String> = required_columns.into_iter().collect();
        columns.sort_unstable();

        // ProjectAll needs the whole row value when no specific columns are identified
        let force_whole_value = project_all.is_some();

        // Build scan layout based on reader capabilities
        let input_start = output_count;
        let mut column_slots = FxHashMap::default();
        let mut projections = Vec::new();

        // Try to resolve all required columns
        let mut all_resolved = true;
        if !columns.is_empty() && !force_whole_value {
            for (idx, name) in columns.iter().enumerate() {
                if let Some(source) = reader_factory.resolve(name) {
                    let slot = (input_start + idx) as SlotId;
                    column_slots.insert(name.clone(), slot);
                    projections.push(ScanProjection {
                        source,
                        target_slot: slot,
                    });
                } else {
                    // Column not resolvable - need to fall back to whole value
                    all_resolved = false;
                    break;
                }
            }
        }

        // Fall back to whole value if:
        // - Any column couldn't be resolved
        // - ProjectAll is used (always needs the whole row for SELECT *)
        let needs_whole_value =
            force_whole_value || (!all_resolved && !columns.is_empty()) || columns.is_empty();
        let can_project = !needs_whole_value && all_resolved && !columns.is_empty();
        if needs_whole_value {
            projections.clear();
            column_slots.clear();
            projections.push(ScanProjection {
                source: ScanSource::whole_value(),
                target_slot: input_start as SlotId,
            });
        }

        let layout = ScanLayout { projections };
        let base_row_slot = if needs_whole_value {
            Some(input_start as SlotId)
        } else {
            None
        };

        let mut slot_count = input_start + if can_project { columns.len().max(0) } else { 1 };
        let predicate_slot = if filters.is_empty() {
            None
        } else {
            let slot = slot_count as SlotId;
            slot_count += 1;
            Some(slot)
        };

        let resolver = PipelineSlotResolver {
            base_row_slot,
            scan_alias: scan.as_key.clone(),
            column_slots: column_slots.clone(),
        };
        let expr_compiler = LogicalExprCompiler::new(&resolver);

        let mut steps: Vec<StepSpec> = Vec::new();
        let mut max_registers = 0usize;

        if let Some(predicate_slot) = predicate_slot {
            for filter_expr in filters {
                let program = expr_compiler.compile_to_program(
                    filter_expr,
                    predicate_slot,
                    slot_count as u16,
                )?;
                max_registers = max_registers.max(program.reg_count as usize);
                steps.push(StepSpec::Filter {
                    program,
                    predicate_slot,
                });
            }
        }

        let shape = if let Some(project_op) = project {
            // SELECT a, b, ... — named column projection
            let mut exprs = Vec::with_capacity(project_op.exprs.len());
            let mut fields = Vec::with_capacity(project_op.exprs.len());
            for (idx, (name, expr)) in project_op.exprs.iter().enumerate() {
                exprs.push((idx as SlotId, expr.clone()));
                // For now, all projected columns are Dynamic type since we don't have
                // type inference yet. The register index matches the output position.
                fields.push(FieldShape {
                    name: FieldName::Static(name.clone()),
                    value: RowShape::Register(idx, PhysicalType::Dynamic),
                });
            }
            let program = expr_compiler.compile_to_program_multi(&exprs, slot_count as u16)?;
            max_registers = max_registers.max(program.reg_count as usize);
            steps.push(StepSpec::Project { program });
            // Scans typically produce bags of structs
            Shape::Bag(RowShape::Struct(fields))
        } else if let Some(pv) = project_value {
            // SELECT VALUE <expr> — each row produces a single value
            let program =
                expr_compiler.compile_to_program(&pv.expr, 0 as SlotId, slot_count as u16)?;
            max_registers = max_registers.max(program.reg_count as usize);
            steps.push(StepSpec::Project { program });
            Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic))
        } else if let Some(_mode) = project_all {
            // SELECT * — emit the whole row value
            // For both Unwrap and PassThrough modes in the streaming engine,
            // we need to pass through the scanned row as-is. The scan already
            // places the whole value into `base_row_slot` (whole-value mode)
            // or individual columns into `column_slots` (column-projected mode).
            //
            // In whole-value mode: copy base_row_slot to output slot 0
            // In column-projected mode: pass through (no extra step needed,
            //   but we still need the shape metadata)
            if let Some(_base_slot) = base_row_slot {
                // Whole-value mode: compile a trivial expression that copies
                // the base row slot to output slot 0
                let copy_expr = ValueExpr::VarRef(
                    BindingsName::CaseInsensitive(scan.as_key.clone().into()),
                    VarRefType::Local,
                );
                let program =
                    expr_compiler.compile_to_program(&copy_expr, 0 as SlotId, slot_count as u16)?;
                max_registers = max_registers.max(program.reg_count as usize);
                steps.push(StepSpec::Project { program });
                // Shape is a bag of dynamic values (the whole row tuples)
                Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic))
            } else {
                // Column-projected mode or no columns: rows are already in slots.
                // Build a struct shape from the column slots.
                if !column_slots.is_empty() {
                    let mut fields: Vec<FieldShape> = column_slots
                        .iter()
                        .map(|(name, slot)| FieldShape {
                            name: FieldName::Static(name.clone()),
                            value: RowShape::Register(*slot as usize, PhysicalType::Dynamic),
                        })
                        .collect();
                    // Sort by slot index for deterministic ordering
                    fields.sort_by_key(|f| match &f.value {
                        RowShape::Register(idx, _) => *idx,
                        _ => 0,
                    });
                    Shape::Bag(RowShape::Struct(fields))
                } else {
                    // No columns resolved - emit the whole value
                    Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic))
                }
            }
        } else {
            // No projection - emit the whole value as a single dynamic column
            Shape::Bag(RowShape::Register(0, PhysicalType::Dynamic))
        };

        if let Some(limit) = limit {
            steps.push(StepSpec::Limit { limit });
        }

        // Generate unique ScanId and build scan metadata
        let scan_id = self.alloc_scan_id();
        let entry_id = reader_factory.entry_id;
        let object_id = ObjectId::new(catalog_id, entry_id);
        let scan_meta = ScanMetadata { layout, object_id };

        // Build scan_metadata map
        let mut scan_metadata = HashMap::new();
        scan_metadata.insert(scan_id, scan_meta);

        let pipeline = PipelineSpec { scan_id, steps };

        Ok(CompiledPlan {
            nodes: vec![RelOpSpec::Pipeline(pipeline)],
            root: 0,
            shape,
            slot_count,
            max_registers,
            scan_metadata,
        })
    }

    /// Resolve a DataSourceHandle for a scan, returning (CatalogId, DataSourceHandle)
    ///
    /// Resolves table references through the CompilationContext's catalog system.
    fn resolve_reader_factory(
        &self,
        scan: &Scan,
    ) -> Result<(partiql_common::catalog::CatalogId, DataSourceHandle)> {
        match &scan.expr {
            // Catalog-based scan via DBRef - resolve through CatalogRegistry
            ValueExpr::DBRef(db_ref) => self.resolve_catalog_table(db_ref),

            // Unqualified table reference (VarRef) - resolve through default catalog
            ValueExpr::VarRef(table_name, _) => {
                // Treat unqualified table names as belonging to the "default" catalog
                let db_ref = DBRef {
                    catalog: "default".to_string(),
                    path: vec![table_name.clone()],
                };
                self.resolve_catalog_table(&db_ref)
            }

            // Other expression types are not supported for scans
            _ => Err(EngineError::InvalidPlan(
                "Unsupported scan expression type - expected DBRef or VarRef".to_string(),
            )),
        }
    }

    /// Resolve a table from a catalog using DBRef, returning (CatalogId, DataSourceHandle)
    fn resolve_catalog_table(
        &self,
        db_ref: &DBRef,
    ) -> Result<(partiql_common::catalog::CatalogId, DataSourceHandle)> {
        // Look up the catalog by name - returns (CatalogId, &dyn CompilationCatalog)
        let (catalog_id, catalog) = self
            .compilation_context
            .get_catalog(&db_ref.catalog)
            .ok_or_else(|| {
                EngineError::InvalidPlan(format!("Catalog '{}' not found", db_ref.catalog))
            })?;

        // Resolve the table within the catalog
        let handle = catalog.get_table(&db_ref.path).ok_or_else(|| {
            let path_str = db_ref
                .path
                .iter()
                .map(|component| match component {
                    BindingsName::CaseSensitive(s) => format!("\"{}\"", s),
                    BindingsName::CaseInsensitive(s) => s.to_string(),
                })
                .collect::<Vec<_>>()
                .join(".");

            EngineError::InvalidPlan(format!(
                "Table '{}' not found in catalog '{}'",
                path_str, db_ref.catalog
            ))
        })?;

        Ok((catalog_id, handle))
    }

    /// Compile an ExprQuery (expression-only query without a scan)
    fn compile_expr_query(
        &mut self,
        expr_query: &partiql_logical::ExprQuery,
    ) -> Result<CompiledPlan> {
        let slot_count = 1; // Single output register

        // Empty resolver - expression should be self-contained
        let resolver = EmptySlotResolver;
        let expr_compiler = LogicalExprCompiler::new(&resolver);

        // Compile expression to register 0
        let program = expr_compiler.compile_to_program(&expr_query.expr, 0, slot_count as u16)?;
        let max_registers = program.reg_count as usize;

        let expr_query_spec = ExprQuerySpec { program };

        Ok(CompiledPlan {
            nodes: vec![RelOpSpec::ExprQuery(expr_query_spec)],
            root: 0,
            shape: Shape::Single(RowShape::Register(0, PhysicalType::Dynamic)),
            slot_count,
            max_registers,
            scan_metadata: HashMap::new(), // No scans in expression queries
        })
    }
}

/// Empty slot resolver for ExprQuery expressions that don't reference any variables
struct EmptySlotResolver;

impl SlotResolver for EmptySlotResolver {
    fn resolve_var(&self, _name: &BindingsName<'_>, _scope: VarRefType) -> Option<SlotId> {
        None // No variables in expression queries
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

struct PipelineSlotResolver {
    base_row_slot: Option<SlotId>,
    scan_alias: String,
    column_slots: FxHashMap<String, SlotId>,
}

impl SlotResolver for PipelineSlotResolver {
    fn resolve_var(&self, name: &BindingsName<'_>, _scope: VarRefType) -> Option<SlotId> {
        if bindings_name_matches(name, &self.scan_alias) {
            return self.base_row_slot;
        }
        self.resolve_field(name)
    }

    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId> {
        if bindings_name_matches(name, &self.scan_alias) {
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
        bindings_name_matches(name, &self.scan_alias)
    }
}

fn bindings_name_matches(name: &BindingsName<'_>, target: &str) -> bool {
    match name {
        BindingsName::CaseSensitive(s) => s.as_ref() == target,
        BindingsName::CaseInsensitive(s) => s.as_ref().eq_ignore_ascii_case(target),
    }
}

/// Column requirements collection that considers Project, ProjectValue, and filter expressions.
///
/// ProjectAll does not contribute column requirements because it needs the whole row
/// (handled by `force_whole_value` in the caller).
fn collect_column_requirements_extended(
    filters: &[&ValueExpr],
    project: Option<&Project>,
    project_value: Option<&ProjectValue>,
    binding_name: &str,
) -> Result<FxHashSet<String>> {
    let mut columns = FxHashSet::default();

    for expr in filters {
        extract_column_refs(expr, binding_name, &mut columns)?;
    }
    if let Some(project) = project {
        for (_, expr) in &project.exprs {
            extract_column_refs(expr, binding_name, &mut columns)?;
        }
    }
    if let Some(pv) = project_value {
        extract_column_refs(&pv.expr, binding_name, &mut columns)?;
    }

    Ok(columns)
}

fn extract_column_refs(
    expr: &ValueExpr,
    binding_name: &str,
    out: &mut FxHashSet<String>,
) -> Result<()> {
    match expr {
        ValueExpr::Path(base, components) => {
            let base_expr = match base.as_ref() {
                ValueExpr::DynamicLookup(lookups) => lookups
                    .iter()
                    .find(|lookup| match lookup {
                        ValueExpr::VarRef(name, _) => bindings_name_matches(name, binding_name),
                        _ => false,
                    })
                    .unwrap_or_else(|| lookups.first().unwrap()),
                other => other,
            };

            // Check if base references the scan binding (either exact match or VarRef(Local))
            let is_scan_ref = match base_expr {
                ValueExpr::VarRef(name, scope) => {
                    // Match if name matches binding OR if it's a Local scope VarRef
                    // (Local scope VarRefs to table names need column extraction)
                    bindings_name_matches(name, binding_name) || *scope == VarRefType::Local
                }
                ValueExpr::DBRef(db_ref) => {
                    // DBRef paths also need column extraction
                    if let Some(first_component) = db_ref.path.first() {
                        bindings_name_matches(first_component, binding_name)
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_scan_ref {
                if let Some(PathComponent::Key(key_name)) = components.first() {
                    out.insert(bindings_name_to_string(key_name));
                }
            }
        }
        ValueExpr::VarRef(name, scope) => {
            // For VarRef(Local), always extract as a column reference
            // For other scopes, only extract if it doesn't match the binding name
            if *scope == VarRefType::Local || !bindings_name_matches(name, binding_name) {
                out.insert(bindings_name_to_string(name));
            }
        }
        ValueExpr::DBRef(db_ref) => {
            // DBRef without path components - shouldn't happen in column context
            // but handle gracefully
            if let Some(first_component) = db_ref.path.first() {
                if !bindings_name_matches(first_component, binding_name) {
                    out.insert(bindings_name_to_string(first_component));
                }
            }
        }
        ValueExpr::DynamicLookup(lookups) => {
            for lookup in lookups.iter() {
                extract_column_refs(lookup, binding_name, out)?;
            }
        }
        ValueExpr::BinaryExpr(_, left, right) => {
            extract_column_refs(left, binding_name, out)?;
            extract_column_refs(right, binding_name, out)?;
        }
        ValueExpr::UnExpr(_, inner) => {
            extract_column_refs(inner, binding_name, out)?;
        }
        ValueExpr::Lit(_) => {}
        _ => {}
    }
    Ok(())
}

fn bindings_name_to_string(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseSensitive(s) => s.to_string(),
        BindingsName::CaseInsensitive(s) => s.to_string(),
    }
}

fn linearize(plan: &LogicalPlan<BindingsOp>) -> Result<Vec<&BindingsOp>> {
    let mut incoming: FxHashMap<OpId, usize> = FxHashMap::default();
    let mut outgoing: FxHashMap<OpId, OpId> = FxHashMap::default();

    for (src, dst, branch) in plan.flows() {
        if *branch != 0 {
            return Err(EngineError::InvalidPlan(
                "multi-branch flow unsupported".to_string(),
            ));
        }
        if outgoing.contains_key(src) {
            return Err(EngineError::InvalidPlan(
                "multiple outputs unsupported".to_string(),
            ));
        }
        outgoing.insert(*src, *dst);
        *incoming.entry(*dst).or_insert(0) += 1;
    }

    let mut start: Option<OpId> = None;
    for (id, _) in plan.operators_by_id() {
        let count = incoming.get(&id).copied().unwrap_or(0);
        if count == 0 {
            if start.is_some() {
                return Err(EngineError::InvalidPlan(
                    "multiple roots unsupported".to_string(),
                ));
            }
            start = Some(id);
        }
    }

    let start = start.ok_or_else(|| EngineError::InvalidPlan("empty plan".to_string()))?;
    let mut order = Vec::new();
    let mut current = start;
    loop {
        let op = plan
            .operator(current)
            .ok_or_else(|| EngineError::InvalidPlan("missing operator for flow".to_string()))?;
        order.push(op);
        match outgoing.get(&current).copied() {
            Some(next) => current = next,
            None => break,
        }
    }

    if order.len() != plan.operator_count() {
        return Err(EngineError::InvalidPlan(
            "plan is not a single chain".to_string(),
        ));
    }
    Ok(order)
}

fn parse_limit(limit: &LimitOffset) -> Result<Option<usize>> {
    if limit.offset.is_some() {
        return Err(EngineError::InvalidPlan("offset not supported".to_string()));
    }
    let expr = match &limit.limit {
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
                "limit literal must be integer".to_string(),
            )),
        },
        _ => Err(EngineError::InvalidPlan(
            "limit must be a literal integer".to_string(),
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
