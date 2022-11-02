use partiql_conformance_test_generator::Config;

use std::process::Command;

fn main() -> miette::Result<()> {
    println!("cargo:rerun-if-changed=partiql-tests");
    println!("cargo:rerun-if-changed=tests/partiql_tests");

    Config::new().process_dir("partiql-tests/partiql-tests-data", "tests/partiql-tests")?;

    Command::new("cargo")
        .arg("fmt")
        .arg("--")
        .spawn()
        .expect("cargo fmt of tests/ failed");

    Ok(())
}
