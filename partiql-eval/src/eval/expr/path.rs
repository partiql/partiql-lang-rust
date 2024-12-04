pub use core::borrow::Borrow;

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_value::Value::Missing;
use partiql_value::{BindingsName, Tuple, Value};

use partiql_catalog::context::Bindings;
use partiql_value::datum::{
    DatumCategory, DatumCategoryOwned, DatumCategoryRef, OwnedSequenceView, OwnedTupleView,
    RefSequenceView, RefTupleView,
};
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
fn as_name(v: &Value) -> Option<BindingsName<'_>> {
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
    fn get_val<'a, 'c>(
        &self,
        value: &'a Value,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Option<Cow<'a, Value>> {
        let category = value.category();
        match (self, category) {
            (EvalPathComponent::Key(k), DatumCategoryRef::Tuple(tuple)) => tuple.get_val(k),
            (EvalPathComponent::Index(idx), DatumCategoryRef::Sequence(seq)) => seq.get_val(*idx),
            (EvalPathComponent::KeyExpr(ke), DatumCategoryRef::Tuple(tuple)) => {
                as_name(ke.evaluate(bindings, ctx).borrow()).and_then(|key| tuple.get_val(&key))
            }
            (EvalPathComponent::IndexExpr(ie), DatumCategoryRef::Sequence(seq)) => {
                as_int(ie.evaluate(bindings, ctx).borrow()).and_then(|i| seq.get_val(i))
            }
            _ => None,
        }
    }

    #[inline]
    fn take_val<'a, 'c>(
        &self,
        value: Value,
        bindings: &Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Option<Cow<'a, Value>> {
        let category = value.into_category();
        match (self, category) {
            (EvalPathComponent::Key(k), DatumCategoryOwned::Tuple(tuple)) => tuple.take_val(k),
            (EvalPathComponent::Index(idx), DatumCategoryOwned::Sequence(seq)) => {
                seq.take_val(*idx)
            }
            (EvalPathComponent::KeyExpr(ke), DatumCategoryOwned::Tuple(tuple)) => {
                as_name(ke.evaluate(bindings, ctx).borrow()).and_then(|key| tuple.take_val(&key))
            }
            (EvalPathComponent::IndexExpr(ie), DatumCategoryOwned::Sequence(seq)) => {
                as_int(ie.evaluate(bindings, ctx).borrow()).and_then(|i| seq.take_val(i))
            }
            _ => None,
        }
        .map(Cow::Owned)
    }
}

impl EvalExpr for EvalPath {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let evaluated = self.expr.evaluate(bindings, ctx);
        let mut path_componenents = self.components.iter();

        path_componenents
            .try_fold(evaluated, |value, path| match value {
                Cow::Borrowed(borrowed) => path.get_val(borrowed, bindings, ctx),
                Cow::Owned(owned) => path.take_val(owned, bindings, ctx),
            })
            .unwrap_or_else(|| Cow::Owned(Value::Missing))
    }
}

/// Represents an operator for dynamic variable name resolution of a (sub)query.
#[derive(Debug)]
pub(crate) struct EvalDynamicLookup {
    pub(crate) lookups: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalDynamicLookup {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let mut lookups = self.lookups.iter().filter_map(|lookup| {
            let val = lookup.evaluate(bindings, ctx);
            match val.as_ref() {
                Missing => None,
                _ => Some(val),
            }
        });

        lookups.next().unwrap_or(Cow::Owned(Value::Missing))
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
        self,
        _: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        Ok(match self {
            EvalVarRef::Global(name) => Box::new(EvalGlobalVarRef { name: name.clone() }),
            EvalVarRef::Local(name) => Box::new(EvalLocalVarRef { name: name.clone() }),
        })
    }
}

#[inline]
fn borrow_or_missing(value: Option<&Value>) -> Cow<'_, Value> {
    value.map_or_else(|| Cow::Owned(Missing), Cow::Borrowed)
}

/// Represents a local variable reference in a (sub)query, e.g. `b` in `SELECT t.b as a FROM T as t`.
#[derive(Clone)]
pub(crate) struct EvalLocalVarRef {
    pub(crate) name: BindingsName<'static>,
}

impl EvalExpr for EvalLocalVarRef {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        _ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
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
    fn evaluate<'a, 'c>(
        &'a self,
        _bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        borrow_or_missing(ctx.bindings().get(&self.name))
    }
}
