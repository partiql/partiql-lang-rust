use crate::batch::{Field, LogicalType, SourceTypeDef};
use crate::compiler::CompilerContext;
use crate::error::PlanError;
use crate::expr::{CompiledExpr, ExprInput, ExprOp, ExpressionExecutor};
use crate::operators::{VectorizedFilter, VectorizedOperator, VectorizedProject, VectorizedScan};
use partiql_logical::{BindingsOp, LogicalPlan, OpId, PathComponent, ValueExpr, VarRefType};
use std::collections::{HashMap, HashSet};
use smallvec::smallvec;

/// Tracks column requirements for each scan operator
pub struct ColumnRequirements {
    scan_requirements: HashMap<(String, OpId), HashSet<String>>,
}

impl ColumnRequirements {
    pub fn new() -> Self {
        Self {
            scan_requirements: HashMap::new(),
        }
    }

    /// Analyze the logical plan and determine column requirements
    pub fn analyze(&mut self, plan: &LogicalPlan<BindingsOp>) -> Result<(), PlanError> {
        let sink_id = find_sink(plan)?;
        self.analyze_operator(sink_id, plan, &HashSet::new())?;
        Ok(())
    }

    /// Get column requirements for a specific scan
    pub fn get_requirements(&self, data_source: &str, op_id: OpId) -> Option<&HashSet<String>> {
        self.scan_requirements
            .get(&(data_source.to_string(), op_id))
    }

    /// Recursively analyze operator and propagate column requirements backwards
    fn analyze_operator(
        &mut self,
        op_id: OpId,
        plan: &LogicalPlan<BindingsOp>,
        required_cols: &HashSet<String>,
    ) -> Result<(), PlanError> {
        let op = plan
            .operator(op_id)
            .ok_or_else(|| PlanError::General(format!("Operator {:?} not found", op_id)))?;

        match op {
            BindingsOp::Scan(scan) => {
                // Extract data source name and store requirements
                let data_source_name = extract_data_source_name(&scan.expr)?;
                self.scan_requirements
                    .entry((data_source_name, op_id))
                    .or_insert_with(HashSet::new)
                    .extend(required_cols.clone());
                Ok(())
            }

            BindingsOp::Filter(filter) => {
                // Filter needs all columns from output PLUS columns in filter expression
                let mut needed = required_cols.clone();
                
                // Extract the binding name from the predecessor scan
                let input_id = find_predecessor(op_id, plan)?;
                let binding_name = extract_binding_name(plan, input_id)?;
                
                needed.extend(extract_column_refs(&filter.expr, &binding_name)?);

                // Propagate to input
                self.analyze_operator(input_id, plan, &needed)
            }

            BindingsOp::Project(project) => {
                // Project only needs columns referenced in projection expressions
                let mut needed = HashSet::new();
                
                // Extract the binding name from the predecessor
                let input_id = find_predecessor(op_id, plan)?;
                let binding_name = extract_binding_name(plan, input_id)?;
                
                for (_alias, expr) in &project.exprs {
                    needed.extend(extract_column_refs(expr, &binding_name)?);
                }

                // Propagate to input
                self.analyze_operator(input_id, plan, &needed)
            }

            BindingsOp::Sink => {
                // Sink needs all columns from its input
                let input_id = find_predecessor(op_id, plan)?;
                self.analyze_operator(input_id, plan, required_cols)
            }

            _ => Err(PlanError::General(format!(
                "Unsupported operator: {:?}",
                op
            ))),
        }
    }
}

/// Extract column references from a ValueExpr
fn extract_column_refs(expr: &ValueExpr, binding_name: &str) -> Result<HashSet<String>, PlanError> {
    let mut columns = HashSet::new();

    match expr {
        ValueExpr::Path(base, components) => {
            // Handle both direct VarRef and DynamicLookup
            let base_expr = match base.as_ref() {
                ValueExpr::DynamicLookup(lookups) => {
                    // Try to find a matching binding in the lookups
                    lookups.iter().find(|lookup| {
                        if let ValueExpr::VarRef(name, _) = lookup {
                            bindings_name_to_string(name) == binding_name
                        } else {
                            false
                        }
                    }).unwrap_or_else(|| lookups.first().unwrap())
                }
                other => other,
            };
            
            if let ValueExpr::VarRef(name, _) = base_expr {
                let name_str = bindings_name_to_string(name);
                if name_str == binding_name {
                    // Extract first key from path
                    if let Some(PathComponent::Key(key_name)) = components.first() {
                        columns.insert(bindings_name_to_string(key_name));
                    }
                }
            }
        }

        ValueExpr::VarRef(name, _) => {
            columns.insert(bindings_name_to_string(name));
        }

        ValueExpr::DynamicLookup(lookups) => {
            // Process each lookup option
            for lookup in lookups.iter() {
                columns.extend(extract_column_refs(lookup, binding_name)?);
            }
        }

        ValueExpr::BinaryExpr(_, left, right) => {
            columns.extend(extract_column_refs(left, binding_name)?);
            columns.extend(extract_column_refs(right, binding_name)?);
        }

        ValueExpr::UnExpr(_, inner) => {
            columns.extend(extract_column_refs(inner, binding_name)?);
        }

        // Literals don't reference columns
        ValueExpr::Lit(_) => {}

        _ => {
            // For now, ignore other complex expressions
        }
    }

    Ok(columns)
}

/// Translates LogicalPlan to physical VectorizedOperators
pub struct LogicalToPhysical {
    context: CompilerContext,
    column_requirements: ColumnRequirements,
}

impl LogicalToPhysical {
    pub fn new(context: CompilerContext, column_requirements: ColumnRequirements) -> Self {
        Self {
            context,
            column_requirements,
        }
    }

    /// Translate the logical plan to a physical operator tree
    pub fn translate(
        mut self,
        plan: &LogicalPlan<BindingsOp>,
    ) -> Result<Box<dyn VectorizedOperator>, PlanError> {
        let sink_id = find_sink(plan)?;
        let input_id = find_predecessor(sink_id, plan)?;
        self.translate_node(input_id, plan)
    }

    fn translate_node(
        &mut self,
        op_id: OpId,
        plan: &LogicalPlan<BindingsOp>,
    ) -> Result<Box<dyn VectorizedOperator>, PlanError> {
        let op = plan
            .operator(op_id)
            .ok_or_else(|| PlanError::General(format!("Operator {:?} not found", op_id)))?;

        match op {
            BindingsOp::Scan(scan) => self.build_scan(op_id, scan),
            BindingsOp::Filter(filter) => self.build_filter(op_id, filter, plan),
            BindingsOp::Project(project) => self.build_project(op_id, project, plan),
            _ => Err(PlanError::General(format!(
                "Unsupported operator: {:?}",
                op
            ))),
        }
    }

    fn build_scan(
        &mut self,
        op_id: OpId,
        scan: &partiql_logical::Scan,
    ) -> Result<Box<dyn VectorizedOperator>, PlanError> {
        // 1. Extract data source name
        let data_source_name = extract_data_source_name(&scan.expr)?;

        // 2. Get the reader
        let reader = self
            .context
            .get_data_source(&data_source_name)
            .ok_or_else(|| {
                PlanError::General(format!("Data source '{}' not found", data_source_name))
            })?;

        // 3. Get required columns for this scan
        let required_cols = self
            .column_requirements
            .get_requirements(&data_source_name, op_id)
            .ok_or_else(|| {
                PlanError::General(format!(
                    "No column requirements found for scan {}",
                    data_source_name
                ))
            })?;

        // 4. Resolve column names to ProjectionSources and build output schema
        let mut projections = Vec::new();
        let mut output_fields = Vec::new();

        for col_name in required_cols {
            let proj_source = reader.resolve(col_name).ok_or_else(|| {
                PlanError::General(format!("Column '{}' not found in reader", col_name))
            })?;

            projections.push(proj_source);

            // For now, assume Int64 type - in Phase 2 we'll infer types properly
            let field_type = LogicalType::Int64;
            output_fields.push(Field {
                name: col_name.clone(),
                type_info: field_type,
            });
        }

        let output_schema = SourceTypeDef::new(output_fields);

        // 5. Create VectorizedScan
        Ok(Box::new(VectorizedScan::new(
            reader,
            projections,
            output_schema,
        )))
    }

    fn build_filter(
        &mut self,
        op_id: OpId,
        _filter: &partiql_logical::Filter,
        plan: &LogicalPlan<BindingsOp>,
    ) -> Result<Box<dyn VectorizedOperator>, PlanError> {
        // 1. Build input operator
        let input_id = find_predecessor(op_id, plan)?;
        let input_op = self.translate_node(input_id, plan)?;

        // input_op.output_schema --> a = V0

        // 2. Create stub filter executor (pass-through for now)
        let stub_executor = create_stub_filter_executor();

        // 3. Create filter operator
        Ok(Box::new(VectorizedFilter::new(input_op, stub_executor)))
    }

    fn build_project(
        &mut self,
        op_id: OpId,
        project: &partiql_logical::Project,
        plan: &LogicalPlan<BindingsOp>,
    ) -> Result<Box<dyn VectorizedOperator>, PlanError> {
        // 1. Build input operator
        let input_id = find_predecessor(op_id, plan)?;
        let input_op = self.translate_node(input_id, plan)?;
        let input_schema = input_op.output_schema();

        // 2. Build output schema and executor
        let mut output_fields = Vec::new();
        let mut compiled_exprs = Vec::new();
        let mut output_types = Vec::new();

        for (idx, (alias, expr)) in project.exprs.iter().enumerate() {
            // Try to extract simple column reference
            let col_name = try_extract_simple_column_ref(expr).ok_or_else(|| {
                PlanError::General("Only simple column references supported in Phase 1".to_string())
            })?;

            let col_idx = input_schema.get_column_index(&col_name)?;
            let col_type = input_schema.get_type(&col_name)?;

            // Identity operation: pass through input column
            compiled_exprs.push(CompiledExpr {
                op: ExprOp::Identity,
                inputs: smallvec![ExprInput::InputCol(col_idx)],
                output: idx,
            });

            output_fields.push(Field {
                name: alias.clone(),
                type_info: col_type,
            });
            output_types.push(col_type);
        }

        let output_schema = SourceTypeDef::new(output_fields);
        let output_indices: Vec<usize> = (0..project.exprs.len()).collect();
        let executor = ExpressionExecutor::new(compiled_exprs, output_types, output_indices);

        // 3. Create project operator
        Ok(Box::new(VectorizedProject::new(
            input_op,
            executor,
            output_schema,
        )))
    }
}

// ===== Helper Functions =====

/// Convert BindingsName to String
fn bindings_name_to_string(name: &partiql_value::BindingsName) -> String {
    match name {
        partiql_value::BindingsName::CaseSensitive(s) => s.to_string(),
        partiql_value::BindingsName::CaseInsensitive(s) => s.to_string(),
    }
}

/// Find the Sink operator in the plan
fn find_sink(plan: &LogicalPlan<BindingsOp>) -> Result<OpId, PlanError> {
    plan.operators_by_id()
        .find(|(_, op)| matches!(op, BindingsOp::Sink))
        .map(|(id, _)| id)
        .ok_or_else(|| PlanError::General("No Sink operator found in plan".to_string()))
}

/// Find the predecessor (input) of an operator
fn find_predecessor(op_id: OpId, plan: &LogicalPlan<BindingsOp>) -> Result<OpId, PlanError> {
    plan.flows()
        .iter()
        .find(|(_, dst, _)| *dst == op_id)
        .map(|(src, _, _)| *src)
        .ok_or_else(|| {
            PlanError::General(format!("No input found for operator {:?}", op_id))
        })
}

/// Extract data source name from Scan.expr
fn extract_data_source_name(expr: &ValueExpr) -> Result<String, PlanError> {
    match expr {
        ValueExpr::VarRef(name, VarRefType::Global) => Ok(bindings_name_to_string(name)),
        ValueExpr::DynamicLookup(lookups) => {
            // DynamicLookup contains multiple possible interpretations
            // Find the Global VarRef (table reference)
            for lookup in lookups.iter() {
                if let ValueExpr::VarRef(name, VarRefType::Global) = lookup {
                    return Ok(bindings_name_to_string(name));
                }
            }
            Err(PlanError::General(format!(
                "No global VarRef found in DynamicLookup: {:?}",
                expr
            )))
        }
        _ => Err(PlanError::General(format!(
            "Scan expression must be a global VarRef or DynamicLookup, but found: {:?}",
            expr
        ))),
    }
}

/// Extract binding name (as_key) from a scan operator
fn extract_binding_name(plan: &LogicalPlan<BindingsOp>, op_id: OpId) -> Result<String, PlanError> {
    let op = plan
        .operator(op_id)
        .ok_or_else(|| PlanError::General(format!("Operator {:?} not found", op_id)))?;

    match op {
        BindingsOp::Scan(scan) => Ok(scan.as_key.clone()),
        _ => {
            // Recursively look for the scan
            let input_id = find_predecessor(op_id, plan)?;
            extract_binding_name(plan, input_id)
        }
    }
}

/// Try to extract a simple column reference like "v.a" -> "a"
fn try_extract_simple_column_ref(expr: &ValueExpr) -> Option<String> {
    match expr {
        ValueExpr::Path(base, components) => {
            // Handle both direct VarRef and DynamicLookup
            let base_expr = match base.as_ref() {
                ValueExpr::DynamicLookup(lookups) => lookups.first()?,
                other => other,
            };
            
            if let ValueExpr::VarRef(_, _) = base_expr {
                if let Some(PathComponent::Key(key_name)) = components.first() {
                    return Some(bindings_name_to_string(key_name));
                }
            }
            None
        }
        ValueExpr::VarRef(name, _) => Some(bindings_name_to_string(name)),
        ValueExpr::DynamicLookup(lookups) => {
            // Unwrap DynamicLookup and try again
            try_extract_simple_column_ref(lookups.first()?)
        }
        _ => None,
    }
}

/// Create stub filter executor that passes all rows (for Phase 1)
/// Uses AndBool with empty inputs as a workaround until we add proper literal support
fn create_stub_filter_executor() -> ExpressionExecutor {
    // Stub: Use AndBool as placeholder that effectively passes through
    // In Phase 2, we'll implement proper expression compilation
    // TODO!
    // Given some input logical expr
    // And, given some input SourceTypeDef
    // Resolve which column `a` corresponds to
    // Resolve which column `b` corresponds to
    // Resolve the operator.
    ExpressionExecutor::new(
        vec![CompiledExpr {
            op: ExprOp::LtI64,
            inputs: smallvec![
                ExprInput::InputCol(0),
                ExprInput::InputCol(1)
            ],
            output: 0,
        }],
        vec![LogicalType::Boolean],
        vec![0],
    )
}
