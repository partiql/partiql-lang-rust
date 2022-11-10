use crate::generator::{Generator, GeneratorConfig};
use crate::reader::read_schema;
use crate::writer::Writer;
use std::path::Path;

mod generator;
mod reader;
mod schema;
mod util;
mod writer;

// TODO docs
#[derive(Debug, Copy, Clone)]
pub enum OverwriteStrategy {
    Overwrite,
    Backup,
}

/// Configuration for the generation of conformance tests.
#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub overwrite: OverwriteStrategy,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            overwrite: OverwriteStrategy::Overwrite,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        Config::default()
    }

    pub fn process_dir(
        &self,
        test_data: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
    ) -> miette::Result<()> {
        let schema = read_schema(&test_data)?;

        let config = GeneratorConfig::new(generator::TreeDepth::N(3));
        let tests = Generator::new(config).generate(schema)?;

        // TODO overwrite vs. backup old content?
        Writer::new().write(out_path, tests)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO: add tests checking the conversions between Ion and test schema structs
    //  https://github.com/partiql/partiql-lang-rust/issues/100
}
