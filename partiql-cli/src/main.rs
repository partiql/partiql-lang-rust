#![deny(rustdoc::broken_intra_doc_links)]

use clap::Parser;
use partiql_cli::error::CLIErrors;
use partiql_cli::{args, repl};

use partiql_parser::Parsed;

#[allow(dead_code)]
fn parse(query: &str) -> Result<Parsed, CLIErrors> {
    partiql_parser::Parser::default()
        .parse(query)
        .map_err(CLIErrors::from_parser_error)
}

fn main() -> miette::Result<()> {
    let args = args::Args::parse();

    match &args.command {
        args::Commands::Repl => repl::repl(),

        #[cfg(feature = "visualize")]
        args::Commands::Ast { format, query } => {
            use partiql_cli::args::Format;
            use partiql_cli::visualize::render::{display, to_dot, to_json, to_png, to_svg};
            use std::io::Write;

            let parsed = parse(&query)?;
            match format {
                Format::Json => println!("{}", to_json(&parsed.ast)),
                Format::Dot => println!("{}", to_dot(&parsed.ast)),
                Format::Svg => println!("{}", to_svg(&parsed.ast)),
                Format::Png => {
                    std::io::stdout()
                        .write(&to_png(&parsed.ast))
                        .expect("png write");
                }
                Format::Display => display(&parsed.ast),
            }

            Ok(())
        }
    }
}
