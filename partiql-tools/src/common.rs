use indexmap::IndexSet;
use ion_rs::ReaderBuilder;
use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::call_defs::{CallDef, CallSpec};
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog, TypeEnvEntry};
use partiql_catalog::context::SessionContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_eval::error::PlanErr;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_eval::source::DataSourceHandle;
use partiql_eval::DataCatalog;
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical::LogicalPlan;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::{Parsed, Parser, ParserError};
use partiql_types::{PartiqlShapeBuilder, Static, StructConstraint, StructField, StructType};
use partiql_value::{BindingsName, Tuple, Value};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

/// Format a number with comma separators (e.g., 1000000 -> "1,000,000")
pub fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();

    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }

    result
}

/// Parse PartiQL query
pub fn parse(statement: &str) -> Result<Parsed<'_>, ParserError<'_>> {
    Parser::default().parse(statement)
}

/// Lower AST to logical plan
pub fn lower(
    catalog: &dyn SharedCatalog,
    parsed: &Parsed<'_>,
) -> Result<LogicalPlan<partiql_logical::BindingsOp>, AstTransformationError> {
    let planner = LogicalPlanner::new(catalog);
    planner.lower(parsed)
}

/// Compile logical plan to evaluation plan
pub fn compile(
    mode: EvaluationMode,
    catalog: &dyn SharedCatalog,
    logical: LogicalPlan<partiql_logical::BindingsOp>,
) -> Result<EvalPlan, PlanErr> {
    let mut planner = EvaluatorPlanner::new(mode, catalog);
    planner.compile(&logical)
}

/// Count total rows from a file-based data source
pub fn count_rows_from_file(data_source: &str, file_path: &str) -> usize {
    match data_source {
        "ion" | "ionb" => {
            // For Ion (text or binary), try to parse the filename pattern first (e.g., data_b4096_n244.ion or .10n)
            // If that fails, parse the Ion file to count rows
            if let Some(filename) = std::path::Path::new(file_path).file_name() {
                if let Some(name_str) = filename.to_str() {
                    // Try to extract batch_size and num_batches from filename
                    // Pattern: data_b<batch_size>_n<num_batches>.ion or .10n
                    if let Some(b_pos) = name_str.find("_b") {
                        if let Some(n_pos) = name_str.find("_n") {
                            if let Some(ext_pos) = name_str.rfind('.') {
                                if let Ok(batch_size) = name_str[b_pos + 2..n_pos].parse::<usize>()
                                {
                                    if let Ok(num_batches) =
                                        name_str[n_pos + 2..ext_pos].parse::<usize>()
                                    {
                                        return batch_size * num_batches;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Fallback: parse Ion file to count rows (more expensive)
            // For text Ion, use read_to_string; for binary Ion, use read (handles both)
            if data_source == "ion" {
                // Text Ion
                if let Ok(contents) = std::fs::read_to_string(file_path) {
                    if let Ok(reader) = ReaderBuilder::new().build(contents) {
                        if let Ok(decoder) = IonDecoderBuilder::new(
                            IonDecoderConfig::default().with_mode(Encoding::Ion),
                        )
                        .build(reader)
                        {
                            let mut count = 0;
                            for result in decoder {
                                if result.is_ok() {
                                    count += 1;
                                }
                            }
                            return count;
                        }
                    }
                }
            } else {
                // Binary Ion
                if let Ok(contents) = std::fs::read(file_path) {
                    if let Ok(reader) = ReaderBuilder::new().build(contents) {
                        if let Ok(decoder) = IonDecoderBuilder::new(
                            IonDecoderConfig::default().with_mode(Encoding::Ion),
                        )
                        .build(reader)
                        {
                            let mut count = 0;
                            for result in decoder {
                                if result.is_ok() {
                                    count += 1;
                                }
                            }
                            return count;
                        }
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

/// Table function that generates or reads data with fields 'a' and 'b'
#[derive(Debug)]
pub struct DataTableFunction {
    data_source: String,
    data_path: Option<String>,
}

impl DataTableFunction {
    pub fn new(data_source: String, data_path: Option<String>) -> Self {
        Self {
            data_source,
            data_path,
        }
    }
}

impl BaseTableFunctionInfo for DataTableFunction {
    fn call_def(&self) -> &CallDef {
        // Define the function signature (no arguments)
        static CALL_DEF: std::sync::OnceLock<CallDef> = std::sync::OnceLock::new();
        CALL_DEF.get_or_init(|| CallDef {
            names: vec!["data"],
            overloads: vec![CallSpec {
                input: vec![],
                output: Box::new(|args| {
                    partiql_logical::ValueExpr::Call(partiql_logical::CallExpr {
                        name: partiql_logical::CallName::ByName("data".to_string()),
                        arguments: args,
                    })
                }),
            }],
        })
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(DataTableExpr {
            data_source: self.data_source.clone(),
            data_path: self.data_path.clone(),
        })
    }
}

#[derive(Debug)]
pub struct DataTableExpr {
    data_source: String,
    data_path: Option<String>,
}

impl BaseTableExpr for DataTableExpr {
    fn evaluate<'c>(
        &self,
        _args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> BaseTableExprResult<'c> {
        match self.data_source.as_str() {
            "mem" => {
                // Generate in-memory data
                let total_rows = if let Ok(rows_str) = std::env::var("TOTAL_ROWS") {
                    rows_str.parse().unwrap_or_else(|_| {
                        let batch_size = std::env::var("BATCH_SIZE")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1024);
                        let num_batches = std::env::var("NUM_BATCHES")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(10_000);
                        batch_size * num_batches
                    })
                } else {
                    let batch_size = std::env::var("BATCH_SIZE")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1024);
                    let num_batches = std::env::var("NUM_BATCHES")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10_000);
                    batch_size * num_batches
                };

                let iter = (0..total_rows as i64).map(|i| {
                    let tuple =
                        Tuple::from([("a", Value::Integer(i)), ("b", Value::Integer(i + 100))]);
                    Ok(Value::Tuple(Box::new(tuple)))
                });

                Ok(Box::new(iter))
            }
            "ion" => {
                // Read from Ion text file - streaming approach
                let file_path = self
                    .data_path
                    .as_ref()
                    .expect("Ion data source requires --data-path");

                // Open the file
                let file = File::open(file_path)
                    .unwrap_or_else(|_| panic!("Failed to open Ion file: {}", file_path));

                let buf_reader = BufReader::new(file);

                // Create Ion reader from the file
                let reader = ReaderBuilder::new()
                    .build(buf_reader)
                    .expect("Failed to create Ion reader");

                // Create Ion decoder - this will stream values lazily
                let decoder =
                    IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                        .build(reader)
                        .expect("Failed to create Ion decoder");

                // Return the decoder directly as an iterator (lazy evaluation)
                Ok(Box::new(decoder.map(|result| {
                    result.map_err(|e| {
                        ExtensionResultError::ReadError(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Ion decode error: {:?}", e),
                        )))
                    })
                })))
            }
            "ionb" => {
                // Read from Ion binary file - streaming approach
                let file_path = self
                    .data_path
                    .as_ref()
                    .expect("Ion binary data source requires --data-path");

                // Open the file
                let file = File::open(file_path)
                    .unwrap_or_else(|_| panic!("Failed to open Ion binary file: {}", file_path));

                let buf_reader = BufReader::new(file);

                // Create Ion reader from the file (automatically detects binary format)
                let reader = ReaderBuilder::new()
                    .build(buf_reader)
                    .expect("Failed to create Ion reader");

                // Create Ion decoder - this will stream values lazily
                let decoder =
                    IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                        .build(reader)
                        .expect("Failed to create Ion decoder");

                // Return the decoder directly as an iterator (lazy evaluation)
                Ok(Box::new(decoder.map(|result| {
                    result.map_err(|e| {
                        ExtensionResultError::ReadError(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Ion decode error: {:?}", e),
                        )))
                    })
                })))
            }
            _ => {
                // Unsupported data source
                panic!(
                    "Unsupported data source: {}. Only 'mem', 'ion', and 'ionb' are supported.",
                    self.data_source
                )
            }
        }
    }
}

/// Create a catalog with the data table function
pub fn create_catalog(data_source: String, data_path: Option<String>) -> Box<dyn SharedCatalog> {
    let mut catalog = PartiqlCatalog::default();

    // Add the data table function
    let data_fn = TableFunction::new(Box::new(DataTableFunction::new(data_source, data_path)));
    catalog
        .add_table_function(data_fn)
        .expect("Failed to add table function");

    // Add type entry for "data" table so it can be referenced without parentheses
    let mut bld = PartiqlShapeBuilder::default();
    let fields = IndexSet::from([
        StructField::new("a", bld.new_static(Static::Int)),
        StructField::new("b", bld.new_static(Static::Int)),
    ]);
    let data_type = bld.new_struct(StructType::new(IndexSet::from([
        StructConstraint::Fields(fields),
        StructConstraint::Open(false),
    ])));

    let data_type_entry = TypeEnvEntry::new("data", &[], data_type);
    catalog
        .add_type_entry(data_type_entry)
        .expect("Failed to add type entry");

    Box::new(catalog.to_shared_catalog())
}

/// Simple catalog implementation for demonstration and testing
///
/// This catalog allows registering tables with DataSourceHandle instances,
/// making it easy to test catalog-based scans without complex setup.
pub struct SimpleDataCatalog {
    catalog_name: String,
    tables: FxHashMap<String, DataSourceHandle>,
}

impl SimpleDataCatalog {
    /// Create a new SimpleDataCatalog with the given name
    pub fn new(name: impl Into<String>) -> Self {
        SimpleDataCatalog {
            catalog_name: name.into(),
            tables: FxHashMap::default(),
        }
    }

    /// Add a table to this catalog
    ///
    /// # Example
    /// ```ignore
    /// let mut catalog = SimpleDataCatalog::new("my_catalog");
    /// catalog.add_table("users", DataSourceHandle::mem(1000, vec!["a".to_string(), "b".to_string()]));
    /// catalog.add_table("orders", DataSourceHandle::ion("data/orders.ion".to_string()));
    /// ```
    pub fn add_table(&mut self, name: impl Into<String>, data_source: DataSourceHandle) {
        self.tables.insert(name.into(), data_source);
    }

    /// Builder-style method to add a table
    pub fn with_table(mut self, name: impl Into<String>, data_source: DataSourceHandle) -> Self {
        self.add_table(name, data_source);
        self
    }
}

impl DataCatalog for SimpleDataCatalog {
    fn name(&self) -> &str {
        &self.catalog_name
    }

    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        // Support simple single-component paths like "table_name"
        if path.len() == 1 {
            let table_name = match &path[0] {
                BindingsName::CaseSensitive(s) => s.as_ref(),
                BindingsName::CaseInsensitive(s) => s.as_ref(),
            };

            // Case-insensitive lookup
            return self
                .tables
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(table_name))
                .map(|(_, factory)| factory.clone());
        }

        // For multi-component paths, try joining with dots
        // e.g., ["schema", "table"] -> "schema.table"
        if path.len() > 1 {
            let full_path = path
                .iter()
                .map(|component| match component {
                    BindingsName::CaseSensitive(s) => s.as_ref(),
                    BindingsName::CaseInsensitive(s) => s.as_ref(),
                })
                .collect::<Vec<_>>()
                .join(".");

            return self
                .tables
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&full_path))
                .map(|(_, factory)| factory.clone());
        }

        None
    }
}
