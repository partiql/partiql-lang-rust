//! Parse + lower + compile + execute one statement.
//!
//! SELECT streams row-by-row into `out`; non-query statements buffer a small
//! `StatementOutcome` for the caller to render.

use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use partiql_ast::ast;
use partiql_eval::value::Shape;
use partiql_logical::{BindingsOp, LogicalPlan, LogicalStatement};

use crate::common;
use crate::session::debug::DebugFlags;
use crate::session::ion_output::stream_query_ion;
use crate::session::naming::{canonical_table_key, normalize_query};
use crate::session::outcome::{DebugCapture, OutputFormat, StatementOutcome, StatementTiming};
use crate::session::planner;
use crate::session::value;
use crate::storage::{self, HeedDB};

/// What `exec::run` streamed to `out`. Query rows are already on the wire; the
/// caller only needs count + timing for the footer. Non-query outcomes carry
/// their own metadata for the caller to render.
pub(super) enum RunResult {
    Query {
        row_count: u64,
        timing: StatementTiming,
    },
    NonQuery(StatementOutcome),
}

struct StatementCtx<'a> {
    debug: &'a DebugFlags,
    db: Option<&'a Arc<HeedDB>>,
    parse_time: Duration,
    lower_time: Duration,
    capture: &'a mut DebugCapture,
}

impl StatementCtx<'_> {
    fn timing(&self, compile: Duration, exec: Duration) -> StatementTiming {
        StatementTiming {
            parse: self.parse_time,
            lower: self.lower_time,
            compile,
            exec,
        }
    }
}

/// Parse + lower + dispatch. On the Query path, AST/plan/program flush to
/// `err` BEFORE row iteration so debug precedes results (dev parity). On Err,
/// any capture built up so far is returned in the second position.
pub(super) fn run(
    db: Option<&Arc<HeedDB>>,
    debug: &DebugFlags,
    sql: &str,
    format: OutputFormat,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> (Result<RunResult, Box<dyn std::error::Error>>, DebugCapture) {
    let sql = normalize_query(sql);
    if sql.is_empty() {
        return (Err("empty query".into()), DebugCapture::default());
    }

    let parse_start = Instant::now();
    let parsed = match common::parse(sql) {
        Ok(p) => p,
        Err(e) => {
            return (
                Err(format!("Parse error: {:?}", e).into()),
                DebugCapture::default(),
            );
        }
    };
    let parse_time = parse_start.elapsed();

    let mut capture = DebugCapture::default();
    if debug.ast {
        capture.ast = Some(format!("[AST] {parsed:?}"));
    }

    let stmt = match parsed.statements.first() {
        Some(s) => s,
        None => return (Err("no statements after parse".into()), capture),
    };

    match dispatch(stmt, debug, db, parse_time, &mut capture, format, out, err) {
        Ok(res) => (Ok(res), DebugCapture::default()),
        Err(e) => (Err(e), capture),
    }
}

/// Bootstrap entry: no debug capture, no rendering. Query statements route
/// their output to `io::sink()` so future SELECTs in the bootstrap script
/// don't brick startup.
pub(super) fn execute_statement_silent(
    stmt: &ast::AstNode<ast::Statement>,
    debug: &DebugFlags,
    db: Arc<HeedDB>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = DebugCapture::default();
    let catalog = common::create_table_fn_catalog();
    let statement =
        common::lower_statement(&*catalog, stmt).map_err(|e| format!("Lower error: {:?}", e))?;

    let mut ctx = StatementCtx {
        debug,
        db: Some(&db),
        parse_time: Duration::ZERO,
        lower_time: Duration::ZERO,
        capture: &mut capture,
    };

    match statement {
        LogicalStatement::Query(plan) => {
            let mut sink_out = std::io::sink();
            let mut sink_err = std::io::sink();
            stream_query(
                plan,
                &mut ctx,
                OutputFormat::Text,
                &mut sink_out,
                &mut sink_err,
            )
            .map(|_| ())
        }
        other => dispatch_write(other, &mut ctx).map(|_| ()),
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    stmt: &ast::AstNode<ast::Statement>,
    debug: &DebugFlags,
    db: Option<&Arc<HeedDB>>,
    parse_time: Duration,
    capture: &mut DebugCapture,
    format: OutputFormat,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<RunResult, Box<dyn std::error::Error>> {
    let catalog = common::create_table_fn_catalog();

    let lower_start = Instant::now();
    let statement =
        common::lower_statement(&*catalog, stmt).map_err(|e| format!("Lower error: {:?}", e))?;
    let lower_time = lower_start.elapsed();

    let mut ctx = StatementCtx {
        debug,
        db,
        parse_time,
        lower_time,
        capture,
    };

    match statement {
        LogicalStatement::Query(plan) => stream_query(plan, &mut ctx, format, out, err),
        other => dispatch_write(other, &mut ctx).map(RunResult::NonQuery),
    }
}

/// Non-query dispatch. Callers must have handled `Query` first.
fn dispatch_write(
    statement: LogicalStatement,
    ctx: &mut StatementCtx<'_>,
) -> Result<StatementOutcome, Box<dyn std::error::Error>> {
    match statement {
        LogicalStatement::Query(_) => Err("dispatch_write called with a Query statement".into()),
        LogicalStatement::CreateTableAs { table_name, query } => exec_ctas(table_name, query, ctx),
        LogicalStatement::InsertInto { table_name, query } => exec_insert(table_name, query, ctx),
        LogicalStatement::CreateTable { table_name } => exec_create_table(table_name, ctx),
    }
}

fn stream_query(
    plan: LogicalPlan<BindingsOp>,
    ctx: &mut StatementCtx<'_>,
    format: OutputFormat,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<RunResult, Box<dyn std::error::Error>> {
    let compile_start = Instant::now();
    let (compiled, catalog_id) =
        planner::build_compiled(&plan, ctx.debug, ctx.db.cloned(), ctx.capture)?;
    let compile_time = compile_start.elapsed();

    let exec_start = Instant::now();
    let exec_context = planner::build_exec_context(ctx.db.cloned(), catalog_id, &compiled);
    let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;
    let shape = vm.shape().clone();

    // Flush AST/plan/program NOW so debug on stderr precedes results on stdout.
    crate::session::flush_debug(ctx.capture, err)?;
    *ctx.capture = DebugCapture::default();

    let row_count = match vm.execute() {
        Ok(partiql_eval::ExecutionResult::Query(iter)) => match format {
            OutputFormat::Text => stream_query_text(iter, &shape, out)?,
            OutputFormat::Ion => stream_query_ion(iter, &shape, out)?,
        },
        Err(e) => return Err(format!("Execution setup error: {:?}", e).into()),
    };
    let exec_time = exec_start.elapsed();

    Ok(RunResult::Query {
        row_count,
        timing: StatementTiming {
            parse: ctx.parse_time,
            lower: ctx.lower_time,
            compile: compile_time,
            exec: exec_time,
        },
    })
}

/// Human-readable text: `<<\n  <row>,\n  ...\n>>\n` for Bag, `[`/`]` for List,
/// bare `{value:?}` for Single.
fn stream_query_text(
    iter: partiql_eval::QueryIterator<'_>,
    shape: &Shape,
    out: &mut dyn Write,
) -> Result<u64, Box<dyn std::error::Error>> {
    let (prefix, tab, suffix) = match shape {
        Shape::Bag(_) => (Some("<<"), "  ", Some(">>")),
        Shape::List(_) => (Some("["), "  ", Some("]")),
        Shape::Single(_) => (None, "", None),
    };
    if let Some(p) = prefix {
        writeln!(out, "{}", p)?;
    }

    let mut count: u64 = 0;
    let mut is_first = true;
    for row_result in iter {
        let row = row_result.map_err(|e| format!("Execution error: {:?}", e))?;
        let v = value::row_to_value(&row, shape).map_err(|e| format!("Execution error: {e}"))?;
        if is_first {
            is_first = false;
        } else {
            writeln!(out, ",")?;
        }
        write!(out, "{tab}{v:?}")?;
        count += 1;
    }
    writeln!(out)?;
    if let Some(s) = suffix {
        writeln!(out, "{}", s)?;
    }
    out.flush()?;
    Ok(count)
}

fn exec_ctas(
    table_name: partiql_value::BindingsName<'static>,
    query: LogicalPlan<BindingsOp>,
    ctx: &mut StatementCtx<'_>,
) -> Result<StatementOutcome, Box<dyn std::error::Error>> {
    let key = canonical_table_key(&table_name);

    // Resolve --db before compile + VM setup so a missing flag fails
    // cleanly before any expensive work.
    let db = ctx
        .db
        .ok_or("Error: The `--db <PATH>` option is required for CREATE TABLE AS.")?;

    // Encode the catalog value before draining so a deterministic encode error
    // fails fast, ahead of any query execution.
    let mut tables_value = Vec::new();
    crate::row_codec::serialize_name_row(&[&key], &mut tables_value)
        .map_err(|e| format!("Error: {}", e))?;

    let (rows, compile_time, exec_start) =
        planner::drain_source_rows(&query, ctx.debug, db, ctx.capture)?;

    let n = {
        let mut writer = db
            .create_table(&key, &tables_value)
            .map_err(|e| format!("Error: {}", e))?;
        for encoded in &rows {
            writer
                .push_row(encoded)
                .map_err(|e| format!("Error: {}", e))?;
        }
        writer.commit().map_err(|e| format!("Error: {}", e))?
    };
    let exec_time = exec_start.elapsed();

    Ok(StatementOutcome::CreateTableAs {
        table_name,
        canonical_key: key,
        rows: n,
        timing: ctx.timing(compile_time, exec_time),
        debug: std::mem::take(ctx.capture),
    })
}

fn exec_insert(
    table_name: partiql_value::BindingsName<'static>,
    query: LogicalPlan<BindingsOp>,
    ctx: &mut StatementCtx<'_>,
) -> Result<StatementOutcome, Box<dyn std::error::Error>> {
    let key = canonical_table_key(&table_name);

    if storage::is_system_table(&key) {
        return Err("Error: cannot INSERT into system table '_tables'".into());
    }

    // Resolve --db before compile + VM setup so a missing flag fails
    // cleanly before any expensive work.
    let db = ctx
        .db
        .ok_or("Error: The `--db <PATH>` option is required for INSERT.")?;

    let (rows, compile_time, exec_start) =
        planner::drain_source_rows(&query, ctx.debug, db, ctx.capture)?;

    // Row count is tallied here, not from commit(): open_table_for_append seeds
    // row_id at the high-water mark, so commit() returns the absolute counter
    // (existing + inserted), not this statement's insert count.
    let inserted = rows.len() as u64;
    {
        let mut writer = db
            .open_table_for_append(&key)
            .map_err(|e| format!("Error: {}", e))?;
        for encoded in &rows {
            writer
                .push_row(encoded)
                .map_err(|e| format!("Error: {}", e))?;
        }
        writer.commit().map_err(|e| format!("Error: {}", e))?;
    }
    let exec_time = exec_start.elapsed();

    Ok(StatementOutcome::InsertInto {
        table_name,
        rows: inserted,
        timing: ctx.timing(compile_time, exec_time),
        debug: std::mem::take(ctx.capture),
    })
}

fn exec_create_table(
    table_name: partiql_value::BindingsName<'static>,
    ctx: &mut StatementCtx<'_>,
) -> Result<StatementOutcome, Box<dyn std::error::Error>> {
    let key = canonical_table_key(&table_name);
    let db = ctx
        .db
        .ok_or("Error: The `--db <PATH>` option is required for CREATE TABLE.")?;

    // Bare `?` (not `.map_err(format!)`) so the typed StorageError /
    // SerializeError boxes intact — the startup bootstrap downcasts to
    // StorageError::TableExists to reconcile a crash-recovery re-run.
    let mut tables_value = Vec::new();
    crate::row_codec::serialize_name_row(&[&key], &mut tables_value)?;
    let exec_start = Instant::now();
    db.create_table(&key, &tables_value)?.commit()?;
    let exec_time = exec_start.elapsed();

    Ok(StatementOutcome::CreateTable {
        table_name,
        canonical_key: key,
        timing: ctx.timing(Duration::ZERO, exec_time),
        debug: std::mem::take(ctx.capture),
    })
}
