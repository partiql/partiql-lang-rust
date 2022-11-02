pub mod structure {
    use crate::schema::spec::TestDocument;

    pub struct TestRoot {
        pub fail: Vec<TestEntry>,
        pub success: Vec<TestEntry>,
    }

    pub enum TestEntry {
        Dir(TestDir),
        Doc(TestFile),
    }

    pub struct TestDir {
        pub dir_name: String,
        pub contents: Vec<TestEntry>,
    }

    pub struct TestFile {
        pub file_name: String,
        pub contents: TestDocument,
    }
}

pub mod spec {
    /// Conformance test document containing namespaces and/or tests
    ///
    /// Follows the `partiql-tests-data` specified in https://github.com/partiql/partiql-tests/blob/main/partiql-tests-data/partiql-tests-schema.isl.
    ///
    /// TODO: when `ion-schema-rust` supports schema code generation based on .isl, replace these
    ///  objects.
    pub struct TestDocument {
        pub namespaces: Namespaces,
        pub test_cases: TestCases,
    }

    pub type Namespaces = Vec<Namespace>;

    /// Namespace can contain other namespaces and/or tests
    pub struct Namespace {
        pub name: String,
        pub namespaces: Namespaces,
        pub test_cases: TestCases,
    }

    pub type TestCases = Vec<TestCase>;

    /// Test cases have a `test_name` and a PartiQL `statement` along with the `assertions` to check for
    /// expected behavior(s). In the future, additional fields will be added the `TestCase` struct for
    /// additional testing configurations.
    pub struct TestCase {
        pub test_name: String,
        pub statement: String,
        pub assertions: Assertions,
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
}
