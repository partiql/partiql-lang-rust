use std::sync::Arc;

use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, TypeEnvEntry};
use partiql_common::catalog::EntryId;
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical_planner::{LogicalPlanner, VarRefResolution};
use partiql_parser::Parser;
use partiql_types::{PartiqlShapeBuilder, StructConstraint, StructType};
use partiql_value::{Bag, BindingsName, List, Tuple, Value};
use partiql_vm::source::{
    BufferStability, CatalogScans, DataSource, DataSourceHandle, DataSourceMetadata, PhysicalType,
    RegisterWriter, ScanLayout, ScanSource, ScanSourceType, ValueWriter,
};
use partiql_vm::value::{RegisterReader, RowShape, Shape, ValueType, ValueView};
use partiql_vm::EvaluationMode;
use partiql_vm::{
    CompilationCatalog, CompilationContext, ExecutionCatalog, ExecutionContext, ExecutionResult,
    PartiQLVM, PlanCompiler,
};

use indexmap::IndexSet;
use rustc_hash::FxHashMap;

// =============================================================================
// Test Infrastructure
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
    // No text: an object without a row
    if ion_text.trim().is_empty() {
        return vec![];
    }
    let data = parse_ion(ion_text);
    match data {
        Value::List(l) => l.into_iter().collect(),
        Value::Bag(b) => b.into_iter().collect(),
        other => vec![other],
    }
}

// --- In-memory DataSource ---

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
        _ => writer.write_null(slot),
    }
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
        _ => vw.put_null(),
    }
}

// --- Catalogs ---

/// A table (its Ion text a list or bag) or a single value (with no row when it has no text).
struct SchemalessMetadata {
    collection: bool,
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
}

struct TestCompilationCatalog {
    /// Each object's entry, and whether it is a collection.
    tables: FxHashMap<String, (EntryId, bool)>,
}

impl CompilationCatalog for TestCompilationCatalog {
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
            .map(|(_, &(entry_id, collection))| {
                DataSourceHandle::new(entry_id, Arc::new(SchemalessMetadata { collection }))
            })
    }
}

/// Stores Ion text (Send+Sync safe) and parses to Value on create().
struct TestExecutionCatalog {
    table_ion: FxHashMap<EntryId, String>,
    scan_mappings: FxHashMap<partiql_vm::source::ScanId, (EntryId, ScanLayout)>,
}

impl ExecutionCatalog for TestExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scan_mappings.clear();
        for (scan_id, entry_id, layout) in scans.iter() {
            self.scan_mappings
                .insert(scan_id, (entry_id, layout.clone()));
        }
    }

    fn create(
        &self,
        scan_id: partiql_vm::source::ScanId,
    ) -> partiql_vm::Result<Box<dyn DataSource>> {
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            partiql_vm::EngineError::IllegalState(format!("ScanId {:?} not prepared", scan_id))
        })?;
        let ion_text = self.table_ion.get(entry_id).ok_or_else(|| {
            partiql_vm::EngineError::IllegalState(format!("EntryId {:?} not found", entry_id))
        })?;
        let rows = ion_to_rows(ion_text);
        Ok(Box::new(InMemorySource::new(rows, layout.clone())))
    }
}

// --- Result extraction ---

fn row_to_value(row: &RegisterReader<'_>, shape: &Shape) -> Value {
    use partiql_vm::value::FieldName;

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

// --- Main test harness ---

fn eval_vm(query: &str, tables: &[(&str, &str)]) -> Value {
    eval_vm_runs(query, tables, &[None]).pop().expect("one run")
}

/// Execute `query` once per entry of `runs` on the same VM, each run reading at most that
/// many rows (all when `None`).
fn eval_vm_runs(query: &str, tables: &[(&str, &str)], runs: &[Option<usize>]) -> Vec<Value> {
    eval_vm_mode(query, tables, runs, EvaluationMode::Permissive)
}

fn eval_vm_mode(
    query: &str,
    tables: &[(&str, &str)],
    runs: &[Option<usize>],
    mode: EvaluationMode,
) -> Vec<Value> {
    // PartiqlCatalog for name resolution during lowering
    let mut catalog = PartiqlCatalog::default();
    for (name, _) in tables.iter() {
        let mut bld = PartiqlShapeBuilder::default();
        let data_type = bld.new_struct(StructType::new(IndexSet::from([StructConstraint::Open(
            true,
        )])));
        let entry = TypeEnvEntry::new(name, &[], data_type);
        catalog.add_type_entry(entry).expect("add type entry");
    }
    let shared_catalog = catalog.to_shared_catalog();

    // Compilation + execution catalogs
    let mut comp_tables = FxHashMap::default();
    let mut exec_ion = FxHashMap::default();

    for (idx, (name, ion_text)) in tables.iter().enumerate() {
        let entry_id = EntryId::from(idx as u64);
        let collection = !ion_text.trim().is_empty()
            && matches!(parse_ion(ion_text), Value::List(_) | Value::Bag(_));
        comp_tables.insert(name.to_string(), (entry_id, collection));
        exec_ion.insert(entry_id, ion_text.to_string());
    }

    let comp_catalog = Arc::new(TestCompilationCatalog {
        tables: comp_tables,
    });

    // Parse + Lower
    let parsed = Parser::default().parse(query).expect("parse failed");
    let planner = LogicalPlanner::with_var_resolution(&shared_catalog, VarRefResolution::Static);
    let logical = planner.lower(&parsed).expect("lower failed");

    // Compile
    let mut context = CompilationContext::new();
    let catalog_id = context.add_catalog("default", comp_catalog);
    let mut compiler = PlanCompiler::new(&context, mode);
    let compiled = compiler.compile(&logical).expect("compile failed");

    // Prepare execution catalog
    let catalog_scans = compiled.scans_for_catalog(catalog_id);
    let mut exec_catalog = TestExecutionCatalog {
        table_ion: exec_ion,
        scan_mappings: FxHashMap::default(),
    };
    exec_catalog.prepare(&catalog_scans);

    // Execute
    let mut exec_context = ExecutionContext::new();
    exec_context.add_catalog(catalog_id, Box::new(exec_catalog));

    let mut vm = PartiQLVM::new(compiled, &exec_context).expect("VM creation failed");
    let shape = vm.shape().clone();

    let mut values = vec![];
    for take in runs {
        let iter = match vm.execute().expect("execute failed") {
            ExecutionResult::Query(iter) => iter,
        };

        let mut results: Vec<Value> = Vec::new();
        for row_result in iter.take(take.unwrap_or(usize::MAX)) {
            let row = row_result.expect("row evaluation failed");
            results.push(row_to_value(&row, &shape));
        }

        values.push(match &shape {
            Shape::Bag(_) => Value::Bag(Box::new(Bag::from(results))),
            Shape::List(_) => Value::List(Box::new(List::from(results))),
            Shape::Single(_) => results.into_iter().next().unwrap_or(Value::Missing),
        });
    }
    values
}

#[track_caller]
fn assert_vm_eval(query: &str, tables: &[(&str, &str)], expected_ion: &str) {
    let actual = eval_vm(query, tables);
    let expected = parse_ion(expected_ion);
    assert_eq!(
        expected, actual,
        "\n\nQuery: {query}\nExpected: {expected:?}\nActual:   {actual:?}\n"
    );
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn select_value_simple() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d",
        &[("data", "[{a: 1}, {a: 2}, {a: 3}]")],
        "$bag::[1, 2, 3]",
    );
}

#[test]
fn select_value_arithmetic() {
    assert_vm_eval(
        "SELECT VALUE d.a + 1 FROM data AS d",
        &[("data", "[{a: 10}, {a: 20}]")],
        "$bag::[11, 21]",
    );
}

#[test]
fn select_value_with_filter() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d WHERE d.a > 1",
        &[("data", "[{a: 1}, {a: 2}, {a: 3}]")],
        "$bag::[2, 3]",
    );
}

#[test]
fn select_value_nested_tuple() {
    assert_vm_eval(
        "SELECT VALUE {'x': d.a, 'y': d.b} FROM data AS d",
        &[("data", "[{a: 1, b: 10}, {a: 2, b: 20}]")],
        "$bag::[{x: 1, y: 10}, {x: 2, y: 20}]",
    );
}

#[test]
fn select_value_nested_list() {
    assert_vm_eval(
        "SELECT VALUE [d.a, d.b] FROM data AS d",
        &[("data", "[{a: 1, b: 2}]")],
        "$bag::[[1, 2]]",
    );
}

#[test]
fn select_value_mul() {
    assert_vm_eval(
        "SELECT VALUE d.a * d.b FROM data AS d",
        &[("data", "[{a: 3, b: 7}, {a: 2, b: 5}]")],
        "$bag::[21, 10]",
    );
}

#[test]
fn select_star() {
    assert_vm_eval(
        "SELECT * FROM data AS d",
        &[("data", "[{a: 1, b: 2}, {a: 3, b: 4}]")],
        "$bag::[{a: 1, b: 2}, {a: 3, b: 4}]",
    );
}

#[test]
fn select_named_columns() {
    assert_vm_eval(
        "SELECT d.a AS x, d.b AS y FROM data AS d",
        &[("data", "[{a: 1, b: 2}, {a: 3, b: 4}]")],
        "$bag::[{x: 1, y: 2}, {x: 3, y: 4}]",
    );
}

// `SchemalessMetadata` resolves every field, so these exercise scan pushdown:
// a binding used both as a whole and through a field must still load whole rows.

#[test]
fn select_star_with_where_keeps_all_fields() {
    assert_vm_eval(
        "SELECT * FROM data AS d WHERE d.a > 1",
        &[("data", "[{a: 1, b: 2}, {a: 3, b: 4, c: 5}]")],
        "$bag::[{a: 3, b: 4, c: 5}]",
    );
}

#[test]
fn whole_row_and_field_in_projection() {
    assert_vm_eval(
        "SELECT VALUE {'row': d, 'a': d.a} FROM data AS d",
        &[("data", "[{a: 1, b: 2}, {a: 3, b: 4}]")],
        "$bag::[{row: {a: 1, b: 2}, a: 1}, {row: {a: 3, b: 4}, a: 3}]",
    );
}

// Path steps after a pushed-down field navigate within that field.

#[test]
fn nested_path_after_pushdown() {
    assert_vm_eval(
        "SELECT VALUE d.a.b FROM data AS d",
        &[("data", "[{a: {b: 1, c: 2}}, {a: {b: 3, c: 4}}]")],
        "$bag::[1, 3]",
    );
}

#[test]
fn deep_nested_path_after_pushdown() {
    assert_vm_eval(
        "SELECT d.a.b.c AS x FROM data AS d WHERE d.a.b.c > 1",
        &[("data", "[{a: {b: {c: 1}}}, {a: {b: {c: 2, d: 0}}}]")],
        "$bag::[{x: 2}]",
    );
}

/// `k.k` reads field `k` of the GROUP BY key `k`, not the key itself.
#[test]
fn path_into_group_key_named_like_its_field() {
    assert_vm_eval(
        "SELECT k.k AS kk FROM data AS d GROUP BY d.k AS k",
        &[("data", "[{k: {k: 1}}]")],
        "$bag::[{kk: 1}]",
    );
}

/// Without GROUP BY the input is one group even when empty.
#[test]
fn aggregate_over_empty_input_is_one_row() {
    assert_vm_eval(
        "SELECT COUNT(*) AS c, SUM(d.a) AS s FROM data AS d",
        &[("data", "[]")],
        "$bag::[{c: 0, s: null}]",
    );
    assert_vm_eval(
        "SELECT COUNT(*) AS c FROM data AS d WHERE d.a > 1",
        &[("data", "[{a: 1}]")],
        "$bag::[{c: 0}]",
    );
}

#[test]
fn group_by_over_empty_input_has_no_groups() {
    assert_vm_eval(
        "SELECT k, COUNT(*) AS c FROM data AS d GROUP BY d.k AS k",
        &[("data", "[]")],
        "$bag::[]",
    );
}

/// NULL and MISSING group keys are one group, keyed NULL.
#[test]
fn group_by_missing_key_is_null() {
    assert_vm_eval(
        "SELECT k, COUNT(*) AS c FROM data AS d GROUP BY d.k AS k",
        &[("data", "[{k: null}, {x: 1}, {k: 2}]")],
        "$bag::[{k: null, c: 2}, {k: 2, c: 1}]",
    );
}

/// ORDER BY compares numbers by value across types, NaN first.
#[test]
fn order_by_numbers() {
    assert_vm_eval(
        "SELECT VALUE d.v FROM data AS d ORDER BY d.v",
        &[(
            "data",
            "[{v: 2.5e0}, {v: 1}, {v: nan}, {v: 3.0}, {v: -inf}, {v: +inf}, {v: 2}]",
        )],
        "[nan, -inf, 1, 2, 2.5e0, 3.0, +inf]",
    );
    assert_vm_eval(
        "SELECT VALUE d.v FROM data AS d ORDER BY d.v DESC",
        &[("data", "[{v: 1.5}, {v: 2e0}, {v: 1}]")],
        "[2e0, 1.5, 1]",
    );
}

/// NULLs nested in a collection sort with the outer ones: last for ASC, first for DESC.
#[test]
fn order_by_nested_nulls() {
    assert_vm_eval(
        "SELECT VALUE d.v FROM data AS d ORDER BY d.v",
        &[("data", "[{v: [1, null]}, {v: [1, 2]}]")],
        "[[1, 2], [1, null]]",
    );
    assert_vm_eval(
        "SELECT VALUE d.v FROM data AS d ORDER BY d.v DESC",
        &[("data", "[{v: [1, 2]}, {v: [1, null]}]")],
        "[[1, null], [1, 2]]",
    );
    assert_vm_eval(
        "SELECT VALUE d.v FROM data AS d ORDER BY d.v",
        &[("data", "[{v: {b: 1}}, {v: {a: 2}}, {v: {a: 1, b: 0}}]")],
        "[{a: 1, b: 0}, {a: 2}, {b: 1}]",
    );
}

/// Ints and floats compare exactly: near 2^53, `as f64` would make the order intransitive
/// and `sort_by` panic.
#[test]
fn order_by_int_float_near_2_pow_53() {
    assert_vm_eval(
        "SELECT VALUE d.v FROM data AS d ORDER BY d.v",
        &[("data", "[{v: 9007199254740992e0}, {v: 9007199254740993}, {v: 9007199254740992e0}, {v: 9007199254740992e0}, {v: 9007199254740992e0}, {v: 9007199254740992e0}, {v: 9007199254740993}, {v: 9007199254740992}, {v: 9007199254740992}, {v: 9007199254740992}, {v: 9007199254740992e0}, {v: 9007199254740993}, {v: 9007199254740993}, {v: 9007199254740992}, {v: 9007199254740992e0}, {v: 9007199254740993}, {v: 9007199254740992e0}, {v: 9007199254740992}, {v: 9007199254740992}, {v: 9007199254740992}, {v: 9007199254740992e0}, {v: 9007199254740992e0}, {v: 9007199254740992}, {v: 9007199254740993}, {v: 9007199254740992}, {v: 9007199254740993}, {v: 9007199254740993}, {v: 9007199254740993}, {v: 9007199254740992e0}, {v: 9007199254740993}, {v: 9007199254740992}, {v: 9007199254740992}, {v: 9007199254740993}]")],
        "[9007199254740992e0, 9007199254740992e0, 9007199254740992e0, 9007199254740992e0, 9007199254740992e0, 9007199254740992, 9007199254740992, 9007199254740992, 9007199254740992e0, 9007199254740992, 9007199254740992e0, 9007199254740992e0, 9007199254740992, 9007199254740992, 9007199254740992, 9007199254740992e0, 9007199254740992e0, 9007199254740992, 9007199254740992, 9007199254740992e0, 9007199254740992, 9007199254740992, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993, 9007199254740993]",
    );
}

/// Each run of a VM starts its sorters empty, even after a run that stopped early.
#[test]
fn rerun_starts_sorters_empty() {
    let query = "SELECT k, COUNT(*) AS c FROM data AS d GROUP BY d.k AS k ORDER BY k";
    let data = [("data", "[{k: \"b\"}, {k: \"a\"}, {k: \"b\"}]")];
    let expected = parse_ion("[{k: \"a\", c: 1}, {k: \"b\", c: 2}]");
    let runs = eval_vm_runs(query, &data, &[Some(1), None, None]);
    assert_eq!(runs[1..], [expected.clone(), expected]);
}

/// An uncorrelated subquery runs once per run of the VM, including after a run that stopped
/// early: each run evaluates it again rather than reading the previous run's freed rows.
#[test]
fn uncorrelated_subquery_reruns() {
    let query = "SELECT VALUE d.a || '!' FROM data AS d \
                 WHERE d.a IN (SELECT VALUE k.a || '' FROM keys AS k)";
    let data = [
        ("data", "[{a: \"x\"}, {a: \"y\"}, {a: \"z\"}]"),
        ("keys", "[{a: \"z\"}, {a: \"x\"}]"),
    ];
    let expected = parse_ion("$bag::[\"x!\", \"z!\"]");
    let runs = eval_vm_runs(query, &data, &[Some(1), None, None]);
    assert_eq!(runs[1..], [expected.clone(), expected]);
}

/// A correlated subquery runs for each row, reading the row's bindings.
#[test]
fn correlated_subquery() {
    assert_vm_eval(
        "SELECT d.a AS a, (SELECT VALUE k.b FROM keys AS k WHERE k.a = d.a) AS bs FROM data AS d",
        &[
            ("data", "[{a: 1}, {a: 2}]"),
            (
                "keys",
                "[{a: 1, b: \"p\"}, {a: 2, b: \"q\"}, {a: 1, b: \"r\"}]",
            ),
        ],
        "$bag::[{a: 1, bs: $bag::[\"p\", \"r\"]}, {a: 2, bs: $bag::[\"q\"]}]",
    );
}

/// Strict `SELECT *` of an Ion struct literal is the struct's fields, not the literal. A
/// value that is not a tuple becomes `{'_1': v}`, in strict mode too, as in the legacy
/// evaluator.
#[test]
fn strict_select_star() {
    let strict = |query| eval_vm_mode(query, &[], &[None], EvaluationMode::Strict);
    assert_eq!(
        strict("SELECT * FROM << `{a: \"hello\", b: 1}` >> x"),
        [parse_ion("$bag::[{a: \"hello\", b: 1}]")]
    );
    assert_eq!(
        strict("SELECT * FROM [TRUE, `1`] x"),
        [parse_ion("$bag::[{'_1': true}, {'_1': 1}]")]
    );
}

/// In strict mode, a LIMIT or OFFSET count that is not a non-negative integer is an error
/// (permissive mode ignores it).
#[test]
fn strict_limit_offset_counts() {
    // An unknown count is no count, in strict mode too
    let rows = eval_vm_mode(
        "SELECT VALUE x FROM <<1, 2>> x LIMIT NULL OFFSET MISSING",
        &[],
        &[None],
        EvaluationMode::Strict,
    );
    assert_eq!(rows, [parse_ion("$bag::[1, 2]")]);
    for query in [
        "SELECT VALUE x FROM <<1, 2>> x LIMIT 'a'",
        "SELECT VALUE x FROM <<1, 2>> x OFFSET 2.5",
        "SELECT VALUE x FROM <<1, 2>> x OFFSET 1 - 2",
    ] {
        let err = strict_error(query);
        assert!(err.contains("non-negative integer"), "{query}: {err}");
    }
}

/// The error a query (over no tables) raises in strict mode.
fn strict_error(query: &str) -> String {
    let parsed = Parser::default().parse(query).expect("parse failed");
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let planner = LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static);
    let logical = planner.lower(&parsed).expect("lower failed");
    let context = CompilationContext::new();
    let compiled = PlanCompiler::new(&context, EvaluationMode::Strict)
        .compile(&logical)
        .expect("compile failed");
    let mut vm = PartiQLVM::new(compiled, &ExecutionContext::new()).expect("VM creation failed");
    let ExecutionResult::Query(mut iter) = vm.execute().expect("execute failed");
    // The error may follow rows (a set operator's right operand fails after its left's rows)
    let err = iter.find_map(|row| row.err());
    format!("{err:?}")
}

/// In strict mode, an aggregate (SQL or `COLL_*`) over an input it can't take, or a
/// `COLL_*` function of a value that is no collection, is an error (permissive mode gives
/// MISSING).
#[test]
fn strict_aggregate_inputs() {
    for query in [
        "SELECT SUM(x) AS s FROM <<1, 'a'>> x",
        "SELECT VALUE ANY(x) FROM <<true, 5>> x",
        "SELECT VALUE COLL_SUM(<<1, 'a'>>) FROM <<0>> d",
        "SELECT VALUE COLL_COUNT('x') FROM <<0>> d",
    ] {
        let err = strict_error(query);
        assert!(err.contains("TypeError"), "{query}: {err}");
    }
}

/// In strict mode, EXTRACT of a value that is no datetime, or whose type has no such field (the
/// time zone of a timestamp without one), is an error (permissive mode gives MISSING).
#[test]
fn strict_extract_of_no_datetime() {
    for query in ["EXTRACT(YEAR FROM 1)", "EXTRACT(HOUR FROM 'a')"] {
        let err = strict_error(query);
        assert!(err.contains("no datetime"), "{query}: {err}");
    }
    for query in [
        "EXTRACT(TIMEZONE_HOUR FROM `2000T`)",
        "EXTRACT(TIMEZONE_MINUTE FROM `2000-01-01T00:00-00:00`)",
    ] {
        let err = strict_error(query);
        assert!(err.contains("without it"), "{query}: {err}");
    }
    assert_eq!(strict_error("EXTRACT(YEAR FROM `2000T`)"), "None");
}

/// In strict mode, comparing values of types that don't compare is an error (permissive mode
/// gives MISSING).
#[test]
fn strict_comparison_of_incomparable_types() {
    for query in ["1 < 'a'", "[1] >= 1", "'a' BETWEEN 1 AND 2"] {
        let err = strict_error(query);
        assert!(err.contains("don't compare"), "{query}: {err}");
    }
}

/// In strict mode, an operand of OUTER UNION, INTERSECT or EXCEPT that is no collection
/// (NULL and MISSING too) is an error (permissive mode makes it a bag: RFC-0007).
#[test]
fn strict_set_operator_operands() {
    for query in [
        "1 OUTER UNION << 2 >>",
        "<< 1 >> OUTER INTERSECT {'a': 1}",
        "NULL OUTER EXCEPT << 1 >>",
        "<< 1 >> OUTER UNION ALL MISSING",
    ] {
        let err = strict_error(query);
        assert!(
            err.contains("require collection operands"),
            "{query}: {err}"
        );
    }
}

/// A correlated subquery reruns cleanly after a run that stopped early, mid-way through the
/// outer rows.
#[test]
fn correlated_subquery_reruns() {
    let query = "SELECT d.a AS a, (SELECT VALUE k.b || '!' FROM keys AS k WHERE k.a = d.a) AS bs \
                 FROM data AS d ORDER BY d.a";
    let data = [
        ("data", "[{a: 1}, {a: 2}, {a: 3}]"),
        (
            "keys",
            "[{a: 1, b: \"p\"}, {a: 2, b: \"q\"}, {a: 1, b: \"r\"}]",
        ),
    ];
    let expected = parse_ion(
        "[{a: 1, bs: $bag::[\"p!\", \"r!\"]}, {a: 2, bs: $bag::[\"q!\"]}, {a: 3, bs: $bag::[]}]",
    );
    let runs = eval_vm_runs(query, &data, &[Some(1), None, None]);
    assert_eq!(runs[1..], [expected.clone(), expected]);
}

/// A subquery's code is emitted once however often it is read, so a FULL JOIN (which emits
/// its sides twice) of FROM subqueries, nested, stays small. Inlined, this one outgrew the
/// program size limit.
#[test]
fn subqueries_under_full_joins_compile_once() {
    let mut q = "<<1, 2>>".to_string();
    for i in 0..9 {
        q = format!("(SELECT VALUE a{i} FROM {q} a{i} FULL JOIN {q} b{i} ON a{i} = b{i})");
    }
    assert_vm_eval(&format!("SELECT VALUE x FROM {q} x"), &[], "$bag::[1, 2]");
}

/// `depth` FROM subqueries nested in one another.
fn nested_from_subqueries(depth: usize) -> String {
    let mut from = "<<1>> t0".to_string();
    for i in 1..=depth {
        from = format!("(SELECT VALUE t{} FROM {from}) t{i}", i - 1);
    }
    format!("SELECT VALUE t{depth} FROM {from}")
}

/// Subqueries nest up to a limit, within the stack of a test thread; past it, compiling
/// fails rather than overflowing the stack.
#[test]
fn subquery_nesting_limit() {
    assert_vm_eval(&nested_from_subqueries(32), &[], "$bag::[1]");

    let query = nested_from_subqueries(33);
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = Parser::default().parse(&query).expect("parse failed");
    let planner = LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static);
    let logical = planner.lower(&parsed).expect("lower failed");
    let context = CompilationContext::new();
    match PlanCompiler::new(&context, EvaluationMode::Permissive).compile(&logical) {
        Err(err) => assert!(format!("{err:?}").contains("subquery nesting"), "{err:?}"),
        Ok(_) => panic!("33 levels compiled"),
    }
}

/// GROUP BY tells collection keys apart.
#[test]
fn group_by_collection_keys() {
    assert_vm_eval(
        "SELECT k, COUNT(*) AS c FROM data AS d GROUP BY d.k AS k",
        &[(
            "data",
            "[{k: [1]}, {k: [2]}, {k: [1]}, {k: {a: 1}}, {k: {a: 2}}, {k: {a: 1}}]",
        )],
        "$bag::[{k: [1], c: 2}, {k: [2], c: 1}, {k: {a: 1}, c: 2}, {k: {a: 2}, c: 1}]",
    );
}

#[test]
fn filter_false_returns_empty() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d WHERE 1 > 2",
        &[("data", "[{a: 1}, {a: 2}]")],
        "$bag::[]",
    );
}

#[test]
fn filter_true_returns_all() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d WHERE 2 > 1",
        &[("data", "[{a: 10}, {a: 20}]")],
        "$bag::[10, 20]",
    );
}

#[test]
fn expr_query_literal() {
    assert_vm_eval("1 + 2", &[], "3");
}

#[test]
fn expr_query_string_concat() {
    assert_vm_eval("'hello' || ' ' || 'world'", &[], "'hello world'");
}

#[test]
fn complex_nested_construction() {
    assert_vm_eval(
        "SELECT VALUE {'a': d.a + 1, 'b': {'c': [d.b * 100]}} FROM data AS d WHERE 2 > 1",
        &[("data", "[{a: 0, b: 100}]")],
        "$bag::[{a: 1, b: {c: [10000]}}]",
    );
}

#[test]
fn multiple_rows_with_filter() {
    assert_vm_eval(
        "SELECT VALUE d.name FROM people AS d WHERE d.age >= 21",
        &[(
            "people",
            "[{name: \"alice\", age: 30}, {name: \"bob\", age: 17}, {name: \"carol\", age: 25}]",
        )],
        "$bag::[\"alice\", \"carol\"]",
    );
}

#[test]
fn limit_basic() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d LIMIT 2",
        &[("data", "[{a: 10}, {a: 20}, {a: 30}, {a: 40}]")],
        "$bag::[10, 20]",
    );
}

#[test]
fn limit_zero() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d LIMIT 0",
        &[("data", "[{a: 1}, {a: 2}, {a: 3}]")],
        "$bag::[]",
    );
}

#[test]
fn limit_exceeds_rows() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d LIMIT 100",
        &[("data", "[{a: 1}, {a: 2}]")],
        "$bag::[1, 2]",
    );
}

#[test]
fn limit_with_filter() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d WHERE d.a > 2 LIMIT 2",
        &[("data", "[{a: 1}, {a: 2}, {a: 3}, {a: 4}, {a: 5}]")],
        "$bag::[3, 4]",
    );
}

#[test]
fn limit_with_true_filter() {
    assert_vm_eval(
        "SELECT VALUE d.a FROM data AS d WHERE 1 = 1 LIMIT 2",
        &[("data", "[{a: 10}, {a: 20}, {a: 30}]")],
        "$bag::[10, 20]",
    );
}

/// The row bank holds both the ingested row and the per-field temporaries
/// computed from it. When temporaries accumulate enough to force the bank to
/// grow, the row must not move — later fields still read it.
#[test]
fn wide_row_crossing_row_bank() {
    let s = "x".repeat(400);
    let n = 48;
    let pairs: Vec<String> = (0..n).map(|i| format!("'k{i}': d.s || 'y'")).collect();
    let query = format!("SELECT VALUE {{{}}} FROM data AS d", pairs.join(", "));
    let expected: Vec<String> = (0..n).map(|i| format!("k{i}: \"{s}y\"")).collect();
    assert_vm_eval(
        &query,
        &[("data", &format!("[{{s: \"{s}\"}}]"))],
        &format!("$bag::[{{{}}}]", expected.join(", ")),
    );
}

// --- Catalog objects named in expressions ---

/// A table named in an expression of a query with FROM (outside its FROM sources) is the
/// collection of its rows, wherever the expression is: SELECT, WHERE, ORDER BY, a subquery.
#[test]
fn global_table_in_query_with_from() {
    let tables = [("t", "[{a: 1}, {a: 2}, {a: 3}]"), ("u", "[{b: 2}]")];
    assert_vm_eval(
        "SELECT u.b, COLL_COUNT(t) AS n FROM u WHERE COLL_COUNT(t) > 2",
        &tables,
        "$bag::[{b: 2, n: 3}]",
    );
    assert_vm_eval(
        "SELECT VALUE COLL_COUNT(t) FROM t AS x ORDER BY x.a",
        &tables,
        "[3, 3, 3]",
    );
    // In a correlated subquery, and named twice in one expression
    assert_vm_eval(
        "SELECT VALUE (SELECT VALUE COLL_COUNT(t) + COLL_COUNT(T) + y FROM [x.a] AS y) FROM t AS x",
        &tables,
        "$bag::[$bag::[7], $bag::[8], $bag::[9]]",
    );
}

/// Catalog objects inside a FROM source's expression: tables in a bag constructor, a path
/// into a single value.
#[test]
fn global_in_from_expression() {
    let tables = [
        ("numbers", "[1, 2, 3]"),
        ("hr", "{employees: [{name: \"ann\"}, {name: \"bob\"}]}"),
        ("i", "7"),
    ];
    assert_vm_eval(
        "SELECT VALUE COLL_COUNT(v) FROM <<numbers, numbers>> AS v",
        &tables,
        "$bag::[3, 3]",
    );
    assert_vm_eval(
        "SELECT VALUE e.name FROM hr.employees AS e",
        &tables,
        "$bag::[\"ann\", \"bob\"]",
    );
    assert_vm_eval(
        "SELECT VALUE x FROM << -1, i, 100 >> AS x WHERE x BETWEEN 0 AND i",
        &tables,
        "$bag::[7]",
    );
}

/// A single value named in a pipeline's expressions: a WHERE, a projection, a GROUP BY key
/// and an aggregate's argument, an operand of a set operator.
#[test]
fn global_value_in_pipeline() {
    let tables = [("i", "2"), ("d", "[1, 2, 3, 4]")];
    assert_vm_eval(
        "SELECT VALUE x * i FROM d AS x WHERE x > i",
        &tables,
        "$bag::[6, 8]",
    );
    assert_vm_eval(
        "SELECT k, SUM(x + i) AS s FROM d AS x GROUP BY x > i AS k",
        &tables,
        "$bag::[{k: false, s: 7}, {k: true, s: 11}]",
    );
    assert_vm_eval(
        "SELECT VALUE x FROM d AS x WHERE x < i OUTER UNION ALL SELECT VALUE i FROM [0] AS y",
        &tables,
        "$bag::[1, 2]",
    );
}

/// A FROM binding that the planner resolves a name to hides a global of the same name; one
/// named before the binding (an earlier ON) is the global.
#[test]
fn global_shadowed_by_a_later_binding() {
    assert_vm_eval(
        "SELECT VALUE [a, b, d] FROM [1, 3] AS a INNER JOIN [1, 2, 3] AS b ON b < d \
         LEFT JOIN [1.1, 2.1] AS d ON b < d AND a <= d",
        &[("d", "2")],
        "$bag::[[1, 1, 1.1], [1, 1, 2.1], [3, 1, null]]",
    );
}

/// A nested single value, read whole and through paths, for every row.
#[test]
fn global_nested_value() {
    let tables = [
        ("hr", "{boss: \"ann\", deep: {xs: [{a: 1}, {a: 2}]}}"),
        ("d", "[1, 2]"),
    ];
    assert_vm_eval(
        "SELECT VALUE [x, hr.deep.xs, hr.boss] FROM d AS x",
        &tables,
        "$bag::[[1, [{a: 1}, {a: 2}], \"ann\"], [2, [{a: 1}, {a: 2}], \"ann\"]]",
    );
    assert_vm_eval(
        "SELECT VALUE y.a FROM hr.deep.xs AS y",
        &tables,
        "$bag::[1, 2]",
    );
}

/// A single value whose source has no row is MISSING.
#[test]
fn global_value_without_a_row() {
    assert_vm_eval("[i, 1]", &[("i", "")], "[$missing::null, 1]");
    assert_vm_eval(
        "SELECT VALUE x FROM << i, 1 >> AS x",
        &[("i", "")],
        "$bag::[$missing::null, 1]",
    );
}

/// Globals in LIMIT, in outer joins' ON, in a LIKE pattern and ESCAPE, and in subqueries two
/// levels deep.
#[test]
fn global_in_other_clauses() {
    let tables = [
        ("n", "2"),
        ("s", "\"h%\""),
        ("e", "\"!\""),
        ("t", "[{a: 1}, {a: 2}, {a: 3}]"),
    ];
    assert_vm_eval(
        "SELECT VALUE x.a FROM t AS x LIMIT n",
        &tables,
        "$bag::[1, 2]",
    );
    assert_vm_eval(
        "SELECT VALUE [x, y] FROM [1, 2] AS x LEFT JOIN [2] AS y ON x = n",
        &tables,
        "$bag::[[1, null], [2, 2]]",
    );
    assert_vm_eval(
        "SELECT VALUE [x, y] FROM [1] AS x FULL JOIN [2] AS y ON y = n AND x = n",
        &tables,
        "$bag::[[1, null], [null, 2]]",
    );
    assert_vm_eval(
        "SELECT VALUE [w LIKE s, w LIKE 'h!%' ESCAPE e] FROM ['hi', 'h%'] AS w",
        &tables,
        "$bag::[[true, false], [true, true]]",
    );
    assert_vm_eval(
        "SELECT VALUE (SELECT VALUE (SELECT VALUE COLL_COUNT(t) * n + x + y FROM [0] AS z) \
         FROM [10] AS y) FROM [100] AS x",
        &tables,
        "$bag::[$bag::[$bag::[116]]]",
    );
}

/// A string read from a global stays valid for every row that reads it, across many rows.
#[test]
fn global_string_across_rows() {
    let s = "x".repeat(1000);
    let rows: Vec<String> = (0..300).map(|i| format!("\"{i}\"")).collect();
    let expected: Vec<String> = (0..300)
        .map(|i: usize| (1000 + i.to_string().len()).to_string())
        .collect();
    assert_vm_eval(
        "SELECT VALUE CHAR_LENGTH(s || r) FROM d AS r",
        &[
            ("s", &format!("\"{s}\"")),
            ("d", &format!("[{}]", rows.join(", "))),
        ],
        &format!("$bag::[{}]", expected.join(", ")),
    );
}

/// Each run of a VM reads its globals again: a run that stops early leaves none half read.
#[test]
fn global_across_runs() {
    let tables = [
        ("t", "[{a: 1}, {a: 2}, {a: 3}]"),
        ("u", "[{b: 10}, {b: 20}]"),
        ("i", "5"),
    ];
    let runs = eval_vm_runs(
        "SELECT VALUE COLL_COUNT(t) + x.b + i FROM u AS x",
        &tables,
        &[None, Some(0), Some(1), None],
    );
    let expected = ["$bag::[18, 28]", "$bag::[]", "$bag::[18]", "$bag::[18, 28]"];
    for (run, expected) in runs.into_iter().zip(expected) {
        assert_eq!(parse_ion(expected), run);
    }
}
