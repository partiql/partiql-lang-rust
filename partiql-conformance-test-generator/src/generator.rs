use crate::schema::spec::*;
use crate::schema::structure::*;

use crate::util::Escaper;
use codegen::{Function, Module, Scope};
use std::collections::{HashMap, HashSet};

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
    seen_fns: Vec<HashSet<String>>,
}

impl Generator {
    pub fn new() -> Generator {
        Self {
            result: Default::default(),
            curr_path: Default::default(),
            seen_fns: Default::default(),
        }
    }

    pub fn generate(mut self, root: TestRoot) -> miette::Result<TestModule> {
        for entry in root.0 {
            self.test_entry(entry)
        }

        Ok(self.result)
    }

    fn test_entry(&mut self, entry: TestEntry) {
        match entry {
            TestEntry::Dir(TestDir { dir_name, contents }) => {
                self.curr_path.push(dir_name.escape_path());
                for c in contents {
                    self.test_entry(c);
                }
                self.curr_path.pop();
            }
            TestEntry::Doc(TestFile {
                file_name,
                contents,
            }) => {
                let mod_name = file_name.replace(".ion", "").escape_path();
                let mut module = Module::new(&mod_name);
                self.gen_tests(module.scope(), &contents);

                let out_file = format!("{}.rs", &mod_name);
                let path: Vec<_> = self
                    .curr_path
                    .iter()
                    .chain(std::iter::once(&out_file))
                    .collect();
                self.result.insert(&path, TestScope { module });
            }
        }
    }

    fn gen_tests(&mut self, scope: &mut Scope, doc: &PartiQLTestDocument) {
        self.seen_fns.push(HashSet::new());
        self.gen_variants(scope, &doc.0);
        self.seen_fns.pop();
    }

    fn gen_variants(&mut self, scope: &mut Scope, variants: &[TestVariant]) {
        for var in variants {
            match var {
                TestVariant::TestCase(test) => self.gen_test(scope, test),
                TestVariant::Namespace(namespace) => self.gen_mod(scope, namespace),
                TestVariant::Environments(envs) => self.gen_envs(scope, envs),
                TestVariant::EquivalenceClass(equivs) => self.gen_equivs(scope, equivs),
            }
        }
    }

    fn gen_envs(&mut self, _scope: &mut Scope, _envs: &Environments) {
        // TODO
    }

    fn gen_equivs(&mut self, _scope: &mut Scope, _equivs: &EquivalenceClass) {
        // TODO
    }

    fn gen_mod(&mut self, scope: &mut Scope, namespace: &Namespace) {
        let module = scope.new_module(&namespace.name.escape_module_name());
        self.seen_fns.push(HashSet::new());
        self.gen_variants(module.scope(), &namespace.contents);
        self.seen_fns.pop();
    }

    fn intern_test_name(&mut self, mut name: String) -> String {
        let seen_fns = self.seen_fns.last_mut().unwrap();

        while seen_fns.contains(&name) {
            name.push('_');
        }

        seen_fns.insert(name.clone());
        name
    }

    fn gen_test(&mut self, scope: &mut Scope, test_case: &TestCase) {
        let escaped_name = test_case.name.escape_test_name();
        let name = self.intern_test_name(escaped_name);

        let test_fn: &mut Function = scope.new_fn(&name);
        test_fn.attr("test");
        test_fn.attr("allow(text_direction_codepoint_in_literal)");

        let doc = format!("Generated test for test named `{}`", &test_case.name);
        test_fn.doc(&doc);

        let mut ignore_test = false;

        for assertion in &test_case.assert {
            match assertion {
                Assertion::SyntaxSuccess(_) => {
                    test_fn.line(format!(
                        r####"crate::pass_syntax(r#"{}"#);"####,
                        &test_case.statement
                    ));
                }
                Assertion::SyntaxFail(_) => {
                    test_fn.line(format!(
                        r####"crate::fail_syntax(r#"{}"#);"####,
                        &test_case.statement
                    ));
                }
                Assertion::StaticAnalysisFail(_) => {
                    // TODO semantics tests are not yet implemented
                    ignore_test = true;

                    test_fn.line(format!(
                        r####"crate::fail_semantics(r#"{}"#);"####,
                        &test_case.statement
                    ));
                }
                Assertion::EvaluationSuccess(_) => {
                    // TODO semantics tests are not yet implemented
                    ignore_test = true;

                    test_fn.line(format!(
                        r####"crate::pass_eval(r##"{}"##);"####,
                        &test_case.statement
                    ));
                }
                Assertion::EvaluationFail(_) => {
                    // TODO semantics tests are not yet implemented
                    ignore_test = true;

                    test_fn.line(format!(
                        r####"crate::fail_eval(r##"{}"##);"####,
                        &test_case.statement
                    ));
                }
            }
        }

        if ignore_test {
            test_fn.attr("ignore = \"not yet implemented\"");
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: add tests checking the conversions between test structs and CodeGen functions
    //  https://github.com/partiql/partiql-lang-rust/issues/101
}
