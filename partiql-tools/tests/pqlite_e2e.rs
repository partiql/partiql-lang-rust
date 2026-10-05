//! Every `tests/pqlite/cases/**/*.test.ion` file runs as an independent case.

mod pqlite_e2e {
    pub mod loader;
    pub mod runner;
}

use rstest::rstest;
use std::path::PathBuf;

#[rstest]
fn case(#[files("tests/pqlite/cases/**/*.test.ion")] path: PathBuf) {
    pqlite_e2e::runner::run_case(&path).unwrap_or_else(|e| panic!("{e}"));
}
