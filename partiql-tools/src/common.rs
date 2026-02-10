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
use partiql_eval::CompilationCatalog;
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
use std::sync::Arc;

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

/// Simple compilation catalog for mem/ion data sources
///
/// Provides compile-time metadata using the two-phase catalog pattern.
pub struct SimpleCompilationCatalog {
    tables: FxHashMap<String, (EntryId, Arc<dyn DataSourceConfig>)>,
}

/// Wrapper to make CompiledSourceFactory implement DataSourceConfig
struct FactoryConfigWrapper {
    factory: CompiledSourceFactory,
}

impl DataSourceConfig for FactoryConfigWrapper {
    fn buffer_stability(&self) -> BufferStability {
        self.factory.buffer_stability()
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        self.factory.resolve(field_name)
    }
}

impl SimpleCompilationCatalog {
    fn new(tables: Vec<(String, CompiledSourceFactory)>) -> Self {
        let mut table_map = FxHashMap::default();

        for (idx, (name, factory)) in tables.into_iter().enumerate() {
            // Wrap the factory to implement DataSourceConfig
            let config: Arc<dyn DataSourceConfig> = Arc::new(FactoryConfigWrapper { factory });
            table_map.insert(name, (EntryId::from(idx as u64), config));
        }

        SimpleCompilationCatalog { tables: table_map }
    }
}

impl CompilationCatalog for SimpleCompilationCatalog {
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        if path.len() != 1 {
            return None;
        }

        let table_name = match &path[0] {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
        };

        self.tables
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(table_name))
            .map(|(_, (entry_id, config))| {
                // Return DataSourceHandle with EntryId and config
                DataSourceHandle::new(*entry_id, config.clone())
            })
    }
}

/// Simple execution catalog for mem/ion data sources
///
/// Creates actual DataSource instances at execution time.
/// Uses the ScanId-based pattern: customers inspect CompiledPlan during setup
/// and build internal mappings from ScanId to data sources.
pub struct SimpleExecutionCatalog {
    tables: FxHashMap<EntryId, CompiledSourceFactory>,
    /// Mapping from ScanId to (EntryId, ScanLayout) built during prepare()
    scan_mappings: FxHashMap<ScanId, (EntryId, ScanLayout)>,
}

impl SimpleExecutionCatalog {
    fn new(tables: Vec<(String, CompiledSourceFactory)>) -> Self {
        let mut table_map = FxHashMap::default();

        for (idx, (_name, factory)) in tables.into_iter().enumerate() {
            table_map.insert(EntryId::from(idx as u64), factory);
        }

        SimpleExecutionCatalog {
            tables: table_map,
            scan_mappings: FxHashMap::default(),
        }
    }
}

impl ExecutionCatalog for SimpleExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scan_mappings.clear();
        for (scan_id, entry_id, layout) in scans.iter() {
            self.scan_mappings
                .insert(scan_id, (entry_id, layout.clone()));
        }
    }

    fn create(&self, scan_id: ScanId) -> EvalResult<Box<dyn DataSource>> {
        // Look up the scan mapping
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!(
                "ScanId {:?} not found in catalog mappings. Did you call prepare()?",
                scan_id
            ))
        })?;

        // Look up the factory by entry_id
        let factory = self.tables.get(entry_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!(
                "Table with entry_id {:?} not found",
                entry_id
            ))
        })?;

        // Use the factory to create the DataSource
        factory.create(layout.clone())
    }
}

/// Create compilation catalog and execution catalog factory for simple data sources (mem/ion)
///
/// This provides the same two-phase catalog pattern as random_catalog,
/// enabling uniform architecture across all data source types.
///
/// # Arguments
/// * `tables` - Vector of (table_name, CompiledSourceFactory) tuples
///
/// # Returns
/// A tuple of (CompilationCatalog, SimpleExecutionCatalog)
/// Note: The execution catalog must have `prepare()` called with the CompiledPlan
/// before it can be used.
///
/// # Example
/// ```ignore
/// use partiql_eval::source::CompiledSourceFactory;
///
/// let (comp_catalog, mut exec_catalog) = simple_catalog(
///     vec![
///         ("data".to_string(), CompiledSourceFactory::mem(10_000, vec!["a".to_string(), "b".to_string()])),
///     ]
/// );
///
/// // After compilation, prepare the execution catalog
/// exec_catalog.prepare(&compiled_plan);
/// ```
pub fn simple_catalog(
    tables: Vec<(String, CompiledSourceFactory)>,
) -> (Arc<dyn CompilationCatalog>, SimpleExecutionCatalog) {
    let comp_catalog = Arc::new(SimpleCompilationCatalog::new(tables.clone()));
    let exec_catalog = SimpleExecutionCatalog::new(tables);
    (comp_catalog, exec_catalog)
}

// =============================================================================
// Random Data Source - Customer-Provided Reader Example
// =============================================================================
//
// This implementation demonstrates the two-phase catalog pattern for custom
// data sources. It shows how customers can:
// 1. Implement DataSource trait for their custom reader
// 2. Implement DataSourceConfig for compile-time metadata
// 3. Implement CompilationCatalog + ExecutionCatalog traits
// 4. Use ObjectId to enable catalog swapping
//
// This is a complete example using ONLY public APIs from partiql-eval.

use partiql_common::catalog::EntryId;
use partiql_eval::source::{
    BufferStability, CatalogScans, CompiledSourceFactory, DataSource, DataSourceConfig,
    PhysicalType, RegisterWriter, ScanId, ScanLayout, ScanSource, ScanSourceType,
};
use partiql_eval::{ExecutionCatalog, Result as EvalResult};
use rand::Rng;

/// Custom DataSource that generates random integer data
///
/// Demonstrates how customers implement the DataSource trait for their custom readers.
struct RandomDataSource {
    current_row: usize,
    total_rows: usize,
    layout: ScanLayout,
    num_columns: usize,
}

impl RandomDataSource {
    fn new(total_rows: usize, num_columns: usize, layout: ScanLayout) -> Self {
        RandomDataSource {
            current_row: 0,
            total_rows,
            layout,
            num_columns,
        }
    }
}

impl DataSource for RandomDataSource {
    fn open(&mut self) -> EvalResult<()> {
        self.current_row = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> EvalResult<bool> {
        if self.current_row >= self.total_rows {
            return Ok(false);
        }

        let mut rng = rand::thread_rng();

        // Generate random values for each projected column
        for proj in &self.layout.projections {
            let target = proj.target_slot;

            match &proj.source.source_type {
                ScanSourceType::ColumnIndex(index) => {
                    if *index < self.num_columns {
                        let random_value: i64 = rng.gen();
                        writer.put_i64(target, random_value)?;
                    } else {
                        return Err(partiql_eval::EngineError::ReaderError(format!(
                            "Column index {} out of bounds (max: {})",
                            index,
                            self.num_columns - 1
                        )));
                    }
                }
                ScanSourceType::WholeValue | ScanSourceType::FieldPath(_) => {
                    return Err(partiql_eval::EngineError::UnsupportedExpr(
                        "Random reader only supports ColumnIndex projections".to_string(),
                    ));
                }
            }
        }

        self.current_row += 1;
        Ok(true)
    }

    fn close(&mut self) -> EvalResult<()> {
        Ok(())
    }
}

/// Compile-time configuration for random data tables
///
/// Provides metadata about the table without needing access to actual data.
struct RandomTableConfig {
    column_names: Vec<String>,
}

impl RandomTableConfig {
    fn new(column_names: Vec<String>) -> Self {
        RandomTableConfig { column_names }
    }
}

impl DataSourceConfig for RandomTableConfig {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        self.column_names
            .iter()
            .position(|name| name.eq_ignore_ascii_case(field_name))
            .map(|index| ScanSource::column(index, PhysicalType::I64))
    }
}

/// Table metadata for random data source
#[derive(Clone)]
struct RandomTableMeta {
    entry_id: EntryId,
    num_rows: usize,
    column_names: Vec<String>,
}

/// Compilation catalog for random data sources
///
/// Provides compile-time metadata and assigns EntryIds for execution-time resolution.
pub struct RandomCompilationCatalog {
    tables: FxHashMap<String, RandomTableMeta>,
}

impl RandomCompilationCatalog {
    fn new(tables: Vec<(String, usize, Vec<String>)>) -> Self {
        let mut table_map = FxHashMap::default();

        for (idx, (name, num_rows, columns)) in tables.into_iter().enumerate() {
            table_map.insert(
                name,
                RandomTableMeta {
                    entry_id: EntryId::from(idx as u64),
                    num_rows,
                    column_names: columns,
                },
            );
        }

        RandomCompilationCatalog { tables: table_map }
    }
}

impl CompilationCatalog for RandomCompilationCatalog {
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        if path.len() != 1 {
            return None;
        }

        let table_name = match &path[0] {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
        };

        self.tables
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(table_name))
            .map(|(_, meta)| {
                let config = Arc::new(RandomTableConfig::new(meta.column_names.clone()));
                // Return DataSourceHandle with EntryId only (CatalogId comes from compiler)
                DataSourceHandle::new(meta.entry_id, config)
            })
    }
}

/// Execution catalog for random data sources
///
/// Creates actual DataSource instances at execution time, enabling different
/// data for the same compiled plan (catalog swapping).
/// Uses the ScanId-based pattern: customers inspect CompiledPlan during setup
/// and build internal mappings from ScanId to data sources.
pub struct RandomExecutionCatalog {
    tables: FxHashMap<EntryId, RandomTableMeta>,
    /// Mapping from ScanId to (EntryId, ScanLayout) built during prepare()
    scan_mappings: FxHashMap<ScanId, (EntryId, ScanLayout)>,
}

impl RandomExecutionCatalog {
    fn new(tables: Vec<(String, usize, Vec<String>)>) -> Self {
        let mut table_map = FxHashMap::default();

        for (idx, (_name, num_rows, columns)) in tables.into_iter().enumerate() {
            table_map.insert(
                EntryId::from(idx as u64),
                RandomTableMeta {
                    entry_id: EntryId::from(idx as u64),
                    num_rows,
                    column_names: columns,
                },
            );
        }

        RandomExecutionCatalog {
            tables: table_map,
            scan_mappings: FxHashMap::default(),
        }
    }
}

impl ExecutionCatalog for RandomExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scan_mappings.clear();
        for (scan_id, entry_id, layout) in scans.iter() {
            self.scan_mappings
                .insert(scan_id, (entry_id, layout.clone()));
        }
    }

    fn create(&self, scan_id: ScanId) -> EvalResult<Box<dyn DataSource>> {
        // Look up the scan mapping
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!(
                "ScanId {:?} not found in catalog mappings. Did you call prepare()?",
                scan_id
            ))
        })?;

        // Look up the table metadata by entry_id
        let meta = self.tables.get(entry_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!(
                "Table with entry_id {:?} not found",
                entry_id
            ))
        })?;

        Ok(Box::new(RandomDataSource::new(
            meta.num_rows,
            meta.column_names.len(),
            layout.clone(),
        )))
    }
}

/// Create compilation catalog and execution catalog factory for random data sources
///
/// This demonstrates the complete two-phase catalog pattern for custom readers.
///
/// # Arguments
/// * `tables` - Vector of (table_name, num_rows, column_names) tuples
///
/// # Returns
/// A tuple of (CompilationCatalog, RandomExecutionCatalog)
/// Note: The execution catalog must have `prepare()` called with the CompiledPlan
/// before it can be used.
///
/// # Example
/// ```ignore
/// let (comp_catalog, mut exec_catalog) = random_catalog(
///     vec![
///         ("users".to_string(), 10_000, vec!["id".to_string(), "age".to_string()]),
///         ("orders".to_string(), 50_000, vec!["order_id".to_string(), "amount".to_string()]),
///     ]
/// );
///
/// // Use in compilation - add_catalog returns the catalog_id
/// let mut comp_context = CompilationContext::new();
/// let catalog_id = comp_context.add_catalog("main", comp_catalog);
///
/// // Compile the plan
/// let compiled = compiler.compile(&logical)?;
///
/// // Prepare execution catalog with the compiled plan
/// exec_catalog.prepare(&compiled);
///
/// // Use in execution with the returned catalog_id
/// let mut exec_context = ExecutionContext::new();
/// exec_context.add_catalog(catalog_id, Arc::new(exec_catalog));
/// ```
pub fn random_catalog(
    tables: Vec<(String, usize, Vec<String>)>,
) -> (Arc<dyn CompilationCatalog>, RandomExecutionCatalog) {
    let comp_catalog = Arc::new(RandomCompilationCatalog::new(tables.clone()));
    let exec_catalog = RandomExecutionCatalog::new(tables);
    (comp_catalog, exec_catalog)
}
