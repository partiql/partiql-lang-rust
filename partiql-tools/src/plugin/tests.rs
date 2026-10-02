//! In-process ABI tests: a Rust plugin implementing `pqlite_plugin.h`, loaded
//! through `load_from_init` (everything except the `dlopen`).

use std::ffi::{c_char, c_void, CString};
use std::sync::{Arc, Mutex};

use arrow_array::builder::{Int32Builder, ListBuilder, StringBuilder};
use arrow_array::ffi_stream::FFI_ArrowArrayStream;
use arrow_array::types::Int8Type;
use arrow_array::{
    ArrayRef, Decimal128Array, DictionaryArray, Float64Array, Int64Array, RecordBatch,
    RecordBatchIterator, StringArray, StructArray,
};
use arrow_schema::ffi::FFI_ArrowSchema;
use arrow_schema::{DataType, Field, Fields, Schema};

use super::ffi::*;
use super::load_from_init;
use crate::session::{render_query_text, Commands, DebugFlags, PqliteSession, RunOutcome};
use crate::table_fns::TableFnRegistry;

/// What the last `open` call asked for: (fn_index, whole_row, fields).
static LAST_OPEN: Mutex<Option<(u32, bool, Vec<String>)>> = Mutex::new(None);
static CONFIG_SEEN: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
/// Serialises tests: they share `LAST_OPEN` and `CONFIG_SEEN`.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

const SEQ: u32 = 0;
const BOOM: u32 = 1;
const TYPED: u32 = 2;
const ECHO: u32 = 3;

fn def(name: &'static str, usage: &'static str, min: u32, max: u32) -> PqliteTableFnDef {
    PqliteTableFnDef {
        struct_size: std::mem::size_of::<PqliteTableFnDef>() as u32,
        name: PqliteStr::new(name),
        usage: PqliteStr::new(usage),
        min_args: min,
        max_args: max,
        static_schema: None,
    }
}

fn typed_schema() -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("label", DataType::Utf8, false),
    ])
}

unsafe extern "C" fn typed_static_schema(
    _data: *mut c_void,
    _fn_index: u32,
    out: *mut FFI_ArrowSchema,
) -> i32 {
    match FFI_ArrowSchema::try_from(&typed_schema()) {
        Ok(s) => {
            std::ptr::write(out, s);
            0
        }
        Err(_) => -1,
    }
}

fn make_vtable(abi_version: u32) -> &'static PqlitePluginV1 {
    let mut typed = def("typed", "typed(n) — fixed schema", 1, 1);
    typed.static_schema = Some(typed_static_schema);
    let fns: &'static [PqliteTableFnDef] = Box::leak(
        vec![
            def("seq", "seq(n[, batch]) — test rows", 1, 2),
            def("boom", "", 0, 0),
            typed,
            def("echo", "echo(x)", 1, 1),
        ]
        .into_boxed_slice(),
    );
    Box::leak(Box::new(PqlitePluginV1 {
        struct_size: std::mem::size_of::<PqlitePluginV1>() as u32,
        abi_version,
        plugin_name: PqliteStr::new("testplug"),
        plugin_version: PqliteStr::new("0.0.1"),
        plugin_data: std::ptr::null_mut(),
        n_functions: fns.len(),
        functions: fns.as_ptr(),
        open: Some(test_open),
        free_string: Some(test_free_string),
    }))
}

unsafe extern "C" fn test_init(host: *const PqliteHostV1, out: *mut *const PqlitePluginV1) -> i32 {
    let host = &*host;
    assert_eq!(host.abi_version, PQLITE_PLUGIN_ABI_VERSION);
    let cfg = std::slice::from_raw_parts(host.config, host.n_config);
    *CONFIG_SEEN.lock().unwrap() = cfg
        .iter()
        .map(|kv| (kv.key.to_string_lossy(), kv.value.to_string_lossy()))
        .collect();
    if let Some(log) = host.log {
        log(
            host.host_data,
            PQLITE_LOG_DEBUG,
            PqliteStr::new("init"),
            PqliteStr::new("hello"),
        );
    }
    *out = make_vtable(PQLITE_PLUGIN_ABI_VERSION);
    0
}

unsafe extern "C" fn bad_abi_init(
    _host: *const PqliteHostV1,
    out: *mut *const PqlitePluginV1,
) -> i32 {
    *out = make_vtable(PQLITE_PLUGIN_ABI_VERSION + 1);
    0
}

unsafe extern "C" fn test_free_string(s: *mut c_char) {
    drop(CString::from_raw(s));
}

unsafe extern "C" fn test_open(
    _data: *mut c_void,
    req: *const PqliteOpenRequest,
    out: *mut FFI_ArrowArrayStream,
    err: *mut *mut c_char,
) -> i32 {
    let req = &*req;
    let args = std::slice::from_raw_parts(req.args, req.n_args);
    let fields: Vec<String> = if req.n_fields == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(req.fields, req.n_fields)
            .iter()
            .map(|f| f.to_string_lossy())
            .collect()
    };
    *LAST_OPEN.lock().unwrap() = Some((req.fn_index, req.whole_row != 0, fields.clone()));
    assert!(!req.cancelled.is_null());

    let batches = match req.fn_index {
        SEQ => {
            let n = args[0].v.i as usize;
            let per = if args.len() > 1 {
                args[1].v.i as usize
            } else {
                4
            };
            seq_batches(n, per, req.whole_row != 0, &fields)
        }
        BOOM => {
            *err = CString::new("boom: nothing to see").unwrap().into_raw();
            return 7;
        }
        TYPED => {
            let n = args[0].v.i;
            let ids: Vec<i64> = (0..n).collect();
            let labels: Vec<String> = ids.iter().map(|i| format!("L{i}")).collect();
            vec![RecordBatch::try_new(
                Arc::new(typed_schema()),
                vec![
                    Arc::new(Int64Array::from(ids)),
                    Arc::new(StringArray::from(labels)),
                ],
            )
            .unwrap()]
        }
        ECHO => {
            let a = &args[0];
            let text = match a.kind {
                PQLITE_ARG_INT => format!("int:{}", a.v.i),
                PQLITE_ARG_STRING => format!("str:{}", a.v.s.to_string_lossy()),
                PQLITE_ARG_ION_TEXT => format!("ion:{}", a.v.s.to_string_lossy()),
                PQLITE_ARG_NULL => "null".to_string(),
                PQLITE_ARG_MISSING => "missing".to_string(),
                k => format!("kind:{k}"),
            };
            let schema = Arc::new(Schema::new(vec![Field::new("arg", DataType::Utf8, false)]));
            vec![
                RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(vec![text]))])
                    .unwrap(),
            ]
        }
        _ => unreachable!(),
    };
    let schema = batches[0].schema();
    let reader = RecordBatchIterator::new(batches.into_iter().map(Ok), schema);
    std::ptr::write(out, FFI_ArrowArrayStream::new(Box::new(reader)));
    0
}

/// `n` rows in batches of `per`, projected to `fields` unless `whole_row`.
fn seq_batches(n: usize, per: usize, whole_row: bool, fields: &[String]) -> Vec<RecordBatch> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < n || out.is_empty() {
        let end = (start + per).min(n);
        let rows: Vec<usize> = (start..end).collect();
        let id: ArrayRef = Arc::new(Int64Array::from_iter_values(rows.iter().map(|&i| i as i64)));
        let name: ArrayRef = Arc::new(StringArray::from_iter(
            rows.iter().map(|&i| (i % 3 != 2).then(|| format!("n{i}"))),
        ));
        let score: ArrayRef = Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|&i| i as f64 * 0.5),
        ));
        let color: ArrayRef = Arc::new(
            DictionaryArray::<Int8Type>::try_new(
                rows.iter().map(|&i| (i % 2) as i8).collect(),
                Arc::new(StringArray::from(vec!["red", "green"])),
            )
            .unwrap(),
        );
        let amount: ArrayRef = Arc::new(
            Decimal128Array::from_iter_values(rows.iter().map(|&i| i as i128 * 125))
                .with_precision_and_scale(10, 2)
                .unwrap(),
        );
        let mut a = Int32Builder::new();
        let mut b = ListBuilder::new(StringBuilder::new());
        for &i in &rows {
            a.append_value(i as i32 * 10);
            for j in 0..(i % 3) {
                b.values().append_value(format!("t{j}"));
            }
            b.append(true);
        }
        let a: ArrayRef = Arc::new(a.finish());
        let b: ArrayRef = Arc::new(b.finish());
        let tags_fields = Fields::from(vec![
            Field::new("a", DataType::Int32, true),
            Field::new("b", b.data_type().clone(), true),
        ]);
        let tags: ArrayRef = Arc::new(StructArray::new(tags_fields.clone(), vec![a, b], None));

        let all: Vec<(&str, ArrayRef)> = vec![
            ("id", id),
            ("name", name),
            ("score", score),
            ("color", color),
            ("amount", amount),
            ("tags", tags),
        ];
        let cols: Vec<(&str, ArrayRef)> = all
            .into_iter()
            .filter(|(n, _)| whole_row || fields.iter().any(|f| f == n))
            .collect();
        let schema = Arc::new(Schema::new(
            cols.iter()
                .map(|(n, c)| Field::new(*n, c.data_type().clone(), true))
                .collect::<Vec<_>>(),
        ));
        let arrays = cols.into_iter().map(|(_, c)| c).collect();
        let batch = if schema.fields().is_empty() {
            RecordBatch::try_new_with_options(
                schema,
                arrays,
                &arrow_array::RecordBatchOptions::new().with_row_count(Some(rows.len())),
            )
        } else {
            RecordBatch::try_new(schema, arrays)
        }
        .unwrap();
        out.push(batch);
        start = end;
    }
    out
}

fn registry() -> TableFnRegistry {
    let mut reg = TableFnRegistry::builtin();
    let cfg = vec![("testplug.k".to_string(), "v".to_string())];
    let info = unsafe { load_from_init("libtest.so", test_init, &cfg, &mut reg) }.unwrap();
    assert_eq!(info.name, "testplug");
    assert_eq!(info.functions, ["seq", "boom", "typed", "echo"]);
    reg
}

fn run(sql: &str) -> Result<String, String> {
    let session = PqliteSession::open_without_db(DebugFlags::default()).with_table_fns(registry());
    let (res, _) = session.run(&Commands::Exec {
        query: sql.to_string(),
    });
    match res.map_err(|e| e.to_string())? {
        RunOutcome::Query(handle) => {
            let mut buf = Vec::new();
            handle
                .drain(|rows, shape| render_query_text(rows, shape, &mut buf))
                .map_err(|e| e.to_string())?;
            Ok(String::from_utf8(buf).unwrap())
        }
        RunOutcome::Statement(_) => Err("expected a query".into()),
    }
}

fn last_open() -> (u32, bool, Vec<String>) {
    LAST_OPEN.lock().unwrap().clone().expect("open was called")
}

#[test]
fn whole_row_scan_maps_every_arrow_type() {
    let _g = serial();
    let out = run("SELECT VALUE t FROM seq(5, 2) AS t").unwrap();
    assert_eq!(last_open(), (SEQ, true, vec![]));
    // Row 2: null name, dictionary-decoded colour, decimal, struct with a list.
    assert!(out.contains("'id': 2"), "{out}");
    assert!(
        out.contains("'name': NULL") || out.contains("'name': null"),
        "{out}"
    );
    assert!(
        out.contains("'color': 'red'") && out.contains("'color': 'green'"),
        "{out}"
    );
    assert!(out.contains("2.50"), "{out}");
    assert!(out.contains("'b': ['t0', 't1']"), "{out}");
    assert_eq!(out.matches("'id':").count(), 5, "{out}");
}

#[test]
fn projection_is_pushed_down() {
    let _g = serial();
    let out = run("SELECT t.id, t.score FROM seq(6) AS t WHERE t.score > 1.5").unwrap();
    let (idx, whole, mut fields) = last_open();
    fields.sort();
    assert_eq!(
        (idx, whole, fields),
        (SEQ, false, vec!["id".into(), "score".into()])
    );
    assert_eq!(out.matches("'id':").count(), 2, "{out}");
    assert!(out.contains("'id': 4") && out.contains("'id': 5"), "{out}");
}

#[test]
fn count_star_and_empty_stream() {
    let _g = serial();
    let out = run("SELECT COUNT(*) AS c FROM seq(10, 3) AS t").unwrap();
    assert!(out.contains("'c': 10"), "{out}");
    // An empty stream must end the scan cleanly. (The VM currently yields no
    // row for COUNT(*) over empty input, with built-in sources too.)
    run("SELECT COUNT(*) AS c FROM seq(0) AS t").unwrap();
    let out = run("SELECT t.id FROM seq(0) AS t").unwrap();
    assert!(!out.contains("'id'"), "{out}");
}

#[test]
fn absent_field_reads_as_missing() {
    let _g = serial();
    let out = run("SELECT t.id, t.nope FROM seq(1) AS t").unwrap();
    assert!(out.contains("{ 'id': 0, 'nope': MISSING }"), "{out}");
}

#[test]
fn plugin_open_error_surfaces() {
    let _g = serial();
    let err = run("SELECT * FROM boom() AS t").unwrap_err();
    assert!(err.contains("boom: nothing to see"), "{err}");
}

#[test]
fn static_schema_resolves_typed_columns() {
    let _g = serial();
    let out = run("SELECT t.label, t.id FROM typed(3) AS t WHERE t.id >= 1").unwrap();
    assert!(
        out.contains("'label': 'L1'") && out.contains("'id': 2"),
        "{out}"
    );
    assert!(!out.contains("'L0'"), "{out}");
}

#[test]
fn arguments_cross_as_scalars_or_ion_text() {
    let _g = serial();
    let out = run("SELECT VALUE t.arg FROM echo(42) AS t").unwrap();
    assert!(out.contains("int:42"), "{out}");
    let out = run("SELECT VALUE t.arg FROM echo('x') AS t").unwrap();
    assert!(out.contains("str:x"), "{out}");
    let out = run("SELECT VALUE t.arg FROM echo({'a': [1, 2.5, 'q'], 'b': null, 'c': <<>>}) AS t")
        .unwrap();
    assert!(
        out.contains(r#"ion:{"a": [1, 2.5, "q"], "b": null, "c": $bag::[]}"#),
        "{out}"
    );
}

#[test]
fn config_is_passed_to_init() {
    let _g = serial();
    let _ = registry();
    assert!(CONFIG_SEEN
        .lock()
        .unwrap()
        .contains(&("testplug.k".to_string(), "v".to_string())));
}

#[test]
fn abi_mismatch_is_rejected() {
    let _g = serial();
    let mut reg = TableFnRegistry::builtin();
    let err = unsafe { load_from_init("libbad.so", bad_abi_init, &[], &mut reg) }
        .err()
        .unwrap();
    assert!(err.contains("ABI version 2"), "{err}");
    assert!(reg.get("seq").is_none());
}

#[test]
fn duplicate_function_names_are_rejected() {
    let _g = serial();
    let mut reg = registry();
    let err = unsafe { load_from_init("libtest.so", test_init, &[], &mut reg) }
        .err()
        .unwrap();
    assert!(err.contains("already registered"), "{err}");
}
