use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::call_defs::{CallDef, CallSpec};
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog};
use partiql_catalog::context::SessionContext;
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_logical_planner::{LogicalPlanner, VarRefResolution};
use partiql_parser::{Parsed, Parser, ParserError};
use partiql_value::{BindingsName, Value};
use partiql_vm::source::DataSourceHandle;
use partiql_vm::source::{
    TableFunction as VmTableFunction, TableFunctionHandle as VmTableFunctionHandle,
};
use partiql_vm::CompilationCatalog;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

/// Parse PartiQL query
pub fn parse(statement: &str) -> Result<Parsed<'_>, ParserError<'_>> {
    Parser::default().parse(statement)
}

/// Parse a `;`-separated PartiQL script into its individual statements.
pub fn parse_statements(script: &str) -> Result<Parsed<'_>, ParserError<'_>> {
    Parser::default().parse_statements(script)
}

/// Lower a single statement to a top-level logical statement (query or DDL).
///
/// This preserves the statement category, so callers can distinguish a
/// `Query` plan from a `CreateTableAs` / `CreateTable` DDL node.
pub fn lower_statement(
    catalog: &dyn SharedCatalog,
    stmt: &partiql_ast::ast::AstNode<partiql_ast::ast::Statement>,
) -> Result<partiql_logical::LogicalStatement, AstTransformationError> {
    let planner = LogicalPlanner::with_var_resolution(catalog, VarRefResolution::Static);
    planner.lower_statement(stmt)
}

/// Create a frontend catalog with only table function stubs.
/// Used by pqlite when table functions are the sole data source mechanism.
pub fn create_table_fn_catalog() -> Box<dyn SharedCatalog> {
    let mut catalog = PartiqlCatalog::default();
    register_table_fn_stubs(&mut catalog);
    Box::new(catalog.to_shared_catalog())
}

fn register_table_fn_stubs(catalog: &mut PartiqlCatalog) {
    catalog
        .add_table_function(TableFunction::new(Box::new(StubTableFn::new(
            "rand",
            vec![
                partiql_catalog::call_defs::CallSpecArg::Positional,
                partiql_catalog::call_defs::CallSpecArg::Positional,
            ],
        ))))
        .expect("Failed to add rand table function");
    catalog
        .add_table_function(TableFunction::new(Box::new(StubTableFn::new(
            "mem",
            vec![
                partiql_catalog::call_defs::CallSpecArg::Positional,
                partiql_catalog::call_defs::CallSpecArg::Positional,
            ],
        ))))
        .expect("Failed to add mem table function");
    catalog
        .add_table_function(TableFunction::new(Box::new(StubTableFn::new(
            "scan_ion",
            vec![partiql_catalog::call_defs::CallSpecArg::Positional],
        ))))
        .expect("Failed to add scan_ion table function");
}

/// Stub table function for the frontend planner. Only provides `call_def()` so
/// the planner can validate the call and produce a `CallExpr` in the logical plan.
/// The actual execution is handled by the VM's `TableFunction` implementations.
#[derive(Debug)]
struct StubTableFn {
    call_def: CallDef,
}

impl StubTableFn {
    fn new(name: &'static str, input: Vec<partiql_catalog::call_defs::CallSpecArg>) -> Self {
        StubTableFn {
            call_def: CallDef {
                names: vec![name],
                overloads: vec![CallSpec {
                    input,
                    output: Box::new(move |args| {
                        partiql_logical::ValueExpr::Call(partiql_logical::CallExpr {
                            name: partiql_logical::CallName::ByName(name.to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl BaseTableFunctionInfo for StubTableFn {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(StubTableExpr)
    }
}

#[derive(Debug)]
struct StubTableExpr;

impl BaseTableExpr for StubTableExpr {
    fn evaluate<'c>(
        &self,
        _args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> BaseTableExprResult<'c> {
        unreachable!("StubTableExpr should never be evaluated — VM handles execution")
    }
}

// =============================================================================
// Random Data Source
// =============================================================================
//
// Generates random integer data. Implemented using only public partiql-vm APIs.

use partiql_vm::source::{
    BufferStability, DataSource, DataSourceMetadata, PhysicalType, RegisterWriter, ScanLayout,
    ScanSource, ScanSourceType,
};
use partiql_vm::value::RegisterReader;
use partiql_vm::Result as EvalResult;

type SlotId = u16;
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
                        writer.write_i64(target, random_value)?;
                    } else {
                        return Err(partiql_vm::EngineError::ReaderError(format!(
                            "Column index {} out of bounds (max: {})",
                            index,
                            self.num_columns - 1
                        )));
                    }
                }
                ScanSourceType::WholeValue => {
                    return Err(partiql_vm::EngineError::UnsupportedExpr(
                        "Random reader only supports ColumnIndex projections".to_string(),
                    ));
                }
                ScanSourceType::FieldPath(_) => {
                    return Err(partiql_vm::EngineError::UnsupportedExpr(
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
                        writer.write_i64(target, row_value)?;
                    } else {
                        return Err(partiql_vm::EngineError::ReaderError(format!(
                            "Column index {} out of bounds (max: {})",
                            index,
                            self.num_columns - 1
                        )));
                    }
                }
                ScanSourceType::WholeValue => {
                    // Build a tuple with all columns
                    let target = proj.target_slot;
                    let mut vw = writer.value_writer(target)?;
                    vw.step_in_tuple()?;
                    for col_idx in 0..self.num_columns {
                        let col_name = match col_idx {
                            0 => "a",
                            1 => "b",
                            _ => "unknown",
                        };
                        vw.put_field_name(col_name)?;
                        vw.put_i64(row_value + col_idx as i64)?;
                    }
                    vw.step_out()?;
                    vw.finish()?;
                }
                ScanSourceType::FieldPath(_) => {
                    return Err(partiql_vm::EngineError::UnsupportedExpr(
                        "InMem reader only supports ColumnIndex and WholeValue projections"
                            .to_string(),
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
    /// If set, we're in whole-value mode: build a Tuple into this slot
    whole_value_slot: Option<u16>,
    string_storage: Vec<String>,
}

impl IonDataSource {
    fn new(path: String, layout: ScanLayout) -> Self {
        let mut field_to_slot = FxHashMap::default();
        let mut whole_value_slot = None;
        for proj in &layout.projections {
            match &proj.source.source_type {
                ScanSourceType::FieldPath(field_name) => {
                    field_to_slot.insert(field_name.clone(), proj.target_slot);
                }
                ScanSourceType::WholeValue => {
                    whole_value_slot = Some(proj.target_slot);
                }
                _ => {}
            }
        }

        IonDataSource {
            path,
            reader: None,
            field_to_slot,
            whole_value_slot,
            string_storage: Vec::new(),
        }
    }
}

/// Advances `reader` to the next field of the current struct, returning its
/// type, or `None` at the end of the struct.
///
/// The Ion reader reports null values as `StreamItem::Null` rather than
/// `StreamItem::Value`; these are returned as `IonType::Null` so the field is
/// written as NULL instead of being dropped (and later read as MISSING).
fn next_struct_field(reader: &mut ion_rs::Reader<'static>) -> EvalResult<Option<IonType>> {
    let item = reader.next().map_err(|e| {
        partiql_vm::EngineError::ReaderError(format!("error reading struct field: {e}"))
    })?;
    Ok(match item {
        ion_rs::StreamItem::Value(ion_type) => Some(ion_type),
        ion_rs::StreamItem::Null(_) => Some(IonType::Null),
        ion_rs::StreamItem::Nothing => None,
    })
}

impl DataSource for IonDataSource {
    fn open(&mut self) -> EvalResult<()> {
        let file = File::open(&self.path)
            .map_err(|e| partiql_vm::EngineError::ReaderError(format!("ion open failed: {e}")))?;
        let buf_reader = BufReader::new(file);

        let ion_reader = IonReaderBuilder::new().build(buf_reader).map_err(|e| {
            partiql_vm::EngineError::ReaderError(format!("ion reader creation failed: {e}"))
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
            .map_err(|e| partiql_vm::EngineError::ReaderError(format!("ion read failed: {e}")))?;

        match stream_item {
            ion_rs::StreamItem::Value(_ion_type) => {
                reader.step_in().map_err(|e| {
                    partiql_vm::EngineError::ReaderError(format!("failed to step into struct: {e}"))
                })?;

                // Whole-value mode: build a Tuple with all fields
                if let Some(target_slot) = self.whole_value_slot {
                    let mut vw = writer.value_writer(target_slot)?;
                    vw.step_in_tuple()?;

                    while let Some(ion_type) = next_struct_field(reader)? {
                        let field_name = reader.field_name().map_err(|e| {
                            partiql_vm::EngineError::ReaderError(format!(
                                "failed to get field name: {e}"
                            ))
                        })?;
                        let field_text = field_name.text().ok_or_else(|| {
                            partiql_vm::EngineError::ReaderError(
                                "field name has no text".to_string(),
                            )
                        })?;

                        // Store field name in string_storage for arena lifetime
                        self.string_storage.push(field_text.to_string());
                        let name_idx = self.string_storage.len() - 1;
                        let name_ref = unsafe {
                            std::mem::transmute::<&str, &str>(
                                self.string_storage[name_idx].as_str(),
                            )
                        };
                        vw.put_field_name(name_ref)?;

                        match ion_type {
                            IonType::Int => {
                                let val = reader.read_i64().map_err(|e| {
                                    partiql_vm::EngineError::ReaderError(format!(
                                        "failed to read i64: {e}"
                                    ))
                                })?;
                                vw.put_i64(val)?;
                            }
                            IonType::Float => {
                                let val = reader.read_f64().map_err(|e| {
                                    partiql_vm::EngineError::ReaderError(format!(
                                        "failed to read f64: {e}"
                                    ))
                                })?;
                                vw.put_f64(val)?;
                            }
                            IonType::Bool => {
                                let val = reader.read_bool().map_err(|e| {
                                    partiql_vm::EngineError::ReaderError(format!(
                                        "failed to read bool: {e}"
                                    ))
                                })?;
                                vw.put_bool(val)?;
                            }
                            IonType::String => {
                                let val = reader.read_str().map_err(|e| {
                                    partiql_vm::EngineError::ReaderError(format!(
                                        "failed to read string: {e}"
                                    ))
                                })?;
                                self.string_storage.push(val.to_string());
                                let str_idx = self.string_storage.len() - 1;
                                let str_ref = unsafe {
                                    std::mem::transmute::<&str, &str>(
                                        self.string_storage[str_idx].as_str(),
                                    )
                                };
                                vw.put_str(str_ref)?;
                            }
                            IonType::Null => {
                                vw.put_null()?;
                            }
                            other_type => {
                                return Err(partiql_vm::EngineError::ReaderError(format!(
                                    "unsupported ion type in whole-value mode: {:?}",
                                    other_type
                                )));
                            }
                        }
                    }

                    vw.step_out()?;
                    vw.finish()?;
                } else {
                    // Column-projection mode: write individual fields to slots
                    while let Some(ion_type) = next_struct_field(reader)? {
                        let field_name = reader.field_name().map_err(|e| {
                            partiql_vm::EngineError::ReaderError(format!(
                                "failed to get field name: {e}"
                            ))
                        })?;

                        let field_text = field_name.text().ok_or_else(|| {
                            partiql_vm::EngineError::ReaderError(
                                "field name has no text".to_string(),
                            )
                        })?;

                        if let Some(&target_slot) = self.field_to_slot.get(field_text) {
                            match ion_type {
                                IonType::Int => {
                                    let val = reader.read_i64().map_err(|e| {
                                        partiql_vm::EngineError::ReaderError(format!(
                                            "failed to read i64: {e}"
                                        ))
                                    })?;
                                    writer.write_i64(target_slot, val)?;
                                }
                                IonType::Float => {
                                    let val = reader.read_f64().map_err(|e| {
                                        partiql_vm::EngineError::ReaderError(format!(
                                            "failed to read f64: {e}"
                                        ))
                                    })?;
                                    writer.write_f64(target_slot, val)?;
                                }
                                IonType::Bool => {
                                    let val = reader.read_bool().map_err(|e| {
                                        partiql_vm::EngineError::ReaderError(format!(
                                            "failed to read bool: {e}"
                                        ))
                                    })?;
                                    writer.write_bool(target_slot, val)?;
                                }
                                IonType::String => {
                                    let val = reader.read_str().map_err(|e| {
                                        partiql_vm::EngineError::ReaderError(format!(
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
                                    writer.write_str(target_slot, str_ref)?;
                                }
                                IonType::Null => {
                                    writer.write_null(target_slot)?;
                                }
                                other_type => {
                                    return Err(partiql_vm::EngineError::ReaderError(format!(
                                        "unsupported ion type for projection: {:?}",
                                        other_type
                                    )));
                                }
                            }
                        }
                    }
                }

                reader.step_out().map_err(|e| {
                    partiql_vm::EngineError::ReaderError(format!(
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

// =============================================================================
// Table Function implementations for the VM engine
// =============================================================================

/// Compile-time metadata for table functions that produce columnar integer data.
/// Used by `rand()` and `mem()` table functions.
pub struct ColumnarIntMetadata {
    column_names: Vec<String>,
}

impl ColumnarIntMetadata {
    pub fn new(column_names: Vec<String>) -> Self {
        ColumnarIntMetadata { column_names }
    }
}

impl DataSourceMetadata for ColumnarIntMetadata {
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

/// Compile-time metadata for Ion table functions (dynamic schema).
pub struct DynamicSchemaMetadata;

impl DataSourceMetadata for DynamicSchemaMetadata {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::field(field_name, PhysicalType::Dynamic))
    }
}

/// Table function: `rand(total_rows, num_columns)`
///
/// Generates random integer data. Each column gets a random i64 value per row.
pub struct RandTableFunction;

impl VmTableFunction for RandTableFunction {
    fn create(
        &self,
        reader: &RegisterReader<'_>,
        arg_slots: &[SlotId],
        layout: &ScanLayout,
    ) -> EvalResult<Box<dyn DataSource>> {
        let total_rows = reader.get_i64(arg_slots[0] as usize).ok_or_else(|| {
            partiql_vm::EngineError::ReaderError("rand: arg 0 must be integer".into())
        })? as usize;
        let num_columns = reader.get_i64(arg_slots[1] as usize).ok_or_else(|| {
            partiql_vm::EngineError::ReaderError("rand: arg 1 must be integer".into())
        })? as usize;
        Ok(Box::new(RandomDataSource::new(
            total_rows,
            num_columns,
            layout.clone(),
        )))
    }
}

/// Table function: `mem(total_rows, num_columns)`
///
/// Generates sequential integer data. Row N has value N in all columns.
pub struct MemTableFunction;

impl VmTableFunction for MemTableFunction {
    fn create(
        &self,
        reader: &RegisterReader<'_>,
        arg_slots: &[SlotId],
        layout: &ScanLayout,
    ) -> EvalResult<Box<dyn DataSource>> {
        let total_rows = reader.get_i64(arg_slots[0] as usize).ok_or_else(|| {
            partiql_vm::EngineError::ReaderError("mem: arg 0 must be integer".into())
        })? as usize;
        let num_columns = reader.get_i64(arg_slots[1] as usize).ok_or_else(|| {
            partiql_vm::EngineError::ReaderError("mem: arg 1 must be integer".into())
        })? as usize;
        Ok(Box::new(InMemGeneratedReader::new(
            total_rows,
            num_columns,
            layout.clone(),
        )))
    }
}

/// Table function: `scan_ion(path)`
///
/// Reads Ion data from a file path, streaming rows lazily.
pub struct ScanIonTableFunction;

impl VmTableFunction for ScanIonTableFunction {
    fn create(
        &self,
        reader: &RegisterReader<'_>,
        arg_slots: &[SlotId],
        layout: &ScanLayout,
    ) -> EvalResult<Box<dyn DataSource>> {
        let path = reader
            .get_str(arg_slots[0] as usize)
            .ok_or_else(|| {
                partiql_vm::EngineError::ReaderError("scan_ion: arg 0 must be string".into())
            })?
            .to_string();
        Ok(Box::new(IonDataSource::new(path, layout.clone())))
    }
}

/// CompilationCatalog that provides table function metadata for pqlite.
pub struct TableFnCompilationCatalog {
    column_names: Vec<String>,
}

impl TableFnCompilationCatalog {
    pub fn new(column_names: Vec<String>) -> Self {
        TableFnCompilationCatalog { column_names }
    }
}

impl CompilationCatalog for TableFnCompilationCatalog {
    fn get_table(&self, _path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        None
    }

    fn get_table_function(&self, name: &str) -> Option<VmTableFunctionHandle> {
        match name {
            "rand" | "mem" => Some(VmTableFunctionHandle::new(Arc::new(
                ColumnarIntMetadata::new(self.column_names.clone()),
            ))),
            "scan_ion" => Some(VmTableFunctionHandle::new(Arc::new(DynamicSchemaMetadata))),
            _ => None,
        }
    }
}
