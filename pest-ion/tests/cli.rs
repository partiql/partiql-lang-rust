// Copyright Amazon.com, Inc. or its affiliates.

use anyhow::Result;
use assert_cmd::Command;
use ion_rs::value::reader::*;
use rstest::*;
use std::fs::File;
use std::io::{Read, Write};
use tempfile::TempDir;

/// The input or output mode of the CLI.
enum FileMode {
    /// Use `STDIN` or `STDOUT`
    Default,
    /// Use a named file.
    Named,
}

#[rstest]
#[case::simple(
    r#"
        a = { "a" ~ "b" }
    "#,
    r#"
        {
            a: {
                type: normal,
                expression: (sequence (string exact "a") (string exact "b")),
            }
        }
    "#
)]
fn simple(
    #[case] pest_src: &str,
    #[case] ion_text: &str,
    #[values("", "-t", "-p", "-b")] format_flag: &str,
    #[values(FileMode::Default, FileMode::Named)] input_mode: FileMode,
    #[values(FileMode::Default, FileMode::Named)] output_mode: FileMode,
) -> Result<()> {
    // working space for our tests when they need files
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("INPUT.pest");
    let output_path = temp_dir.path().join("OUTPUT.ion");

    let mut cmd = Command::cargo_bin("pest2ion")?;
    if format_flag != "" {
        cmd.arg(format_flag);
    }
    match output_mode {
        FileMode::Default => {
            // do nothing
        }
        FileMode::Named => {
            // tell our driver to output to a file
            cmd.arg("-o");
            cmd.arg(&output_path);
        }
    }
    match input_mode {
        FileMode::Default => {
            // do nothing
        }
        FileMode::Named => {
            // dump our test data to input file
            let mut input_file = File::create(&input_path)?;
            input_file.write(pest_src.as_bytes())?;
            input_file.flush()?;

            // make this the input for our driver
            cmd.arg(input_path.to_str().unwrap());
        }
    };
    println!("{:?}", cmd);
    let assert = cmd.write_stdin(pest_src).assert();

    let actual = match output_mode {
        FileMode::Default => {
            let output = assert.get_output();
            element_reader().read_one(&output.stdout)?
        }
        FileMode::Named => {
            let mut output_file = File::open(output_path)?;
            let mut output_buffer = vec![];
            output_file.read_to_end(&mut output_buffer)?;
            element_reader().read_one(&output_buffer)?
        }
    };
    let expected = element_reader().read_one(ion_text.as_bytes())?;
    assert_eq!(expected, actual);

    assert.success();

    Ok(())
}
