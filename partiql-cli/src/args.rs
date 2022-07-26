use clap::{ArgEnum, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[cfg(feature = "visualize")]
    /// Dump the AST for a query
    Ast {
        #[clap(short = 'T', long = "format", value_enum)]
        format: Format,

        /// Query to parse
        #[clap(value_parser)]
        query: String,
    },
    /// interactive REPL (Read Eval Print Loop) shell
    Repl,
}

#[derive(ArgEnum, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    /// JSON
    Json,
    /// Graphviz dot
    Dot,
    /// Graphviz svg output
    Svg,
    /// Graphviz svg rendered to png
    Png,
    /// Display rendered output
    Display,
}
