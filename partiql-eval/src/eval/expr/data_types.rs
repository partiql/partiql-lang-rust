use crate::error::EvaluationError;

use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;

use partiql_value::Value::Missing;
use partiql_value::{Bag, List, Tuple, Value};
use std::borrow::Cow;
use std::fmt::Debug;

use partiql_logical::Type;
use partiql_value::datum::{DatumCategory, DatumCategoryRef, RefTupleView};
use std::ops::Not;

/// Represents an evaluation operator for Tuple expressions such as `{t1.a: t1.b * 2}` in
/// `SELECT VALUE {t1.a: t1.b * 2} FROM table1 AS t1`.
#[derive(Debug)]
pub(crate) struct EvalTupleExpr {
    pub(crate) attrs: Vec<Box<dyn EvalExpr>>,
    pub(crate) vals: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalTupleExpr {
    fn evaluate<'a, 'c, 'o>(
        &'a self,
        bindings: &'a dyn RefTupleView<'a, Value>,
        ctx: &'c dyn EvalContext,
    ) -> Cow<'o, Value>
    where
        'c: 'a,
        'a: 'o,
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
    fn evaluate<'a, 'c, 'o>(
        &'a self,
        bindings: &'a dyn RefTupleView<'a, Value>,
        ctx: &'c dyn EvalContext,
    ) -> Cow<'o, Value>
    where
        'c: 'a,
        'a: 'o,
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
    fn evaluate<'a, 'c, 'o>(
        &'a self,
        bindings: &'a dyn RefTupleView<'a, Value>,
        ctx: &'c dyn EvalContext,
    ) -> Cow<'o, Value>
    where
        'c: 'a,
        'a: 'o,
    {
        let values = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx).into_owned());

        Cow::Owned(Value::from(values.collect::<Bag>()))
    }
}

/// Represents a `PartiQL` evaluation `IS` operator, e.g. `a IS INT`.
#[derive(Debug)]
pub(crate) struct EvalIsTypeExpr {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) is_type: Type,
    pub(crate) invert: bool,
}

impl EvalExpr for EvalIsTypeExpr {
    fn evaluate<'a, 'c, 'o>(
        &'a self,
        bindings: &'a dyn RefTupleView<'a, Value>,
        ctx: &'c dyn EvalContext,
    ) -> Cow<'o, Value>
    where
        'c: 'a,
        'a: 'o,
    {
        let expr = self.expr.evaluate(bindings, ctx);
        let result = match (&self.is_type, expr.category()) {
            (Type::NullType, DatumCategoryRef::Null | DatumCategoryRef::Missing) => true,
            (Type::MissingType, DatumCategoryRef::Missing) => true,
            (Type::NullType, _) => false,
            (Type::MissingType, _) => false,
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
