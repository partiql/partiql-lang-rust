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

/// Convert a `Vec<Box<dyn EvalExpr>>` into a `Box<dyn EvalExpr>; N]` or error on length mismatch
pub(crate) fn unwrap_args<const N: usize>(
    args: Vec<Box<dyn EvalExpr>>,
) -> Result<[Box<dyn EvalExpr>; N], BindError> {
    args.try_into()
        .map_err(|args: Vec<_>| BindError::ArgNumMismatch {
            expected: 1,
            found: args.len(),
        })
}

/// An expression that is evaluated over `N` input arguments
pub(crate) trait ExecuteEvalExpr<const N: usize>: Debug {
    /// Evaluate the expression
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; N],
        ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value>;
}

/// Used to tell argument checking whether it should exit early or go on as usual.
///
/// Analagous to [`ControlFlow`], but with additional states to handle strict error reporting and
/// `NULL`/`MISSING` propagation.
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

/// A type which performs argument checking during evaluation.
pub(crate) trait ArgChecker: Debug {
    /// Check an argument against an expected type.
    fn arg_check<'a>(
        typ: &PartiqlType,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>>;
}

/// How to handle argument mismatch and `MISSING` propagation
pub(crate) trait ArgShortCircuit: Debug {
    /// Whether a mismatch is an error in `STRICT` mode
    fn is_strict_error() -> bool;
    /// What to propagate on mismatch/`MISSING`
    fn propagate() -> Value;
}

/// Propagate `MISSING` on argument check failure
///
/// `[IS_ERR]` determines whether `[is_strict_error]` returns true.
#[derive(Debug)]
pub(crate) struct PropagateMissing<const IS_ERR: bool> {}
impl<const IS_ERR: bool> ArgShortCircuit for PropagateMissing<IS_ERR> {
    fn is_strict_error() -> bool {
        IS_ERR
    }

    #[inline]
    fn propagate() -> Value {
        Missing
    }
}

/// Propagate `NULL` on argument check failure
///
/// `IS_ERR` determines whether `is_strict_error` returns true.
#[derive(Debug)]
pub(crate) struct PropagateNull<const IS_ERR: bool> {}
impl<const IS_ERR: bool> ArgShortCircuit for PropagateNull<IS_ERR> {
    fn is_strict_error() -> bool {
        IS_ERR
    }

    #[inline]
    fn propagate() -> Value {
        Null
    }
}

/// An [`ArgChecker`] that performs checking appropriate for the majority of expressions and
/// functions.
///
/// Specifically:
/// - `MISSING` input arguments propagate according to the [`OnMissing`] generic parameter
/// - `NULL` input arguments propagate `NULL` to expression output
/// - Upon argument mismatch:
///   - In `STRICT` mode: an error according to the [`OnMissing`] generic parameter
///   - In `PERMISSIVE` mode: a short-circuit return according to the [`OnMissing`] generic parameter
#[derive(Debug)]
pub(crate) struct DefaultArgChecker<const STRICT: bool, OnMissing: ArgShortCircuit> {
    marker: PhantomData<OnMissing>,
}

impl<const STRICT: bool, OnMissing: ArgShortCircuit> ArgChecker
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
            Missing => ArgCheckControlFlow::Propagate(OnMissing::propagate()),
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

/// An [`ArgChecker`] that performs no checking.
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

/// An [`EvalExpr`] which checks its `N` input arguments using `ArgC` and then delegates to an
/// [`ExecuteEvalExpr`].
///
/// Bridges between [`EvalExpr`] and [`ExecuteEvalExpr`]
///
///
#[derive(Debug)]
pub(crate) struct ArgCheckEvalExpr<
    const STRICT: bool,
    const N: usize,
    E: ExecuteEvalExpr<N>,
    ArgC: ArgChecker,
> {
    /// The expected type of expression's positional arguments
    pub(crate) types: [PartiqlType; N],
    /// The expression's positional arguments
    pub(crate) args: [Box<dyn EvalExpr>; N],
    /// the expression
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

    /// Evaluate the input argument expressions in [`self.args`] in the environment, type check them,
    /// and convert them into an array of `N` `Cow<Value>`s.
    ///
    /// If type-checking fails, the appropriate failure case of [`ArgCheckControlFlow`] is returned,
    /// else [`ArgCheckControlFlow::Continue`] is returned containg the `N` values.
    pub fn evaluate_args<'a>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'a dyn EvalContext,
    ) -> ControlFlow<Value, [Cow<Value>; N]> {
        let err_arg_count_mismatch = |args: Vec<_>| {
            if STRICT {
                ctx.add_error(EvaluationError::IllegalState(format!(
                    "# of evaluated arguments ({}) does not match expectation {}",
                    args.len(),
                    N
                )));
            }
            ControlFlow::Break(Missing)
        };

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
            // If `propagate` is a `Some`, then argument type checking failed, propagate the value
            ControlFlow::Break(v)
        } else {
            // If `propagate` is `None`, then try to convert the `result` vec into an array of `N`
            match result.try_into() {
                Ok(a) => ControlFlow::Continue(a),
                Err(args) => err_arg_count_mismatch(args),
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

/// Wraps an `Fn` for use as an [`ExecuteEvalExpr`] executed by an [`ArgCheckEvalExpr`].
pub(crate) struct EvalExprWrapper<E, F> {
    pub ident: E,
    pub f: F,
}

impl<E, F> Debug for EvalExprWrapper<E, F>
where
    E: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvalExprWrapper")
            .field("ident", &self.ident)
            .finish()
    }
}

impl<E: 'static, F: 'static> EvalExprWrapper<E, F> {
    #[inline]
    pub(crate) fn create_checked<const STRICT: bool, const N: usize, ArgC: 'static + ArgChecker>(
        ident: E,
        types: [PartiqlType; N],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        EvalExprWrapper<E, F>: ExecuteEvalExpr<N>,
    {
        let args = unwrap_args(args)?;
        let expr = Self { ident, f };
        let expr = ArgCheckEvalExpr::<STRICT, N, _, ArgC>::new(types, args, expr);
        Ok(Box::new(expr))
    }
}

/// An [`ExecuteEvalExpr`] over a single [`Value`] argument
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct UnaryValueExpr {}

impl<F> ExecuteEvalExpr<1> for EvalExprWrapper<UnaryValueExpr, F>
where
    F: Fn(&Value) -> Value,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 1],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [arg] = args;
        Cow::Owned((self.f)(arg.borrow()))
    }
}

impl UnaryValueExpr {
    #[inline]
    pub(crate) fn create_with_any<const STRICT: bool, F: 'static>(
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value) -> Value,
    {
        Self::create_typed::<STRICT, F>([TYPE_ANY; 1], args, f)
    }

    #[inline]
    pub(crate) fn create_typed<const STRICT: bool, F: 'static>(
        types: [PartiqlType; 1],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value) -> Value,
    {
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        Self::create_checked::<{ STRICT }, Check<STRICT>, F>(types, args, f)
    }

    #[inline]
    pub(crate) fn create_checked<const STRICT: bool, ArgC, F: 'static>(
        types: [PartiqlType; 1],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value) -> Value,
        ArgC: 'static + ArgChecker,
    {
        EvalExprWrapper::create_checked::<{ STRICT }, 1, ArgC>(Self::default(), types, args, f)
    }
}

/// An [`ExecuteEvalExpr`] over a pair of [`Value`] arguments
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct BinaryValueExpr {}

impl<F> ExecuteEvalExpr<2> for EvalExprWrapper<BinaryValueExpr, F>
where
    F: Fn(&Value, &Value) -> Value,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 2],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [arg1, arg2] = args;
        Cow::Owned((self.f)(arg1.borrow(), arg2.borrow()))
    }
}

impl BinaryValueExpr {
    #[inline]
    pub(crate) fn create_with_any<const STRICT: bool, F: 'static>(
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value) -> Value,
    {
        Self::create_typed::<STRICT, F>([TYPE_ANY; 2], args, f)
    }

    #[inline]
    pub(crate) fn create_typed<const STRICT: bool, F: 'static>(
        types: [PartiqlType; 2],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value) -> Value,
    {
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        Self::create_checked::<{ STRICT }, Check<STRICT>, F>(types, args, f)
    }

    #[inline]
    pub(crate) fn create_checked<const STRICT: bool, ArgC, F: 'static>(
        types: [PartiqlType; 2],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value) -> Value,
        ArgC: 'static + ArgChecker,
    {
        EvalExprWrapper::create_checked::<{ STRICT }, 2, ArgC>(Self::default(), types, args, f)
    }
}

/// An [`ExecuteEvalExpr`] over a trio of [`Value`] arguments
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct TernaryValueExpr {}

impl<F> ExecuteEvalExpr<3> for EvalExprWrapper<TernaryValueExpr, F>
where
    F: Fn(&Value, &Value, &Value) -> Value,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 3],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [arg1, arg2, arg3] = args;
        Cow::Owned((self.f)(arg1.borrow(), arg2.borrow(), arg3.borrow()))
    }
}

impl TernaryValueExpr {
    #[inline]
    pub(crate) fn create_with_any<const STRICT: bool, F: 'static>(
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value, &Value) -> Value,
    {
        Self::create_typed::<STRICT, F>([TYPE_ANY; 3], args, f)
    }

    #[inline]
    pub(crate) fn create_typed<const STRICT: bool, F: 'static>(
        types: [PartiqlType; 3],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value, &Value) -> Value,
    {
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        Self::create_checked::<{ STRICT }, Check<STRICT>, F>(types, args, f)
    }

    #[inline]
    pub(crate) fn create_checked<const STRICT: bool, ArgC, F: 'static>(
        types: [PartiqlType; 3],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value, &Value) -> Value,
        ArgC: 'static + ArgChecker,
    {
        EvalExprWrapper::create_checked::<{ STRICT }, 3, ArgC>(Self::default(), types, args, f)
    }
}

/// An [`ExecuteEvalExpr`] over a quartet of [`Value`] arguments
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct QuaternaryValueExpr {}

impl<F> ExecuteEvalExpr<4> for EvalExprWrapper<QuaternaryValueExpr, F>
where
    F: Fn(&Value, &Value, &Value, &Value) -> Value,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 4],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [arg1, arg2, arg3, arg4] = args;
        Cow::Owned((self.f)(
            arg1.borrow(),
            arg2.borrow(),
            arg3.borrow(),
            arg4.borrow(),
        ))
    }
}

impl QuaternaryValueExpr {
    #[inline]
    pub(crate) fn create_with_any<const STRICT: bool, F: 'static>(
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value, &Value, &Value) -> Value,
    {
        Self::create_typed::<STRICT, F>([TYPE_ANY; 4], args, f)
    }

    #[inline]
    pub(crate) fn create_typed<const STRICT: bool, F: 'static>(
        types: [PartiqlType; 4],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value, &Value, &Value) -> Value,
    {
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        Self::create_checked::<{ STRICT }, Check<STRICT>, F>(types, args, f)
    }

    #[inline]
    pub(crate) fn create_checked<const STRICT: bool, ArgC, F: 'static>(
        types: [PartiqlType; 4],
        args: Vec<Box<dyn EvalExpr>>,
        f: F,
    ) -> Result<Box<dyn EvalExpr>, BindError>
    where
        F: Fn(&Value, &Value, &Value, &Value) -> Value,
        ArgC: 'static + ArgChecker,
    {
        EvalExprWrapper::create_checked::<{ STRICT }, 4, ArgC>(Self::default(), types, args, f)
    }
}
