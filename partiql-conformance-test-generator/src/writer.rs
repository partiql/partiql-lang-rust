use crate::generator::{TestComponent, TestModule};

use codegen::Scope;
use miette::IntoDiagnostic;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn write_scopes(path: impl AsRef<Path>, root: TestModule) -> miette::Result<()> {
    let path: &Path = path.as_ref();
    std::fs::create_dir_all(&path).into_diagnostic()?;

    write_top_level(path, "partiql_tests")?;

    let mut path: PathBuf = (&path).into();

    path.push("partiql_tests");

    write_module(path, root)?;

    Ok(())
}

fn write_module(path: impl AsRef<Path>, module: TestModule) -> miette::Result<()> {
    write_dir_mod(&path, module.children.keys())?;

    for (name, child) in module.children {
        let mut child_path: PathBuf = path.as_ref().into();
        match child {
            TestComponent::Scope(mut scope) => {
                child_path.push(&name);
                write_scope(child_path, scope.module.scope())?
            }
            TestComponent::Module(module) => {
                child_path.push(&name);

                write_module(child_path, module)?
            }
        }
    }

    Ok(())
}

fn write_top_level(path: impl AsRef<Path>, sub_tests_dir: &str) -> miette::Result<()> {
    std::fs::create_dir_all(&path).into_diagnostic()?;

    // Creates the top-level mod.rs file in the tests/ directory and adds the "conformance_test"
    // flag to this mod
    let file_path = path.as_ref().join("mod.rs");
    File::create(file_path)
        .into_diagnostic()?
        .write_all(
            format!(
                "#[cfg(feature = \"conformance_test\")]\nmod {};\n",
                sub_tests_dir
            )
            .as_bytes(),
        )
        .into_diagnostic()
}

fn write_dir_mod<'a>(
    path: impl AsRef<Path>,
    sub_mods: impl Iterator<Item = &'a String>,
) -> miette::Result<()> {
    std::fs::create_dir_all(&path).into_diagnostic()?;

    let contents = sub_mods
        .map(|m| format!("mod {};", m.replace(".rs", "")))
        .collect::<Vec<String>>()
        .join("\n");

    let file_path = path.as_ref().join("mod.rs");
    File::create(file_path)
        .into_diagnostic()?
        .write_all(contents.as_bytes())
        .into_diagnostic()
}

fn write_scope(path: impl AsRef<Path>, scope: &Scope) -> miette::Result<()> {
    let contents = scope.to_string();

    File::create(path)
        .into_diagnostic()?
        .write_all(contents.as_bytes())
        .into_diagnostic()
}
