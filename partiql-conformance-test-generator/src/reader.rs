use crate::schema::structure::{TestDir, TestEntry, TestFile, TestRoot};
use crate::{
    Assertion, Assertions, Namespace, Namespaces, StringExt, TestCase, TestCases, TestDocument,
};

use ion_rs::value::owned::Element;
use ion_rs::value::reader::{element_reader, ElementReader};
use ion_rs::value::{IonElement, IonSequence, IonStruct};
use ion_rs::IonType;
use miette::IntoDiagnostic;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;

use std::ops::Add;
use std::path::{Path, PathBuf};

pub fn read_schema(root: impl AsRef<Path>) -> miette::Result<TestRoot> {
    let fail = read_root(&root, "fail")?;
    let success = read_root(&root, "success")?;
    Ok(TestRoot { fail, success })
}

fn read_root(root: impl AsRef<Path>, root_type: &str) -> miette::Result<Vec<TestEntry>> {
    let mut dir: PathBuf = PathBuf::from(root.as_ref());
    dir.push(root_type);

    let root = fs::read_dir(root)
        .into_diagnostic()?
        .filter_map(|entry| read_entry(entry.expect("entry")))
        .collect();
    Ok(root)
}

fn read_entry(entry: DirEntry) -> Option<TestEntry> {
    let file_type = entry.file_type().expect("file_type");
    let name = entry.file_name().into_string().expect("file name");

    let ion_file_extension = OsStr::new("ion");
    if file_type.is_file() {
        if entry.path().extension() == Some(ion_file_extension) {
            let contents = read_test_doc(entry);
            Some(TestEntry::Doc(TestFile {
                file_name: name,
                contents,
            }))
        } else {
            None
        }
    } else if file_type.is_dir() {
        let contents = fs::read_dir(entry.path())
            .expect("read_dir")
            .into_iter()
            .filter_map(|x| read_entry(x.expect("entry")))
            .collect();
        Some(TestEntry::Dir(TestDir {
            dir_name: name,
            contents,
        }))
    } else {
        unreachable!()
    }
}

fn read_test_doc(entry: DirEntry) -> TestDocument {
    let buf = fs::read(entry.path()).unwrap();
    let elements = element_reader().read_all(&buf).unwrap();
    ion_data_to_test_document(elements)
}

/// Converts a vector of Ion data into a `TestDocument`, which can be composed of `Namespace`s
/// and `TestCase`s. `Namespace`s must be provided as IonLists while `TestCase`s must be provided
/// as IonStructs. Other Ion types will result in a panic.
///
/// When encountering a duplicate namespace/test case, a namespace/test case will be added with
/// '_0' suffixed to the end of the name.
fn ion_data_to_test_document(all_ion_data: Vec<Element>) -> TestDocument {
    let mut namespaces = Vec::new();
    let mut test_cases = Vec::new();

    let mut encountered_ns_names: HashSet<String> = HashSet::new();
    let mut encountered_tc_names: HashSet<String> = HashSet::new();

    for elem in all_ion_data {
        match elem.ion_type() {
            IonType::List => {
                // namespace in document
                let mut ns = test_namespace(&elem);
                if encountered_ns_names.contains(&ns.name) {
                    ns.name.push_str("_0")
                }
                encountered_ns_names.insert(ns.name.clone());
                namespaces.push(ns)
            }
            IonType::Struct => {
                // test case in document
                let mut tc = test_case(&elem);
                if encountered_tc_names.contains(&tc.test_name) {
                    tc.test_name.push_str("_0")
                }
                encountered_tc_names.insert(tc.test_name.clone());
                test_cases.push(tc)
            }
            _ => panic!("Document parsing requires an IonList or IonStruct"),
        }
    }
    TestDocument {
        namespaces,
        test_cases,
    }
}

/// Parses the given `OwnedElement` to a `Namespace`. Requires an annotation to provided that will
/// be used to create the name. '_namespace' will be suffixed to the first annotation provided.
/// The namespace can contain sub-namespaces and test cases represented by IonLists and IonStructs
/// respectively. When provided with something other than an IonList or IonStruct, this function
/// will panic.
fn test_namespace(element: &Element) -> Namespace {
    let annot: Vec<_> = element
        .annotations()
        .map(|a| a.text().expect("annotation text"))
        .collect();
    let name = annot
        .first()
        .expect("expected an annotation for the namespace")
        .escaped_snake_case()
        .add("_namespace");

    let mut namespaces: Namespaces = Vec::new();
    let mut test_cases: TestCases = Vec::new();

    let mut encountered_ns_names: HashSet<String> = HashSet::new();
    let mut encountered_tc_names: HashSet<String> = HashSet::new();

    for ns_or_test in element.as_sequence().expect("namespace is list").iter() {
        match ns_or_test.ion_type() {
            // namespace within the namespace
            IonType::List => {
                let mut ns = test_namespace(ns_or_test);
                if encountered_ns_names.contains(&ns.name) {
                    ns.name.push_str("_0")
                }
                encountered_ns_names.insert(ns.name.clone());
                namespaces.push(ns)
            }
            // test case within the namespace
            IonType::Struct => {
                let mut tc = test_case(ns_or_test);
                if encountered_tc_names.contains(&tc.test_name) {
                    tc.test_name.push_str("_0")
                }
                encountered_tc_names.insert(tc.test_name.clone());
                test_cases.push(tc)
            }
            _ => panic!("Namespace parsing requires an IonList or IonStruct"),
        }
    }
    Namespace {
        name,
        namespaces,
        test_cases,
    }
}

/// Parses the given IonStruct to a `TestCase`. The IonStruct requires two string fields with the
/// 'name' and 'statement' in addition to an 'assert' field containing one or more `Assertions`.
///
/// For test assertions that are not supported (e.g. StaticAnalysisFail), the assertion of
/// `NotYetImplemented` will be used.
fn test_case(element: &Element) -> TestCase {
    let test_struct = element.as_struct().expect("struct");
    let test_name = test_struct
        .get("name")
        .expect("name")
        .as_str()
        .expect("as_str()")
        .escaped_snake_case()
        .add("_test");
    let statement = test_struct
        .get("statement")
        .expect("statement")
        .as_str()
        .expect("as_str()")
        .to_string();

    let assert_field = test_struct.get("assert").expect("assert field missing");
    let assertions_vec: Vec<_> = match assert_field.ion_type() {
        IonType::Struct => vec![assert_field],
        IonType::List => assert_field
            .as_sequence()
            .expect("as_sequence")
            .iter()
            .collect(),
        _ => panic!("Invalid IonType for the test case assertions"),
    };
    let assertions = assertions(&assertions_vec);
    TestCase {
        test_name,
        statement,
        assertions,
    }
}

/// Converts the vector of Ion values into `Assertions`. Checks that a result field is provided
/// in the vector and has the symbol 'SyntaxSuccess' or 'SyntaxFail', which correspond to
/// `Assertion::SyntaxSuccess` and `Assertion::SyntaxFail` respectively. Other assertion symbols
/// will default to `Assertion::NotYetImplemented`
fn assertions(assertions: &Vec<&Element>) -> Assertions {
    let mut test_case_assertions: Assertions = Vec::new();
    for assertion in assertions {
        let assertion_struct = assertion.as_struct().expect("as_struct()");
        let parse_result = assertion_struct.get("result");

        if let Some(r) = parse_result {
            let r_as_str = r.as_str().expect("as_str()");
            let assertion = match r_as_str {
                "SyntaxSuccess" => Assertion::SyntaxSuccess,
                "SyntaxFail" => Assertion::SyntaxFail,
                _ => Assertion::NotYetImplemented,
            };
            test_case_assertions.push(assertion);
        }
    }
    test_case_assertions
}

#[cfg(test)]
mod test {}
