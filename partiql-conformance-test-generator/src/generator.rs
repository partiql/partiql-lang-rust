use crate::schema::spec::*;
use crate::schema::structure::*;

use crate::util::Escaper;
use codegen::{Function, Module, Scope};
use ion_rs::TextWriterBuilder;

use ion_rs::element::writer::ElementWriter;
use ion_rs::element::{Element, Struct};
use quote::quote;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
#[allow(dead_code)]
pub enum TreeDepth {
    Full,
    N(u8),
}

impl TreeDepth {
    pub fn is_exceeded(&self, depth: &u8) -> bool {
        match self {
            TreeDepth::Full => false,
            TreeDepth::N(n) => depth >= n,
        }
    }
}

#[derive(Debug)]
pub struct GeneratorConfig {
    depth: TreeDepth,
}

impl GeneratorConfig {
    pub fn new(depth: TreeDepth) -> GeneratorConfig {
        GeneratorConfig { depth }
    }
}

#[derive(Debug)]
pub enum TestTree {
    Node(Node),
    Namespace(NamespaceNode),
}

#[derive(Debug)]
pub enum Node {
    Test(TestNode),
    Value(TestValueNode),
}

#[derive(Debug)]
pub struct TestNode {
    pub module: Module,
}

#[derive(Debug)]
pub struct TestValueNode {
    pub value: String,
}

#[derive(Debug, Default)]
pub struct NamespaceNode {
    pub children: HashMap<String, TestTree>,
}

impl NamespaceNode {
    pub fn insert(&mut self, path: &[&String], node: Node) {
        if let Some((first, rest)) = path.split_first() {
            if rest.is_empty() {
                self.children
                    .insert(first.to_string(), TestTree::Node(node));
            } else {
                let child = self
                    .children
                    .entry((*first).clone())
                    .or_insert_with(|| TestTree::Namespace(NamespaceNode::default()));
                if let TestTree::Namespace(child_mod) = child {
                    child_mod.insert(rest, node)
                } else {
                    unreachable!();
                }
            }
        }
    }
}

/// Generates a [`NamespaceNode`] root from a [`TestRoot`] specification.
#[derive(Debug)]
pub struct Generator {
    config: GeneratorConfig,
    result: NamespaceNode,
    curr_path: Vec<String>,
    curr_mod_path: Vec<String>,
    curr_scope_has_mod: Vec<bool>,
    curr_equivs: Vec<HashMap<String, Vec<String>>>,
    seen_fns: Vec<HashSet<String>>,
}

const TEST_DATA_DIR: &str = "_test_data";
const ENV_INLINE_LOWER_BOUND_LINE_COUNT: usize = 10;
const EXPECTED_INLINE_LOWER_BOUND_LINE_COUNT: usize = 25;

impl Generator {
    pub fn new(config: GeneratorConfig) -> Generator {
        Self {
            config,
            result: Default::default(),
            curr_path: Default::default(),
            curr_mod_path: Default::default(),
            curr_scope_has_mod: Default::default(),
            curr_equivs: Default::default(),
            seen_fns: Default::default(),
        }
    }

    pub fn generate(mut self, root: TestRoot) -> miette::Result<NamespaceNode> {
        for entry in root.0 {
            self.test_entry(entry)
        }

        Ok(self.result)
    }

    fn test_entry(&mut self, entry: TestEntry) {
        let depth = self.curr_path.len() + 1;
        if self.config.depth.is_exceeded(&(depth as u8)) {
            self.collapsed_test_entry(entry);
        } else {
            self.nested_test_entry(entry);
        }
    }

    fn push_scope(&mut self, mod_name: Option<String>) {
        self.curr_scope_has_mod.push(mod_name.is_some());
        if let Some(mod_name) = mod_name {
            self.curr_mod_path.push(mod_name);
        }

        self.seen_fns.push(HashSet::new());
        self.curr_equivs.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.curr_equivs.pop();
        self.seen_fns.pop();

        if self.curr_scope_has_mod.pop().unwrap() {
            self.curr_mod_path.pop();
        }
    }

    fn nested_test_entry(&mut self, entry: TestEntry) {
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
                module.attr("allow(unused_imports)");
                module.attr("allow(clippy::module_inception)");
                module.import("super", "*");

                self.push_scope(Some(mod_name.clone()));
                self.gen_tests(module.scope(), &contents);
                self.pop_scope();

                let out_file = format!("{}.rs", &mod_name);
                let path: Vec<_> = self
                    .curr_path
                    .iter()
                    .chain(std::iter::once(&out_file))
                    .collect();
                self.result.insert(&path, Node::Test(TestNode { module }));
            }
        }
    }

    fn collapsed_test_entry(&mut self, entry: TestEntry) {
        let mod_name = match &entry {
            TestEntry::Dir(TestDir { dir_name, .. }) => dir_name.clone(),
            TestEntry::Doc(TestFile { file_name, .. }) => file_name.replace(".ion", ""),
        };

        let mut module = Module::new(&mod_name.escape_module_name());
        module.attr("allow(unused_imports)");
        module.attr("allow(clippy::module_inception)");
        module.import("super", "*");

        self.push_scope(Some(mod_name.clone()));
        self.collapse_test_entry(module.scope(), entry);
        self.pop_scope();

        let out_file = format!("{}.rs", &mod_name.escape_path());
        let path: Vec<_> = self
            .curr_path
            .iter()
            .chain(std::iter::once(&out_file))
            .collect();
        self.result.insert(&path, Node::Test(TestNode { module }));
    }

    fn collapse_test_entry(&mut self, scope: &mut Scope, entry: TestEntry) {
        match entry {
            TestEntry::Dir(TestDir { dir_name, contents }) => {
                let mod_name = dir_name;
                let module = scope.new_module(&mod_name.escape_module_name());
                module.attr("allow(unused_imports)");
                module.attr("allow(clippy::module_inception)");
                module.import("super", "*");

                self.push_scope(Some(mod_name));
                for c in contents {
                    self.collapse_test_entry(module.scope(), c);
                }
                self.pop_scope();
            }
            TestEntry::Doc(TestFile {
                file_name,
                contents,
            }) => {
                let mod_name = file_name.replace(".ion", "");
                let module = scope.new_module(&mod_name.escape_module_name());
                module.attr("allow(unused_imports)");
                module.attr("allow(clippy::module_inception)");
                module.import("super", "*");

                self.push_scope(Some(mod_name));
                self.gen_tests(module.scope(), &contents);
                self.pop_scope();
            }
        }
    }

    fn gen_tests(&mut self, scope: &mut Scope, doc: &PartiQLTestDocument) {
        self.push_scope(None);
        self.gen_variants(scope, &doc.0);
        self.pop_scope();
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

    fn gen_envs(&mut self, scope: &mut Scope, envs: &Environments) {
        let envs = struct_to_string(&envs.envs);

        if envs.lines().count() < ENV_INLINE_LOWER_BOUND_LINE_COUNT {
            self.gen_envs_inline(scope, envs);
        } else {
            self.gen_envs_external(scope, envs);
        }
    }

    fn gen_envs_inline(&mut self, scope: &mut Scope, envs: String) {
        scope.raw(
            quote! {
                const ENV_ION_TEXT : &'static str = #envs;
                fn environment() -> Option<TestValue> {
                    Some(ENV_ION_TEXT.into())
                }
            }
            .to_string()
            .replace("\\n", "\n"),
        );
    }

    fn gen_envs_external(&mut self, scope: &mut Scope, envs: String) {
        let env_file = self
            .curr_mod_path
            .iter()
            .map(|s| s.escape_path())
            .collect::<Vec<_>>()
            .join("___")
            + ".env.ion";

        let data_file = format!("{TEST_DATA_DIR}/{env_file}");
        scope.raw(
            quote! {
                const ENV_ION_TEXT : &'static str = include_str!(#data_file);
                fn environment() -> Option<TestValue> {
                    Some(ENV_ION_TEXT.into())
                }
            }
            .to_string()
            .replace("\\n", "\n"),
        );

        let td_dir = TEST_DATA_DIR.to_string();
        let env_path: Vec<_> = self
            .curr_path
            .iter()
            .chain(std::iter::once(&td_dir))
            .chain(std::iter::once(&env_file))
            .collect();

        self.result.insert(
            env_path.as_slice(),
            Node::Value(TestValueNode { value: envs }),
        );
    }

    fn gen_equivs(&mut self, _scope: &mut Scope, equivs: &EquivalenceClass) {
        self.curr_equivs
            .last_mut()
            .unwrap()
            .insert(equivs.id.to_string(), equivs.statements.clone());
    }

    fn gen_mod(&mut self, scope: &mut Scope, namespace: &Namespace) {
        let mod_name = &namespace.name;
        let module = scope.new_module(&mod_name.escape_module_name());
        module.attr("allow(unused_imports)");
        module.import("super", "*");

        self.push_scope(Some(mod_name.clone()));
        self.gen_variants(module.scope(), &namespace.contents);
        self.pop_scope()
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
        let mut has_env = false;
        let test_case_expr = |gen: &dyn Fn(&str) -> _| match &test_case.statement {
            TestStatement::EquivalenceClass(equiv_id) => {
                let stmts = self
                    .curr_equivs
                    .iter()
                    .rev()
                    .filter_map(|equiv| equiv.get(equiv_id))
                    .next();

                let stmts = stmts.expect("equivalence class named");
                stmts.iter().map(|s| gen(s)).collect::<Vec<_>>()
            }
            TestStatement::Statement(s) => vec![gen(s)],
        };

        if let Some(env) = &test_case.env {
            let env = struct_to_string(env);
            let env = quote! {
                let env_ion_text = #env;
                let env = Some(env_ion_text.into());
            }
            .to_string()
            .replace("\\n", "\n");
            test_fn.line(env);
            has_env = true;
        }

        for assertion in &test_case.assert {
            match assertion {
                Assertion::SyntaxSuccess(_) => {
                    let stmts = test_case_expr(&|stmt: &str| quote! {pass_syntax(#stmt);});
                    let tokens = quote! {
                        #(#stmts)*
                    };
                    test_fn.line(tokens.to_string().replace("\\n", "\n"));
                }
                Assertion::SyntaxFail(_) => {
                    let stmts = test_case_expr(&|stmt: &str| quote! {fail_syntax(#stmt);});
                    let tokens = quote! {
                        #(#stmts)*
                    };
                    test_fn.line(tokens.to_string().replace("\\n", "\n"));
                }
                Assertion::StaticAnalysisFail(_) => {
                    // TODO semantics tests are not yet implemented
                    ignore_test = true;

                    let stmts = test_case_expr(&|stmt: &str| quote! {fail_semantics(#stmt);});
                    let tokens = quote! {
                        #(#stmts)*
                    };
                    test_fn.line(tokens.to_string().replace("\\n", "\n"));
                }
                Assertion::EvaluationSuccess(EvaluationSuccessAssertion {
                    output,
                    eval_mode,
                    ..
                }) => {
                    if !std::mem::replace(&mut has_env, true) {
                        test_fn.line("let env = environment();\n\n");
                    }
                    test_fn.line("\n//**** evaluation success test case(s) ****//");

                    let expected = elt_to_string(output);
                    let expected =
                        if expected.lines().count() > EXPECTED_INLINE_LOWER_BOUND_LINE_COUNT {
                            let expected_file = self
                                .curr_mod_path
                                .iter()
                                .map(|s| s.escape_path())
                                .chain(std::iter::once(test_case.name.escape_path()))
                                .collect::<Vec<_>>()
                                .join("___")
                                + ".expected.ion";

                            let td_dir = TEST_DATA_DIR.to_string();
                            let expected_path: Vec<_> = self
                                .curr_path
                                .iter()
                                .chain(std::iter::once(&td_dir))
                                .chain(std::iter::once(&expected_file))
                                .collect();

                            self.result.insert(
                                expected_path.as_slice(),
                                Node::Value(TestValueNode { value: expected }),
                            );

                            let data_file = format!("{TEST_DATA_DIR}/{expected_file}");
                            quote! {include_str!(#data_file)}
                        } else {
                            quote! {#expected}
                        };

                    let modes: Vec<_> = eval_mode
                        .into_iter()
                        .map(|mode| match mode {
                            EvaluationMode::EvalModeError => quote! { EvaluationMode::Error },
                            EvaluationMode::EvalModeCoerce => quote! { EvaluationMode::Coerce },
                        })
                        .collect();

                    // emit asserts for all statements X all modes
                    let stmts = test_case_expr(&|stmt: &str| {
                        // emit one assert statement per evaluation mode
                        let asserts = modes.iter().map(|mode| {
                            quote! {
                                pass_eval(stmt, #mode, &env, &expected);
                            }
                        });
                        // emit PartiQL statement and evaluation mode asserts
                        quote! {
                            let stmt = #stmt;
                            #(#asserts)*
                        }
                    });

                    let tokens = quote! {
                        let expected = #expected.into();
                        #(#stmts)*
                    };
                    test_fn.line(tokens.to_string().replace("\\n", "\n"));
                }
                Assertion::EvaluationFail(EvaluationFailAssertion { eval_mode, .. }) => {
                    if !std::mem::replace(&mut has_env, true) {
                        test_fn.line("let env = environment();\n\n");
                    }
                    test_fn.line("\n//**** evaluation failure test case(s) ****//");

                    let modes: Vec<_> = eval_mode
                        .into_iter()
                        .map(|mode| match mode {
                            EvaluationMode::EvalModeError => quote! { EvaluationMode::Error },
                            EvaluationMode::EvalModeCoerce => quote! { EvaluationMode::Coerce },
                        })
                        .collect();

                    // emit asserts for all statements X all modes
                    let stmts = test_case_expr(&|stmt: &str| {
                        // emit one assert statement per evaluation mode
                        let asserts = modes.iter().map(|mode| {
                            quote! {
                                fail_eval(stmt, #mode, &env);
                            }
                        });
                        // emit PartiQL statement and evaluation mode asserts
                        quote! {
                            let stmt = #stmt;
                            #(#asserts)*
                        }
                    });

                    let tokens = quote! {
                        #(#stmts)*
                    };
                    test_fn.line(tokens.to_string().replace("\\n", "\n"));
                }
            }
        }

        if ignore_test {
            test_fn.attr("ignore = \"not yet implemented\"");
        }
    }
}

fn struct_to_string(elt: &Struct) -> String {
    elt_to_string(&Element::from(elt.clone()))
}

fn elt_to_string(elt: &Element) -> String {
    let mut buffer = Vec::new();
    {
        let mut writer = TextWriterBuilder::pretty()
            .build(&mut buffer)
            .expect("ion text builder");

        writer.write_element(elt).expect("element write");
    }
    String::from_utf8(buffer).expect("utf8")
}

#[cfg(test)]
mod tests {
    // TODO: add tests checking the conversions between test structs and CodeGen functions
    //  https://github.com/partiql/partiql-lang-rust/issues/101
}
