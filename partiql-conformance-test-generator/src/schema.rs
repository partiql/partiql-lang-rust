use crate::schema::TestCaseKind::Parse;
use codegen::{Function, Module, Scope};

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

pub enum ParseAssertions {
    ParsePass,
    ParseFail,
}

impl TestDocument {
    /// Converts a `TestDocument` into a `Scope`
    pub fn generate_scope(&self) -> Scope {
        let mut scope = Scope::new();
        for namespace in &self.namespaces {
            scope.push_module(namespace.generate_mod());
        }
        for test in &self.test_cases {
            scope.push_fn(test.generate_test_fn());
        }
        scope
    }
}

impl Namespace {
    /// Converts a `Namespace` into a `Module`
    fn generate_mod(&self) -> Module {
        let mut module = Module::new(&*self.name);
        for ns in &self.namespaces {
            module.push_module(ns.generate_mod());
        }
        for test in &self.test_cases {
            module.push_fn(test.generate_test_fn());
        }
        module
    }
}

impl TestCase {
    /// Converts a test case into a testing `Function`
    fn generate_test_fn(&self) -> Function {
        match &self.test_kind {
            Parse(ParseTestCase { parse_assertions }) => {
                let mut test_fn: Function = Function::new(&self.test_name);
                test_fn.attr("test").line(format!(
                    "let parse_result = partiql_parser::lalr_parse(\"{}\");",
                    &self.statement
                ));
                match parse_assertions {
                    ParseAssertions::ParsePass => test_fn.line("assert!(parse_result.is_ok());"),
                    ParseAssertions::ParseFail => test_fn.line("assert!(parse_result.is_err());"),
                };
                test_fn
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // TODO: add tests checking the conversions between test structs and CodeGen functions
}
