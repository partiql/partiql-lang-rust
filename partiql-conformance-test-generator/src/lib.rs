pub mod generator;
mod schema;
pub mod util;

use crate::schema::{
    Assertion, Assertions, Namespace, Namespaces, TestCase, TestCases, TestDocument,
};
use crate::util::StringExt;
use ion_rs::value::owned::OwnedElement;
use ion_rs::value::{Element, Sequence, Struct, SymbolToken};
use ion_rs::IonType;
use std::collections::HashSet;
use std::ops::Add;

// TODO: move these test data parsing functions to own file
/// Converts a vector of Ion data into a `TestDocument`, which can be composed of `Namespace`s
/// and `TestCase`s. `Namespace`s must be provided as IonLists while `TestCase`s must be provided
/// as IonStructs. Other Ion types will result in a panic.
///
/// When encountering a duplicate namespace/test case, a namespace/test case will be added with
/// '_0' suffixed to the end of the name.
pub fn ion_data_to_test_document(all_ion_data: Vec<OwnedElement>) -> TestDocument {
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
pub fn test_namespace(element: &OwnedElement) -> Namespace {
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
fn test_case(element: &OwnedElement) -> TestCase {
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

    let assert_field = test_struct.get("assert").expect("assert");
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
fn assertions(assertions: &Vec<&OwnedElement>) -> Assertions {
    let mut test_case_assertions: Assertions = Vec::new();
    for assertion in assertions {
        let assertion_struct = assertion.as_struct().expect("as_struct()");
        let parse_result = assertion_struct.get("result");
        match parse_result {
            Some(r) => {
                let r_as_str = r.as_str().expect("as_str()");
                let assertion = match r_as_str {
                    "SyntaxSuccess" => Assertion::SyntaxSuccess,
                    "SyntaxFail" => Assertion::SyntaxFail,
                    _ => Assertion::NotYetImplemented,
                };
                test_case_assertions.push(assertion);
            }
            None => (),
        }
    }
    test_case_assertions
}

#[cfg(test)]
mod tests {
    // TODO: add tests checking the conversions between Ion and test schema structs
    //  https://github.com/partiql/partiql-lang-rust/issues/100
}
