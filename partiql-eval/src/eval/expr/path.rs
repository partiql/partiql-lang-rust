use crate::env::Bindings;

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_value::Value::Missing;
use partiql_value::{BindingsName, Tuple, Value};

use std::borrow::Cow;
use std::fmt::Debug;

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

/// Represents a local variable reference in a (sub)query, e.g. `b` in `SELECT t.b as a FROM T as t`.
#[derive(Debug, Clone)]
pub(crate) enum EvalVarRef {
    Local(BindingsName),
    Global(BindingsName),
}

impl BindEvalExpr for EvalVarRef {
    fn bind<const STRICT: bool>(
        &self,
        _: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        Ok(match self {
            EvalVarRef::Global(name) => Box::new(EvalGlobalVarRef { name: name.clone() }),
            EvalVarRef::Local(name) => Box::new(EvalLocalVarRef { name: name.clone() }),
        })
    }
}

#[inline]
fn borrow_or_missing(value: Option<&Value>) -> Cow<Value> {
    value.map_or_else(|| Cow::Owned(Missing), Cow::Borrowed)
}

/// Represents a local variable reference in a (sub)query, e.g. `b` in `SELECT t.b as a FROM T as t`.
#[derive(Debug, Clone)]
pub(crate) struct EvalLocalVarRef {
    pub(crate) name: BindingsName,
}

impl EvalExpr for EvalLocalVarRef {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, _: &'a dyn EvalContext) -> Cow<'a, Value> {
        borrow_or_missing(Bindings::get(bindings, &self.name))
    }
}

/// Represents a global variable reference in a (sub)query, e.g. `T` in `SELECT t.b as a FROM T as t`.
#[derive(Debug, Clone)]
pub(crate) struct EvalGlobalVarRef {
    pub(crate) name: BindingsName,
}

impl EvalExpr for EvalGlobalVarRef {
    fn evaluate<'a>(&'a self, _: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        borrow_or_missing(ctx.bindings().get(&self.name))
    }
}
