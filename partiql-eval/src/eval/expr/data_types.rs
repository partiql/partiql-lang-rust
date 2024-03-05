use crate::error::EvaluationError;

use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;

use partiql_value::Value::{Missing, Null};
use partiql_value::{Bag, List, Tuple, Value};
use std::borrow::Cow;
use std::fmt::Debug;

use partiql_logical::Type;
use std::ops::Not;

/// Represents an evaluation operator for Tuple expressions such as `{t1.a: t1.b * 2}` in
/// `SELECT VALUE {t1.a: t1.b * 2} FROM table1 AS t1`.
#[derive(Debug)]
pub(crate) struct EvalTupleExpr {
    pub(crate) attrs: Vec<Box<dyn EvalExpr>>,
    pub(crate) vals: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalTupleExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let tuple = self
            .attrs
            .iter()
            .zip(self.vals.iter())
            .filter_map(|(attr, val)| {
                let key = attr.evaluate(bindings, ctx);
                match key.as_ref() {
                    Value::String(key) => {
                        let val = val.evaluate(bindings, ctx);
                        match val.as_ref() {
                            Missing => None,
                            _ => Some((key.to_string(), val.into_owned())),
                        }
                    }
                    _ => None,
                }
            })
            .collect::<Tuple>();

        Cow::Owned(Value::from(tuple))
    }
}

/// Represents an evaluation operator for List (ordered array) expressions such as
/// `[t1.a, t1.b * 2]` in `SELECT VALUE [t1.a, t1.b * 2] FROM table1 AS t1`.
#[derive(Debug)]
pub(crate) struct EvalListExpr {
    pub(crate) elements: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalListExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let values = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx).into_owned());

        Cow::Owned(Value::from(values.collect::<List>()))
    }
}

/// Represents an evaluation operator for Bag (unordered array) expressions such as
/// `<<t1.a, t1.b * 2>>` in `SELECT VALUE <<t1.a, t1.b * 2>> FROM table1 AS t1`.
#[derive(Debug)]
pub(crate) struct EvalBagExpr {
    pub(crate) elements: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalBagExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let values = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx).into_owned());

        Cow::Owned(Value::from(values.collect::<Bag>()))
    }
}

/// Represents a PartiQL evaluation `IS` operator, e.g. `a IS INT`.
#[derive(Debug)]
pub(crate) struct EvalIsTypeExpr {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) is_type: Type,
    pub(crate) invert: bool,
}

impl EvalExpr for EvalIsTypeExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let expr = self.expr.evaluate(bindings, ctx);
        let expr = expr.as_ref();
        let result = match self.is_type {
            Type::NullType => matches!(expr, Missing | Null),
            Type::MissingType => matches!(expr, Missing),
            _ => {
                ctx.add_error(EvaluationError::NotYetImplemented(
                    "`IS` for other types".to_string(),
                ));
                false
            }
        };
        let result = if self.invert { result.not() } else { result };

        Cow::Owned(result.into())
    }
}
