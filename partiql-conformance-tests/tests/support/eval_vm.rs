use std::sync::Arc;

use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, TypeEnvEntry};
use partiql_common::catalog::EntryId;
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical_planner::{LogicalPlanner, VarRefResolution};
use partiql_types::{PartiqlShapeBuilder, StructConstraint, StructType};
use partiql_value::{Bag, BindingsName, List, Tuple, Value};
use partiql_vm::source::{
    BufferStability, CatalogScans, DataSource, DataSourceHandle, DataSourceMetadata, PhysicalType,
    RegisterWriter, ScanId, ScanLayout, ScanSource, ScanSourceType, ValueWriter,
};
use partiql_vm::value::{FieldName, RegisterReader, RowShape, Shape, ValueType, ValueView};
use partiql_vm::{
    CompilationCatalog, CompilationContext, EngineError, ExecutionCatalog, ExecutionContext,
    ExecutionResult, PartiQLVM, PlanCompiler,
};

use indexmap::IndexSet;
use rustc_hash::FxHashMap;

use partiql_eval::error::{EvalErr, EvaluationError, PlanErr, PlanningError};

use super::{parse, EvaluationMode, TestError, TestValue};

// =============================================================================
// Value Conversion (VM → partiql_value::Value)
// =============================================================================

fn row_to_value(row: &RegisterReader<'_>, shape: &Shape) -> Value {
    match shape.row_shape() {
        RowShape::Struct(fields) => {
            let mut tuple = Tuple::new();
            for field in fields.iter() {
                let name = match &field.name {
                    FieldName::Static(s) => s.clone(),
                    FieldName::Register(reg) => row.get_str(*reg).unwrap_or("?").to_string(),
                };
                let reg_idx = match &field.value {
                    RowShape::Register(idx, _) => *idx,
                    _ => continue,
                };
                if let Some(mut view) = row.get_value_view(reg_idx) {
                    tuple.insert(&name, view_to_value(&mut view));
                }
            }
            Value::Tuple(Box::new(tuple))
        }
        RowShape::Register(idx, _) => {
            if let Some(mut view) = row.get_value_view(*idx) {
                view_to_value(&mut view)
            } else {
                Value::Missing
            }
        }
    }
}

/// A VM datetime as `partiql_value` has it, if it can: the `time` crate it is built on holds
/// the years -9999 to 9999.
fn date_to_value(date: partiql_vm::value::Date) -> Option<partiql_value::DateTime> {
    let (year, month, day) = date.ymd();
    let month = std::num::NonZeroU8::new(month)?;
    (-9999..=9999)
        .contains(&year)
        .then(|| partiql_value::DateTime::from_ymd(year, month, day))
}

fn time_to_value(time: partiql_vm::value::Time) -> partiql_value::DateTime {
    let (hour, minute, second, micros) = time.hms_micro();
    let (tz_hours, tz_minutes) = match time.offset().get() {
        Some(m) => (Some((m / 60) as i8), Some((m % 60) as i8)),
        None => (None, None),
    };
    partiql_value::DateTime::from_hms_nano_tz(
        hour,
        minute,
        second,
        micros * 1_000,
        tz_hours,
        tz_minutes,
    )
}

fn timestamp_to_value(ts: partiql_vm::value::Timestamp) -> Option<partiql_value::DateTime> {
    let (date, time) = ts.local();
    let (year, month, day) = date.ymd();
    let (hour, minute, second, micros) = time.hms_micro();
    let month = std::num::NonZeroU8::new(month)?;
    (-9999..=9999).contains(&year).then(|| {
        partiql_value::DateTime::from_ymdhms_nano_offset_minutes(
            year,
            month,
            day,
            hour,
            minute,
            second,
            micros * 1_000,
            ts.offset().get().map(i32::from),
        )
    })
}

fn view_to_value(view: &mut ValueView<'_>) -> Value {
    match view.get_type() {
        ValueType::Missing => Value::Missing,
        ValueType::Null => Value::Null,
        ValueType::Bool => Value::Boolean(view.get_bool().unwrap()),
        ValueType::Integer => Value::Integer(view.get_i64().unwrap()),
        ValueType::Float => Value::Real(ordered_float::OrderedFloat(view.get_f64().unwrap())),
        ValueType::Decimal => Value::Decimal(Box::new(view.get_decimal().unwrap())),
        ValueType::String => Value::String(Box::new(view.get_str().unwrap().to_string())),
        ValueType::Bytes => Value::Null,
        ValueType::Tuple => {
            let mut tuple = Tuple::new();
            if view.step_in().is_ok() {
                loop {
                    let key = view.get_field_name().unwrap_or("?").to_string();
                    let val = view_to_value(view);
                    tuple.insert(&key, val);
                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::Tuple(Box::new(tuple))
        }
        ValueType::List => {
            let mut items: Vec<Value> = Vec::new();
            if view.step_in().is_ok() {
                loop {
                    items.push(view_to_value(view));
                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::List(Box::new(List::from(items)))
        }
        ValueType::Bag => {
            let mut items: Vec<Value> = Vec::new();
            if view.step_in().is_ok() {
                loop {
                    items.push(view_to_value(view));
                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::Bag(Box::new(Bag::from(items)))
        }
        ValueType::Date => Value::DateTime(Box::new(
            date_to_value(view.get_date().unwrap()).expect("a date partiql-value holds"),
        )),
        ValueType::Time => Value::DateTime(Box::new(time_to_value(view.get_time().unwrap()))),
        ValueType::Timestamp => Value::DateTime(Box::new(
            timestamp_to_value(view.get_timestamp().unwrap())
                .expect("a timestamp partiql-value holds"),
        )),
        ValueType::Variant => {
            use partiql_extension_ion::boxed_ion::BoxedIonType;
            use partiql_value::boxed_variant::DynBoxedVariantTypeFactory;
            let (bytes, _type_name) = view.get_variant().unwrap();
            let type_tag = BoxedIonType::default().to_dyn_type_tag();
            let variant =
                partiql_value::Variant::new(bytes.to_vec(), type_tag).expect("variant decode");
            Value::from(variant)
        }
    }
}

// =============================================================================
// In-Memory DataSource
// =============================================================================

enum SlotMapping {
    WholeValue(u16),
    Field { name: String, slot: u16 },
}

struct InMemorySource {
    rows: Vec<Value>,
    cursor: usize,
    mappings: Vec<SlotMapping>,
}

impl InMemorySource {
    fn new(rows: Vec<Value>, layout: ScanLayout) -> Self {
        let mappings = layout
            .projections
            .iter()
            .map(|p| match &p.source.source_type {
                ScanSourceType::WholeValue => SlotMapping::WholeValue(p.target_slot),
                ScanSourceType::FieldPath(name) => SlotMapping::Field {
                    name: name.clone(),
                    slot: p.target_slot,
                },
                _ => SlotMapping::WholeValue(p.target_slot),
            })
            .collect();
        InMemorySource {
            rows,
            cursor: 0,
            mappings,
        }
    }
}

impl DataSource for InMemorySource {
    fn open(&mut self) -> partiql_vm::Result<()> {
        self.cursor = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> partiql_vm::Result<bool> {
        if self.cursor >= self.rows.len() {
            return Ok(false);
        }
        let row = &self.rows[self.cursor];
        self.cursor += 1;

        for mapping in &self.mappings {
            match mapping {
                SlotMapping::WholeValue(slot) => {
                    write_value_to_slot(writer, *slot, row)?;
                }
                SlotMapping::Field { name, slot } => {
                    let field_value = match row {
                        Value::Tuple(t) => t
                            .pairs()
                            .find(|(k, _)| k.eq_ignore_ascii_case(name))
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::Missing),
                        _ => Value::Missing,
                    };
                    write_value_to_slot(writer, *slot, &field_value)?;
                }
            }
        }
        Ok(true)
    }

    fn close(&mut self) -> partiql_vm::Result<()> {
        Ok(())
    }
}

fn write_value_to_slot(
    writer: &mut RegisterWriter<'_, '_>,
    slot: u16,
    value: &Value,
) -> partiql_vm::Result<()> {
    match value {
        Value::Null => writer.write_null(slot),
        Value::Missing => writer.write_missing(slot),
        Value::Boolean(b) => writer.write_bool(slot, *b),
        Value::Integer(i) => writer.write_i64(slot, *i),
        Value::Real(f) => writer.write_f64(slot, f.into_inner()),
        Value::Decimal(d) => writer.write_decimal(slot, **d),
        Value::String(s) => {
            let leaked: &'static str = Box::leak(s.clone().into_boxed_str());
            writer.write_str(slot, leaked)
        }
        Value::Tuple(t) => {
            let mut vw = writer.value_writer(slot)?;
            vw.step_in_tuple()?;
            for (key, val) in t.pairs() {
                let leaked: &'static str = Box::leak(key.to_string().into_boxed_str());
                vw.put_field_name(leaked)?;
                write_nested_value(&mut vw, val)?;
            }
            vw.step_out()?;
            vw.finish()
        }
        Value::List(l) => {
            let mut vw = writer.value_writer(slot)?;
            vw.step_in_list()?;
            for val in l.iter() {
                write_nested_value(&mut vw, val)?;
            }
            vw.step_out()?;
            vw.finish()
        }
        Value::Bag(b) => {
            let mut vw = writer.value_writer(slot)?;
            vw.step_in_bag()?;
            for val in b.iter() {
                write_nested_value(&mut vw, val)?;
            }
            vw.step_out()?;
            vw.finish()
        }
        Value::DateTime(dt) => match datetime_to_vm(dt) {
            Some(VmDateTime::Date(d)) => writer.write_date(slot, d),
            Some(VmDateTime::Time(t)) => writer.write_time(slot, t),
            Some(VmDateTime::Timestamp(t)) => writer.write_timestamp(slot, t),
            None => Err(unsupported_datetime(dt)),
        },
        _ => writer.write_null(slot),
    }
}

/// A `partiql_value` datetime written into the VM, its fraction of a second rounded to
/// microseconds; `None` for one the VM can't hold (an offset with seconds).
enum VmDateTime {
    Date(partiql_vm::value::Date),
    Time(partiql_vm::value::Time),
    Timestamp(partiql_vm::value::Timestamp),
}

/// A timestamp whose fraction rounds to microseconds, but not past year 9999, `partiql_value`'s
/// last, so that the value converts back.
fn until_9999(
    date: partiql_vm::value::Date,
    hms: (u8, u8, u8),
    nanos: u32,
    offset: partiql_vm::value::Offset,
    precision: u8,
) -> Option<partiql_vm::value::Timestamp> {
    use partiql_vm::value::Timestamp;
    let rounded = Timestamp::from_local_nano(date, hms, nanos, offset, precision)?;
    match rounded.local().0.ymd().0 > 9999 {
        true => Timestamp::from_local_nano(date, hms, nanos / 1_000 * 1_000, offset, precision),
        false => Some(rounded),
    }
}

fn datetime_to_vm(value: &partiql_value::DateTime) -> Option<VmDateTime> {
    use partiql_value::DateTime as D;
    use partiql_vm::value::{Date, Offset, Time, MAX_PRECISION};
    let precision = |nanos: u32| if nanos == 0 { 0 } else { MAX_PRECISION };
    Some(match value {
        D::Date(d) => VmDateTime::Date(Date::from_ymd(d.year(), u8::from(d.month()), d.day())?),
        D::Time(t) | D::TimeWithTz(t, _) => {
            let offset = match value {
                D::TimeWithTz(_, o) if o.seconds_past_minute() == 0 => {
                    Offset::minutes(i32::from(o.whole_minutes()))?
                }
                D::TimeWithTz(..) => return None,
                _ => Offset::NONE,
            };
            let nanos = t.nanosecond();
            VmDateTime::Time(Time::from_hms_nano(
                t.hour(),
                t.minute(),
                t.second(),
                nanos,
                offset,
                precision(nanos),
            )?)
        }
        D::Timestamp(ts) => {
            let date = Date::from_ymd(ts.year(), u8::from(ts.month()), ts.day())?;
            let nanos = ts.nanosecond();
            VmDateTime::Timestamp(until_9999(
                date,
                (ts.hour(), ts.minute(), ts.second()),
                nanos,
                Offset::NONE,
                precision(nanos),
            )?)
        }
        D::TimestampWithTz(ts) => {
            let o = ts.offset();
            if o.seconds_past_minute() != 0 {
                return None;
            }
            let offset = Offset::minutes(i32::from(o.whole_minutes()))?;
            let date = Date::from_ymd(ts.year(), u8::from(ts.month()), ts.day())?;
            let nanos = ts.nanosecond();
            VmDateTime::Timestamp(until_9999(
                date,
                (ts.hour(), ts.minute(), ts.second()),
                nanos,
                offset,
                precision(nanos),
            )?)
        }
        _ => return None,
    })
}

/// A datetime the VM can't hold (an offset with seconds).
fn unsupported_datetime(dt: &partiql_value::DateTime) -> EngineError {
    EngineError::TypeError(format!("a datetime the VM can't hold: {dt:?}"))
}

fn write_nested_value(vw: &mut ValueWriter<'_, '_>, value: &Value) -> partiql_vm::Result<()> {
    match value {
        Value::Null | Value::Missing => vw.put_null(),
        Value::Boolean(b) => vw.put_bool(*b),
        Value::Integer(i) => vw.put_i64(*i),
        Value::Real(f) => vw.put_f64(f.into_inner()),
        Value::Decimal(d) => vw.put_decimal(**d),
        Value::String(s) => {
            let leaked: &'static str = Box::leak(s.clone().into_boxed_str());
            vw.put_str(leaked)
        }
        Value::Tuple(t) => {
            vw.step_in_tuple()?;
            for (key, val) in t.pairs() {
                let leaked: &'static str = Box::leak(key.to_string().into_boxed_str());
                vw.put_field_name(leaked)?;
                write_nested_value(vw, val)?;
            }
            vw.step_out()
        }
        Value::List(l) => {
            vw.step_in_list()?;
            for val in l.iter() {
                write_nested_value(vw, val)?;
            }
            vw.step_out()
        }
        Value::Bag(b) => {
            vw.step_in_bag()?;
            for val in b.iter() {
                write_nested_value(vw, val)?;
            }
            vw.step_out()
        }
        Value::DateTime(dt) => match datetime_to_vm(dt) {
            Some(VmDateTime::Date(d)) => vw.put_date(d),
            Some(VmDateTime::Time(t)) => vw.put_time(t),
            Some(VmDateTime::Timestamp(t)) => vw.put_timestamp(t),
            None => Err(unsupported_datetime(dt)),
        },
        _ => vw.put_null(),
    }
}

// =============================================================================
// Catalog Implementations
// =============================================================================

/// A global of the test environment: a collection (read row by row; a list is ordered) or
/// a single value.
struct SchemalessMetadata {
    collection: bool,
    ordered: bool,
}

impl DataSourceMetadata for SchemalessMetadata {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::field(field_name, PhysicalType::Dynamic))
    }

    fn is_collection(&self) -> bool {
        self.collection
    }

    fn is_ordered(&self) -> bool {
        self.ordered
    }
}

struct ConformanceCompilationCatalog {
    /// Each global's entry, and whether it is a collection, and ordered.
    tables: FxHashMap<String, (EntryId, bool, bool)>,
}

impl CompilationCatalog for ConformanceCompilationCatalog {
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        if path.len() != 1 {
            return None;
        }
        let name = match &path[0] {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
            _ => return None,
        };
        self.tables
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, &(entry_id, collection, ordered))| {
                let metadata = SchemalessMetadata {
                    collection,
                    ordered,
                };
                DataSourceHandle::new(entry_id, Arc::new(metadata))
            })
    }
}

struct ConformanceExecutionCatalog {
    table_ion: FxHashMap<EntryId, String>,
    scan_mappings: FxHashMap<ScanId, (EntryId, ScanLayout)>,
}

impl ExecutionCatalog for ConformanceExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scan_mappings.clear();
        for (scan_id, entry_id, layout) in scans.iter() {
            self.scan_mappings
                .insert(scan_id, (entry_id, layout.clone()));
        }
    }

    fn create(&self, scan_id: ScanId) -> partiql_vm::Result<Box<dyn DataSource>> {
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            EngineError::IllegalState(format!("ScanId {:?} not prepared", scan_id))
        })?;
        let ion_text = self
            .table_ion
            .get(entry_id)
            .map(|s| s.as_str())
            .unwrap_or("[]");
        let rows = ion_to_rows(ion_text);
        Ok(Box::new(InMemorySource::new(rows, layout.clone())))
    }
}

// =============================================================================
// Ion Parsing
// =============================================================================

fn parse_ion(ion_text: &str) -> Value {
    let reader = ion_rs_old::ReaderBuilder::new()
        .build(ion_text)
        .expect("reading Ion text");
    let mut iter = IonDecoderBuilder::new(
        IonDecoderConfig::default().with_mode(Encoding::PartiqlEncodedAsIon),
    )
    .build(reader)
    .expect("building decoder");
    iter.next()
        .expect("expected a value")
        .expect("value decode failed")
}

fn ion_to_rows(ion_text: &str) -> Vec<Value> {
    let data = parse_ion(ion_text);
    match data {
        Value::List(l) => l.into_iter().collect(),
        Value::Bag(b) => b.into_iter().collect(),
        other => vec![other],
    }
}

// =============================================================================
// Environment → Catalog Translation
// =============================================================================

fn value_to_ion_text(value: &Value) -> String {
    use ion_rs_old::element::writer::TextKind;
    use partiql_extension_ion::encode::{IonEncoderBuilder, IonEncoderConfig};

    let mut buf = Vec::new();
    let mut writer = ion_rs_old::TextWriterBuilder::new(TextKind::Compact)
        .build(&mut buf)
        .expect("writer");
    let mut encoder = IonEncoderBuilder::new(
        IonEncoderConfig::default().with_mode(Encoding::PartiqlEncodedAsIon),
    )
    .build(&mut writer)
    .expect("encoder");

    encoder.write_value(value).expect("write value");

    drop(encoder);
    drop(writer);
    String::from_utf8(buf).expect("valid utf8")
}

// =============================================================================
// Error Mapping
// =============================================================================

fn engine_error_to_plan_err(e: EngineError) -> PlanErr {
    PlanErr {
        errors: vec![PlanningError::NotYetImplemented(e.to_string())],
    }
}

fn engine_error_to_eval_err(e: EngineError) -> EvalErr {
    EvalErr {
        errors: vec![EvaluationError::IllegalState(e.to_string())],
    }
}

// =============================================================================
// Main Evaluation Entry Point
// =============================================================================

pub(crate) fn eval_via_vm<'a>(
    statement: &'a str,
    mode: EvaluationMode,
    env: &Option<TestValue>,
) -> Result<Value, TestError<'a>> {
    let mut catalog = PartiqlCatalog::default();
    let mut comp_tables = FxHashMap::default();
    let mut exec_ion: FxHashMap<EntryId, String> = FxHashMap::default();
    let mut next_entry_id: u64 = 0;

    if let Some(test_val) = env {
        if let Value::Tuple(ref t) = test_val.value {
            for (key, value) in t.pairs() {
                let entry_id = EntryId::from(next_entry_id);
                next_entry_id += 1;

                let mut bld = PartiqlShapeBuilder::default();
                let data_type =
                    bld.new_struct(StructType::new(IndexSet::from([StructConstraint::Open(
                        true,
                    )])));
                let entry = TypeEnvEntry::new(key, &[], data_type);
                catalog.add_type_entry(entry).expect("add type entry");

                let collection = matches!(value, Value::List(_) | Value::Bag(_));
                let ordered = matches!(value, Value::List(_));
                comp_tables.insert(key.to_string(), (entry_id, collection, ordered));
                exec_ion.insert(entry_id, value_to_ion_text(value));
            }
        }
    }

    let shared_catalog = catalog.to_shared_catalog();

    let parsed = parse(statement)?;
    let planner = LogicalPlanner::with_var_resolution(&shared_catalog, VarRefResolution::Static);
    let logical = planner.lower(&parsed).map_err(TestError::Lower)?;

    let comp_catalog = Arc::new(ConformanceCompilationCatalog {
        tables: comp_tables,
    });

    let mut context = CompilationContext::new();
    let catalog_id = context.add_catalog("default", comp_catalog);
    let eval_mode: partiql_vm::EvaluationMode = mode.into();
    let mut compiler = PlanCompiler::new(&context, eval_mode);
    let compiled = compiler
        .compile(&logical)
        .map_err(|e| TestError::Plan(engine_error_to_plan_err(e)))?;

    let catalog_scans = compiled.scans_for_catalog(catalog_id);
    let mut exec_catalog = ConformanceExecutionCatalog {
        table_ion: exec_ion,
        scan_mappings: FxHashMap::default(),
    };
    exec_catalog.prepare(&catalog_scans);

    let mut exec_context = ExecutionContext::new();
    exec_context.add_catalog(catalog_id, Box::new(exec_catalog));

    let mut vm = PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| TestError::Eval(engine_error_to_eval_err(e)))?;
    let shape = vm.shape().clone();

    let iter = match vm
        .execute()
        .map_err(|e| TestError::Eval(engine_error_to_eval_err(e)))?
    {
        ExecutionResult::Query(iter) => iter,
    };

    let mut results: Vec<Value> = Vec::new();
    for row_result in iter {
        let row = row_result.map_err(|e| TestError::Eval(engine_error_to_eval_err(e)))?;
        results.push(row_to_value(&row, &shape));
    }

    let result = match &shape {
        Shape::Bag(_) => Value::Bag(Box::new(Bag::from(results))),
        Shape::List(_) => Value::List(Box::new(List::from(results))),
        Shape::Single(_) => results.into_iter().next().unwrap_or(Value::Missing),
    };

    Ok(result)
}
