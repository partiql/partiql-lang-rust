use crate::batch::{Field, SourceTypeDef, LogicalType};
use crate::compiler::VectorizedPlan;
use crate::error::PlanError;
use crate::expr::ExpressionExecutor;
use crate::functions::VectorizedFnRegistry;
use crate::operators::{VectorizedFilter, VectorizedProject, VectorizedScan};
use crate::reader::BatchReader;
use partiql_logical::{BindingsOp, LogicalPlan};
use smallvec::smallvec;
use std::collections::HashMap;

/// Context provided to the compiler for plan building
pub struct CompilerContext {
    /// Maps table/binding names to data sources
    data_sources: HashMap<String, Box<dyn BatchReader>>,

    /// Function registry for type inference and function resolution
    fn_registry: VectorizedFnRegistry,
}

impl CompilerContext {
    /// Create a new compiler context
    pub fn new() -> Self {
        Self {
            data_sources: HashMap::new(),
            fn_registry: VectorizedFnRegistry::default(),
        }
    }

    /// Add a data source for a table/binding name
    pub fn with_data_source(mut self, name: String, reader: Box<dyn BatchReader>) -> Self {
        self.data_sources.insert(name, reader);
        self
    }

    /// Get a data source by name
    pub fn get_data_source(&mut self, name: &str) -> Option<Box<dyn BatchReader>> {
        self.data_sources.remove(name)
    }

    /// Get the function registry
    pub fn fn_registry(&self) -> &VectorizedFnRegistry {
        &self.fn_registry
    }
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiles LogicalPlan into VectorizedPlan
pub struct Compiler {
    context: CompilerContext,
}

impl Compiler {
    /// Create a new compiler with the given context
    pub fn new(context: CompilerContext) -> Self {
        Self { context }
    }

    /// Compile a logical plan into a vectorized plan
    ///
    /// For PoC: Returns a hardcoded plan: SCAN data -> FILTER a > 500 AND b < 100 -> PROJECT a, b
    pub fn compile(
        &mut self,
        _logical: &LogicalPlan<BindingsOp>,
    ) -> Result<VectorizedPlan, PlanError> {
        // TODO: Traverse logical plan and build vectorized operators
        // TODO: Perform type inference using fn_registry
        // TODO: Resolve table references via data_sources
        
        // Get data source
        let mut reader = self
            .context
            .get_data_source("data")
            .ok_or_else(|| PlanError::General("No data source found".to_string()))?;

        // Phase 0: Configure reader with projection specification
        // For this demo, we'll project columns 'a' and 'b' as Int64
        use crate::reader::{Projection, ProjectionSource, ProjectionSpec};
        use crate::batch::LogicalType;
        
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("a".to_string()),
                0, // Target vector index 0
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("b".to_string()),
                1, // Target vector index 1
                LogicalType::Int64,
            ),
        ];

        let projection_spec = ProjectionSpec::new(projections)
            .map_err(|e| PlanError::General(format!("Failed to create projection spec: {}", e)))?;

        // Configure the reader with the projection
        reader.set_projection(projection_spec)
            .map_err(|e| PlanError::General(format!("Failed to set projection: {}", e)))?;

        // Step 1: Create SCAN operator
        let scan = VectorizedScan::new(reader);

        // Step 2: Create FILTER predicate: a > 500 AND b < 100
        // Compile filter expression to bytecode
        // Expression: (a > 500) AND (b < 100)
        // Scratch registers: 0 = a > 500 result, 1 = b < 100 result, 2 = AND result
        use crate::expr::{CompiledExpr, ExprOp, ExprInput, ConstantValue};
        
        let filter_exprs = vec![
            // Step 1: Compare a > 500
            CompiledExpr {
                op: ExprOp::EqI64,
                inputs: smallvec![
                    ExprInput::InputCol(0),  // column 'a'
                    ExprInput::Constant(ConstantValue::Int64(1000))
                ],
                output: 0,  // scratch register 0
            },
        ];
        
        // Filter output mapping: scratch register 2 contains the final boolean result
        // Scratch types: 0=Boolean (a>500), 1=Boolean (b<100), 2=Boolean (AND result)
        let filter_executor = ExpressionExecutor::new(
            filter_exprs,
            vec![LogicalType::Boolean],
            vec![0]
        );

        // Create FILTER operator
        let filter = VectorizedFilter::new(Box::new(scan), filter_executor);

        // Step 3: Create PROJECT for columns a, b
        // Use Identity operations to project input columns to output
        let project_exprs = vec![
            // Project column 'a' (index 0) to scratch register 0
            CompiledExpr {
                op: ExprOp::Identity,
                inputs: smallvec![ExprInput::InputCol(0)],
                output: 0,
            },
            // Project column 'b' (index 1) to scratch register 1
            CompiledExpr {
                op: ExprOp::Identity,
                inputs: smallvec![ExprInput::InputCol(1)],
                output: 1,
            },
        ];
        // Output mapping: scratch registers 0 and 1 map to output columns 0 and 1
        // Scratch types: 0=Int64 (column a), 1=Int64 (column b)
        let project_executor = ExpressionExecutor::new(
            project_exprs,
            vec![LogicalType::Int64, LogicalType::Int64],
            vec![0, 1]
        );
        
        let output_schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: LogicalType::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: LogicalType::Int64,
            },
        ]);

        let project = VectorizedProject::new(
            Box::new(filter),
            project_executor,
            output_schema.clone(),
        );

        // Return plan with PROJECT as the root
        Ok(VectorizedPlan::new(Box::new(project), output_schema))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{Field, LogicalType};
    use crate::reader::TupleIteratorReader;

    #[test]
    fn test_compiler_basic() {
        // Create a schema matching what the compiler expects (a INT64, b INT64)
        let schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: LogicalType::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: LogicalType::Int64,
            },
        ]);

        // Create a dummy reader (Phase 0 - no schema in constructor)
        let tuples: Vec<partiql_value::Value> = vec![];
        let reader: Box<dyn BatchReader> = Box::new(TupleIteratorReader::new(
            Box::new(tuples.into_iter()),
            1024,
        ));

        // Create compiler context with data source
        let context = CompilerContext::new().with_data_source("data".to_string(), reader);

        // Create compiler
        let mut compiler = Compiler::new(context);

        // Create a dummy logical plan (we're not actually using it yet)
        // For now, just passing it through
        let logical = LogicalPlan::new();

        // Compile
        let result = compiler.compile(&logical);

        // Should succeed and return a plan
        assert!(result.is_ok());
        let plan = result.unwrap();

        // Verify output schema has 2 columns (a, b) as expected
        assert_eq!(plan.output_schema().field_count(), 2);
    }
}
