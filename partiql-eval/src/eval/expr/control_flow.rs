use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;

use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::fmt::Debug;

/// Represents a searched case operator, e.g. CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END.
#[derive(Debug)]
pub(crate) struct EvalSearchedCaseExpr {
    pub(crate) cases: Vec<(Box<dyn EvalExpr>, Box<dyn EvalExpr>)>,
    pub(crate) default: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalSearchedCaseExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        for (when_expr, then_expr) in &self.cases {
            let when_expr_evaluated = when_expr.evaluate(bindings, ctx);
            if when_expr_evaluated.as_ref() == &Value::Boolean(true) {
                return then_expr.evaluate(bindings, ctx);
            }
        }
        self.default.evaluate(bindings, ctx)
    }
}
