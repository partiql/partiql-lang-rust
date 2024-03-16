#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

mod subquery_tests;

#[cfg(test)]
mod tests {
    use partiql_ast_passes::error::AstTransformationError;
    use partiql_catalog::context::SystemContext;
    use partiql_catalog::{Catalog, PartiqlCatalog};
    use partiql_eval as eval;
    use partiql_eval::env::basic::MapBindings;
    use partiql_eval::error::{EvalErr, PlanErr};
    use partiql_eval::eval::{BasicContext, EvalPlan, EvalResult, Evaluated};
    use partiql_eval::plan::EvaluationMode;
    use partiql_logical as logical;
    use partiql_parser::{Parsed, ParserError, ParserResult};
    use partiql_value::{DateTime, Value};
    use thiserror::Error;

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
    fn parse(statement: &str) -> ParserResult<'_> {
        partiql_parser::Parser::default().parse(statement)
    }

    #[track_caller]
    #[inline]
    fn lower(
        catalog: &dyn Catalog,
        parsed: &Parsed<'_>,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
        planner.lower(parsed)
    }

    #[track_caller]
    #[inline]
    fn compile(
        mode: EvaluationMode,
        catalog: &dyn Catalog,
        logical: logical::LogicalPlan<logical::BindingsOp>,
    ) -> Result<EvalPlan, PlanErr> {
        let mut planner = eval::plan::EvaluatorPlanner::new(mode, catalog);
        planner.compile(&logical)
    }

    #[track_caller]
    #[inline]
    fn evaluate(mut plan: EvalPlan, bindings: MapBindings<Value>) -> EvalResult {
        let sys = SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);
        plan.execute_mut(&ctx)
    }

    #[track_caller]
    #[inline]
    fn eval(statement: &str, mode: EvaluationMode) -> Result<Evaluated, TestError<'_>> {
        let catalog = PartiqlCatalog::default();

        let parsed = parse(statement)?;
        let lowered = lower(&catalog, &parsed)?;
        let bindings = Default::default();
        let plan = compile(mode, &catalog, lowered)?;
        Ok(evaluate(plan, bindings)?)
    }

    #[test]
    fn order_by_count() {
        let query = "select foo, count(1) as n from
            <<
                { 'foo': 'foo' },
                { 'foo': 'bar' },
                { 'foo': 'qux' },
                { 'foo': 'bar' },
                { 'foo': 'baz' },
                { 'foo': 'bar' },
                { 'foo': 'baz' }
            >>  group by foo order by n desc";

        let res = eval(query, EvaluationMode::Permissive);
        assert!(res.is_ok());
    }
}
