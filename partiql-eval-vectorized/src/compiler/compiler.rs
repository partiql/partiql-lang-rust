use crate::compiler::{ColumnRequirements, LogicalToPhysical, VectorizedPlan};
use crate::error::PlanError;
use crate::functions::VectorizedFnRegistry;
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

    /// Compile a logical plan into a vectorized plan using two-pass strategy:
    /// Pass 1: Analyze column requirements (projection pushdown)
    /// Pass 2: Build physical operators
    pub fn compile(
        &mut self,
        logical: &LogicalPlan<BindingsOp>,
    ) -> Result<VectorizedPlan, PlanError> {
        // Pass 1: Analyze column requirements
        let mut col_reqs = ColumnRequirements::new();
        col_reqs.analyze(logical)?;

        // Pass 2: Build physical operators
        let context = std::mem::replace(&mut self.context, CompilerContext::new());
        let translator = LogicalToPhysical::new(context, col_reqs);
        let root_op = translator.translate(logical)?;
        let output_schema = root_op.output_schema().clone();

        Ok(VectorizedPlan::new(root_op, output_schema))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{Field, LogicalType, SourceTypeDef};
    use crate::reader::InMemoryGeneratedReader;
    use partiql_logical::{Filter, PathComponent, Project, Scan, ValueExpr, VarRefType};
    use partiql_value::BindingsName;

    #[test]
    fn test_compiler_basic() {
        // Create a schema matching what the compiler expects (a INT64, b INT64)
        let _schema = SourceTypeDef::new(vec![
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
        let reader: Box<dyn BatchReader> =
            Box::new(InMemoryGeneratedReader::new());

        // Create compiler context with data source
        let context = CompilerContext::new().with_data_source("data".to_string(), reader);

        // Create compiler
        let mut compiler = Compiler::new(context);

        // Create a logical plan: SELECT a, b FROM data AS x WHERE true
        let mut logical = LogicalPlan::new();

        // Scan(data AS x)
        let scan = logical.add_operator(BindingsOp::Scan(Scan {
            expr: ValueExpr::VarRef(
                BindingsName::CaseInsensitive("data".into()),
                VarRefType::Global,
            ),
            as_key: "x".to_string(),
            at_key: None,
        }));

        // Filter(true) - stub filter for now
        let filter = logical.add_operator(BindingsOp::Filter(Filter {
            expr: ValueExpr::Lit(Box::new(partiql_logical::Lit::Bool(true))),
        }));

        // Project(a, b)
        let project = logical.add_operator(BindingsOp::Project(Project {
            exprs: vec![
                (
                    "a".to_string(),
                    ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(
                            BindingsName::CaseInsensitive("x".into()),
                            VarRefType::Local,
                        )),
                        vec![PathComponent::Key(BindingsName::CaseInsensitive("a".into()))],
                    ),
                ),
                (
                    "b".to_string(),
                    ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(
                            BindingsName::CaseInsensitive("x".into()),
                            VarRefType::Local,
                        )),
                        vec![PathComponent::Key(BindingsName::CaseInsensitive("b".into()))],
                    ),
                ),
            ],
        }));

        // Sink
        let sink = logical.add_operator(BindingsOp::Sink);

        // Connect operators: Scan -> Filter -> Project -> Sink
        logical.add_flow(scan, filter);
        logical.add_flow(filter, project);
        logical.add_flow(project, sink);

        // Compile
        let result = compiler.compile(&logical);

        // Should succeed and return a plan
        assert!(result.is_ok());
        let plan = result.unwrap();

        // Verify output schema has 2 columns (a, b) as expected
        assert_eq!(plan.output_schema().field_count(), 2);
    }
}
