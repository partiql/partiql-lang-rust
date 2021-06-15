// Copyright Amazon.com, Inc. or its affiliates.

use anyhow::Result;
use assert_cmd::Command;
use ion_rs::value::owned::OwnedElement;
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

/// Simple wrapper for a CLI test case.
struct TestCase<S: AsRef<str>> {
    /// The text of the Pest grammar to test
    pest_text: S,
    /// The expected Ion
    expected_ion: OwnedElement,
}

impl From<(&'static str, &'static str)> for TestCase<&'static str> {
    /// Simple conversion for static `str` slices into a test acse
    fn from((pest_text, ion_text): (&'static str, &'static str)) -> Self {
        let expected_ion = element_reader().read_one(ion_text.as_bytes()).unwrap();
        Self {
            pest_text,
            expected_ion,
        }
    }
}

/// Loads the actual PartiQL grammar to make sure we can parse/serialize it
/// This is not testing the serialization is actually correct, just that it works through
/// the CLI.
fn partiql_test_case() -> TestCase<String> {
    use pest_ion::*;
    use std::fs::read_to_string;
    use std::path::Path;

    let pest_text = read_to_string(Path::new("../partiql-parser/src/peg/partiql.pest"))
        .expect("Could not load PartiQL grammar");
    let expected_ion = Path::new("../partiql-parser/src/peg/partiql.pest")
        .try_pest_to_element()
        .expect("Could not convert PartiQL grammar to Ion");

    TestCase {
        pest_text,
        expected_ion,
    }
}

#[rstest]
#[case::simple((
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
).into())]
#[case::partiql(partiql_test_case())]
fn run_it<S: AsRef<str>>(
    #[case] test_case: TestCase<S>,
    #[values("", "-t", "-p", "-b")] format_flag: &str,
    #[values(FileMode::Default, FileMode::Named)] input_mode: FileMode,
    #[values(FileMode::Default, FileMode::Named)] output_mode: FileMode,
) -> Result<()> {
    let TestCase {
        pest_text,
        expected_ion,
    } = test_case;

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
            cmd.write_stdin(pest_text.as_ref());
        }
        FileMode::Named => {
            // dump our test data to input file
            let mut input_file = File::create(&input_path)?;
            input_file.write(pest_text.as_ref().as_bytes())?;
            input_file.flush()?;

            // make this the input for our driver
            cmd.arg(input_path.to_str().unwrap());
        }
    };
    println!("{:?}", cmd);
    let assert = cmd.assert();

    let actual_ion = match output_mode {
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
    assert_eq!(expected_ion, actual_ion);

    assert.success();

    Ok(())
}
