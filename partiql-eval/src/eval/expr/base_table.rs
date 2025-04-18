use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;
use itertools::Itertools;
use partiql_catalog::table_fn::BaseTableExpr;

use partiql_value::Value::Missing;
use partiql_value::{Bag, Value};

use partiql_catalog::extension::ExtensionResultError;
use partiql_value::datum::RefTupleView;
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
    fn evaluate<'a, 'c, 'o>(
        &'a self,
        bindings: &'a dyn RefTupleView<'a, Value>,
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
                let bag: Result<Bag, _> = it
                    .map(|r| {
                        match r {
                            Ok(v) => Ok(v),
                            Err(err) => match err {
                                err @ ExtensionResultError::DataError(_) => {
                                    ctx.add_error(err.into());
                                    // This is an error for this data item; coerce it to `Missing` and continue
                                    Ok(Missing)
                                }
                                err => Err(err),
                            },
                        }
                    })
                    .collect();
                match bag {
                    Ok(bag) => Value::from(bag),
                    Err(err) => {
                        // Error on read and/or stream; Treat whole stream as `Missing`
                        ctx.add_error(err.into());
                        Missing
                    }
                }
            }
            Err(err) => {
                // Error on read and/or stream; Treat whole stream as `Missing`
                ctx.add_error(err.into());
                Missing
            }
        };
        Cow::Owned(result)
    }
}
