use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::catalog::{PartiqlCatalog, SharedCatalog};
use partiql_catalog::context::SystemContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_eval as eval;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::error::{EvalErr, PlanErr};
use partiql_eval::eval::{BasicContext, EvalPlan, EvalResult, Evaluated};
use partiql_eval::plan::EvaluationMode;
use partiql_logical as logical;
use partiql_parser::{Parsed, ParserError, ParserResult};
use partiql_value::{DateTime, Value};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError<'a> {
    #[error("Parse error: {0:?}")]
    Parse(ParserError<'a>),
    #[error("Lower error: {0:?}")]
    Lower(AstTransformationError),
    #[error("Plan error: {0:?}")]
    Plan(PlanErr),
    #[error("Evaluation error: {0:?}")]
    Eval(EvalErr),
    #[error("Extension error: {0:?}")]
    Extension(ExtensionResultError),
    #[error("Other: {0:?}")]
    Other(Box<dyn Error>),
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

impl From<Box<dyn Error>> for TestError<'_> {
    fn from(err: Box<dyn Error>) -> Self {
        TestError::Other(err)
    }
}

impl From<ExtensionResultError> for TestError<'_> {
    fn from(err: ExtensionResultError) -> Self {
        TestError::Extension(err)
    }
}

#[allow(dead_code)]
#[track_caller]
#[inline]
pub fn parse(statement: &str) -> ParserResult<'_> {
    partiql_parser::Parser::default().parse(statement)
}

#[allow(dead_code)]
#[track_caller]
#[inline]
pub fn lower(
    catalog: &dyn SharedCatalog,
    parsed: &Parsed<'_>,
) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
    let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
    planner.lower(parsed)
}

#[allow(dead_code)]
#[track_caller]
#[inline]
pub fn compile(
    mode: EvaluationMode,
    catalog: &dyn SharedCatalog,
    logical: logical::LogicalPlan<logical::BindingsOp>,
) -> Result<EvalPlan, PlanErr> {
    let mut planner = eval::plan::EvaluatorPlanner::new(mode, catalog);
    planner.compile(&logical)
}

#[allow(dead_code)]
#[track_caller]
#[inline]
pub fn evaluate(plan: EvalPlan, bindings: MapBindings<Value>) -> EvalResult {
    let sys = SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);
    plan.execute(&ctx)
}

#[allow(dead_code)]
#[track_caller]
#[inline]
pub fn eval_query_with_catalog<'a>(
    statement: &'a str,
    catalog: &dyn SharedCatalog,
    mode: EvaluationMode,
) -> Result<Evaluated, TestError<'a>> {
    let parsed = parse(statement)?;
    let lowered = lower(catalog, &parsed)?;
    let bindings = Default::default();
    let plan = compile(mode, catalog, lowered)?;
    Ok(evaluate(plan, bindings)?)
}

#[allow(dead_code)]
#[track_caller]
#[inline]
pub fn eval_query(statement: &str, mode: EvaluationMode) -> Result<Evaluated, TestError<'_>> {
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    eval_query_with_catalog(statement, &catalog, mode)
}
