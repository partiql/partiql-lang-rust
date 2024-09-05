use crate::schema::spec::*;
use crate::schema::structure::*;

use crate::util::{escape_fn_code, Escaper};
use codegen::{Function, Module, Scope};
use ion_rs_old::TextWriterBuilder;

use ion_rs_old::element::writer::ElementWriter;
use ion_rs_old::element::{Element, Struct};
use quote::__private::TokenStream;
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
                if self.curr_path.iter().any(|s| s.contains("experimental")) {
                    module.attr(r#"cfg(feature = "experimental")"#);
                }
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
        if self.curr_path.iter().any(|s| s.contains("experimental")) {
            module.attr(r#"cfg(feature = "experimental")"#);
        }
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

                if self.curr_path.iter().any(|s| s.contains("experimental")) {
                    module.attr(r#"cfg(feature = "experimental")"#);
                }
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

                if self.curr_path.iter().any(|s| s.contains("experimental")) {
                    module.attr(r#"cfg(feature = "experimental")"#);
                }
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
        if self.curr_path.iter().any(|s| s.contains("experimental")) {
            module.attr(r#"cfg(feature = "experimental")"#);
        }

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

    fn test_case_stmts(&self, test_case: &TestCase) -> Vec<String> {
        match &test_case.statement {
            TestStatement::EquivalenceClass(equiv_id) => {
                let stmts = self
                    .curr_equivs
                    .iter()
                    .rev()
                    .filter_map(|equiv| equiv.get(equiv_id))
                    .next();

                let stmts = stmts.expect("equivalence class named");
                stmts.to_vec()
            }
            TestStatement::Statement(s) => vec![s.clone()],
        }
    }

    fn create_test<F>(
        &mut self,
        scope: &mut Scope,
        test_case: &TestCase,
        name_prefix: Option<&str>,
        needs_env: bool,
        mut body: F,
    ) where
        F: FnMut(&mut Function),
    {
        let bare_name = &test_case.name;
        let prefixed_name = if let Some(prefix) = name_prefix {
            format!("{prefix}_{bare_name}")
        } else {
            bare_name.clone()
        };
        let escaped_name = prefixed_name.escape_test_name();
        let name = self.intern_test_name(escaped_name);

        let test_fn: &mut Function = scope.new_fn(&name);
        test_fn.attr("test");
        test_fn.attr("allow(text_direction_codepoint_in_literal)");

        let doc = format!("Generated test for test named `{}`", &test_case.name);
        test_fn.doc(&doc);

        let env = if let Some(env) = &test_case.env {
            let env = struct_to_string(env);
            quote! {
                let env_ion_text = #env;
                let env = Some(env_ion_text.into());
            }
        } else if needs_env {
            quote! {
                let env = environment();
            }
        } else {
            quote! {}
        };
        test_fn.line(escape_fn_code(env));

        body(test_fn)
    }

    fn write_aside_expected(&mut self, test_case: &TestCase, expected: String) -> (String, bool) {
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

            (format!("{TEST_DATA_DIR}/{expected_file}"), true)
        } else {
            (expected, false)
        }
    }

    fn gen_test(&mut self, scope: &mut Scope, test_case: &TestCase) {
        let stmts = self.test_case_stmts(test_case);
        let test_case_expr =
            |gen: &dyn Fn(&str) -> _| stmts.iter().map(|s| gen(s)).collect::<Vec<_>>();

        fn mode_data(eval_mode: &EvaluationMode) -> (&'static str, &'static str, TokenStream) {
            match eval_mode {
                EvaluationMode::EvalModeError => {
                    ("strict", "strict", quote! { EvaluationMode::Error })
                }
                EvaluationMode::EvalModeCoerce => (
                    "permissive",
                    "permissive",
                    quote! { EvaluationMode::Coerce },
                ),
            }
        }

        for assertion in &test_case.assert {
            match assertion {
                Assertion::SyntaxSuccess(_) => {
                    self.create_test(scope, test_case, None, false, |test_fn| {
                        test_fn.attr(r#"cfg(feature = "syntax")"#);
                        let stmts = test_case_expr(&|stmt: &str| quote! {pass_syntax(#stmt);});
                        test_fn.line(escape_fn_code(quote! {
                            #(#stmts)*
                        }));
                    })
                }
                Assertion::SyntaxFail(_) => {
                    self.create_test(scope, test_case, None, false, |test_fn| {
                        test_fn.attr(r#"cfg(feature = "syntax")"#);
                        let stmts = test_case_expr(&|stmt: &str| quote! {fail_syntax(#stmt);});
                        test_fn.line(escape_fn_code(quote! {
                            #(#stmts)*
                        }));
                    })
                }
                Assertion::StaticAnalysisFail(_) => {
                    self.create_test(scope, test_case, None, false, |test_fn| {
                        test_fn.attr(r#"cfg(feature = "semantics")"#);
                        let stmts = test_case_expr(&|stmt: &str| quote! {fail_semantics(#stmt);});
                        test_fn.line(escape_fn_code(quote! {
                            #(#stmts)*
                        }));
                    })
                }
                Assertion::EvaluationSuccess(EvaluationSuccessAssertion {
                    output,
                    eval_mode,
                    ..
                }) => {
                    for mode in eval_mode {
                        let (prefix, feature, mode) = mode_data(mode);
                        let (expected, is_file) =
                            self.write_aside_expected(test_case, elt_to_string(output));

                        self.create_test(scope, test_case, Some(prefix), true, |test_fn| {
                            test_fn.attr(&format!(r#"cfg(feature = "{feature}")"#));
                            test_fn.line("\n//**** evaluation success test case(s) ****//");

                            let expected = if is_file {
                                quote! {include_str!(#expected)}
                            } else {
                                quote! {#expected}
                            };

                            // emit asserts for all statements
                            let stmts = test_case_expr(&|stmt: &str| {
                                // emit PartiQL statement and evaluation mode assert
                                quote! {
                                    let stmt = #stmt;
                                    pass_eval(stmt, #mode, &env, &expected);
                                }
                            });

                            test_fn.line(escape_fn_code(quote! {
                                let expected = #expected.into();
                                #(#stmts)*
                            }));
                        });
                    }
                }
                Assertion::EvaluationFail(EvaluationFailAssertion { eval_mode, .. }) => {
                    for mode in eval_mode {
                        let (prefix, feature, mode) = mode_data(mode);

                        self.create_test(scope, test_case, Some(prefix), true, |test_fn| {
                            test_fn.attr(&format!(r#"cfg(feature = "{feature}")"#));
                            test_fn.line("\n//**** evaluation failure test case(s) ****//");

                            // emit asserts for all statements
                            let stmts = test_case_expr(&|stmt: &str| {
                                // emit PartiQL statement and evaluation mode assert
                                quote! {
                                    let stmt = #stmt;
                                    fail_eval(stmt, #mode, &env);
                                }
                            });

                            test_fn.line(escape_fn_code(quote! {
                                #(#stmts)*
                            }));
                        });
                    }
                }
            }
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
