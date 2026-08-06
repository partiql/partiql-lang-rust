//! Non-query outcomes (CTAS / INSERT / CREATE TABLE). Query results are
//! delivered as a live handle in `RunOutcome::Query` and do not flow through
//! this type — the caller renders rows and their footer directly from the
//! handle's timing/row-count fields.

use std::time::Duration;

use clap::ValueEnum;
use partiql_value::BindingsName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Ion,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OutputFormat::Text => "text",
            OutputFormat::Ion => "ion",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StatementTiming {
    pub parse: Duration,
    pub lower: Duration,
    /// `Duration::ZERO` for pure-DDL CREATE TABLE (no compile step).
    pub compile: Duration,
    pub exec: Duration,
}

/// AST/plan/program captured under `--debug`, flushed to stderr.
#[derive(Debug, Default)]
pub struct DebugCapture {
    pub ast: Option<String>,
    pub plan: Option<String>,
    pub program: Option<String>,
}

#[derive(Debug)]
pub enum StatementOutcome {
    CreateTableAs {
        table_name: BindingsName<'static>,
        canonical_key: String,
        rows: u64,
        timing: StatementTiming,
        debug: DebugCapture,
    },
    InsertInto {
        table_name: BindingsName<'static>,
        rows: u64,
        timing: StatementTiming,
        debug: DebugCapture,
    },
    CreateTable {
        table_name: BindingsName<'static>,
        canonical_key: String,
        /// `compile` is `Duration::ZERO`; CREATE TABLE has no compile step.
        timing: StatementTiming,
        debug: DebugCapture,
    },
}

impl StatementOutcome {
    pub fn debug(&self) -> &DebugCapture {
        match self {
            StatementOutcome::CreateTableAs { debug, .. }
            | StatementOutcome::InsertInto { debug, .. }
            | StatementOutcome::CreateTable { debug, .. } => debug,
        }
    }

    pub fn timing(&self) -> &StatementTiming {
        match self {
            StatementOutcome::CreateTableAs { timing, .. }
            | StatementOutcome::InsertInto { timing, .. }
            | StatementOutcome::CreateTable { timing, .. } => timing,
        }
    }
}
