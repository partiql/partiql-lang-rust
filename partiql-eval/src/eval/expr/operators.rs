use crate::eval::eval_expr_wrapper::{
    ArgCheckControlFlow, ArgChecker, ArgShortCircuit, ArgValidateError, BinaryValueExpr,
    DefaultArgChecker, ExecuteEvalExpr, NullArgChecker, PropagateMissing, PropagateNull,
    TernaryValueExpr, UnaryValueExpr,
};

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;

use partiql_types::{
    type_bool, type_dynamic, type_numeric, PartiqlNoIdShapeBuilder, PartiqlShape,
    ShapeBuilderExtensions,
};
use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{BinaryAnd, Comparable, EqualityValue, NullableEq, NullableOrd, Tuple, Value};

use std::borrow::{Borrow, Cow};
use std::fmt::{Debug, Formatter};

use std::marker::PhantomData;

use partiql_value::datum::{DatumCategory, DatumCategoryRef, SequenceDatum, TupleDatum};
use std::ops::ControlFlow;

/// Represents a literal in (sub)query, e.g. `1` in `a + 1`.
#[derive(Clone)]
pub(crate) struct EvalLitExpr {
    pub(crate) val: Value,
}

impl EvalLitExpr {
    pub(crate) fn new(val: Value) -> Self {
        Self { val }
    }
}

impl Debug for EvalLitExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.val.fmt(f)
    }
}

impl BindEvalExpr for EvalLitExpr {
    fn bind<const STRICT: bool>(
        self,
        _args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        Ok(Box::new(self))
    }
}

impl EvalExpr for EvalLitExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        _bindings: &'a Tuple,
        _ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        Cow::Borrowed(&self.val)
    }
}

impl ExecuteEvalExpr<0> for Value {
    fn evaluate<'a, 'c>(
        &'a self,
        _args: [Cow<'a, Value>; 0],
        _ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
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
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();
        let any_num = type_numeric!(&mut bld);

        let unop = |types, f: fn(&Value) -> Value| {
            UnaryValueExpr::create_typed::<{ STRICT }, _>(types, args, f)
        };

        match self {
            EvalOpUnary::Pos => unop([any_num], std::clone::Clone::clone),
            EvalOpUnary::Neg => unop([any_num], |operand| -operand),
            EvalOpUnary::Not => unop([type_bool!(bld)], |operand| !operand),
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
        _typ: &PartiqlShape,
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

#[derive(Debug)]
pub(crate) struct ComparisonArgChecker<const STRICT: bool, OnMissing: ArgShortCircuit> {
    check: PhantomData<DefaultArgChecker<STRICT, OnMissing>>,
}

impl<const STRICT: bool, OnMissing: ArgShortCircuit> ArgChecker
    for ComparisonArgChecker<STRICT, OnMissing>
{
    #[inline]
    fn arg_check<'a>(
        typ: &PartiqlShape,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>> {
        DefaultArgChecker::<{ STRICT }, OnMissing>::arg_check(typ, arg)
    }

    fn validate_args(args: Vec<Cow<'_, Value>>) -> Result<Vec<Cow<'_, Value>>, ArgValidateError> {
        if args.len() == 2 && args[0].is_comparable_to(&args[1]) {
            Ok(args)
        } else {
            Err(ArgValidateError {
                message: "data-type mismatch".to_string(),
                propagate: OnMissing::propagate(),
            })
        }
    }
}

impl BindEvalExpr for EvalOpBinary {
    #[inline]
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        type AndCheck = BoolShortCircuitArgChecker<false, PropagateNull<false>>;
        type OrCheck = BoolShortCircuitArgChecker<true, PropagateNull<false>>;
        type InCheck<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateNull<false>>;
        type Check<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;
        type EqCheck<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<false>>;
        type CompCheck<const STRICT: bool> = ComparisonArgChecker<STRICT, PropagateMissing<true>>;
        type MathCheck<const STRICT: bool> = DefaultArgChecker<STRICT, PropagateMissing<true>>;

        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();

        macro_rules! create {
            ($check: ty, $types: expr, $f:expr) => {
                BinaryValueExpr::create_checked::<{ STRICT }, $check, _>($types, args, $f)
            };
        }

        macro_rules! logical {
            ($check: ty, $f:expr) => {
                create!($check, [type_bool!(bld), type_bool!(bld)], $f)
            };
        }

        macro_rules! equality {
            ($f:expr) => {
                create!(
                    EqCheck<STRICT>,
                    [PartiqlShape::Dynamic, PartiqlShape::Dynamic],
                    $f
                )
            };
        }

        macro_rules! comparison {
            ($f:expr) => {
                create!(
                    CompCheck<STRICT>,
                    [PartiqlShape::Dynamic, PartiqlShape::Dynamic],
                    $f
                )
            };
        }

        macro_rules! math {
            ($f:expr) => {{
                let nums = type_numeric!(&mut bld);
                create!(MathCheck<STRICT>, [nums.clone(), nums], $f)
            }};
        }

        match self {
            EvalOpBinary::And => logical!(AndCheck, partiql_value::BinaryAnd::and),
            EvalOpBinary::Or => logical!(OrCheck, partiql_value::BinaryOr::or),
            EvalOpBinary::Eq => equality!(|lhs, rhs| {
                let wrap = EqualityValue::<false, false, Value>;
                NullableEq::eq(&wrap(lhs), &wrap(rhs))
            }),
            EvalOpBinary::Neq => equality!(|lhs, rhs| {
                let wrap = EqualityValue::<false, false, Value>;
                NullableEq::neq(&wrap(lhs), &wrap(rhs))
            }),
            EvalOpBinary::Gt => comparison!(NullableOrd::gt),
            EvalOpBinary::Gteq => comparison!(NullableOrd::gteq),
            EvalOpBinary::Lt => comparison!(NullableOrd::lt),
            EvalOpBinary::Lteq => comparison!(NullableOrd::lteq),
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
                        type_dynamic!(bld),
                        [bld.new_array_of_dyn(), bld.new_bag_of_dyn()].into_any_of(&mut bld)
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
                create!(
                    Check<STRICT>,
                    [PartiqlShape::Dynamic, PartiqlShape::Dynamic],
                    |lhs, rhs| {
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
                    }
                )
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct BetweenArgChecker<const STRICT: bool> {
    check: PhantomData<NullArgChecker>,
}

impl<const STRICT: bool> ArgChecker for BetweenArgChecker<STRICT> {
    #[inline]
    fn arg_check<'a>(
        typ: &PartiqlShape,
        arg: Cow<'a, Value>,
    ) -> ArgCheckControlFlow<Value, Cow<'a, Value>> {
        NullArgChecker::arg_check(typ, arg)
    }

    fn validate_args(args: Vec<Cow<'_, Value>>) -> Result<Vec<Cow<'_, Value>>, ArgValidateError> {
        if args.len() == 3
            && args[0].is_comparable_to(&args[1])
            && args[0].is_comparable_to(&args[2])
        {
            Ok(args)
        } else {
            Err(ArgValidateError {
                message: "data-type mismatch".to_string(),
                propagate: Value::Missing,
            })
        }
    }
}

/// Represents an evaluation `PartiQL` `BETWEEN` operator, e.g. `x BETWEEN 10 AND 20`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalBetweenExpr {}

impl BindEvalExpr for EvalBetweenExpr {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let types = [
            PartiqlShape::Dynamic,
            PartiqlShape::Dynamic,
            PartiqlShape::Dynamic,
        ];
        TernaryValueExpr::create_checked::<{ STRICT }, BetweenArgChecker<{ STRICT }>, _>(
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
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        UnaryValueExpr::create_with_any::<{ STRICT }, _>(args, |v| {
            Value::from(match v.category() {
                DatumCategoryRef::Tuple(tuple) => !tuple.is_empty(),
                DatumCategoryRef::Sequence(seq) => !seq.is_empty(),
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
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();
        let nums = type_numeric!(&mut bld);
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
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();
        let collections = [
            bld.new_array_of_dyn(),
            bld.new_bag_of_dyn(),
            bld.new_struct_of_dyn(),
        ]
        .into_any_of(&mut bld);

        UnaryValueExpr::create_typed::<{ STRICT }, _>([collections], args, |v| match v.category() {
            DatumCategoryRef::Tuple(tuple) => Value::from(tuple.len()),
            DatumCategoryRef::Sequence(seq) => Value::from(seq.len()),
            _ => Missing,
        })
    }
}
