use crate::generator::{NamespaceNode, Node, TestTree};
use crate::util::Escaper;

use codegen::Scope;
use miette::IntoDiagnostic;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const FILE_HEADER: &str = "\
// ********************************************************************************************** //
//  *NOTE*: This file is generated by partiql-conformance-test-generator. Do not edit directly.
// ********************************************************************************************** //

#[allow(unused_imports)]
";

#[derive(Debug)]
pub struct Writer {}

impl Writer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write(&self, path: impl AsRef<Path>, root: NamespaceNode) -> miette::Result<()> {
        let path: PathBuf = path
            .as_ref()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string().escape_path())
            .collect();
        std::fs::create_dir_all(&path).into_diagnostic()?;

        write_module(&path, root)?;

        Ok(())
    }
}

fn write_module(path: impl AsRef<Path>, module: NamespaceNode) -> miette::Result<()> {
    let sub_mods = module.children.iter().filter_map(|(name, tree)| {
        if let TestTree::Node(Node::Env(_)) = tree {
            None
        } else {
            Some(name)
        }
    });
    write_dir_mod(&path, sub_mods)?;

    for (name, child) in module.children {
        let mut child_path: PathBuf = path.as_ref().into();
        child_path.push(&name);
        match child {
            TestTree::Node(Node::Test(mut s)) => write_scope(child_path, s.module.scope())?,
            TestTree::Node(Node::Env(e)) => write_file(child_path, &e.env)?,
            TestTree::Namespace(m) => write_module(child_path, m)?,
        }
    }
    Ok(())
}

fn write_dir_mod<'a>(
    path: impl AsRef<Path>,
    sub_mods: impl Iterator<Item = &'a String>,
) -> miette::Result<()> {
    std::fs::create_dir_all(&path).into_diagnostic()?;

    let mut contents = FILE_HEADER.to_string();
    contents.push_str("use super::*;");
    for sub_mod in sub_mods {
        contents.push_str(&format!("mod {};\n", sub_mod.replace(".rs", "")))
    }

    let file_path = path.as_ref().join("mod.rs");
    File::create(file_path)
        .into_diagnostic()?
        .write_all(contents.as_bytes())
        .into_diagnostic()
}

fn write_scope(path: impl AsRef<Path>, scope: &Scope) -> miette::Result<()> {
    let contents = format!("{}{}", FILE_HEADER, scope.to_string());
    write_file(path, &contents)
}

fn write_file(path: impl AsRef<Path>, contents: &str) -> miette::Result<()> {
    let mut file = File::create(path).into_diagnostic()?;
    file.write_all(contents.as_bytes()).into_diagnostic()?;
    file.sync_all().into_diagnostic()
}
