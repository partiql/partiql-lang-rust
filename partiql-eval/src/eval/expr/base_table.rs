use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;
use itertools::Itertools;
use partiql_catalog::BaseTableExpr;

use partiql_value::Value::Missing;
use partiql_value::{Bag, Tuple, Value};

use std::borrow::Cow;
use std::fmt::Debug;

/// Represents a Base Table Expr
#[derive(Debug)]
pub(crate) struct EvalFnBaseTableExpr {
    pub(crate) args: Vec<Box<dyn EvalExpr>>,
    pub(crate) expr: Box<dyn BaseTableExpr>,
}

impl EvalExpr for EvalFnBaseTableExpr {
    #[inline]
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let args = self
            .args
            .iter()
            .map(|arg| arg.evaluate(bindings, ctx))
            .collect_vec();
        let results = self.expr.evaluate(&args, ctx.as_session());
        let result = match results {
            Ok(it) => {
                let bag: Result<Bag, _> = it.collect();
                match bag {
                    Ok(b) => Value::from(b),
                    Err(err) => {
                        ctx.add_error(err.into());
                        Missing
                    }
                }
            }
            Err(err) => {
                ctx.add_error(err.into());
                Missing
            }
        };
        Cow::Owned(result)
    }
}
