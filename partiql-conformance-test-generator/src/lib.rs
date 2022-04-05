mod schema;
pub mod util;

use crate::schema::TestCaseKind::Parse;
use crate::schema::{
    Namespace, Namespaces, ParseAssertions, ParseTestCase, TestCase, TestCases, TestDocument,
};
use crate::util::StringExt;
use ion_rs::value::owned::OwnedElement;
use ion_rs::value::{Element, Sequence, Struct, SymbolToken};
use ion_rs::IonType;
use std::collections::HashSet;
use std::ops::Add;

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

/// Parses the given IonStruct to a `TestCase`. Requires for there to be an annotation indicating
/// the test case category (currently limited to just 'parse'). The IonStruct requires two string
/// fields with the 'name' and 'statement'.
fn test_case(element: &OwnedElement) -> TestCase {
    let annot: Vec<_> = element.annotations().map(|a| a.text().expect("")).collect();

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

    if annot.contains(&"parse") {
        TestCase {
            test_name,
            statement,
            test_kind: Parse(parse_test_case(element)),
        }
    } else {
        panic!("Invalid test category annotation provided")
    }
}

/// Converts the IonStruct into a `ParseTestCase`. Requires the 'assert' field to be present in the
/// struct and for it to be an IonStruct or IonList.
fn parse_test_case(element: &OwnedElement) -> ParseTestCase {
    let parse_struct = element.as_struct().expect("struct");
    let assert_field = parse_struct.get("assert").expect("assert");
    let assertions: Vec<_> = match assert_field.ion_type() {
        IonType::Struct => vec![assert_field],
        IonType::List => assert_field
            .as_sequence()
            .expect("as_sequence")
            .iter()
            .collect(),
        _ => panic!("Invalid IonType for the parse test case assertions"),
    };

    ParseTestCase {
        parse_assertions: parse_assertions(&assertions),
    }
}

/// Converts the vector of Ion values into `ParseAssertions`. Checks that a result field is provided
/// in the vector and has the symbol 'ParseOk' or 'ParserError', which correspond to
/// `ParseAssertions::ParsePass` and `ParseAssertions::ParseFail` respectively.
fn parse_assertions(assertions: &Vec<&OwnedElement>) -> ParseAssertions {
    for assertion in assertions {
        let assertion_struct = assertion.as_struct().expect("as_struct()");
        let parse_result = assertion_struct.get("result");
        match parse_result {
            Some(r) => {
                let r_as_str = r.as_str().expect("as_str()");
                return match r_as_str {
                    "ParseOk" => ParseAssertions::ParsePass,
                    "ParserError" => ParseAssertions::ParseFail,
                    _ => panic!("Unexpected parse result {}", r_as_str),
                };
            }
            None => (),
        }
    }
    panic!("Expected a parse result field");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // TODO: add tests checking the conversions between Ion and test schema structs
}
