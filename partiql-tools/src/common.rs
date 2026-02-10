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

// =============================================================================
// Random Data Source - Customer-Provided Reader Example
// =============================================================================
//
// This implementation demonstrates the two-phase catalog pattern for custom
// data sources. It shows how customers can:
// 1. Implement DataSource trait for their custom reader
// 2. Implement DataSourceMetadata for compile-time metadata
// 3. Implement CompilationCatalog + ExecutionCatalog traits
// 4. Use ObjectId to enable catalog swapping
//
// This is a complete example using ONLY public APIs from partiql-eval.

use partiql_common::catalog::EntryId;
use partiql_eval::source::{
    BufferStability, CatalogScans, DataSource, DataSourceMetadata, PhysicalType, RegisterWriter,
    ScanId, ScanLayout, ScanSource, ScanSourceType,
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

impl DataSourceMetadata for RandomTableConfig {
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
                let metadata: Arc<dyn DataSourceMetadata> =
                    Arc::new(RandomTableConfig::new(meta.column_names.clone()));
                // Return DataSourceHandle with EntryId only (CatalogId comes from compiler)
                DataSourceHandle::new(meta.entry_id, metadata)
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

// =============================================================================
// In-Memory Generated Data Source
// =============================================================================
//
// Generates sequential integer data on-the-fly. Demonstrates a simple DataSource
// implementation that doesn't require external data.

/// In-memory row reader that generates rows on-the-fly
///
/// Generates sequential integer data. All columns start at 0 and increment
/// by 1 for each row.
struct InMemGeneratedReader {
    current_row: i64,
    total_rows: usize,
    layout: ScanLayout,
    num_columns: usize,
}

impl InMemGeneratedReader {
    fn new(total_rows: usize, num_columns: usize, layout: ScanLayout) -> Self {
        InMemGeneratedReader {
            current_row: 0,
            total_rows,
            layout,
            num_columns,
        }
    }
}

impl DataSource for InMemGeneratedReader {
    fn open(&mut self) -> EvalResult<()> {
        self.current_row = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> EvalResult<bool> {
        if self.current_row >= self.total_rows as i64 {
            return Ok(false);
        }

        let row_value = self.current_row;

        for proj in &self.layout.projections {
            let target = proj.target_slot;

            match &proj.source.source_type {
                ScanSourceType::ColumnIndex(index) => {
                    if *index < self.num_columns {
                        writer.put_i64(target, row_value)?;
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
                        "InMem reader only supports ColumnIndex projections".to_string(),
                    ));
                }
            };
        }

        self.current_row += 1;
        Ok(true)
    }

    fn close(&mut self) -> EvalResult<()> {
        Ok(())
    }
}

/// Compile-time configuration for in-memory generated tables
struct InMemTableConfig {
    column_names: Vec<String>,
}

impl InMemTableConfig {
    fn new(column_names: Vec<String>) -> Self {
        InMemTableConfig { column_names }
    }
}

impl DataSourceMetadata for InMemTableConfig {
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

// =============================================================================
// Ion Streaming Data Source
// =============================================================================
//
// High-performance streaming Ion reader with projection pushdown.
// Reads Ion data directly into row slots, avoiding materialization to Value objects.

use ion_rs::{IonReader, IonType, ReaderBuilder as IonReaderBuilder};

/// Streaming Ion text reader with projection pushdown
///
/// Uses the ion_rs streaming API to read Ion data directly into row slots.
///
/// # Performance Characteristics
/// - Zero-copy for primitives (i64, f64, bool)
/// - Minimal string allocations (only for projected string fields)
/// - True projection pushdown (only reads requested fields)
/// - Uses FxHashMap for O(1) field lookups
pub struct IonDataSource {
    path: String,
    reader: Option<Box<ion_rs::Reader<'static>>>,
    field_to_slot: FxHashMap<String, u16>,
    string_storage: Vec<String>,
}

impl IonDataSource {
    fn new(path: String, layout: ScanLayout) -> Self {
        let mut field_to_slot = FxHashMap::default();
        for proj in &layout.projections {
            if let ScanSourceType::FieldPath(field_name) = &proj.source.source_type {
                field_to_slot.insert(field_name.clone(), proj.target_slot);
            }
        }

        IonDataSource {
            path,
            reader: None,
            field_to_slot,
            string_storage: Vec::new(),
        }
    }
}

impl DataSource for IonDataSource {
    fn open(&mut self) -> EvalResult<()> {
        let file = File::open(&self.path)
            .map_err(|e| partiql_eval::EngineError::ReaderError(format!("ion open failed: {e}")))?;
        let buf_reader = BufReader::new(file);

        let ion_reader = IonReaderBuilder::new().build(buf_reader).map_err(|e| {
            partiql_eval::EngineError::ReaderError(format!("ion reader creation failed: {e}"))
        })?;

        let boxed_reader: Box<ion_rs::Reader<'static>> =
            unsafe { std::mem::transmute(Box::new(ion_reader)) };

        self.reader = Some(boxed_reader);
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> EvalResult<bool> {
        let reader = match self.reader.as_mut() {
            Some(r) => r,
            None => return Ok(false),
        };

        self.string_storage.clear();

        let stream_item = reader
            .next()
            .map_err(|e| partiql_eval::EngineError::ReaderError(format!("ion read failed: {e}")))?;

        match stream_item {
            ion_rs::StreamItem::Value(_ion_type) => {
                reader.step_in().map_err(|e| {
                    partiql_eval::EngineError::ReaderError(format!(
                        "failed to step into struct: {e}"
                    ))
                })?;

                loop {
                    match reader.next().map_err(|e| {
                        partiql_eval::EngineError::ReaderError(format!(
                            "error reading struct field: {e}"
                        ))
                    })? {
                        ion_rs::StreamItem::Value(ion_type) => {
                            let field_name = reader.field_name().map_err(|e| {
                                partiql_eval::EngineError::ReaderError(format!(
                                    "failed to get field name: {e}"
                                ))
                            })?;

                            let field_text = field_name.text().ok_or_else(|| {
                                partiql_eval::EngineError::ReaderError(
                                    "field name has no text".to_string(),
                                )
                            })?;

                            if let Some(&target_slot) = self.field_to_slot.get(field_text) {
                                match ion_type {
                                    IonType::Int => {
                                        let val = reader.read_i64().map_err(|e| {
                                            partiql_eval::EngineError::ReaderError(format!(
                                                "failed to read i64: {e}"
                                            ))
                                        })?;
                                        writer.put_i64(target_slot, val)?;
                                    }
                                    IonType::Float => {
                                        let val = reader.read_f64().map_err(|e| {
                                            partiql_eval::EngineError::ReaderError(format!(
                                                "failed to read f64: {e}"
                                            ))
                                        })?;
                                        writer.put_f64(target_slot, val)?;
                                    }
                                    IonType::Bool => {
                                        let val = reader.read_bool().map_err(|e| {
                                            partiql_eval::EngineError::ReaderError(format!(
                                                "failed to read bool: {e}"
                                            ))
                                        })?;
                                        writer.put_bool(target_slot, val)?;
                                    }
                                    IonType::String => {
                                        let val = reader.read_str().map_err(|e| {
                                            partiql_eval::EngineError::ReaderError(format!(
                                                "failed to read string: {e}"
                                            ))
                                        })?;
                                        self.string_storage.push(val.to_string());
                                        let idx = self.string_storage.len() - 1;
                                        let str_ref = unsafe {
                                            std::mem::transmute::<&str, &str>(
                                                self.string_storage[idx].as_str(),
                                            )
                                        };
                                        writer.put_str(target_slot, str_ref)?;
                                    }
                                    IonType::Null => {
                                        writer.put_null(target_slot)?;
                                    }
                                    other_type => {
                                        return Err(partiql_eval::EngineError::ReaderError(
                                            format!(
                                                "unsupported ion type for projection: {:?}",
                                                other_type
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                        ion_rs::StreamItem::Nothing => break,
                        ion_rs::StreamItem::Null(_) => continue,
                    }
                }

                reader.step_out().map_err(|e| {
                    partiql_eval::EngineError::ReaderError(format!(
                        "failed to step out of struct: {e}"
                    ))
                })?;

                Ok(true)
            }
            ion_rs::StreamItem::Nothing => Ok(false),
            ion_rs::StreamItem::Null(_) => self.next_row(writer),
        }
    }

    fn close(&mut self) -> EvalResult<()> {
        self.reader = None;
        self.string_storage.clear();
        self.field_to_slot.clear();
        Ok(())
    }
}

/// Compile-time configuration for Ion data sources
struct IonTableConfig {
    // Ion is schemaless - all fields are accepted at compile time
}

impl IonTableConfig {
    fn new() -> Self {
        IonTableConfig {}
    }
}

impl DataSourceMetadata for IonTableConfig {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        // Ion reader accepts any field name at compile time
        // Type is Dynamic since Ion is schemaless
        Some(ScanSource::field(field_name, PhysicalType::Dynamic))
    }
}

// =============================================================================
// Simple Catalog - Unified Compilation/Execution Catalog
// =============================================================================
//
// A simplified catalog that stores table metadata and creates data sources.
// Supports both in-memory and Ion data sources.

/// Factory configuration for creating data sources
#[derive(Clone)]
pub enum CompiledSourceFactory {
    /// In-memory generated data
    Mem {
        total_rows: usize,
        column_names: Vec<String>,
    },
    /// Ion file data
    Ion { path: String },
}

impl CompiledSourceFactory {
    /// Create a factory for in-memory generated data
    pub fn mem(total_rows: usize, column_names: Vec<String>) -> Self {
        CompiledSourceFactory::Mem {
            total_rows,
            column_names,
        }
    }

    /// Create a factory for Ion file data
    pub fn ion(path: String) -> Self {
        CompiledSourceFactory::Ion { path }
    }
}

/// Table metadata for simple catalog
struct SimpleTableMeta {
    entry_id: EntryId,
    factory: CompiledSourceFactory,
}

/// Compilation catalog for simple data sources
pub struct SimpleCompilationCatalog {
    tables: FxHashMap<String, SimpleTableMeta>,
}

impl SimpleCompilationCatalog {
    fn new(tables: Vec<(String, CompiledSourceFactory)>) -> Self {
        let mut table_map = FxHashMap::default();

        for (idx, (name, factory)) in tables.into_iter().enumerate() {
            table_map.insert(
                name,
                SimpleTableMeta {
                    entry_id: EntryId::from(idx as u64),
                    factory,
                },
            );
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
            .map(|(_, meta)| {
                let metadata: Arc<dyn DataSourceMetadata> = match &meta.factory {
                    CompiledSourceFactory::Mem { column_names, .. } => {
                        Arc::new(InMemTableConfig::new(column_names.clone()))
                    }
                    CompiledSourceFactory::Ion { .. } => Arc::new(IonTableConfig::new()),
                };
                DataSourceHandle::new(meta.entry_id, metadata)
            })
    }
}

/// Execution catalog for simple data sources
pub struct SimpleExecutionCatalog {
    tables: FxHashMap<EntryId, SimpleTableMeta>,
    scan_mappings: FxHashMap<ScanId, (EntryId, ScanLayout)>,
}

impl SimpleExecutionCatalog {
    fn new(tables: Vec<(String, CompiledSourceFactory)>) -> Self {
        let mut table_map = FxHashMap::default();

        for (idx, (_name, factory)) in tables.into_iter().enumerate() {
            let entry_id = EntryId::from(idx as u64);
            table_map.insert(entry_id, SimpleTableMeta { entry_id, factory });
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
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!(
                "ScanId {:?} not found in catalog mappings. Did you call prepare()?",
                scan_id
            ))
        })?;

        let meta = self.tables.get(entry_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!(
                "Table with entry_id {:?} not found",
                entry_id
            ))
        })?;

        match &meta.factory {
            CompiledSourceFactory::Mem {
                total_rows,
                column_names,
            } => Ok(Box::new(InMemGeneratedReader::new(
                *total_rows,
                column_names.len(),
                layout.clone(),
            ))),
            CompiledSourceFactory::Ion { path } => {
                Ok(Box::new(IonDataSource::new(path.clone(), layout.clone())))
            }
        }
    }
}

/// Create compilation and execution catalogs for simple data sources
///
/// # Arguments
/// * `tables` - Vector of (table_name, CompiledSourceFactory) tuples
///
/// # Returns
/// A tuple of (CompilationCatalog, SimpleExecutionCatalog)
pub fn simple_catalog(
    tables: Vec<(String, CompiledSourceFactory)>,
) -> (Arc<dyn CompilationCatalog>, SimpleExecutionCatalog) {
    let comp_catalog = Arc::new(SimpleCompilationCatalog::new(tables.clone()));
    let exec_catalog = SimpleExecutionCatalog::new(tables);
    (comp_catalog, exec_catalog)
}
