use std::sync::Arc;

use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, TypeEnvEntry};
use partiql_common::catalog::EntryId;
use partiql_eval::source::{
    BufferStability, CatalogScans, DataSource, DataSourceHandle, DataSourceMetadata, PhysicalType,
    RegisterWriter, ScanId, ScanLayout, ScanSource, ScanSourceType, ValueWriter,
};
use partiql_eval::value::{FieldName, RegisterReader, RowShape, Shape, ValueType, ValueView};
use partiql_eval::{
    CompilationCatalog, CompilationContext, EngineError, ExecutionCatalog, ExecutionContext,
    ExecutionResult, PartiQLVM, PlanCompiler,
};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical_planner::LogicalPlanner;
use partiql_types::{PartiqlShapeBuilder, StructConstraint, StructType};
use partiql_value::{Bag, BindingsName, List, Tuple, Value};

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

// =============================================================================
// Catalog Implementations
// =============================================================================

struct SchemalessMetadata;

impl DataSourceMetadata for SchemalessMetadata {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::field(field_name, PhysicalType::Dynamic))
    }
}

struct ConformanceCompilationCatalog {
    tables: FxHashMap<String, EntryId>,
}

impl CompilationCatalog for ConformanceCompilationCatalog {
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

    fn create(&self, scan_id: ScanId) -> partiql_eval::Result<Box<dyn DataSource>> {
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

                comp_tables.insert(key.to_string(), entry_id);
                exec_ion.insert(entry_id, value_to_ion_text(value));
            }
        }
    }

    let shared_catalog = catalog.to_shared_catalog();

    let parsed = parse(statement)?;
    let planner = LogicalPlanner::new(&shared_catalog);
    let logical = planner.lower(&parsed).map_err(TestError::Lower)?;

    let comp_catalog = Arc::new(ConformanceCompilationCatalog {
        tables: comp_tables,
    });

    let mut context = CompilationContext::new();
    let catalog_id = context.add_catalog("default", comp_catalog);
    let eval_mode: partiql_eval::plan::EvaluationMode = mode.into();
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
        Shape::Bag(_) | Shape::List(_) => Value::Bag(Box::new(Bag::from(results))),
        Shape::Single(_) => results.into_iter().next().unwrap_or(Value::Missing),
    };

    Ok(result)
}
