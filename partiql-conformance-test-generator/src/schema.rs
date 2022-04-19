/// Conformance test document containing namespaces and/or tests
pub struct TestDocument {
    pub(crate) namespaces: Namespaces,
    pub(crate) test_cases: TestCases,
}
pub type Namespaces = Vec<Namespace>;

/// Namespace can contain other namespaces and/or tests
pub struct Namespace {
    pub(crate) name: String,
    pub(crate) namespaces: Namespaces,
    pub(crate) test_cases: TestCases,
}
pub type TestCases = Vec<TestCase>;

/// Test cases have a `test_name` and a PartiQL `statement` along with additional test fields stored
/// depending on the `test_kind`.
pub struct TestCase {
    pub(crate) test_name: String,
    pub(crate) statement: String,
    pub(crate) test_kind: TestCaseKind,
}

/// Test case kind
///
/// Currently, just supports `Parse` test cases. In the future, other test case variants will be
/// added (e.g. evaluation, type-checking)
pub enum TestCaseKind {
    Parse(ParseTestCase),
}

/// Test case to test the parsing behavior of a PartiQL statement
pub struct ParseTestCase {
    pub(crate) parse_assertions: ParseAssertions,
}

/// Indicates whether a parsing test passes (`ParsePass`) without errors or fails (`ParseFail`) with
/// a parsing-related error
pub enum ParseAssertions {
    ParsePass,
    ParseFail,
}
