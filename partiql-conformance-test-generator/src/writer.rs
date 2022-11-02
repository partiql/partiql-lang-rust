use crate::generator::{TestComponent, TestModule};
use crate::util::Escaper;

use codegen::Scope;
use miette::IntoDiagnostic;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const GENERATED_BANNER: &'static str = "\
// ********************************************************************************************** //
//  *NOTE*: This file is generated by partiql-conformance-test-generator. Do not edit directly.   
// ********************************************************************************************** //



";

#[derive(Debug)]
pub struct WriterConfig {
    root: String,
}

impl WriterConfig {
    pub fn new(root: &str) -> WriterConfig {
        WriterConfig {
            root: root.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Writer {
    config: WriterConfig,
}

impl Writer {
    pub fn new(config: WriterConfig) -> Writer {
        Self { config }
    }

    pub fn write(&self, path: impl AsRef<Path>, root: TestModule) -> miette::Result<()> {
        let path: PathBuf = path
            .as_ref()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string().escape_path())
            .collect();
        std::fs::create_dir_all(&path).into_diagnostic()?;

        self.write_module(&path, root)?;

        Ok(())
    }

    fn write_module(&self, path: impl AsRef<Path>, module: TestModule) -> miette::Result<()> {
        self.write_dir_mod(&path, module.children.keys())?;

        for (name, child) in module.children {
            let mut child_path: PathBuf = path.as_ref().into();
            child_path.push(&name);
            match child {
                TestComponent::Scope(mut s) => self.write_scope(child_path, s.module.scope())?,
                TestComponent::Module(m) => self.write_module(child_path, m)?,
            }
        }

        Ok(())
    }

    fn write_dir_mod<'a>(
        &self,
        path: impl AsRef<Path>,
        sub_mods: impl Iterator<Item = &'a String>,
    ) -> miette::Result<()> {
        std::fs::create_dir_all(&path).into_diagnostic()?;

        let mut contents = GENERATED_BANNER.to_string();
        for sub_mod in sub_mods {
            contents.push_str(&format!("mod {};\n", sub_mod.replace(".rs", "")))
        }

        let file_path = path.as_ref().join("mod.rs");
        File::create(file_path)
            .into_diagnostic()?
            .write_all(contents.as_bytes())
            .into_diagnostic()
    }

    fn write_scope(&self, path: impl AsRef<Path>, scope: &Scope) -> miette::Result<()> {
        let contents = scope.to_string();

        let mut file = File::create(path).into_diagnostic()?;
        file.write_all(contents.as_bytes()).into_diagnostic()?;
        file.sync_all().into_diagnostic()
    }
}
