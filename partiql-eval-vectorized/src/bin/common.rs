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
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_eval::eval::EvalPlan;
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical::LogicalPlan;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::{Parsed, Parser, ParserError};
use partiql_types::{PartiqlShapeBuilder, StructField, Static, StructType, StructConstraint};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use indexmap::IndexSet;

/// Format a number with comma separators (e.g., 1000000 -> "1,000,000")
pub fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
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
        "arrow" => {
            use arrow_ipc::reader::FileReader;
            use std::fs::File;
            if let Ok(file) = File::open(file_path) {
                if let Ok(reader) = FileReader::try_new(file, None) {
                    let mut total = 0;
                    for batch_result in reader {
                        if let Ok(batch) = batch_result {
                            total += batch.num_rows();
                        }
                    }
                    return total;
                }
            }
            0
        }
        "parquet" => {
            use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
            use std::fs::File;
            if let Ok(file) = File::open(file_path) {
                if let Ok(builder) = ParquetRecordBatchReaderBuilder::try_new(file) {
                    if let Ok(reader) = builder.build() {
                        let mut total = 0;
                        for batch_result in reader {
                            if let Ok(batch) = batch_result {
                                total += batch.num_rows();
                            }
                        }
                        return total;
                    }
                }
            }
            0
        }
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
                                if let Ok(batch_size) = name_str[b_pos + 2..n_pos].parse::<usize>() {
                                    if let Ok(num_batches) = name_str[n_pos + 2..ext_pos].parse::<usize>() {
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
                        if let Ok(mut decoder) = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                            .build(reader) {
                            let mut count = 0;
                            while let Some(result) = decoder.next() {
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
                        if let Ok(mut decoder) = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                            .build(reader) {
                            let mut count = 0;
                            while let Some(result) = decoder.next() {
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
                    let tuple = Tuple::from([("a", Value::Integer(i)), ("b", Value::Integer(i + 100))]);
                    Ok(Value::Tuple(Box::new(tuple)))
                });

                Ok(Box::new(iter))
            }
            "ion" => {
                // Read from Ion text file - streaming approach
                let file_path = self.data_path.as_ref()
                    .expect("Ion data source requires --data-path");

                // Open the file
                let file = File::open(file_path)
                    .expect(&format!("Failed to open Ion file: {}", file_path));
                
                let buf_reader = BufReader::new(file);

                // Create Ion reader from the file
                let reader = ReaderBuilder::new().build(buf_reader)
                    .expect("Failed to create Ion reader");

                // Create Ion decoder - this will stream values lazily
                let decoder = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                    .build(reader)
                    .expect("Failed to create Ion decoder");

                // Return the decoder directly as an iterator (lazy evaluation)
                Ok(Box::new(decoder.map(|result| {
                    result.map_err(|e| ExtensionResultError::ReadError(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Ion decode error: {:?}", e)
                    ))))
                })))
            }
            "ionb" => {
                // Read from Ion binary file - streaming approach
                let file_path = self.data_path.as_ref()
                    .expect("Ion binary data source requires --data-path");

                // Open the file
                let file = File::open(file_path)
                    .expect(&format!("Failed to open Ion binary file: {}", file_path));
                
                let buf_reader = BufReader::new(file);

                // Create Ion reader from the file (automatically detects binary format)
                let reader = ReaderBuilder::new().build(buf_reader)
                    .expect("Failed to create Ion reader");

                // Create Ion decoder - this will stream values lazily
                let decoder = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                    .build(reader)
                    .expect("Failed to create Ion decoder");

                // Return the decoder directly as an iterator (lazy evaluation)
                Ok(Box::new(decoder.map(|result| {
                    result.map_err(|e| ExtensionResultError::ReadError(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Ion decode error: {:?}", e)
                    ))))
                })))
            }
            "arrow" => {
                // Read from Arrow IPC file - streaming approach
                use arrow_ipc::reader::FileReader;
                
                let file_path = self.data_path.as_ref()
                    .expect("Arrow data source requires --data-path");

                // Open the file
                let file = File::open(file_path)
                    .expect(&format!("Failed to open Arrow file: {}", file_path));

                // Create Arrow file reader
                let reader = FileReader::try_new(file, None)
                    .expect("Failed to create Arrow reader");

                // Create an iterator that converts Arrow batches to PartiQL tuples
                let iter = ArrowBatchIterator::new(reader);
                
                Ok(Box::new(iter))
            }
            "parquet" => {
                // Read from Parquet file - streaming approach
                use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
                
                let file_path = self.data_path.as_ref()
                    .expect("Parquet data source requires --data-path");

                // Open the file
                let file = File::open(file_path)
                    .expect(&format!("Failed to open Parquet file: {}", file_path));

                // Create Parquet reader
                let builder = ParquetRecordBatchReaderBuilder::try_new(file)
                    .expect("Failed to create Parquet reader");
                
                let reader = builder.build()
                    .expect("Failed to build Parquet reader");

                // Create an iterator that converts Parquet batches to PartiQL tuples
                let iter = ParquetBatchIterator::new(reader);
                
                Ok(Box::new(iter))
            }
            _ => {
                // Unsupported data source
                panic!("Unsupported data source: {}. Only 'mem', 'ion', 'ionb', 'arrow', and 'parquet' are supported for non-vectorized evaluator.", self.data_source)
            }
        }
    }
}

/// Iterator that converts Arrow record batches to PartiQL tuples
struct ArrowBatchIterator {
    reader: arrow_ipc::reader::FileReader<File>,
    current_batch: Option<arrow::record_batch::RecordBatch>,
    current_row: usize,
}

impl ArrowBatchIterator {
    fn new(reader: arrow_ipc::reader::FileReader<File>) -> Self {
        Self {
            reader,
            current_batch: None,
            current_row: 0,
        }
    }
}

impl Iterator for ArrowBatchIterator {
    type Item = Result<Value, ExtensionResultError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current batch, try to get the next row
            if let Some(batch) = &self.current_batch {
                if self.current_row < batch.num_rows() {
                    // Extract values from columns 0 (a) and 1 (b)
                    let col_a = batch.column(0);
                    let col_b = batch.column(1);
                    
                    // Downcast to Int64Array
                    let a_array = col_a.as_any().downcast_ref::<arrow::array::Int64Array>()
                        .expect("Column 'a' should be Int64");
                    let b_array = col_b.as_any().downcast_ref::<arrow::array::Int64Array>()
                        .expect("Column 'b' should be Int64");
                    
                    // Get values at current row
                    let a_val = a_array.value(self.current_row);
                    let b_val = b_array.value(self.current_row);
                    
                    self.current_row += 1;
                    
                    // Create tuple
                    let tuple = Tuple::from([
                        ("a", Value::Integer(a_val)),
                        ("b", Value::Integer(b_val)),
                    ]);
                    
                    return Some(Ok(Value::Tuple(Box::new(tuple))));
                }
            }
            
            // Need to fetch next batch
            match self.reader.next() {
                Some(Ok(batch)) => {
                    self.current_batch = Some(batch);
                    self.current_row = 0;
                    // Continue loop to process first row of new batch
                }
                Some(Err(e)) => {
                    return Some(Err(ExtensionResultError::ReadError(Box::new(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Arrow read error: {}", e)
                        )
                    ))));
                }
                None => {
                    // No more batches
                    return None;
                }
            }
        }
    }
}

/// Iterator that converts Parquet record batches to PartiQL tuples
struct ParquetBatchIterator {
    reader: parquet::arrow::arrow_reader::ParquetRecordBatchReader,
    current_batch: Option<arrow::record_batch::RecordBatch>,
    current_row: usize,
}

impl ParquetBatchIterator {
    fn new(reader: parquet::arrow::arrow_reader::ParquetRecordBatchReader) -> Self {
        Self {
            reader,
            current_batch: None,
            current_row: 0,
        }
    }
}

impl Iterator for ParquetBatchIterator {
    type Item = Result<Value, ExtensionResultError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current batch, try to get the next row
            if let Some(batch) = &self.current_batch {
                if self.current_row < batch.num_rows() {
                    // Extract values from columns 0 (a) and 1 (b)
                    let col_a = batch.column(0);
                    let col_b = batch.column(1);
                    
                    // Downcast to Int64Array
                    let a_array = col_a.as_any().downcast_ref::<arrow::array::Int64Array>()
                        .expect("Column 'a' should be Int64");
                    let b_array = col_b.as_any().downcast_ref::<arrow::array::Int64Array>()
                        .expect("Column 'b' should be Int64");
                    
                    // Get values at current row
                    let a_val = a_array.value(self.current_row);
                    let b_val = b_array.value(self.current_row);
                    
                    self.current_row += 1;
                    
                    // Create tuple
                    let tuple = Tuple::from([
                        ("a", Value::Integer(a_val)),
                        ("b", Value::Integer(b_val)),
                    ]);
                    
                    return Some(Ok(Value::Tuple(Box::new(tuple))));
                }
            }
            
            // Need to fetch next batch
            match self.reader.next() {
                Some(Ok(batch)) => {
                    self.current_batch = Some(batch);
                    self.current_row = 0;
                    // Continue loop to process first row of new batch
                }
                Some(Err(e)) => {
                    return Some(Err(ExtensionResultError::ReadError(Box::new(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Parquet read error: {}", e)
                        )
                    ))));
                }
                None => {
                    // No more batches
                    return None;
                }
            }
        }
    }
}

/// Create a catalog with the data table function
pub fn create_catalog(data_source: String, data_path: Option<String>) -> Box<dyn SharedCatalog> {
    let mut catalog = PartiqlCatalog::default();
    
    // Add the data table function
    let data_fn = TableFunction::new(Box::new(DataTableFunction::new(
        data_source,
        data_path,
    )));
    catalog.add_table_function(data_fn).expect("Failed to add table function");
    
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
    catalog.add_type_entry(data_type_entry).expect("Failed to add type entry");
    
    Box::new(catalog.to_shared_catalog())
}
