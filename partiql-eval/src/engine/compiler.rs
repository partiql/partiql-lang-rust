use crate::engine::catalog::CatalogRegistry;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::LogicalExprCompiler;
use crate::engine::plan::{Column, CompiledPlan, PipelineSpec, RelOpSpec, Schema, StepSpec};
use crate::engine::row::SlotId;
use crate::engine::source::{DataSourceHandle, ScanLayout, ScanProjection, ScanSource, TypeHint};
use crate::engine::SlotResolver;
use partiql_logical::{
    BindingsOp, DBRef, LimitOffset, LogicalPlan, OpId, PathComponent, Project, Scan, ValueExpr,
    VarRefType,
};
use partiql_value::BindingsName;
use rustc_hash::{FxHashMap, FxHashSet};

pub trait ScanProvider {
    fn data_source(&self, scan: &Scan) -> Result<DataSourceHandle>;
}

pub struct PlanCompiler<'a> {
    scan_provider: &'a dyn ScanProvider,
    catalog_registry: Option<&'a CatalogRegistry>,
}

impl<'a> PlanCompiler<'a> {
    pub fn new(scan_provider: &'a dyn ScanProvider) -> Self {
        PlanCompiler {
            scan_provider,
            catalog_registry: None,
        }
    }

    /// Create a new PlanCompiler with optional catalog registry support
    pub fn with_catalogs(
        scan_provider: &'a dyn ScanProvider,
        catalog_registry: Option<&'a CatalogRegistry>,
    ) -> Self {
        PlanCompiler {
            scan_provider,
            catalog_registry,
        }
    }

    pub fn compile(&self, plan: &LogicalPlan<BindingsOp>) -> Result<CompiledPlan> {
        let order = linearize(plan)?;

        let mut scan: Option<&Scan> = None;
        let mut filters: Vec<&ValueExpr> = Vec::new();
        let mut project: Option<&Project> = None;
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
                    if project.is_some() {
                        return Err(EngineError::InvalidPlan("multiple projects".to_string()));
                    }
                    project = Some(project_op);
                }
                BindingsOp::LimitOffset(limit_op) => {
                    if limit.is_some() {
                        return Err(EngineError::InvalidPlan("multiple limits".to_string()));
                    }
                    limit = parse_limit(limit_op)?;
                }
                BindingsOp::Sink => {}
                _ => {
                    return Err(EngineError::InvalidPlan(
                        "unsupported operator in streaming pipeline".to_string(),
                    ));
                }
            }
        }

        let scan = scan.ok_or_else(|| EngineError::InvalidPlan("missing scan".to_string()))?;
        let reader_factory = self.resolve_reader_factory(scan)?;

        // Check reader capabilities at compile time
        let reader_caps = reader_factory.caps();
        let can_project = reader_caps.can_project;

        let has_project = project.is_some();
        let output_count = if has_project {
            project.map(|proj| proj.exprs.len()).unwrap_or_default()
        } else {
            1
        };

        let required_columns = collect_column_requirements(&filters, project, &scan.as_key)?;
        let mut columns: Vec<String> = required_columns.into_iter().collect();
        columns.sort_unstable();

        // Build scan layout based on reader capabilities
        let input_start = output_count;
        let mut column_slots = FxHashMap::default();
        let mut projections = Vec::new();

        if can_project && !columns.is_empty() {
            // Projection pushdown: resolve and project only required columns
            for (idx, name) in columns.iter().enumerate() {
                if let Some(source) = reader_factory.resolve(name) {
                    let slot = (input_start + idx) as SlotId;
                    column_slots.insert(name.clone(), slot);
                    projections.push(ScanProjection {
                        source,
                        target_slot: slot,
                        type_hint: TypeHint::Any,
                    });
                }
                // If resolve returns None, column doesn't exist in reader
            }
        } else if !can_project {
            // Reader doesn't support projection - fall back to base row
            projections.push(ScanProjection {
                source: ScanSource::BaseRow,
                target_slot: input_start as SlotId,
                type_hint: TypeHint::Any,
            });
            // Note: When using BaseRow, column_slots remains empty and
            // expression evaluation will work directly on the base row slot
        }
        // else: can_project but no columns needed - empty projection is fine

        let layout = ScanLayout { projections };
        let base_row_slot = if !can_project {
            Some(input_start as SlotId)
        } else {
            None
        };

        let mut slot_count = input_start + if can_project { columns.len() } else { 1 };
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
            column_slots,
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

        let schema = if let Some(project_op) = project {
            let mut exprs = Vec::with_capacity(project_op.exprs.len());
            let mut columns = Vec::with_capacity(project_op.exprs.len());
            for (idx, (name, expr)) in project_op.exprs.iter().enumerate() {
                exprs.push((idx as SlotId, expr.clone()));
                columns.push(Column { name: name.clone() });
            }
            let program = expr_compiler.compile_to_program_multi(&exprs, slot_count as u16)?;
            max_registers = max_registers.max(program.reg_count as usize);
            steps.push(StepSpec::Project { program });
            Schema { columns }
        } else {
            Schema {
                columns: vec![Column {
                    name: "value".to_string(),
                }],
            }
        };

        if let Some(limit) = limit {
            steps.push(StepSpec::Limit { limit });
        }

        let pipeline = PipelineSpec {
            layout,
            steps,
            data_source: reader_factory,
        };

        Ok(CompiledPlan {
            nodes: vec![RelOpSpec::Pipeline(pipeline)],
            root: 0,
            schema,
            slot_count,
            max_registers,
        })
    }

    /// Resolve a DataSourceHandle for a scan, handling both VarRef and DBRef expressions
    fn resolve_reader_factory(&self, scan: &Scan) -> Result<DataSourceHandle> {
        match &scan.expr {
            // Catalog-based scan via DBRef - resolve through CatalogRegistry
            ValueExpr::DBRef(db_ref) => self.resolve_catalog_table(db_ref),

            // TODO: VarRef and other expression types will be compiled differently in the future
            // Traditional scan via VarRef - use ScanProvider for now
            ValueExpr::VarRef(_, _) => self.scan_provider.data_source(scan),

            // Other expression types - delegate to ScanProvider for now
            _ => self.scan_provider.data_source(scan),
        }
    }

    /// Resolve a table from a catalog using DBRef
    fn resolve_catalog_table(&self, db_ref: &DBRef) -> Result<DataSourceHandle> {
        // Ensure we have a catalog registry
        let registry = self.catalog_registry.ok_or_else(|| {
            EngineError::InvalidPlan(format!(
                "Catalog '{}' referenced but no catalog registry configured",
                db_ref.catalog
            ))
        })?;

        // Look up the catalog by name
        let catalog = registry.get_catalog(&db_ref.catalog).ok_or_else(|| {
            EngineError::InvalidPlan(format!("Catalog '{}' not found", db_ref.catalog))
        })?;

        // Resolve the table within the catalog
        catalog.get_table(&db_ref.path).ok_or_else(|| {
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
        })
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

fn collect_column_requirements(
    filters: &[&ValueExpr],
    project: Option<&Project>,
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
