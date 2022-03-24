use codegen::{Formatter, Function, Scope};
use ion_rs::value::reader::{element_reader, ElementReader};
use ion_rs::value::{Element, Sequence, Struct, SymbolToken};
use ion_rs::IonType;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const PASS_PARSE_TESTS_DIR: &str = "partiql-tests/partiql-test-data/pass/parser";
const FAIL_PARSE_TESTS_DIR: &str = "partiql-tests/partiql-test-data/fail/parser";
const PARSER_IMPORT: (&str, &str) = ("partiql_parser", "lalr_parse");

fn main() {
    let pass_parse_test_files: Vec<PathBuf> = all_ion_files_in(PASS_PARSE_TESTS_DIR);
    let fail_parse_test_files: Vec<PathBuf> = all_ion_files_in(FAIL_PARSE_TESTS_DIR);

    for pass_parse_test_file in &pass_parse_test_files {
        let mut scope = create_pass_parse_scope(pass_parse_test_file);
        scope.import(PARSER_IMPORT.0, PARSER_IMPORT.1);
        let full_file_name = to_full_file_name("pass_parse", pass_parse_test_file);
        let dest_path: PathBuf = ["tests", full_file_name.as_str()].iter().collect();
        File::create(dest_path)
            .expect("File creation failed")
            .write_all(scope_to_formatted_string(scope).as_bytes())
            .unwrap_or_else(|error| panic!("Failure when writing to file: {:?}", error));
    }

    for fail_parse_test_file in &fail_parse_test_files {
        let mut scope = create_fail_parse_scope(fail_parse_test_file);
        scope.import(PARSER_IMPORT.0, PARSER_IMPORT.1);
        let full_file_name = to_full_file_name("fail_parse", fail_parse_test_file);
        let dest_path: PathBuf = ["tests", full_file_name.as_str()].iter().collect();
        File::create(dest_path)
            .expect("File creation failed")
            .write_all(scope_to_formatted_string(scope).as_bytes())
            .unwrap_or_else(|error| panic!("Failure when writing to file: {:?}", error));
    }
}

/// Returns a vector of all .ion files in the directory `dir`
fn all_ion_files_in(dir: &str) -> Vec<PathBuf> {
    let ion_file_extension: &OsStr = OsStr::new("ion");
    let ion_file_iterator = WalkDir::new(dir)
        .into_iter()
        .map(|entry| {
            entry.unwrap_or_else(|error| panic!("Failure during dir traversal: {:?}", error))
        })
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().to_owned())
        .filter(|f| f.extension() == Some(ion_file_extension));
    ion_file_iterator.collect()
}

/// Returns a file name with prefix prepended and 'rs' as the extension
fn to_full_file_name(prefix: &str, ion_file: &Path) -> String {
    format!(
        "{}_{}",
        prefix,
        ion_file
            .with_extension("rs")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    )
}

/// Converts the scope to a String that is passed through codegen's default formatter
fn scope_to_formatted_string(scope: Scope) -> String {
    let mut dest_str = String::new();
    scope
        .fmt(&mut Formatter::new(&mut dest_str))
        .unwrap_or_else(|error| panic!("Failure during scope formatting: {:?}", error));
    dest_str
}

trait TestFunction {
    fn to_test_function(&self) -> Function;
}

/// Represents a conformance test with a PartiQL statement that will parse without errors
/// TODO: add additional parse checks (e.g. ast)
struct ParsePassTestCase {
    test_name: String,
    statement: String,
    namespace: Option<String>,
}

impl TestFunction for ParsePassTestCase {
    /// Creates a [`codegen::Function`] that checks the parsing of `statement` does not result in
    /// a parsing error.
    fn to_test_function(&self) -> Function {
        let complete_test_name = match &self.namespace {
            Some(ns) => format!("{}_{}", ns, &self.test_name),
            None => self.test_name.to_owned(),
        };
        let mut generated_fun = Function::new(complete_test_name.replace(' ', "_").as_str());
        generated_fun
            .attr("test")
            .line(format!(
                "let parse_result = lalr_parse(\"{}\");",
                &self.statement
            ))
            .line("assert!(parse_result.is_ok());");
        generated_fun
    }
}

/// Represents a conformance test with a PartiQL statement that will give an error when parsed
/// TODO: add additional fields for further error-checking (e.g. error code)
struct ParseFailTestCase {
    test_name: String,
    statement: String,
    namespace: Option<String>,
}

impl TestFunction for ParseFailTestCase {
    /// Creates a [`codegen::Function`] that checks the parsing of `statement` results in a parsing
    /// error.
    fn to_test_function(&self) -> Function {
        let complete_test_name = match &self.namespace {
            Some(ns) => format!("{}_{}", ns, &self.test_name),
            None => self.test_name.to_owned(),
        };
        let mut generated_fun = Function::new(complete_test_name.replace(' ', "_").as_str());
        generated_fun
            .attr("test")
            .line(format!(
                "let parse_result = lalr_parse(\"{}\");",
                &self.statement
            ))
            .line("assert!(parse_result.is_err());");
        generated_fun
    }
}

/// Parses the given .ion file `path` and creates a new [`codegen::Scope`] with a
/// [`codegen::Function`] for each test in `path`.
///
/// Assumes the given tests in `path` all parse without errors.
fn create_pass_parse_scope(path: &Path) -> Scope {
    let all_data = fs::read(path).unwrap();
    let all_ion_data = element_reader().read_all(&all_data).unwrap();

    let mut scope = Scope::new();

    for ion_data in all_ion_data {
        match ion_data.ion_type() {
            IonType::Struct => {
                let test_struct = ion_data.as_struct().unwrap();
                let test_name = test_struct
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned();
                let statement = test_struct
                    .get("statement")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned();

                let test_case = ParsePassTestCase {
                    test_name,
                    statement,
                    namespace: None,
                };
                scope.push_fn(test_case.to_test_function());
            }
            IonType::List => {
                let test_list = ion_data.as_sequence().unwrap();
                for tc in test_list.iter() {
                    match tc.ion_type() {
                        IonType::Struct => {
                            let namespace = Some(
                                ion_data
                                    .annotations()
                                    .next()
                                    .unwrap()
                                    .text()
                                    .unwrap()
                                    .to_owned(),
                            );

                            let test_struct = tc.as_struct().unwrap();
                            let test_name = test_struct
                                .get("name")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_owned();
                            let statement = test_struct
                                .get("statement")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_owned();
                            let test_case = ParsePassTestCase {
                                test_name,
                                statement,
                                namespace,
                            };
                            scope.push_fn(test_case.to_test_function());
                        }
                        _ => panic!("Expected tests within a namespace to be a struct"),
                    }
                }
            }
            _ => panic!("Unexpected Ion type received"),
        };
    }
    scope
}

/// Parses the given .ion file `path` and creates a new [`codegen::Scope`] with a
/// [`codegen::Function`] for each test in `path`.
///
/// Assumes the given tests in `path` give errors when parsed.
fn create_fail_parse_scope(path: &Path) -> Scope {
    let all_data = fs::read(path).unwrap();
    let all_ion_data = element_reader().read_all(&all_data).unwrap();

    let mut scope = Scope::new();

    for ion_data in all_ion_data {
        match ion_data.ion_type() {
            IonType::Struct => {
                let test_struct = ion_data.as_struct().unwrap();
                let test_name = test_struct
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned();
                let statement = test_struct
                    .get("statement")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned();

                let test_case = ParseFailTestCase {
                    test_name,
                    statement,
                    namespace: None,
                };
                scope.push_fn(test_case.to_test_function());
            }
            IonType::List => {
                let test_list = ion_data.as_sequence().unwrap();
                for tc in test_list.iter() {
                    match tc.ion_type() {
                        IonType::Struct => {
                            let namespace = Some(
                                ion_data
                                    .annotations()
                                    .next()
                                    .unwrap()
                                    .text()
                                    .unwrap()
                                    .to_owned(),
                            );

                            let test_struct = tc.as_struct().unwrap();
                            let test_name = test_struct
                                .get("name")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_owned();
                            let statement = test_struct
                                .get("statement")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_owned();
                            let test_case = ParseFailTestCase {
                                test_name,
                                statement,
                                namespace,
                            };
                            scope.push_fn(test_case.to_test_function());
                        }
                        _ => panic!("Expected tests within a namespace to be a struct"),
                    }
                }
            }
            _ => panic!("Unexpected Ion type received"),
        }
    }
    scope
}
