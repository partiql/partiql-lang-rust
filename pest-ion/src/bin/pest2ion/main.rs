// Copyright Amazon.com, Inc. or its affiliates.

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, App, Arg, ArgGroup};
use ion_rs::value::writer::{ElementWriter, Format, TextKind};
use pest_ion::{from_read, TryPestToElement};
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::path::Path;

fn main() -> Result<()> {
    let matches = App::new("Pest to Ion Converter")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("INPUT_FILE")
                .help("The input file to parse (defaults to STDIN)")
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT_FILE")
                .long("output")
                .short("o")
                .takes_value(true)
                .help("Writes output to the given file (defaults to STDOUT)"),
        )
        .arg(
            Arg::with_name("text")
                .long("text")
                .short("t")
                .help("Generate Ion text (default)"),
        )
        .arg(
            Arg::with_name("binary")
                .long("binary")
                .short("b")
                .help("Generate Ion binary"),
        )
        .arg(
            Arg::with_name("pretty")
                .long("pretty")
                .short("p")
                .help("Generate Ion pretty printed text"),
        )
        .group(ArgGroup::with_name("format").args(&["text", "binary", "pretty"]))
        .get_matches();

    let elem = if let Some(file_name) = matches.value_of("INPUT_FILE") {
        Path::new(file_name).try_pest_to_element()?
    } else {
        // no file argument means read from stdin
        from_read(stdin()).try_pest_to_element()?
    };

    // currently Ion element requires a fixed buffer to serialize to, let's choose something
    // relatively big until this limitation is lifted
    const BUFFER_SIZE: usize = 32 * 1024 * 1024;
    let mut out_buf = vec![0u8; BUFFER_SIZE];

    let format = if matches.is_present("binary") {
        Format::Binary
    } else if matches.is_present("pretty") {
        Format::Text(TextKind::Pretty)
    } else {
        Format::Text(TextKind::Compact)
    };

    let mut writer = format.element_writer_for_slice(&mut out_buf)?;
    writer.write(&elem)?;
    let out_slice = writer.finish()?;

    // TODO make output (file) configurable
    let mut out: Box<dyn Write> = if let Some(out_file_name) = matches.value_of("OUTPUT_FILE") {
        Box::new(File::create(out_file_name)?)
    } else {
        Box::new(stdout())
    };
    out.write_all(out_slice)?;

    Ok(())
}
