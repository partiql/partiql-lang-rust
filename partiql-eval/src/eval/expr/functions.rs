use crate::eval::eval_expr_wrapper::{
    evaluate_and_validate_args, DefaultArgChecker, PropagateMissing,
};

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_types::PartiqlNoIdShapeBuilder;
use partiql_value::{Tuple, Value};

use std::borrow::Cow;
use std::fmt::Debug;

use crate::error::EvaluationError;
use partiql_catalog::call_defs::ScalarFnCallSpec;
use partiql_catalog::scalar_fn::ScalarFnExpr;
use std::ops::ControlFlow;

impl BindEvalExpr for ScalarFnCallSpec {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let plan = self.output;
        Ok(Box::new(EvalExprFnScalar::<{ STRICT }> { plan, args }))
    }
}

#[derive(Debug)]
pub(crate) struct EvalExprFnScalar<const STRICT: bool> {
    plan: Box<dyn ScalarFnExpr>,
    args: Vec<Box<dyn EvalExpr>>,
}

impl<const STRICT: bool> EvalExpr for EvalExprFnScalar<STRICT> {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();
        let typ = bld.new_struct_of_dyn();
        match evaluate_and_validate_args::<{ STRICT }, Check<STRICT>, _>(
            &self.args,
            |_| &typ,
            bindings,
            ctx,
        ) {
            ControlFlow::Break(v) => Cow::Owned(v),
            ControlFlow::Continue(args) => match self.plan.evaluate(&args, ctx.as_session()) {
                Ok(v) => v,
                Err(e) => {
                    ctx.add_error(EvaluationError::ExtensionResultError(e));
                    Cow::Owned(Value::Missing)
                }
            },
        }
    }
}
