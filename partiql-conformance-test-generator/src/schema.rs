/// Conformance test document containing namespaces and/or tests
///
/// Follows the `partiql-tests-data` specified in https://github.com/partiql/partiql-tests/blob/main/partiql-tests-data/partiql-tests-schema.isl.
///
/// TODO: when `ion-schema-rust` supports schema code generation based on .isl, replace these
///  objects.
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

/// Test cases have a `test_name` and a PartiQL `statement` along with the `assertions` to check for
/// expected behavior(s). In the future, additional fields will be added the `TestCase` struct for
/// additional testing configurations.
pub struct TestCase {
    pub(crate) test_name: String,
    pub(crate) statement: String,
    pub(crate) assertions: Assertions,
}
pub type Assertions = Vec<Assertion>;

/// Assertion specifies expected behaviors to be checked in the given test case.
///
/// Currently just supports 'Syntax'-related assertions. In the future, other assertion variants
/// will be added (e.g. static analysis, evaluation). For now, the other assertions will be
/// `NotYetImplemented`.
pub enum Assertion {
    /// Asserts statement is syntactically correct
    SyntaxSuccess,
    /// Asserts statement has at least one syntax error
    SyntaxFail,
    /// Assertion that has yet to be implemented
    NotYetImplemented,
}
