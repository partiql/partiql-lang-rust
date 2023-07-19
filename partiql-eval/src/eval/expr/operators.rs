use crate::eval::expr::arg_check::{
    unwrap_args, ArgCheckControlFlow, ArgCheckEvalExpr, ArgChecker, DefaultArgChecker,
    ExecuteEvalExpr, GenericFn, MissingArgShortCircuit, MissingToMissing, MissingToNull,
    NullArgChecker,
};

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_types::{
    ArrayType, BagType, PartiqlType, StructType, TypeKind, TYPE_ANY, TYPE_BOOL, TYPE_DECIMAL,
    TYPE_DOUBLE, TYPE_INT, TYPE_REAL,
};
use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{BinaryAnd, BinaryOr, NullableEq, NullableOrd, Value};

use std::borrow::{Borrow, Cow};
use std::fmt::Debug;

use std::marker::PhantomData;

use std::ops::ControlFlow;

/// Represents a literal in (sub)query, e.g. `1` in `a + 1`.
#[derive(Debug)]
pub(crate) struct EvalLitExpr {
    pub(crate) lit: Box<Value>,
}

impl BindEvalExpr for EvalLitExpr {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let expr: ArgCheckEvalExpr<
            STRICT,
            0,
            _,
            DefaultArgChecker<{ STRICT }, MissingToMissing<false>>,
        > = ArgCheckEvalExpr::new([], unwrap_args(args)?, self.lit.as_ref().clone());
        Ok(Box::new(expr))
    }
}

impl ExecuteEvalExpr<0> for Value {
    fn evaluate<'a>(
        &'a self,
        _args: [Cow<'a, Value>; 0],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        Cow::Borrowed(self)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EvalOpUnary {
    Pos,
    Neg,
    Not,
}

impl BindEvalExpr for EvalOpUnary {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let any_num = PartiqlType::any_of([TYPE_INT, TYPE_REAL, TYPE_DOUBLE, TYPE_DECIMAL]);

        let unop = |types, f: fn(&Value) -> Value| {
            GenericFn::create_with_checker::<
                { STRICT },
                1,
                DefaultArgChecker<{ STRICT }, MissingToMissing<true>>,
            >(*self, types, args, f)
        };

        match self {
            EvalOpUnary::Pos => unop([any_num], |operand| operand.clone()),
            EvalOpUnary::Neg => unop([any_num], |operand| -operand),
            EvalOpUnary::Not => unop([TYPE_BOOL], |operand| !operand),
        }
    }
}

impl<F, R> ExecuteEvalExpr<1> for GenericFn<EvalOpUnary, F>
where
    F: Fn(&Value) -> R,
    R: Into<Value>,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 1],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [val] = args;
        Cow::Owned(((self.f)(val.borrow())).into())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EvalOpBinary {
    // Logical ops
    And,
    Or,

    // Equality ops
    Eq,
    Neq,
    Gt,
    Gteq,
    Lt,
    Lteq,

    // Arithmetic ops
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,

    // other ops
    In,
    Concat,
}

#[derive(Debug)]
struct BoolShortCircuitArgChecker<const B: bool, OnMissing: MissingArgShortCircuit> {
    marker: PhantomData<OnMissing>,
}

impl<const B: bool, OnMissing: MissingArgShortCircuit> ArgChecker
    for BoolShortCircuitArgChecker<B, OnMissing>
{
    fn arg_check<'a>(
        _typ: &PartiqlType,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>> {
        match arg.borrow() {
            Boolean(b) if b == &B => ArgCheckControlFlow::ShortCircuit(Value::Boolean(*b)),
            Missing => ArgCheckControlFlow::ShortCircuit(OnMissing::propagate()),
            Null => ArgCheckControlFlow::Propagate(Null),
            _ => ArgCheckControlFlow::Continue(arg),
        }
    }
}

impl BindEvalExpr for EvalOpBinary {
    #[inline]
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        type AndCheck = BoolShortCircuitArgChecker<false, MissingToNull<false>>;
        type OrCheck = BoolShortCircuitArgChecker<true, MissingToNull<false>>;
        type InCheck<const STRICT: bool> = DefaultArgChecker<STRICT, MissingToNull<false>>;
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, MissingToMissing<true>>;
        type EqCheck<const STRICT: bool> = DefaultArgChecker<STRICT, MissingToMissing<false>>;
        type MathCheck<const STRICT: bool> = DefaultArgChecker<STRICT, MissingToMissing<true>>;

        macro_rules! create {
            ($check: ty, $types: expr, $f:expr) => {
                GenericFn::create_with_checker::<{ STRICT }, 2, $check>(*self, $types, args, $f)
            };
        }

        macro_rules! logical {
            ($check: ty, $f:expr) => {
                create!($check, [TYPE_BOOL, TYPE_BOOL], $f)
            };
        }

        macro_rules! equality {
            ($f:expr) => {
                create!(EqCheck<STRICT>, [TYPE_ANY, TYPE_ANY], $f)
            };
        }

        macro_rules! math {
            ($f:expr) => {{
                let nums = PartiqlType::any_of([TYPE_INT, TYPE_REAL, TYPE_DOUBLE, TYPE_DECIMAL]);
                create!(MathCheck<STRICT>, [nums.clone(), nums], $f)
            }};
        }

        match self {
            EvalOpBinary::And => logical!(AndCheck, |lhs, rhs| lhs.and(rhs)),
            EvalOpBinary::Or => logical!(OrCheck, |lhs, rhs| lhs.or(rhs)),
            EvalOpBinary::Eq => equality!(|lhs, rhs| NullableEq::eq(lhs, rhs)),
            EvalOpBinary::Neq => equality!(|lhs, rhs| NullableEq::neq(lhs, rhs)),
            EvalOpBinary::Gt => equality!(|lhs, rhs| NullableOrd::gt(lhs, rhs)),
            EvalOpBinary::Gteq => equality!(|lhs, rhs| NullableOrd::gteq(lhs, rhs)),
            EvalOpBinary::Lt => equality!(|lhs, rhs| NullableOrd::lt(lhs, rhs)),
            EvalOpBinary::Lteq => equality!(|lhs, rhs| NullableOrd::lteq(lhs, rhs)),
            EvalOpBinary::Add => math!(|lhs, rhs| lhs + rhs),
            EvalOpBinary::Sub => math!(|lhs, rhs| lhs - rhs),
            EvalOpBinary::Mul => math!(|lhs, rhs| lhs * rhs),
            EvalOpBinary::Div => math!(|lhs, rhs| lhs / rhs),
            EvalOpBinary::Mod => math!(|lhs, rhs| lhs % rhs),
            EvalOpBinary::Exp => Err(BindError::NotYetImplemented("exp".to_string())),
            EvalOpBinary::In => {
                create!(
                    InCheck<STRICT>,
                    [
                        TYPE_ANY,
                        PartiqlType::any_of([
                            PartiqlType::new(TypeKind::Array(ArrayType::new_any())),
                            PartiqlType::new(TypeKind::Bag(BagType::new_any())),
                        ])
                    ],
                    |lhs, rhs| {
                        match rhs.sequence_iter() {
                            // TODO apply the changes once we clarify the rules of coercion for `IN` RHS.
                            // See also:
                            // - https://github.com/partiql/partiql-docs/pull/13
                            // - https://github.com/partiql/partiql-lang-kotlin/issues/524
                            // - https://github.com/partiql/partiql-lang-kotlin/pull/621#issuecomment-1147754213

                            // TODO change the Null propagation if required.
                            // Current implementation propagates `Null` as described in PartiQL spec section 8 [1]
                            // and differs from `partiql-lang-kotlin` impl [2].
                            // [1] https://github.com/partiql/partiql-lang-kotlin/issues/896
                            // [2] https://partiql.org/assets/PartiQL-Specification.pdf#section.8
                            Some(mut iter) => {
                                let ret = iter
                                    // init accum to `false`, look for the value
                                    .try_fold(Boolean(false), |accum, elem| match elem {
                                        // found it, short circuit and immediately return `true`
                                        x if x == lhs => ControlFlow::Break(Boolean(true)),
                                        // found null/missing; return `Null` if `lhs` is not subsequently found
                                        Null | Missing => ControlFlow::Continue(Null),
                                        // something else; carry forward knowledge of previous null/missing and keep looking
                                        _ => ControlFlow::Continue(accum),
                                    });
                                match ret {
                                    ControlFlow::Continue(v) => v,
                                    ControlFlow::Break(v) => v,
                                }
                            }
                            None => Null,
                        }
                    }
                )
            }
            EvalOpBinary::Concat => {
                create!(Check<STRICT>, [TYPE_ANY, TYPE_ANY], |lhs, rhs| {
                    // TODO non-naive concat (i.e., don't just use debug print for non-strings).
                    let lhs = if let Value::String(s) = lhs {
                        s.as_ref().clone()
                    } else {
                        format!("{lhs:?}")
                    };
                    let rhs = if let Value::String(s) = rhs {
                        s.as_ref().clone()
                    } else {
                        format!("{rhs:?}")
                    };
                    Value::String(Box::new(format!("{lhs}{rhs}")))
                })
            }
        }
    }
}

impl<F, R> ExecuteEvalExpr<2> for GenericFn<EvalOpBinary, F>
where
    F: Fn(&Value, &Value) -> R,
    R: Into<Value>,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 2],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [lhs, rhs] = args;
        Cow::Owned(((self.f)(lhs.borrow(), rhs.borrow())).into())
    }
}

/// Represents an evaluation PartiQL `BETWEEN` operator, e.g. `x BETWEEN 10 AND 20`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalBetweenExpr {}

impl BindEvalExpr for EvalBetweenExpr {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let types = [TYPE_ANY, TYPE_ANY, TYPE_ANY];

        let args = unwrap_args(args)?;
        Ok(Box::new(
            ArgCheckEvalExpr::<STRICT, 3, _, NullArgChecker>::new(types, args, self.clone()),
        ))
    }
}

impl ExecuteEvalExpr<3> for EvalBetweenExpr {
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 3],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [value, from, to] = args;

        Cow::Owned(value.gteq(&from).and(&value.lteq(&to)))
    }
}

/// Represents an `EXISTS` function, e.g. `exists(`(1)`)`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalFnExists {}

impl BindEvalExpr for EvalFnExists {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        GenericFn::create_generic::<STRICT, 1>(args, |[value]| {
            let exists = match value.borrow() {
                Value::Bag(b) => !b.is_empty(),
                Value::List(l) => !l.is_empty(),
                Value::Tuple(t) => !t.is_empty(),
                _ => false,
            };
            Cow::Owned(Value::Boolean(exists))
        })
    }
}

/// Represents an `ABS` function, e.g. `abs(-1)`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalFnAbs {}

impl BindEvalExpr for EvalFnAbs {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let nums = PartiqlType::any_of([TYPE_INT, TYPE_REAL, TYPE_DOUBLE, TYPE_DECIMAL]);
        GenericFn::create_typed_generic::<STRICT, 1>([nums], args, |[value]| {
            let zero: Value = 0.into();
            Cow::Owned(match NullableOrd::lt(value.borrow(), &zero) {
                Null => Null,
                Missing => Missing,
                Value::Boolean(true) => -value.into_owned(),
                _ => value.into_owned(),
            })
        })
    }
}

/// Represents an `CARDINALITY` function, e.g. `cardinality([1,2,3])`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalFnCardinality {}

impl BindEvalExpr for EvalFnCardinality {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let collections = PartiqlType::any_of([
            PartiqlType::new(TypeKind::Array(ArrayType::new_any())),
            PartiqlType::new(TypeKind::Bag(BagType::new_any())),
            PartiqlType::new(TypeKind::Struct(StructType::new_any())),
        ]);
        GenericFn::create_typed_generic::<STRICT, 1>([collections], args, |[value]| {
            let result = match value.borrow() {
                Value::List(l) => Value::from(l.len()),
                Value::Bag(b) => Value::from(b.len()),
                Value::Tuple(t) => Value::from(t.len()),
                _ => Missing,
            };
            Cow::Owned(result)
        })
    }
}
