/// Conformance test document containing namespaces and/or tests
/// TODO: once test ISL is defined in `partiql-tests` (https://github.com/partiql/partiql-tests/issues/3),
///  add link to ISL. Also, when `ion-schema-rust` supports schema code generation based on .isl,
///  replace these objects.
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
/// added (e.g. evaluation, type-checking). For now, the other test case variants will be `Ignore`.
pub enum TestCaseKind {
    Parse(ParseTestCase),
    Ignore,
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
