use crate::eval::eval_expr_wrapper::{
    unwrap_args, ArgCheckControlFlow, ArgCheckEvalExpr, ArgChecker, ArgShortCircuit,
    BinaryValueExpr, DefaultArgChecker, ExecuteEvalExpr, NullArgChecker, PropagateMissing,
    PropagateNull, TernaryValueExpr, UnaryValueExpr,
};

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_types::{
    ArrayType, BagType, PartiqlType, StructType, TypeKind, TYPE_ANY, TYPE_BOOL, TYPE_NUMERIC_TYPES,
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
            DefaultArgChecker<{ STRICT }, PropagateMissing<false>>,
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
        let any_num = PartiqlType::any_of(TYPE_NUMERIC_TYPES);

        let unop = |types, f: fn(&Value) -> Value| {
            UnaryValueExpr::create_typed::<{ STRICT }, _>(types, args, f)
        };

        match self {
            EvalOpUnary::Pos => unop([any_num], |operand| operand.clone()),
            EvalOpUnary::Neg => unop([any_num], |operand| -operand),
            EvalOpUnary::Not => unop([TYPE_BOOL], |operand| !operand),
        }
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

/// An [`ArgChecker`] for short-circuiting boolean logical operators.
///
/// `Target` is the value upon which, if found, to short-circuit
/// `OnMissing` contains the logic for how to handle `Missing` values.
#[derive(Debug)]
struct BoolShortCircuitArgChecker<const TARGET: bool, OnMissing: ArgShortCircuit> {
    marker: PhantomData<OnMissing>,
}

impl<const TARGET: bool, OnMissing: ArgShortCircuit> ArgChecker
    for BoolShortCircuitArgChecker<TARGET, OnMissing>
{
    fn arg_check<'a>(
        _typ: &PartiqlType,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>> {
        match arg.borrow() {
            Boolean(b) if b == &TARGET => ArgCheckControlFlow::ShortCircuit(Value::Boolean(*b)),
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
        type AndCheck = BoolShortCircuitArgChecker<false, PropagateNull<false>>;
        type OrCheck = BoolShortCircuitArgChecker<true, PropagateNull<false>>;
        type InCheck<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateNull<false>>;
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        type EqCheck<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<false>>;
        type MathCheck<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;

        macro_rules! create {
            ($check: ty, $types: expr, $f:expr) => {
                BinaryValueExpr::create_checked::<{ STRICT }, $check, _>($types, args, $f)
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
                let nums = PartiqlType::any_of(TYPE_NUMERIC_TYPES);
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

/// Represents an evaluation PartiQL `BETWEEN` operator, e.g. `x BETWEEN 10 AND 20`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalBetweenExpr {}

impl BindEvalExpr for EvalBetweenExpr {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let types = [TYPE_ANY, TYPE_ANY, TYPE_ANY];
        TernaryValueExpr::create_checked::<{ STRICT }, NullArgChecker, _>(
            types,
            args,
            |value, from, to| value.gteq(from).and(&value.lteq(to)),
        )
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
        UnaryValueExpr::create_with_any::<{ STRICT }, _>(args, |v| {
            Value::from(match v {
                Value::Bag(b) => !b.is_empty(),
                Value::List(l) => !l.is_empty(),
                Value::Tuple(t) => !t.is_empty(),
                _ => false,
            })
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
        let nums = PartiqlType::any_of(TYPE_NUMERIC_TYPES);
        UnaryValueExpr::create_typed::<{ STRICT }, _>([nums], args, |v| {
            match NullableOrd::lt(v, &Value::from(0)) {
                Null => Null,
                Missing => Missing,
                Value::Boolean(true) => -v,
                _ => v.clone(),
            }
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

        UnaryValueExpr::create_typed::<{ STRICT }, _>([collections], args, |v| match v {
            Value::List(l) => Value::from(l.len()),
            Value::Bag(b) => Value::from(b.len()),
            Value::Tuple(t) => Value::from(t.len()),
            _ => Missing,
        })
    }
}
