pub mod structure {
    use crate::schema::spec::PartiQLTestDocument;

    #[derive(Debug, Clone)]
    pub struct TestRoot(pub Vec<TestEntry>);

    #[derive(Debug, Clone)]
    pub enum TestEntry {
        Dir(TestDir),
        Doc(TestFile),
    }

    #[derive(Debug, Clone)]
    pub struct TestDir {
        pub dir_name: String,
        pub contents: Vec<TestEntry>,
    }

    #[derive(Debug, Clone)]
    pub struct TestFile {
        pub file_name: String,
        pub contents: PartiQLTestDocument,
    }
}

pub mod spec {
    use ion_rs::value::owned::{Element, Struct};

    #[derive(Debug, Clone)]
    pub enum TestVariant {
        TestCase(TestCase),
        Namespace(Namespace),
        Environments(Environments),
        EquivalenceClass(EquivalenceClass),
    }

    #[derive(Debug, Clone)]
    pub struct PartiQLTestDocument(pub Vec<TestVariant>);

    #[derive(Debug, Clone)]
    pub struct Namespace {
        pub name: String,
        pub contents: Vec<TestVariant>,
    }

    #[derive(Debug, Clone)]
    pub struct Environments {
        pub envs: Struct,
    }

    #[derive(Debug, Clone)]
    pub struct EquivalenceClass {
        pub id: String,
        pub statements: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct TestCase {
        pub name: String,
        pub statement: String,
        pub env: Option<Struct>,
        pub assert: Vec<Assertion>,
    }

    #[derive(Debug, Clone)]
    pub enum Assertion {
        SyntaxSuccess(SyntaxSuccessAssertion),
        SyntaxFail(SyntaxFailAssertion),
        StaticAnalysisFail(StaticAnalysisFailAssertion),
        EvaluationSuccess(EvaluationSuccessAssertion),
        EvaluationFail(EvaluationFailAssertion),
    }

    impl From<SyntaxSuccessAssertion> for Assertion {
        fn from(assertion: SyntaxSuccessAssertion) -> Self {
            Assertion::SyntaxSuccess(assertion)
        }
    }

    impl From<SyntaxFailAssertion> for Assertion {
        fn from(assertion: SyntaxFailAssertion) -> Self {
            Assertion::SyntaxFail(assertion)
        }
    }

    impl From<StaticAnalysisFailAssertion> for Assertion {
        fn from(assertion: StaticAnalysisFailAssertion) -> Self {
            Assertion::StaticAnalysisFail(assertion)
        }
    }

    impl From<EvaluationSuccessAssertion> for Assertion {
        fn from(assertion: EvaluationSuccessAssertion) -> Self {
            Assertion::EvaluationSuccess(assertion)
        }
    }

    impl From<EvaluationFailAssertion> for Assertion {
        fn from(assertion: EvaluationFailAssertion) -> Self {
            Assertion::EvaluationFail(assertion)
        }
    }

    #[derive(Debug, Clone)]
    pub enum EvaluationMode {
        EvalModeError,
        EvalModeCoerce,
    }

    #[derive(Debug, Clone)]
    pub enum EvaluationModeSymbolOrList {
        Mode(EvaluationMode),
        List(Vec<EvaluationMode>),
    }

    impl From<EvaluationMode> for EvaluationModeSymbolOrList {
        fn from(mode: EvaluationMode) -> Self {
            EvaluationModeSymbolOrList::Mode(mode)
        }
    }

    impl From<Vec<EvaluationMode>> for EvaluationModeSymbolOrList {
        fn from(mode: Vec<EvaluationMode>) -> Self {
            EvaluationModeSymbolOrList::List(mode)
        }
    }

    #[derive(Debug, Clone)]
    pub struct SyntaxSuccessAssertion {
        pub result: String,
    }

    #[derive(Debug, Clone)]
    pub struct SyntaxFailAssertion {
        pub result: String,
    }

    #[derive(Debug, Clone)]
    pub struct StaticAnalysisFailAssertion {
        pub result: String,
    }

    #[derive(Debug, Clone)]
    pub struct EvaluationSuccessAssertion {
        pub result: String,
        pub output: Element,
        pub eval_mode: EvaluationModeSymbolOrList,
    }

    #[derive(Debug, Clone)]
    pub struct EvaluationFailAssertion {
        pub result: String,
        pub eval_mode: EvaluationModeSymbolOrList,
    }
}
