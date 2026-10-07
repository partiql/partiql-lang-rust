//! Rendering primitives shared by the CLI and the test harness. Session core
//! does not call any of these — it returns structured results and the caller
//! picks a renderer.

use std::io::{self, Write};
use std::time::Duration;

use partiql_vm::value::{RegisterReader, Shape};

use crate::session::exec::QueryFooter;
use crate::session::ion_output::{escape_control_chars_in_strings, write_outcome_ion};
use crate::session::naming::format_table_name;
use crate::session::outcome::{DebugCapture, StatementOutcome, StatementTiming};
use crate::session::value::{row_to_value, value_to_element, RowConvertError};

/// Flush a captured AST/plan/program block to `err`. Callers invoke this in
/// the right ordering (before query rows on stdout, or on error).
pub fn flush_debug(debug: &DebugCapture, err: &mut dyn Write) -> io::Result<()> {
    if let Some(ast) = &debug.ast {
        writeln!(err, "{}", ast)?;
    }
    if let Some(plan) = &debug.plan {
        writeln!(err, "{}", plan)?;
    }
    if let Some(program) = &debug.program {
        writeln!(err, "{}", program)?;
    }
    err.flush()
}

/// Text-format streaming query renderer. Emits `<<\n  row,\n  ...\n>>\n` for
/// Bag, `[...]` for List, bare `{value:?}` for Single. Row count is tracked
/// by whatever `Iterator` the caller passes (typically session's
/// `CountingRows` so the row count is authoritative).
pub fn render_query_text<'vm, I>(
    iter: I,
    shape: &Shape,
    out: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>>
where
    I: Iterator<Item = Result<RegisterReader<'vm>, partiql_vm::EngineError>>,
{
    let (prefix, tab, suffix) = match shape {
        Shape::Bag(_) => (Some("<<"), "  ", Some(">>")),
        Shape::List(_) => (Some("["), "  ", Some("]")),
        Shape::Single(_) => (None, "", None),
    };
    if let Some(p) = prefix {
        writeln!(out, "{}", p)?;
    }

    let mut is_first = true;
    for row_result in iter {
        let row = row_result.map_err(|e| format!("Execution error: {:?}", e))?;
        let v = row_to_value(&row, shape).map_err(|e| format!("Execution error: {e}"))?;
        if is_first {
            is_first = false;
        } else {
            writeln!(out, ",")?;
        }
        write!(out, "{tab}{v:?}")?;
    }
    writeln!(out)?;
    if let Some(s) = suffix {
        writeln!(out, "{}", s)?;
    }
    out.flush()?;
    Ok(())
}

/// PartiQL-encoded-Ion streaming query renderer. Emits `{rows: $bag::[...]}`
/// (Bag), `{rows: [...]}` (List), or `{rows: v}` (Single). Streams one row at
/// a time; peak memory is O(1) in row count.
pub fn render_query_ion<'vm, I>(
    iter: I,
    shape: &Shape,
    out: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>>
where
    I: Iterator<Item = Result<RegisterReader<'vm>, partiql_vm::EngineError>>,
{
    let (open, close) = match shape {
        Shape::Bag(_) => ("{rows: $bag::[", "]}"),
        Shape::List(_) => ("{rows: [", "]}"),
        Shape::Single(_) => ("{rows: ", "}"),
    };

    let mut count: u64 = 0;
    let mut opened = false;
    for row_result in iter {
        let row = row_result.map_err(|e| format!("Execution error: {:?}", e))?;
        let value = row_to_value(&row, shape).map_err(|e| format!("Execution error: {e}"))?;
        let element = value_to_element(&value).map_err(|e: RowConvertError| {
            io::Error::new(io::ErrorKind::InvalidData, e.to_string())
        })?;
        let text = element
            .to_text(ion_rs::element::writer::TextKind::Compact)
            .map_err(|e| io::Error::other(e.to_string()))?;
        let escaped = escape_control_chars_in_strings(&text);

        if matches!(shape, Shape::Single(_)) && count >= 1 {
            return Err(
                "single-row query produced more than one row (invalid Ion envelope)".into(),
            );
        }

        if !opened {
            out.write_all(open.as_bytes())?;
        } else {
            out.write_all(b", ")?;
        }
        out.write_all(escaped.as_bytes())?;
        opened = true;
        count += 1;
    }

    if matches!(shape, Shape::Single(_)) && count == 0 {
        return Err("single-row query produced no row".into());
    }

    if !opened {
        out.write_all(open.as_bytes())?;
    }
    out.write_all(close.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

/// Ion envelope for a completed non-query outcome.
pub fn render_outcome_ion(outcome: &StatementOutcome, out: &mut dyn Write) -> io::Result<()> {
    write_outcome_ion(outcome, out)
}

/// Human-readable summary for a completed non-query outcome, written to
/// `err`, followed by its `Statement N:` timing block. `statement` is the
/// 1-based position of the statement in its script.
pub fn render_outcome_text(
    outcome: &StatementOutcome,
    statement: usize,
    err: &mut dyn Write,
) -> io::Result<()> {
    match outcome {
        StatementOutcome::CreateTableAs {
            table_name, rows, ..
        } => writeln!(
            err,
            "Created table {} ({} rows)",
            format_table_name(table_name),
            rows
        )?,
        StatementOutcome::InsertInto {
            table_name, rows, ..
        } => writeln!(
            err,
            "Inserted {} rows into {}",
            rows,
            format_table_name(table_name)
        )?,
        StatementOutcome::CreateTable { table_name, .. } => {
            writeln!(err, "Created table {}", format_table_name(table_name))?
        }
    }
    write_timing(err, statement, None, outcome.timing())?;
    err.flush()
}

/// `Statement N: (R rows in ...)` timing block for a completed Query. Ion
/// mode skips this to keep stderr clean for downstream Ion consumers.
pub fn render_query_footer_text(
    footer: &QueryFooter,
    statement: usize,
    err: &mut dyn Write,
) -> io::Result<()> {
    write_timing(err, statement, Some(footer.row_count), &footer.timing)?;
    err.flush()
}

/// Two-line timing block, the per-phase breakdown nested under the label:
///
/// ```text
/// Statement 1: (3 rows in 0.5ms)
///   parse: 0.1ms, lower: 0.1ms, compile: 0.2ms, exec: 0.1ms
/// ```
///
/// Non-query statements omit the `R rows in` part.
fn write_timing(
    err: &mut dyn Write,
    statement: usize,
    rows: Option<u64>,
    timing: &StatementTiming,
) -> io::Result<()> {
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let total = timing.parse + timing.lower + timing.compile + timing.exec;
    let rows = rows.map(|n| format!("{n} rows in ")).unwrap_or_default();
    writeln!(err, "Statement {statement}: ({rows}{:.1}ms)", ms(total))?;
    writeln!(
        err,
        "  parse: {:.1}ms, lower: {:.1}ms, compile: {:.1}ms, exec: {:.1}ms",
        ms(timing.parse),
        ms(timing.lower),
        ms(timing.compile),
        ms(timing.exec),
    )
}
