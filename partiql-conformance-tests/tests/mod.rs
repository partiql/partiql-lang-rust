use partiql_ast_passes::error::AstTransformationError;
use partiql_eval as eval;
use std::ops::Deref;

use partiql_eval::error::{EvalErr, PlanErr};
use partiql_eval::eval::{BasicContext, EvalContext, EvalPlan, EvalResult, Evaluated};
use partiql_logical as logical;
use partiql_parser::{Parsed, ParserError, ParserResult};
use partiql_value::DateTime;

use partiql_catalog::catalog::{PartiqlCatalog, PartiqlSharedCatalog, SharedCatalog};
use partiql_catalog::context::SystemContext;
use thiserror::Error;

mod test_value;
pub(crate) use test_value::TestValue;

use once_cell::sync::Lazy;
pub(crate) static SHARED_CATALOG: Lazy<PartiqlSharedCatalog> = Lazy::new(init_shared_catalog);

fn init_shared_catalog() -> PartiqlSharedCatalog {
    PartiqlCatalog::default().to_shared_catalog()
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub(crate) enum EvaluationMode {
    Coerce,
    Error,
}

impl From<EvaluationMode> for eval::plan::EvaluationMode {
    fn from(value: EvaluationMode) -> Self {
        match value {
            EvaluationMode::Coerce => eval::plan::EvaluationMode::Permissive,
            EvaluationMode::Error => eval::plan::EvaluationMode::Strict,
        }
    }
}

#[track_caller]
#[inline]
pub(crate) fn parse(statement: &str) -> ParserResult {
    let result = partiql_parser::Parser::default().parse(statement);

    #[cfg(feature = "test_pretty_print")]
    if let Ok(result) = &result {
        use partiql_common::pretty::ToPretty;
        let pretty = result.ast.to_pretty_string(80);
        if let Ok(pretty) = pretty {
            println!("{pretty}");
        } else {
            panic!("failed pretty print");
        }
    }

    result
}

#[track_caller]
#[inline]
pub(crate) fn lower(
    catalog: &dyn SharedCatalog,
    parsed: &Parsed<'_>,
) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
    let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
    planner.lower(parsed)
}

#[track_caller]
#[inline]
pub(crate) fn compile(
    mode: EvaluationMode,
    catalog: &dyn SharedCatalog,
    logical: logical::LogicalPlan<logical::BindingsOp>,
) -> Result<EvalPlan, PlanErr> {
    let mut planner = eval::plan::EvaluatorPlanner::new(mode.into(), catalog);
    planner.compile(&logical)
}

#[track_caller]
#[inline]
pub(crate) fn evaluate(plan: EvalPlan, ctx: &dyn EvalContext) -> EvalResult {
    plan.execute(ctx)
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn fail_syntax(statement: &str) {
    let res = parse(statement);
    assert!(
        res.is_err(),
        "When parsing `{statement}`, expected `Err(_)`, but was `{res:#?}`"
    );
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn pass_syntax(statement: &str) -> Parsed {
    let res = parse(statement);
    assert!(
        res.is_ok(),
        "When parsing `{statement}`, expected `Ok(_)`, but was `{res:#?}`"
    );
    res.unwrap()
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn fail_semantics(statement: &str) {
    let catalog: &PartiqlSharedCatalog = SHARED_CATALOG.deref();
    if let Ok(parsed) = parse(statement) {
        let lowered = lower(catalog, &parsed);

        assert!(
            lowered.is_err(),
            "When semantically verifying `{statement}`, expected `Err(_)`, but was `{lowered:#?}`"
        );
    }
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn pass_semantics(statement: &str) {
    let catalog: &PartiqlSharedCatalog = SHARED_CATALOG.deref();
    let parsed = pass_syntax(statement);
    let lowered = lower(catalog, &parsed);
    assert!(
        lowered.is_ok(),
        "When semantically verifying `{statement}`, expected `Ok(_)`, but was `{lowered:#?}`"
    );
}

#[derive(Error, Debug)]
enum TestError<'a> {
    #[error("Parse error: {0:?}")]
    Parse(ParserError<'a>),
    #[error("Lower error: {0:?}")]
    Lower(AstTransformationError),
    #[error("Plan error: {0:?}")]
    Plan(PlanErr),
    #[error("Evaluation error: {0:?}")]
    Eval(EvalErr),
}

impl<'a> From<ParserError<'a>> for TestError<'a> {
    fn from(err: ParserError<'a>) -> Self {
        TestError::Parse(err)
    }
}

impl From<AstTransformationError> for TestError<'_> {
    fn from(err: AstTransformationError) -> Self {
        TestError::Lower(err)
    }
}

impl From<PlanErr> for TestError<'_> {
    fn from(err: PlanErr) -> Self {
        TestError::Plan(err)
    }
}

impl From<EvalErr> for TestError<'_> {
    fn from(err: EvalErr) -> Self {
        TestError::Eval(err)
    }
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn eval<'a>(
    statement: &'a str,
    mode: EvaluationMode,
    env: &Option<TestValue>,
) -> Result<Evaluated, TestError<'a>> {
    let catalog: &PartiqlSharedCatalog = SHARED_CATALOG.deref();

    let parsed = parse(statement)?;
    let lowered = lower(catalog, &parsed)?;

    let bindings = env.as_ref().map(|e| (&e.value).into()).unwrap_or_default();
    let sys = SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);
    let plan = compile(mode, catalog, lowered)?;

    Ok(evaluate(plan, &ctx)?)
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn fail_eval(statement: &str, mode: EvaluationMode, env: &Option<TestValue>) {
    let result = eval(statement, mode, env);

    match result {
        Ok(result) => panic!("When evaluating (mode = {mode:#?}) `{statement}`, expected `Err(_)`, but was `{result:#?}`"),
        Err(TestError::Parse(_)) => panic!("When evaluating (mode = {mode:#?}) `{statement}`, unexpected parse error"),
        Err(TestError::Lower(err)) => panic!("When evaluating (mode = {mode:#?}) `{statement}`, unexpected lowering error `{err:?}`"),
        Err(TestError::Plan(_)) | Err(TestError::Eval(_)) => {}
    }
}

#[track_caller]
#[inline]
#[allow(dead_code)]
pub(crate) fn pass_eval(
    statement: &str,
    mode: EvaluationMode,
    env: &Option<TestValue>,
    expected: &TestValue,
) {
    match eval(statement, mode, env) {
        Ok(v) => {
            assert_eq!(&TestValue::from(v), expected)
        },
        Err(TestError::Parse(err)) => {
            panic!("When evaluating (mode = {mode:#?}) `{statement}`, unexpected parse error: {err:#?}")
        }
        Err(TestError::Lower(err)) => panic!("When evaluating (mode = {mode:#?}) `{statement}`, unexpected lowering error `{err:?}`"),
        Err(TestError::Plan(err)) => panic!("When evaluating (mode = {mode:#?}) `{statement}`, unexpected planning error `{err:?}`"),
        Err(TestError::Eval(err)) => panic!(
            "When evaluating (mode = {mode:#?}) `{statement}`, expected `Ok(_)`, but was `Err({err:#?})`"
        )
    }
}

#[allow(dead_code)]
pub(crate) fn environment() -> Option<TestValue> {
    None
}

// The `partiql_tests` module will be generated by `build.rs` build script.
#[cfg(feature = "conformance_test")]
mod partiql_tests;
