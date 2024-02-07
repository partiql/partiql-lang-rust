use crate::env::Bindings;

pub use core::borrow::Borrow;

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_value::Value::Missing;
use partiql_value::{BindingsName, Tuple, Value};

use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

/// Represents an evaluation operator for path navigation expressions as outlined in Section `4` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
pub(crate) struct EvalPath {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) components: Vec<EvalPathComponent>,
}

pub(crate) enum EvalPathComponent {
    Key(BindingsName<'static>),
    KeyExpr(Box<dyn EvalExpr>),
    Index(i64),
    IndexExpr(Box<dyn EvalExpr>),
}

impl Debug for EvalPathComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            EvalPathComponent::Key(name) => match name {
                BindingsName::CaseSensitive(s) => write!(f, ".\"{s}\""),
                BindingsName::CaseInsensitive(s) => write!(f, ".{s}"),
            },
            EvalPathComponent::KeyExpr(ke) => {
                write!(f, "[")?;
                ke.fmt(f)?;
                write!(f, "]")
            }
            EvalPathComponent::Index(i) => write!(f, "[{i}]"),
            EvalPathComponent::IndexExpr(ie) => {
                write!(f, "[")?;
                ie.fmt(f)?;
                write!(f, "]")
            }
        }
    }
}

impl Debug for EvalPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.expr.fmt(f)?;
        for cmp in &self.components {
            cmp.fmt(f)?;
        }
        Ok(())
    }
}

#[inline]
fn as_str(v: &Value) -> Option<&str> {
    match v {
        Value::String(s) => Some(s.as_ref()),
        _ => None,
    }
}

#[inline]
fn as_name(v: &Value) -> Option<BindingsName> {
    as_str(v).map(|key| BindingsName::CaseInsensitive(Cow::Borrowed(key)))
}

#[inline]
fn as_int(v: &Value) -> Option<i64> {
    match v {
        Value::Integer(i) => Some(*i),
        _ => None,
    }
}

impl EvalPathComponent {
    #[inline]
    fn get_val<'a>(
        &self,
        value: &'a Value,
        bindings: &'a Tuple,
        ctx: &dyn EvalContext,
    ) -> Option<&'a Value> {
        match (self, value) {
            (EvalPathComponent::Key(k), Value::Tuple(tuple)) => tuple.get(k),
            (EvalPathComponent::Index(idx), Value::List(list)) => list.get(*idx),
            (EvalPathComponent::KeyExpr(ke), Value::Tuple(tuple)) => {
                as_name(ke.evaluate(bindings, ctx).borrow()).and_then(|key| tuple.get(&key))
            }
            (EvalPathComponent::IndexExpr(ie), Value::List(list)) => {
                as_int(ie.evaluate(bindings, ctx).borrow()).and_then(|i| list.get(i))
            }
            _ => None,
        }
    }

    #[inline]
    fn take_val(&self, value: Value, bindings: &Tuple, ctx: &dyn EvalContext) -> Option<Value> {
        match (self, value) {
            (EvalPathComponent::Key(k), Value::Tuple(tuple)) => tuple.take_val(k),
            (EvalPathComponent::Index(idx), Value::List(list)) => list.take_val(*idx),
            (EvalPathComponent::KeyExpr(ke), Value::Tuple(tuple)) => {
                as_name(ke.evaluate(bindings, ctx).borrow()).and_then(|key| tuple.take_val(&key))
            }
            (EvalPathComponent::IndexExpr(ie), Value::List(list)) => {
                as_int(ie.evaluate(bindings, ctx).borrow()).and_then(|i| list.take_val(i))
            }
            _ => None,
        }
    }
}

impl EvalExpr for EvalPath {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.expr.evaluate(bindings, ctx);
        let mut path_componenents = self.components.iter();
        match value {
            Cow::Borrowed(borrowed) => path_componenents
                .try_fold(borrowed, |v, path| path.get_val(v, bindings, ctx))
                .map(Cow::Borrowed),
            Cow::Owned(owned) => path_componenents
                .try_fold(owned, |v, path| path.take_val(v, bindings, ctx))
                .map(Cow::Owned),
        }
        .unwrap_or_else(|| Cow::Owned(Value::Missing))
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
    Local(BindingsName<'static>),
    Global(BindingsName<'static>),
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
#[derive(Clone)]
pub(crate) struct EvalLocalVarRef {
    pub(crate) name: BindingsName<'static>,
}

impl EvalExpr for EvalLocalVarRef {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, _: &'a dyn EvalContext) -> Cow<'a, Value> {
        borrow_or_missing(Bindings::get(bindings, &self.name))
    }
}

impl Debug for EvalLocalVarRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            BindingsName::CaseSensitive(s) => write!(f, "@\"{s}\"",),
            BindingsName::CaseInsensitive(s) => write!(f, "@{s}",),
        }
    }
}

/// Represents a global variable reference in a (sub)query, e.g. `T` in `SELECT t.b as a FROM T as t`.
#[derive(Clone)]
pub(crate) struct EvalGlobalVarRef {
    pub(crate) name: BindingsName<'static>,
}

impl Debug for EvalGlobalVarRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            BindingsName::CaseSensitive(s) => write!(f, "^\"{s}\"",),
            BindingsName::CaseInsensitive(s) => write!(f, "^{s}",),
        }
    }
}

impl EvalExpr for EvalGlobalVarRef {
    fn evaluate<'a>(&'a self, _: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        borrow_or_missing(ctx.bindings().get(&self.name))
    }
}
