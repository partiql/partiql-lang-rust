//! Adapter from a plugin's Arrow stream to the VM's `DataSource` contract.
//!
//! `PluginTableFn::create` copies the call's arguments; `PluginDataSource::open`
//! calls the plugin's `open` and wraps the returned `ArrowArrayStream`;
//! `next_row` walks each record batch row by row and copies every projected
//! cell into its register slot.
//!
//! Type mapping (Arrow → PartiQL register value):
//! * Null → NULL; Boolean → bool; Int8..Int64, UInt8..UInt32 → i64; UInt64 →
//!   i64, or decimal above `i64::MAX`; Float16/32/64 → f64
//! * Utf8 / LargeUtf8 / Utf8View → string; Binary / LargeBinary / BinaryView /
//!   FixedSizeBinary → bytes
//! * Decimal128 with scale ≤ 28 → decimal; other decimals → string
//! * Date32/64, Time32/64, Timestamp → ISO-8601 string (timestamps with a time
//!   zone end in `Z`; Arrow stores them as UTC)
//! * Struct → tuple; List / LargeList / FixedSizeList → list; Map → list of
//!   `{key, value}` tuples; Dictionary → its decoded value
//! * A field the batch doesn't have → MISSING
//!
//! Strings and bytes are borrowed straight from the batch's buffers. That is
//! sound under `BufferStability::UntilNext`: the VM does not hold a row past
//! the following `next_row` call, and a batch is only dropped two batches
//! later (see `PluginDataSource::prev`).

use std::ffi::{c_char, CStr};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream};
use arrow_array::types::*;
use arrow_array::{temporal_conversions as tc, Array, ArrayRef, RecordBatch};
use arrow_schema::{DataType, Field, SchemaRef, TimeUnit};
use partiql_vm::source::{
    BufferStability, DataSource, DataSourceMetadata, PhysicalType, RegisterWriter, ScanLayout,
    ScanSource, ScanSourceType, TableFunction, ValueWriter,
};
use partiql_vm::value::RegisterReader;
use partiql_vm::{EngineError, Result};
use rust_decimal::Decimal;

use super::args::{read_args, OwnedArg};
use super::ffi::{self, PqliteOpenRequest, PqliteStr};
use super::LoadedPlugin;

fn reader_err(msg: impl Into<String>) -> EngineError {
    EngineError::ReaderError(msg.into())
}

// =============================================================================
// Compile time
// =============================================================================

/// Compile-time metadata. Without a static schema every field resolves as a
/// dynamically typed field path. With one, known columns resolve by index,
/// and non-nullable primitive columns get a concrete physical type.
pub(crate) struct PluginMetadata {
    pub(crate) static_schema: Option<SchemaRef>,
}

impl DataSourceMetadata for PluginMetadata {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        let Some(schema) = &self.static_schema else {
            return Some(ScanSource::field(field_name, PhysicalType::Dynamic));
        };
        match find_field(
            schema.fields().iter().map(|f| f.name().as_str()),
            field_name,
        ) {
            Some(i) => Some(ScanSource::column(i, physical_type(&schema.fields()[i]))),
            // Unknown names still resolve (to MISSING at run time) so one typo
            // doesn't force the whole scan into whole-row mode.
            None => Some(ScanSource::field(field_name, PhysicalType::Dynamic)),
        }
    }
}

fn physical_type(field: &Field) -> PhysicalType {
    if field.is_nullable() {
        return PhysicalType::Dynamic;
    }
    match field.data_type() {
        DataType::Boolean => PhysicalType::Bool,
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => PhysicalType::I64,
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 => PhysicalType::I64,
        DataType::Float32 | DataType::Float64 => PhysicalType::F64,
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => PhysicalType::Str,
        DataType::Binary | DataType::LargeBinary | DataType::BinaryView => PhysicalType::Bytes,
        _ => PhysicalType::Dynamic,
    }
}

/// Exact match first, then ASCII-case-insensitive (PartiQL's default).
fn find_field<'s>(names: impl Iterator<Item = &'s str> + Clone, wanted: &str) -> Option<usize> {
    names.clone().position(|n| n == wanted).or_else(|| {
        names
            .into_iter()
            .position(|n| n.eq_ignore_ascii_case(wanted))
    })
}

// =============================================================================
// Runtime factory
// =============================================================================

pub(crate) struct PluginTableFn {
    pub(crate) plugin: Arc<LoadedPlugin>,
    pub(crate) fn_index: u32,
    pub(crate) name: &'static str,
    pub(crate) static_schema: Option<SchemaRef>,
}

impl TableFunction for PluginTableFn {
    fn create(
        &self,
        reader: &RegisterReader<'_>,
        arg_slots: &[u16],
        layout: &ScanLayout,
    ) -> Result<Box<dyn DataSource>> {
        let args = read_args(reader, arg_slots)?;
        let mut targets = Vec::with_capacity(layout.projections.len());
        let mut whole_row = false;
        for proj in &layout.projections {
            let target = match &proj.source.source_type {
                ScanSourceType::WholeValue => {
                    whole_row = true;
                    Target::WholeRow
                }
                ScanSourceType::FieldPath(name) => Target::Field(name.clone()),
                ScanSourceType::ColumnIndex(i) => {
                    let name = self
                        .static_schema
                        .as_ref()
                        .and_then(|s| s.fields().get(*i))
                        .map(|f| f.name().clone())
                        .ok_or_else(|| {
                            reader_err(format!("{}: no column at index {i}", self.name))
                        })?;
                    Target::Field(name)
                }
            };
            targets.push((proj.target_slot, target));
        }
        Ok(Box::new(PluginDataSource {
            stream: None,
            current: None,
            prev: None,
            row: 0,
            bindings: Vec::new(),
            scratch: Vec::new(),
            plugin: Arc::clone(&self.plugin),
            fn_index: self.fn_index,
            name: self.name,
            args,
            whole_row,
            targets,
            cancelled: Box::new(AtomicU32::new(0)),
        }))
    }
}

enum Target {
    WholeRow,
    Field(String),
}

// =============================================================================
// Runtime data source
// =============================================================================

/// Field order matters: `stream` must drop before `cancelled`, which the
/// plugin may read until the stream is released.
pub(crate) struct PluginDataSource {
    stream: Option<ArrowArrayStreamReader>,
    current: Option<RecordBatch>,
    /// The batch before `current`, kept so the previous row's borrowed
    /// strings outlive the `next_row` call that crosses a batch boundary.
    prev: Option<RecordBatch>,
    row: usize,
    /// Per target: the column index in the current batch, if present.
    bindings: Vec<Option<usize>>,
    /// Owned strings rendered for the current row (dates, wide decimals).
    scratch: Vec<String>,
    plugin: Arc<LoadedPlugin>,
    fn_index: u32,
    name: &'static str,
    args: Vec<OwnedArg>,
    whole_row: bool,
    targets: Vec<(u16, Target)>,
    cancelled: Box<AtomicU32>,
}

impl PluginDataSource {
    fn open_stream(&mut self) -> Result<ArrowArrayStreamReader> {
        let vt = self.plugin.vtable;
        let ffi_args: Vec<_> = self.args.iter().map(OwnedArg::as_ffi).collect();
        let fields: Vec<PqliteStr> = if self.whole_row {
            Vec::new()
        } else {
            self.targets
                .iter()
                .filter_map(|(_, t)| match t {
                    Target::Field(n) => Some(PqliteStr::new(n)),
                    Target::WholeRow => None,
                })
                .collect()
        };
        let req = PqliteOpenRequest {
            struct_size: std::mem::size_of::<PqliteOpenRequest>() as u32,
            fn_index: self.fn_index,
            args: ffi_args.as_ptr(),
            n_args: ffi_args.len(),
            whole_row: self.whole_row as i32,
            fields: fields.as_ptr(),
            n_fields: fields.len(),
            cancelled: self.cancelled.as_ptr(),
        };
        let mut stream = FFI_ArrowArrayStream::empty();
        let mut err: *mut c_char = std::ptr::null_mut();
        let open = vt.open.expect("validated at load");
        // SAFETY: every pointer in `req` borrows locals that outlive the call;
        // `stream` and `err` are valid out-params.
        let rc = unsafe { open(vt.plugin_data, &req, &mut stream, &mut err) };
        if rc != 0 {
            // The plugin's message is reported as-is; it usually names the
            // function already.
            let msg = self
                .take_plugin_string(err)
                .unwrap_or_else(|| format!("{}: plugin open failed ({rc})", self.name));
            return Err(reader_err(msg));
        }
        if !err.is_null() {
            let _ = self.take_plugin_string(err);
        }
        ArrowArrayStreamReader::try_new(stream)
            .map_err(|e| reader_err(format!("{}: invalid Arrow stream: {e}", self.name)))
    }

    fn take_plugin_string(&self, s: *mut c_char) -> Option<String> {
        if s.is_null() {
            return None;
        }
        // SAFETY: the plugin returned a NUL-terminated string it owns; we copy
        // it and hand it back to the plugin's own allocator.
        let out = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
        if let Some(free) = self.plugin.vtable.free_string {
            unsafe { free(s) };
        }
        Some(out)
    }

    /// Advance to a batch with at least one unread row. `Ok(false)` at end.
    fn ensure_row(&mut self) -> Result<bool> {
        loop {
            if let Some(b) = &self.current {
                if self.row < b.num_rows() {
                    return Ok(true);
                }
            }
            let Some(stream) = self.stream.as_mut() else {
                return Ok(false);
            };
            match stream.next() {
                None => {
                    self.stream = None;
                    return Ok(false);
                }
                Some(Err(e)) => return Err(reader_err(format!("{}: {e}", self.name))),
                Some(Ok(batch)) => {
                    let schema = batch.schema();
                    let names = || schema.fields().iter().map(|f| f.name().as_str());
                    self.bindings = self
                        .targets
                        .iter()
                        .map(|(_, t)| match t {
                            Target::Field(n) => find_field(names(), n),
                            Target::WholeRow => None,
                        })
                        .collect();
                    self.prev = self.current.replace(batch);
                    self.row = 0;
                }
            }
        }
    }
}

impl DataSource for PluginDataSource {
    fn open(&mut self) -> Result<()> {
        let stream = self.open_stream()?;
        self.stream = Some(stream);
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
        if !self.ensure_row()? {
            return Ok(false);
        }
        self.scratch.clear();
        let batch = self.current.as_ref().expect("ensure_row");
        let row = self.row;
        for (i, (slot, target)) in self.targets.iter().enumerate() {
            match target {
                Target::WholeRow => {
                    let mut vw = writer.value_writer(*slot)?;
                    put_struct_fields(
                        &mut vw,
                        batch
                            .schema_ref()
                            .fields()
                            .iter()
                            .map(|f| f.name().as_str()),
                        batch.columns(),
                        row,
                        &mut self.scratch,
                    )?;
                    vw.finish()?;
                }
                Target::Field(_) => match self.bindings[i] {
                    None => writer.write_missing(*slot)?,
                    Some(col) => {
                        write_slot(writer, *slot, batch.column(col), row, &mut self.scratch)?
                    }
                },
            }
        }
        self.row += 1;
        Ok(true)
    }

    fn close(&mut self) -> Result<()> {
        self.cancelled.store(1, Ordering::Release);
        self.stream = None;
        self.current = None;
        self.prev = None;
        Ok(())
    }
}

impl Drop for PluginDataSource {
    fn drop(&mut self) {
        self.cancelled.store(1, Ordering::Release);
    }
}

// =============================================================================
// Arrow cell → register value
// =============================================================================

/// A cell decoded to a register-compatible value. Borrowed data is laundered
/// to `'static`; see the module docs for why that is sound.
enum Cell {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(&'static str),
    Bytes(&'static [u8]),
    Decimal(Decimal),
    Struct,
    List,
    Map,
}

/// Erase the batch lifetime. Callers uphold the `UntilNext` argument in the
/// module docs.
unsafe fn launder<T: ?Sized>(r: &T) -> &'static T {
    std::mem::transmute::<&T, &'static T>(r)
}

fn keep(scratch: &mut Vec<String>, s: String) -> &'static str {
    scratch.push(s);
    // SAFETY: the String's heap buffer doesn't move when `scratch` grows, and
    // `scratch` is only cleared at the start of the next row.
    unsafe { launder(scratch.last().expect("just pushed").as_str()) }
}

fn cell(arr: &dyn Array, row: usize, scratch: &mut Vec<String>) -> Result<Cell> {
    if arr.is_null(row) {
        return Ok(Cell::Null);
    }
    macro_rules! int {
        ($t:ty) => {
            Cell::I64(arr.as_primitive::<$t>().value(row) as i64)
        };
    }
    // SAFETY (all `launder` calls): see module docs.
    Ok(match arr.data_type() {
        DataType::Null => Cell::Null,
        DataType::Boolean => Cell::Bool(arr.as_boolean().value(row)),
        DataType::Int8 => int!(Int8Type),
        DataType::Int16 => int!(Int16Type),
        DataType::Int32 => int!(Int32Type),
        DataType::Int64 => int!(Int64Type),
        DataType::UInt8 => int!(UInt8Type),
        DataType::UInt16 => int!(UInt16Type),
        DataType::UInt32 => int!(UInt32Type),
        DataType::UInt64 => {
            let v = arr.as_primitive::<UInt64Type>().value(row);
            match i64::try_from(v) {
                Ok(i) => Cell::I64(i),
                Err(_) => Cell::Decimal(Decimal::from(v)),
            }
        }
        DataType::Float16 => Cell::F64(arr.as_primitive::<Float16Type>().value(row).to_f64()),
        DataType::Float32 => Cell::F64(arr.as_primitive::<Float32Type>().value(row) as f64),
        DataType::Float64 => Cell::F64(arr.as_primitive::<Float64Type>().value(row)),
        DataType::Utf8 => Cell::Str(unsafe { launder(arr.as_string::<i32>().value(row)) }),
        DataType::LargeUtf8 => Cell::Str(unsafe { launder(arr.as_string::<i64>().value(row)) }),
        DataType::Utf8View => Cell::Str(unsafe { launder(arr.as_string_view().value(row)) }),
        DataType::Binary => Cell::Bytes(unsafe { launder(arr.as_binary::<i32>().value(row)) }),
        DataType::LargeBinary => Cell::Bytes(unsafe { launder(arr.as_binary::<i64>().value(row)) }),
        DataType::BinaryView => Cell::Bytes(unsafe { launder(arr.as_binary_view().value(row)) }),
        DataType::FixedSizeBinary(_) => {
            Cell::Bytes(unsafe { launder(arr.as_fixed_size_binary().value(row)) })
        }
        DataType::Decimal128(_, scale) => {
            let v = arr.as_primitive::<Decimal128Type>().value(row);
            match u32::try_from(*scale)
                .ok()
                .and_then(|s| Decimal::try_from_i128_with_scale(v, s).ok())
            {
                Some(d) => Cell::Decimal(d),
                None => Cell::Str(keep(
                    scratch,
                    arr.as_primitive::<Decimal128Type>().value_as_string(row),
                )),
            }
        }
        DataType::Decimal256(_, _) => Cell::Str(keep(
            scratch,
            arr.as_primitive::<Decimal256Type>().value_as_string(row),
        )),
        DataType::Date32 => {
            let v = arr.as_primitive::<Date32Type>().value(row);
            Cell::Str(keep(scratch, fmt_opt(tc::as_date::<Date32Type>(v as i64))))
        }
        DataType::Date64 => {
            let v = arr.as_primitive::<Date64Type>().value(row);
            Cell::Str(keep(scratch, fmt_opt(tc::as_date::<Date64Type>(v))))
        }
        DataType::Time32(unit) => {
            let s = match unit {
                TimeUnit::Second => tc::as_time::<Time32SecondType>(
                    arr.as_primitive::<Time32SecondType>().value(row) as i64,
                ),
                _ => tc::as_time::<Time32MillisecondType>(
                    arr.as_primitive::<Time32MillisecondType>().value(row) as i64,
                ),
            };
            Cell::Str(keep(scratch, fmt_opt(s)))
        }
        DataType::Time64(unit) => {
            let s = match unit {
                TimeUnit::Microsecond => tc::as_time::<Time64MicrosecondType>(
                    arr.as_primitive::<Time64MicrosecondType>().value(row),
                ),
                _ => tc::as_time::<Time64NanosecondType>(
                    arr.as_primitive::<Time64NanosecondType>().value(row),
                ),
            };
            Cell::Str(keep(scratch, fmt_opt(s)))
        }
        DataType::Timestamp(unit, tz) => {
            macro_rules! ts {
                ($t:ty) => {
                    tc::as_datetime::<$t>(arr.as_primitive::<$t>().value(row))
                };
            }
            let dt = match unit {
                TimeUnit::Second => ts!(TimestampSecondType),
                TimeUnit::Millisecond => ts!(TimestampMillisecondType),
                TimeUnit::Microsecond => ts!(TimestampMicrosecondType),
                TimeUnit::Nanosecond => ts!(TimestampNanosecondType),
            };
            let s = match dt {
                Some(dt) => {
                    let z = if tz.is_some() { "Z" } else { "" };
                    format!("{}{z}", dt.format("%Y-%m-%dT%H:%M:%S%.f"))
                }
                None => "<invalid timestamp>".to_string(),
            };
            Cell::Str(keep(scratch, s))
        }
        DataType::Struct(_) => Cell::Struct,
        DataType::List(_) | DataType::LargeList(_) | DataType::FixedSizeList(_, _) => Cell::List,
        DataType::Map(_, _) => Cell::Map,
        DataType::Dictionary(_, _) => {
            let dict = arr.as_any_dictionary();
            let key = dict_key(dict.keys(), row)?;
            return cell(dict.values().as_ref(), key, scratch);
        }
        other => {
            return Err(EngineError::UnsupportedExpr(format!(
                "plugin column type {other} is not supported"
            )))
        }
    })
}

fn fmt_opt<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map(|v| v.to_string())
        .unwrap_or_else(|| "<invalid temporal value>".to_string())
}

fn dict_key(keys: &dyn Array, row: usize) -> Result<usize> {
    macro_rules! k {
        ($t:ty) => {
            keys.as_primitive::<$t>().value(row) as usize
        };
    }
    Ok(match keys.data_type() {
        DataType::Int8 => k!(Int8Type),
        DataType::Int16 => k!(Int16Type),
        DataType::Int32 => k!(Int32Type),
        DataType::Int64 => k!(Int64Type),
        DataType::UInt8 => k!(UInt8Type),
        DataType::UInt16 => k!(UInt16Type),
        DataType::UInt32 => k!(UInt32Type),
        DataType::UInt64 => k!(UInt64Type),
        other => return Err(reader_err(format!("invalid dictionary key type {other}"))),
    })
}

/// Resolve a dictionary to its values array and index; identity otherwise.
fn undict(arr: &dyn Array, row: usize) -> Result<(&dyn Array, usize)> {
    match arr.data_type() {
        DataType::Dictionary(_, _) if !arr.is_null(row) => {
            let dict = arr.as_any_dictionary();
            let key = dict_key(dict.keys(), row)?;
            undict(dict.values().as_ref(), key)
        }
        _ => Ok((arr, row)),
    }
}

fn write_slot(
    w: &mut RegisterWriter<'_, '_>,
    slot: u16,
    arr: &ArrayRef,
    row: usize,
    scratch: &mut Vec<String>,
) -> Result<()> {
    let (arr, row) = undict(arr.as_ref(), row)?;
    match cell(arr, row, scratch)? {
        Cell::Null => w.write_null(slot),
        Cell::Bool(b) => w.write_bool(slot, b),
        Cell::I64(i) => w.write_i64(slot, i),
        Cell::F64(f) => w.write_f64(slot, f),
        Cell::Str(s) => w.write_str(slot, s),
        Cell::Bytes(b) => w.write_bytes(slot, b),
        Cell::Decimal(d) => w.write_decimal(slot, d),
        Cell::Struct | Cell::List | Cell::Map => {
            let mut vw = w.value_writer(slot)?;
            put_nested(&mut vw, arr, row, scratch)?;
            vw.finish()
        }
    }
}

fn put_value(
    vw: &mut ValueWriter<'_, '_>,
    arr: &dyn Array,
    row: usize,
    scratch: &mut Vec<String>,
) -> Result<()> {
    let (arr, row) = undict(arr, row)?;
    match cell(arr, row, scratch)? {
        Cell::Null => vw.put_null(),
        Cell::Bool(b) => vw.put_bool(b),
        Cell::I64(i) => vw.put_i64(i),
        Cell::F64(f) => vw.put_f64(f),
        Cell::Str(s) => vw.put_str(s),
        Cell::Bytes(b) => vw.put_bytes(b),
        Cell::Decimal(d) => vw.put_decimal(d),
        Cell::Struct | Cell::List | Cell::Map => put_nested(vw, arr, row, scratch),
    }
}

/// Write a non-null struct / list / map cell as a tuple / list.
fn put_nested(
    vw: &mut ValueWriter<'_, '_>,
    arr: &dyn Array,
    row: usize,
    scratch: &mut Vec<String>,
) -> Result<()> {
    match arr.data_type() {
        DataType::Struct(fields) => {
            let s = arr.as_struct();
            put_struct_fields(
                vw,
                fields.iter().map(|f| f.name().as_str()),
                s.columns(),
                row,
                scratch,
            )
        }
        DataType::List(_) => put_list(vw, &arr.as_list::<i32>().value(row), scratch),
        DataType::LargeList(_) => put_list(vw, &arr.as_list::<i64>().value(row), scratch),
        DataType::FixedSizeList(_, _) => {
            put_list(vw, &arr.as_fixed_size_list().value(row), scratch)
        }
        DataType::Map(_, _) => {
            let entries = arr.as_map().value(row);
            vw.step_in_list()?;
            for i in 0..entries.len() {
                put_struct_fields(
                    vw,
                    ["key", "value"].into_iter(),
                    entries.columns(),
                    i,
                    scratch,
                )?;
            }
            vw.step_out()
        }
        other => Err(reader_err(format!("{other} is not a container type"))),
    }
}

fn put_list(
    vw: &mut ValueWriter<'_, '_>,
    items: &ArrayRef,
    scratch: &mut Vec<String>,
) -> Result<()> {
    vw.step_in_list()?;
    for i in 0..items.len() {
        put_value(vw, items.as_ref(), i, scratch)?;
    }
    vw.step_out()
}

/// One tuple from parallel `names` / `columns` at `row`. Field names borrow
/// the batch schema; see the module docs.
fn put_struct_fields<'n>(
    vw: &mut ValueWriter<'_, '_>,
    names: impl Iterator<Item = &'n str>,
    columns: &[ArrayRef],
    row: usize,
    scratch: &mut Vec<String>,
) -> Result<()> {
    vw.step_in_tuple()?;
    for (name, col) in names.zip(columns) {
        // SAFETY: see module docs; `"key"`/`"value"` are already 'static.
        vw.put_field_name(unsafe { launder(name) })?;
        put_value(vw, col.as_ref(), row, scratch)?;
    }
    vw.step_out()
}

/// Read the plugin-declared static schema, if any.
pub(crate) fn read_static_schema(
    plugin: &LoadedPlugin,
    fn_index: u32,
    def: &ffi::PqliteTableFnDef,
) -> std::result::Result<Option<SchemaRef>, String> {
    let Some(f) = def.static_schema else {
        return Ok(None);
    };
    let mut out = arrow_schema::ffi::FFI_ArrowSchema::empty();
    // SAFETY: `out` is a valid out-param; the plugin fills and owns its release.
    let rc = unsafe { f(plugin.vtable.plugin_data, fn_index, &mut out) };
    if rc != 0 {
        return Err(format!("static_schema failed ({rc})"));
    }
    let schema = arrow_schema::Schema::try_from(&out).map_err(|e| e.to_string())?;
    Ok(Some(Arc::new(schema)))
}
