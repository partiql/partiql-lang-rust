//! Parse + lower + compile + execute one statement.
//!
//! Non-query statements are executed to completion and returned as a
//! `StatementOutcome`. Queries return `RunOutcome::Query` carrying a handle
//! the caller drives via `QueryHandle::drain(render_fn)`. Session never writes
//! to stdout/stderr.

use std::sync::Arc;
use std::time::{Duration, Instant};

use partiql_ast::ast;
use partiql_eval::value::{RegisterReader, Shape};
use partiql_logical::{BindingsOp, LogicalPlan, LogicalStatement};

use crate::common;
use crate::session::debug::DebugFlags;
use crate::session::naming::{canonical_table_key, normalize_query};
use crate::session::outcome::{DebugCapture, StatementOutcome, StatementTiming};
use crate::session::planner;
use crate::storage::{self, HeedDB};

/// Result of dispatching a statement. Queries hand back a `QueryHandle` so the
/// caller can drain rows on its own terms; non-query statements are already
/// complete.
pub enum RunOutcome {
    Query(QueryHandle),
    Statement(StatementOutcome),
}

/// A compiled query the caller must drive to completion via `drain`. Owns the
/// VM so rows stay valid for the drain call's lifetime; the exec-time clock
/// starts inside `drain` (NOT at handle construction) so caller-side stalls
/// between `run` and `drain` don't count as execution.
pub struct QueryHandle {
    vm: Box<partiql_eval::PartiQLVM>,
    shape: Shape,
    debug: DebugCapture,
    parse_time: Duration,
    lower_time: Duration,
    compile_time: Duration,
}

impl QueryHandle {
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Take the debug capture out of the handle. Call this BEFORE `drain` if
    /// the caller wants `--debug` output on stderr to precede rows on stdout.
    pub fn take_debug(&mut self) -> DebugCapture {
        std::mem::take(&mut self.debug)
    }

    /// Drive the query to completion. The renderer receives a row-counting
    /// iterator plus the shape; whatever it returns comes back paired with a
    /// finalized `QueryFooter`. Session owns the exec-time clock. After the
    /// renderer returns `Ok`, any rows it did NOT consume are drained here so
    /// `row_count` is authoritative — a lazy renderer cannot silently under-
    /// count. A row error surfaced during that post-drain becomes the return
    /// value's error.
    ///
    /// Consumes `self`: a handle cannot be executed twice.
    pub fn drain<F, R>(mut self, render: F) -> Result<(R, QueryFooter), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut CountingRows<'_>, &Shape) -> Result<R, Box<dyn std::error::Error>>,
    {
        let exec_start = Instant::now();
        let iter = match self.vm.execute() {
            Ok(partiql_eval::ExecutionResult::Query(it)) => it,
            Err(e) => return Err(format!("Execution setup error: {:?}", e).into()),
        };
        let mut counter = CountingRows {
            inner: iter,
            count: 0,
        };
        let user_result = render(&mut counter, &self.shape)?;
        // Post-drain: if the renderer stopped early, finish consuming the
        // iterator so row_count is complete and any late row error surfaces.
        for row in counter.by_ref() {
            row.map_err(|e| format!("Execution error: {:?}", e))?;
        }
        let row_count = counter.count;
        Ok((
            user_result,
            QueryFooter {
                row_count,
                timing: StatementTiming {
                    parse: self.parse_time,
                    lower: self.lower_time,
                    compile: self.compile_time,
                    exec: exec_start.elapsed(),
                },
            },
        ))
    }
}

/// Row-count + timing for a completed Query, ready to render.
pub struct QueryFooter {
    pub row_count: u64,
    pub timing: StatementTiming,
}

/// Row iterator adapter: session owns the counter, renderer just pulls rows.
pub struct CountingRows<'vm> {
    inner: partiql_eval::QueryIterator<'vm>,
    count: u64,
}

impl<'vm> Iterator for CountingRows<'vm> {
    type Item = Result<RegisterReader<'vm>, partiql_eval::EngineError>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        if item.is_ok() {
            self.count += 1;
        }
        Some(item)
    }
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

/// Parse + lower + dispatch. On error, the compile-time debug capture is
/// returned in the second position so the caller can flush it.
pub(super) fn run(
    db: Option<&Arc<HeedDB>>,
    debug: &DebugFlags,
    sql: &str,
) -> (Result<RunOutcome, Box<dyn std::error::Error>>, DebugCapture) {
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

    match dispatch(stmt, debug, db, parse_time, &mut capture) {
        Ok(out) => (Ok(out), DebugCapture::default()),
        Err(e) => (Err(e), capture),
    }
}

/// Bootstrap entry: no debug capture, no rendering. Query statements drain
/// their rows internally so their side effects run without needing a renderer.
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
            let handle = compile_query(plan, &mut ctx)?;
            handle.drain(|rows, _shape| {
                for row in rows {
                    row.map_err(|e| format!("Execution error: {:?}", e))?;
                }
                Ok(())
            })?;
            Ok(())
        }
        other => dispatch_write(other, &mut ctx).map(|_| ()),
    }
}

fn dispatch(
    stmt: &ast::AstNode<ast::Statement>,
    debug: &DebugFlags,
    db: Option<&Arc<HeedDB>>,
    parse_time: Duration,
    capture: &mut DebugCapture,
) -> Result<RunOutcome, Box<dyn std::error::Error>> {
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
        LogicalStatement::Query(plan) => {
            let handle = compile_query(plan, &mut ctx)?;
            Ok(RunOutcome::Query(handle))
        }
        other => dispatch_write(other, &mut ctx).map(RunOutcome::Statement),
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

/// Compile a query plan into a handle ready to execute. VM setup
/// (build_exec_context + PartiQLVM::new) is folded into `compile_time` so the
/// footer accounts for all pre-execution work.
fn compile_query(
    plan: LogicalPlan<BindingsOp>,
    ctx: &mut StatementCtx<'_>,
) -> Result<QueryHandle, Box<dyn std::error::Error>> {
    let compile_start = Instant::now();
    let (compiled, catalog_id) =
        planner::build_compiled(&plan, ctx.debug, ctx.db.cloned(), ctx.capture)?;
    let exec_context = planner::build_exec_context(ctx.db.cloned(), catalog_id, &compiled);
    let vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;
    let compile_time = compile_start.elapsed();

    let shape = vm.shape().clone();

    Ok(QueryHandle {
        vm: Box::new(vm),
        shape,
        debug: std::mem::take(ctx.capture),
        parse_time: ctx.parse_time,
        lower_time: ctx.lower_time,
        compile_time,
    })
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
