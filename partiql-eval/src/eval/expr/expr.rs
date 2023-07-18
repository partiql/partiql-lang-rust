use crate::env::Bindings;
use crate::error::EvaluationError;

use crate::eval::expr::pattern_match::like_to_re_pattern;
use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;
use itertools::Itertools;
use partiql_catalog::BaseTableExpr;
use partiql_logical::Type;

use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{Bag, BindingsName, List, Tuple, Value};
use regex::{Regex, RegexBuilder};

use std::borrow::Cow;
use std::fmt::Debug;

use std::ops::Not;

/// Represents an evaluation operator for Tuple expressions such as `{t1.a: t1.b * 2}` in
/// `SELECT VALUE {t1.a: t1.b * 2} FROM table1 AS t1`.
#[derive(Debug)]
pub(crate) struct EvalTupleExpr {
    pub(crate) attrs: Vec<Box<dyn EvalExpr>>,
    pub(crate) vals: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalTupleExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
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
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
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
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let values = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx).into_owned());

        Cow::Owned(Value::from(values.collect::<Bag>()))
    }
}

/// Represents an evaluation operator for path navigation expressions as outlined in Section `4` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub(crate) struct EvalPath {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) components: Vec<EvalPathComponent>,
}

#[derive(Debug)]
pub(crate) enum EvalPathComponent {
    Key(BindingsName),
    KeyExpr(Box<dyn EvalExpr>),
    Index(i64),
    IndexExpr(Box<dyn EvalExpr>),
}

impl EvalExpr for EvalPath {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        #[inline]
        fn path_into<'a>(
            value: &'a Value,
            path: &EvalPathComponent,
            bindings: &'a Tuple,
            ctx: &dyn EvalContext,
        ) -> Option<&'a Value> {
            match path {
                EvalPathComponent::Key(k) => match value {
                    Value::Tuple(tuple) => tuple.get(k),
                    _ => None,
                },
                EvalPathComponent::Index(idx) => match value {
                    Value::List(list) if (*idx as usize) < list.len() => list.get(*idx),
                    _ => None,
                },
                EvalPathComponent::KeyExpr(ke) => {
                    let key = ke.evaluate(bindings, ctx);
                    match (value, key.as_ref()) {
                        (Value::Tuple(tuple), Value::String(key)) => {
                            tuple.get(&BindingsName::CaseInsensitive(key.as_ref().clone()))
                        }
                        _ => None,
                    }
                }
                EvalPathComponent::IndexExpr(ie) => {
                    if let Value::Integer(idx) = ie.evaluate(bindings, ctx).as_ref() {
                        match value {
                            Value::List(list) if (*idx as usize) < list.len() => list.get(*idx),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
            }
        }
        let value = self.expr.evaluate(bindings, ctx);
        self.components
            .iter()
            .fold(Some(value.as_ref()), |v, path| {
                v.and_then(|v| path_into(v, path, bindings, ctx))
            })
            .map_or_else(|| Cow::Owned(Value::Missing), |v| Cow::Owned(v.clone()))
    }
}

/// Represents an operator for dynamic variable name resolution of a (sub)query.
#[derive(Debug)]
pub(crate) struct EvalDynamicLookup {
    pub(crate) lookups: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalDynamicLookup {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let mut lookups = self.lookups.iter().filter_map(|lookup| {
            let val = lookup.evaluate(bindings, ctx);
            match val.as_ref() {
                Missing => None,
                _ => Some(val),
            }
        });

        lookups.next().unwrap_or_else(|| Cow::Owned(Value::Missing))
    }
}

/// Represents a variable reference in a (sub)query, e.g. `a` in `SELECT b as a FROM`.
#[derive(Debug)]
pub(crate) struct EvalVarRef {
    pub(crate) name: BindingsName,
}

impl EvalExpr for EvalVarRef {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = Bindings::get(bindings, &self.name).or_else(|| ctx.bindings().get(&self.name));

        match value {
            None => Cow::Owned(Missing),
            Some(v) => Cow::Borrowed(v),
        }
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
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
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

/// Represents an evaluation `LIKE` operator, e.g. in `s LIKE 'h%llo'`.
#[derive(Debug)]
pub(crate) struct EvalLikeMatch {
    pub(crate) value: Box<dyn EvalExpr>,
    pub(crate) pattern: Regex,
}

// TODO make configurable?
// Limit chosen somewhat arbitrarily, but to be smaller than the default of `10 * (1 << 20)`
pub(crate) const RE_SIZE_LIMIT: usize = 1 << 16;

impl EvalLikeMatch {
    pub(crate) fn new(value: Box<dyn EvalExpr>, pattern: Regex) -> Self {
        EvalLikeMatch { value, pattern }
    }
}

impl EvalExpr for EvalLikeMatch {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.as_ref() {
            Null => Null,
            Missing => Missing,
            Value::String(s) => Boolean(self.pattern.is_match(s.as_ref())),
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents an evaluation `LIKE` operator without string literals in the match and/or escape
/// pattern, e.g. in `s LIKE match_str ESCAPE escape_char`.
#[derive(Debug)]
pub(crate) struct EvalLikeNonStringNonLiteralMatch {
    pub(crate) value: Box<dyn EvalExpr>,
    pub(crate) pattern: Box<dyn EvalExpr>,
    pub(crate) escape: Box<dyn EvalExpr>,
}

impl EvalLikeNonStringNonLiteralMatch {
    pub(crate) fn new(
        value: Box<dyn EvalExpr>,
        pattern: Box<dyn EvalExpr>,
        escape: Box<dyn EvalExpr>,
    ) -> Self {
        EvalLikeNonStringNonLiteralMatch {
            value,
            pattern,
            escape,
        }
    }
}

impl EvalExpr for EvalLikeNonStringNonLiteralMatch {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let pattern = self.pattern.evaluate(bindings, ctx);
        let escape = self.escape.evaluate(bindings, ctx);

        let result = match (value.as_ref(), pattern.as_ref(), escape.as_ref()) {
            (Missing, _, _) => Missing,
            (_, Missing, _) => Missing,
            (_, _, Missing) => Missing,
            (Null, _, _) => Null,
            (_, Null, _) => Null,
            (_, _, Null) => Null,
            (Value::String(v), Value::String(p), Value::String(e)) => {
                if e.chars().count() > 1 {
                    ctx.add_error(EvaluationError::IllegalState(
                        "escape longer than 1 character".to_string(),
                    ));
                }
                let escape = e.chars().next();
                let regex_pattern = RegexBuilder::new(&like_to_re_pattern(p, escape))
                    .size_limit(RE_SIZE_LIMIT)
                    .build();
                match regex_pattern {
                    Ok(pattern) => Boolean(pattern.is_match(v.as_ref())),
                    Err(err) => {
                        ctx.add_error(EvaluationError::IllegalState(err.to_string()));
                        Missing
                    }
                }
            }
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a searched case operator, e.g. CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END.
#[derive(Debug)]
pub(crate) struct EvalSearchedCaseExpr {
    pub(crate) cases: Vec<(Box<dyn EvalExpr>, Box<dyn EvalExpr>)>,
    pub(crate) default: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalSearchedCaseExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        for (when_expr, then_expr) in &self.cases {
            let when_expr_evaluated = when_expr.evaluate(bindings, ctx);
            if when_expr_evaluated.as_ref() == &Value::Boolean(true) {
                return then_expr.evaluate(bindings, ctx);
            }
        }
        self.default.evaluate(bindings, ctx)
    }
}

/// Represents a Base Table Expr
#[derive(Debug)]
pub(crate) struct EvalFnBaseTableExpr {
    pub(crate) args: Vec<Box<dyn EvalExpr>>,
    pub(crate) expr: Box<dyn BaseTableExpr>,
}

impl EvalExpr for EvalFnBaseTableExpr {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let args = self
            .args
            .iter()
            .map(|arg| arg.evaluate(bindings, ctx))
            .collect_vec();
        let results = self.expr.evaluate(&args);
        let result = match results {
            Ok(it) => {
                let bag: Result<Bag, _> = it.collect();
                match bag {
                    Ok(b) => Value::from(b),
                    Err(_) => {
                        // TODO hook into pending eval errors
                        Missing
                    }
                }
            }
            Err(_) => {
                // TODO hook into pending eval errors
                Missing
            }
        };
        Cow::Owned(result)
    }
}
