use crate::batch::{Field, PVector, SourceTypeDef, TypeInfo, TypedVector};
use crate::compiler::VectorizedPlan;
use crate::error::PlanError;
use crate::expr::{ColumnRef, FnCallExpr, LiteralExpr};
use crate::functions::{VecAnd, VecGtInt64, VecLtInt64, VectorizedFnRegistry};
use crate::operators::{VectorizedFilter, VectorizedProject, VectorizedScan};
use crate::reader::BatchReader;
use partiql_logical::{BindingsOp, LogicalPlan};
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
        let reader = self
            .context
            .get_data_source("data")
            .ok_or_else(|| PlanError::General("No data source found".to_string()))?;

        // Get schema from reader (assumes columns: a INT64, b INT64)
        let _input_schema = reader.schema().clone();

        // Step 1: Create SCAN operator
        let scan = VectorizedScan::new(reader);

        // Step 2: Create FILTER predicate: a > 500 AND b < 100
        // We need to allocate scratch columns for intermediate expression results
        // Source columns: [0=a, 1=b]
        // Scratch columns will be: [2, 3, 4, 5, 6]
        
        // Scratch column allocation:
        let scratch_col_literal_500 = 2;  // For literal 500
        let scratch_col_a_gt_500 = 3;     // For result of a > 500
        let scratch_col_literal_100 = 4;  // For literal 100
        let scratch_col_b_lt_100 = 5;     // For result of b < 100
        let scratch_col_and = 6;           // For final AND result
        
        // Build: a > 500
        let col_a = Box::new(ColumnRef::new(0, TypeInfo::Int64)); // Column 'a' at index 0
        let literal_500 = Box::new(LiteralExpr::new(
            PVector::Int64(TypedVector::from_vec(vec![500])),
            TypeInfo::Int64
        ));
        let a_gt_500 = Box::new(FnCallExpr::new(
            Box::new(VecGtInt64),
            vec![col_a, literal_500],
            vec![0, scratch_col_literal_500], // Input columns: a is at 0, literal writes to scratch 2
            TypeInfo::Boolean,
        ));

        // Build: b < 100
        let col_b = Box::new(ColumnRef::new(1, TypeInfo::Int64)); // Column 'b' at index 1
        let literal_100 = Box::new(LiteralExpr::new(
            PVector::Int64(TypedVector::from_vec(vec![100])),
            TypeInfo::Int64
        ));
        let b_lt_100 = Box::new(FnCallExpr::new(
            Box::new(VecLtInt64),
            vec![col_b, literal_100],
            vec![1, scratch_col_literal_100], // Input columns: b is at 1, literal writes to scratch 4
            TypeInfo::Boolean,
        ));

        // Build: (a > 500) AND (b < 100)
        let predicate = Box::new(FnCallExpr::new(
            Box::new(VecAnd),
            vec![a_gt_500, b_lt_100],
            vec![scratch_col_a_gt_500, scratch_col_b_lt_100], // Read from previous results
            TypeInfo::Boolean,
        ));

        // Create FILTER operator - it will evaluate predicate which writes to scratch_col_and
        let filter = VectorizedFilter::new(Box::new(scan), predicate);

        // Step 3: Create PROJECT for columns a, b
        let proj_col_a = Box::new(ColumnRef::new(0, TypeInfo::Int64)); // Column 'a'
        let proj_col_b = Box::new(ColumnRef::new(1, TypeInfo::Int64)); // Column 'b'
        
        let output_schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: TypeInfo::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: TypeInfo::Int64,
            },
        ]);

        let project = VectorizedProject::new(
            Box::new(filter),
            vec![
                ("a".to_string(), proj_col_a),
                ("b".to_string(), proj_col_b),
            ],
            output_schema.clone(),
        );

        // Return plan with PROJECT as the root
        Ok(VectorizedPlan::new(Box::new(project), output_schema))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{Field, TypeInfo};
    use crate::reader::{Tuple, TupleIteratorReader};

    #[test]
    fn test_compiler_basic() {
        // Create a schema matching what the compiler expects (a INT64, b INT64)
        let schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: TypeInfo::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: TypeInfo::Int64,
            },
        ]);

        // Create a dummy reader
        let tuples: Vec<Tuple> = vec![];
        let reader: Box<dyn BatchReader> = Box::new(TupleIteratorReader::new(
            Box::new(tuples.into_iter()),
            schema.clone(),
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
