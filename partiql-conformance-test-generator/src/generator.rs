use crate::schema::spec::{Assertion, Namespace, TestCase, TestDocument};
use crate::schema::structure::{TestDir, TestEntry, TestFile, TestRoot};

use crate::StringExt;
use codegen::{Function, Module, Scope};
use std::collections::HashMap;

#[derive(Debug)]
pub enum TestComponent {
    Scope(TestScope),
    Module(TestModule),
}

#[derive(Debug)]
pub struct TestScope {
    pub module: Module,
}

#[derive(Debug, Default)]
pub struct TestModule {
    pub children: HashMap<String, TestComponent>,
}

impl TestModule {
    pub fn insert(&mut self, path: &[&String], scope: TestScope) {
        if let Some((first, rest)) = path.split_first() {
            if rest.is_empty() {
                self.children
                    .insert(first.to_string(), TestComponent::Scope(scope));
            } else {
                let child = self
                    .children
                    .entry((*first).clone())
                    .or_insert_with(|| TestComponent::Module(TestModule::default()));
                if let TestComponent::Module(child_mod) = child {
                    child_mod.insert(rest, scope)
                } else {
                    unreachable!();
                }
            }
        }
    }
}

/// Generates a [`TestModule`] root from a [`TestRoot`] specification.
#[derive(Debug)]
pub struct Generator {
    result: TestModule,
    curr_path: Vec<String>,
}

impl Generator {
    pub fn new() -> Generator {
        Self {
            result: Default::default(),
            curr_path: Default::default(),
        }
    }

    pub fn generate(mut self, root: TestRoot) -> miette::Result<TestModule> {
        let TestRoot { fail, success } = root;
        for f in fail {
            self.test_entry(f)
        }
        for s in success {
            self.test_entry(s)
        }

        Ok(self.result)
    }

    fn test_entry(&mut self, entry: TestEntry) {
        match entry {
            TestEntry::Dir(TestDir { dir_name, contents }) => {
                self.curr_path.push(dir_name);
                for c in contents {
                    self.test_entry(c);
                }
                self.curr_path.pop();
            }
            TestEntry::Doc(TestFile {
                file_name,
                contents,
            }) => {
                let mod_name = file_name.replace(".ion", "").escaped_snake_case();
                let out_file = format!("{}.rs", &mod_name);
                let path: Vec<_> = self
                    .curr_path
                    .iter()
                    .chain(std::iter::once(&out_file))
                    .collect();
                let mut module = Module::new(&mod_name);
                gen_tests(module.scope(), &contents);
                self.result.insert(&path, TestScope { module });
            }
        }
    }
}

fn gen_tests(scope: &mut Scope, test_document: &TestDocument) {
    for namespace in &test_document.namespaces {
        gen_mod(scope, namespace);
    }
    for test in &test_document.test_cases {
        gen_test(scope, test);
    }
}

fn gen_mod(scope: &mut Scope, namespace: &Namespace) {
    let module = scope.new_module(&namespace.name);
    for ns in &namespace.namespaces {
        gen_mod(module.scope(), ns);
    }
    for test in &namespace.test_cases {
        gen_test(module.scope(), test);
    }
}
fn gen_test(scope: &mut Scope, test_case: &TestCase) {
    let test_fn: &mut Function = scope.new_fn(&test_case.test_name);
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
}

#[cfg(test)]
mod tests {
    // TODO: add tests checking the conversions between test structs and CodeGen functions
    //  https://github.com/partiql/partiql-lang-rust/issues/101
}
