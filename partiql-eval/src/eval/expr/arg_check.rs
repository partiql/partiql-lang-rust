use crate::error::EvaluationError;

use crate::eval::expr::{BindError, EvalExpr};
use crate::eval::EvalContext;
use itertools::Itertools;

use partiql_types::{PartiqlType, TypeKind, TYPE_ANY};
use partiql_value::Value::{Missing, Null};
use partiql_value::{Tuple, Value};

use std::borrow::{Borrow, Cow};
use std::fmt::{Debug, Formatter};
use std::hash::Hash;

use std::marker::PhantomData;

use std::ops::ControlFlow;

// TODO replace with type system's subsumption once it is in place
#[inline]
pub(crate) fn subsumes(typ: &PartiqlType, value: &Value) -> bool {
    match (typ.kind(), value) {
        (TypeKind::Any, _) => true,
        (TypeKind::AnyOf(anyof), val) => anyof.types().any(|typ| subsumes(typ, val)),
        (TypeKind::Null, Value::Null) => true,
        (TypeKind::Missing, Value::Missing) => true,
        (
            TypeKind::Int | TypeKind::Int8 | TypeKind::Int16 | TypeKind::Int32 | TypeKind::Int64,
            Value::Integer(_),
        ) => true,
        (TypeKind::Bool, Value::Boolean(_)) => true,
        (TypeKind::Decimal, Value::Decimal(_)) => true,
        (TypeKind::DecimalP(_, _), Value::Decimal(_)) => true,
        (TypeKind::Float32 | TypeKind::Float64, Value::Real(_)) => true,
        (
            TypeKind::String | TypeKind::StringFixed(_) | TypeKind::StringVarying(_),
            Value::String(_),
        ) => true,
        (TypeKind::Struct(_), Value::Tuple(_)) => true,
        (TypeKind::Bag(_), Value::Bag(_)) => true,
        (TypeKind::DateTime, Value::DateTime(_)) => true,

        (TypeKind::Array(_), Value::List(_)) => true,
        _ => false,
    }
}

pub(crate) fn unwrap_args<const N: usize>(
    args: Vec<Box<dyn EvalExpr>>,
) -> Result<[Box<dyn EvalExpr>; N], BindError> {
    args.try_into()
        .map_err(|args: Vec<_>| BindError::ArgNumMismatch {
            expected: 1,
            found: args.len(),
        })
}

pub(crate) trait ExecuteEvalExpr<const N: usize>: Debug {
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; N],
        ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ArgCheckControlFlow<B, C, R = B> {
    /// Continue checking args; This arg is a `C`
    Continue(C),
    /// Immediately short-circuit return a `B`
    ShortCircuit(B),
    /// Immediately Error in strict mode or short-circuit return a `B` in permissive mode
    ErrorOrShortCircuit(B),
    /// Continue checking args, but propagate a `R` rather than executing the function
    Propagate(R),
}

pub(crate) trait ArgChecker: Debug {
    fn arg_check<'a>(
        typ: &PartiqlType,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>>;
}

pub(crate) trait MissingArgShortCircuit: Debug {
    fn is_strict_error() -> bool;
    fn propagate() -> Value;
}

#[derive(Debug)]
pub(crate) struct MissingToMissing<const IS_ERR: bool> {}
impl<const IS_ERR: bool> MissingArgShortCircuit for MissingToMissing<IS_ERR> {
    fn is_strict_error() -> bool {
        IS_ERR
    }

    #[inline]
    fn propagate() -> Value {
        Missing
    }
}
#[derive(Debug)]
pub(crate) struct MissingToNull<const IS_ERR: bool> {}
impl<const IS_ERR: bool> MissingArgShortCircuit for MissingToNull<IS_ERR> {
    fn is_strict_error() -> bool {
        IS_ERR
    }

    #[inline]
    fn propagate() -> Value {
        Null
    }
}

#[derive(Debug)]
pub(crate) struct DefaultArgChecker<const STRICT: bool, OnMissing: MissingArgShortCircuit> {
    marker: PhantomData<OnMissing>,
}

impl<const STRICT: bool, OnMissing: MissingArgShortCircuit> ArgChecker
    for DefaultArgChecker<STRICT, OnMissing>
{
    fn arg_check<'a>(
        typ: &PartiqlType,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>> {
        let err = || {
            if OnMissing::is_strict_error() {
                ArgCheckControlFlow::ErrorOrShortCircuit(OnMissing::propagate())
            } else {
                ArgCheckControlFlow::ShortCircuit(OnMissing::propagate())
            }
        };

        match arg.borrow() {
            Missing => {
                if STRICT {
                    ArgCheckControlFlow::ShortCircuit(OnMissing::propagate())
                } else {
                    ArgCheckControlFlow::Propagate(OnMissing::propagate())
                }
            }
            Null => ArgCheckControlFlow::Propagate(Null),
            val => {
                if subsumes(typ, val) {
                    ArgCheckControlFlow::Continue(arg)
                } else {
                    err()
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct NullArgChecker {}

impl ArgChecker for NullArgChecker {
    fn arg_check<'a>(
        _typ: &PartiqlType,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>> {
        ArgCheckControlFlow::Continue(arg)
    }
}

#[derive(Debug)]
pub(crate) struct ArgCheckEvalExpr<
    const STRICT: bool,
    const N: usize,
    E: ExecuteEvalExpr<N>,
    ArgC: ArgChecker,
> {
    pub(crate) types: [PartiqlType; N],
    pub(crate) args: [Box<dyn EvalExpr>; N],
    pub(crate) expr: E,
    pub(crate) arg_check: PhantomData<ArgC>,
}

impl<const STRICT: bool, const N: usize, E: ExecuteEvalExpr<N>, ArgC: ArgChecker>
    ArgCheckEvalExpr<STRICT, N, E, ArgC>
{
    pub fn new(types: [PartiqlType; N], args: [Box<dyn EvalExpr>; N], expr: E) -> Self {
        Self {
            types,
            args,
            expr,
            arg_check: PhantomData {},
        }
    }

    pub fn evaluate_args<'a>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'a dyn EvalContext,
    ) -> ControlFlow<Value, [Cow<Value>; N]> {
        let mut result = Vec::with_capacity(N);

        let mut propagate = None;
        for i in 0..N {
            let typ = &self.types[i];
            let arg = self.args[i].evaluate(bindings, ctx);

            match ArgC::arg_check(typ, arg) {
                ArgCheckControlFlow::Continue(v) => {
                    if propagate.is_none() {
                        result.push(v)
                    }
                }
                ArgCheckControlFlow::Propagate(v) => {
                    propagate = match propagate {
                        None => Some(v),
                        Some(prev) => match (prev, v) {
                            (Null, Missing) => Missing,
                            (Missing, _) => Missing,
                            (Null, _) => Null,
                            (_, new) => new,
                        }
                        .into(),
                    };
                }
                ArgCheckControlFlow::ShortCircuit(v) => return ControlFlow::Break(v),
                ArgCheckControlFlow::ErrorOrShortCircuit(v) => {
                    if STRICT {
                        let signature = self
                            .types
                            .iter()
                            .map(|typ| format!("{}", typ.kind()))
                            .join(",");
                        let before = (0..i).map(|_| "_");
                        let arg = "MISSING"; // TODO display actual argument?
                        let after = (i + 1..N).map(|_| "_");
                        let arg_pattern = before.chain(std::iter::once(arg)).chain(after).join(",");
                        let msg = format!("expected `({signature})`, found `({arg_pattern})`");
                        ctx.add_error(EvaluationError::IllegalState(msg));
                    }
                    return ControlFlow::Break(v);
                }
            }
        }

        if let Some(v) = propagate {
            ControlFlow::Break(v)
        } else {
            match result.try_into() {
                Ok(a) => ControlFlow::Continue(a),
                Err(e) => {
                    ctx.add_error(EvaluationError::IllegalState(format!(
                        "# of evaluated arguments ({}) does not match expectation {}",
                        e.len(),
                        N
                    )));
                    ControlFlow::Break(Missing)
                }
            }
        }
    }
}

impl<const STRICT: bool, const N: usize, E: ExecuteEvalExpr<N>, ArgC: ArgChecker> EvalExpr
    for ArgCheckEvalExpr<STRICT, N, E, ArgC>
{
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        if STRICT && ctx.has_errors() {
            return Cow::Owned(Missing);
        }
        match self.evaluate_args(bindings, ctx) {
            ControlFlow::Continue(args) => self.expr.evaluate(args, ctx),
            ControlFlow::Break(short_circuit) => Cow::Owned(short_circuit),
        }
    }
}

pub(crate) struct GenericFn<E, F> {
    pub ident: E,
    pub f: F,
}

impl<E, F> Debug for GenericFn<E, F>
where
    E: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericFn")
            .field("ident", &self.ident)
            .finish()
    }
}

impl<E: 'static, F: 'static> GenericFn<E, F> {
    #[inline]
    pub(crate) fn create<const STRICT: bool, const N: usize>(
        ident: E,
        types: [PartiqlType; N],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        GenericFn<E, F>: ExecuteEvalExpr<N>,
    {
        Self::create_with_checker::<
            { STRICT },
            N,
            DefaultArgChecker<{ STRICT }, MissingToMissing<true>>,
        >(ident, types, args, f)
    }

    #[inline]
    pub(crate) fn create_with_checker<
        const STRICT: bool,
        const N: usize,
        ArgC: 'static + ArgChecker,
    >(
        ident: E,
        types: [PartiqlType; N],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        GenericFn<E, F>: ExecuteEvalExpr<N>,
    {
        let args = unwrap_args(args)?;
        let expr = Self { ident, f };
        let expr = ArgCheckEvalExpr::<STRICT, N, _, ArgC>::new(types, args, expr);
        Ok(Box::new(expr))
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct GenericValueFn {}

impl<const N: usize, F> ExecuteEvalExpr<N> for GenericFn<GenericValueFn, F>
where
    F: Fn([Cow<Value>; N]) -> Cow<Value>,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; N],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        (self.f)(args)
    }
}

impl<F: 'static> GenericFn<GenericValueFn, F> {
    #[inline]
    pub(crate) fn create_generic<const STRICT: bool, const N: usize>(
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: for<'a> Fn([Cow<'a, Value>; N]) -> Cow<'a, Value>,
    {
        let types = [TYPE_ANY; N];
        Self::create_typed_generic::<STRICT, N>(types, args, f)
    }

    #[inline]
    pub(crate) fn create_typed_generic<const STRICT: bool, const N: usize>(
        types: [PartiqlType; N],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: for<'a> Fn([Cow<'a, Value>; N]) -> Cow<'a, Value>,
    {
        Self::create_with_checker::<
            { STRICT },
            N,
            DefaultArgChecker<{ STRICT }, MissingToMissing<true>>,
        >(GenericValueFn::default(), types, args, f)
    }
}
