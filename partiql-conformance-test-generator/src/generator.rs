use crate::schema::{Assertion, Namespace, TestCase, TestDocument};
use codegen::{Function, Module, Scope};

/// Defines a test code generation object
pub struct Generator {
    pub test_document: TestDocument,
}

impl Generator {
    /// Generates a `Scope` from the `Generator`'s `test_document`
    pub fn generate_scope(&self) -> Scope {
        test_document_to_scope(&self.test_document)
    }
}

/// Converts a `TestDocument` into a `Scope`
fn test_document_to_scope(test_document: &TestDocument) -> Scope {
    let mut scope = Scope::new();
    for namespace in &test_document.namespaces {
        scope.push_module(namespace_to_module(namespace));
    }
    for test in &test_document.test_cases {
        scope.push_fn(test_case_to_function(test));
    }
    scope
}

/// Converts a `Namespace` into a `Module`
fn namespace_to_module(namespace: &Namespace) -> Module {
    let mut module = Module::new(&*namespace.name);
    for ns in &namespace.namespaces {
        module.push_module(namespace_to_module(ns));
    }
    for test in &namespace.test_cases {
        module.push_fn(test_case_to_function(test));
    }
    module
}

/// Converts a test case into a testing `Function`
fn test_case_to_function(test_case: &TestCase) -> Function {
    let mut test_fn: Function = Function::new(&test_case.test_name);
    test_fn.attr("test");
    test_fn.line(format!("let statement = r#\"{}\"#;", &test_case.statement));
    for assertion in &test_case.assertions {
        match assertion {
            Assertion::SyntaxSuccess => {
                test_fn.line("let res = partiql_parser::Parser::default().parse(statement);");
                test_fn.line(r#"assert!(res.is_ok(), "For `{}`, expected `Ok(_)`, but was `{:#?}`", statement, res);"#);
            }
            Assertion::SyntaxFail => {
                test_fn.line("let res = partiql_parser::Parser::default().parse(statement);");
                test_fn.line(r#"assert!(res.is_err(), "For `{}`, expected `Err(_)`, but was `{:#?}`", statement, res);"#);
            }
            Assertion::NotYetImplemented => {
                // for `NotYetImplemented` assertions, add the 'ignore' annotation to the test case
                test_fn.attr("ignore = \"not yet implemented\"");
                test_fn.attr("allow(unused_variables)");
            }
        }
    }
    test_fn
}

#[cfg(test)]
mod tests {
    // TODO: add tests checking the conversions between test structs and CodeGen functions
    //  https://github.com/partiql/partiql-lang-rust/issues/101
}
