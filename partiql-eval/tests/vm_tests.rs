use std::sync::Arc;

use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, TypeEnvEntry};
use partiql_common::catalog::EntryId;
use partiql_eval::plan::EvaluationMode;
use partiql_eval::source::{
    BufferStability, CatalogScans, DataSource, DataSourceHandle, DataSourceMetadata, PhysicalType,
    RegisterWriter, ScanLayout, ScanSource, ScanSourceType, ValueWriter,
};
use partiql_eval::value::{RegisterReader, RowShape, Shape, ValueType, ValueView};
use partiql_eval::{
    CompilationCatalog, CompilationContext, ExecutionCatalog, ExecutionContext, ExecutionResult,
    PartiQLVM, PlanCompiler,
};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::Parser;
use partiql_types::{PartiqlShapeBuilder, StructConstraint, StructType};
use partiql_value::{Bag, BindingsName, List, Tuple, Value};

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
    fn open(&mut self) -> partiql_eval::Result<()> {
        self.cursor = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> partiql_eval::Result<bool> {
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

    fn close(&mut self) -> partiql_eval::Result<()> {
        Ok(())
    }
}

fn write_value_to_slot(
    writer: &mut RegisterWriter<'_, '_>,
    slot: u16,
    value: &Value,
) -> partiql_eval::Result<()> {
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

fn write_nested_value(vw: &mut ValueWriter<'_, '_>, value: &Value) -> partiql_eval::Result<()> {
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

struct SchemalessMetadata;

impl DataSourceMetadata for SchemalessMetadata {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::field(field_name, PhysicalType::Dynamic))
    }
}

struct TestCompilationCatalog {
    tables: FxHashMap<String, EntryId>,
}

impl CompilationCatalog for TestCompilationCatalog {
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        if path.len() != 1 {
            return None;
        }
        let name = match &path[0] {
            BindingsName::CaseSensitive(s) => s.as_ref(),
            BindingsName::CaseInsensitive(s) => s.as_ref(),
        };
        self.tables
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, entry_id)| DataSourceHandle::new(*entry_id, Arc::new(SchemalessMetadata)))
    }
}

/// Stores Ion text (Send+Sync safe) and parses to Value on create().
struct TestExecutionCatalog {
    table_ion: FxHashMap<EntryId, String>,
    scan_mappings: FxHashMap<partiql_eval::source::ScanId, (EntryId, ScanLayout)>,
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
        scan_id: partiql_eval::source::ScanId,
    ) -> partiql_eval::Result<Box<dyn DataSource>> {
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!("ScanId {:?} not prepared", scan_id))
        })?;
        let ion_text = self.table_ion.get(entry_id).ok_or_else(|| {
            partiql_eval::EngineError::IllegalState(format!("EntryId {:?} not found", entry_id))
        })?;
        let rows = ion_to_rows(ion_text);
        Ok(Box::new(InMemorySource::new(rows, layout.clone())))
    }
}

// --- Result extraction ---

fn row_to_value(row: &RegisterReader<'_>, shape: &Shape) -> Value {
    use partiql_eval::value::FieldName;

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
    }
}

// --- Main test harness ---

fn eval_vm(query: &str, tables: &[(&str, &str)]) -> Value {
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
        comp_tables.insert(name.to_string(), entry_id);
        exec_ion.insert(entry_id, ion_text.to_string());
    }

    let comp_catalog = Arc::new(TestCompilationCatalog {
        tables: comp_tables,
    });

    // Parse + Lower
    let parsed = Parser::default().parse(query).expect("parse failed");
    let planner = LogicalPlanner::new(&shared_catalog);
    let logical = planner.lower(&parsed).expect("lower failed");

    // Compile
    let mut context = CompilationContext::new();
    let catalog_id = context.add_catalog("default", comp_catalog);
    let mut compiler = PlanCompiler::new(&context, EvaluationMode::Permissive);
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

    let iter = match vm.execute().expect("execute failed") {
        ExecutionResult::Query(iter) => iter,
    };

    let mut results: Vec<Value> = Vec::new();
    for row_result in iter {
        let row = row_result.expect("row evaluation failed");
        results.push(row_to_value(&row, &shape));
    }

    match &shape {
        Shape::Bag(_) | Shape::List(_) => Value::Bag(Box::new(Bag::from(results))),
        Shape::Single(_) => results.into_iter().next().unwrap_or(Value::Missing),
    }
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
