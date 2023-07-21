use crate::eval::expr::eval_wrapper::{
    BinaryValueExpr, EvalExprWrapper, ExecuteEvalExpr, QuaternaryValueExpr, TernaryValueExpr,
    UnaryValueExpr,
};

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::EvalContext;
use itertools::Itertools;

use partiql_types::{TYPE_INT, TYPE_STRING};
use partiql_value::Value;
use partiql_value::Value::Missing;

use std::borrow::{Borrow, Cow};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EvalStringFn {
    /// Represents a built-in `lower` string function, e.g. lower('AdBd').
    Lower,
    /// Represents a built-in `upper` string function, e.g. upper('AdBd').
    Upper,
    /// Represents a built-in character length string function, e.g. `char_length('123456789')`.
    CharLength,
    /// Represents a built-in octet length string function, e.g. `octet_length('123456789')`.
    OctetLength,
    /// Represents a built-in bit length string function, e.g. `bit_length('123456789')`.
    BitLength,
}

impl BindEvalExpr for EvalStringFn {
    #[inline]
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        #[inline]
        fn create<const STRICT: bool, F, R>(
            args: Vec<Box<dyn EvalExpr>>,
            f: F,
        ) -> Result<Box<dyn EvalExpr>, BindError>
        where
            F: Fn(&Box<String>) -> R + 'static,
            R: Into<Value> + 'static,
        {
            UnaryValueExpr::create_typed::<{ STRICT }, _>([TYPE_STRING], args, move |value| {
                match value {
                    Value::String(value) => (f(value)).into(),
                    _ => Missing,
                }
            })
        }
        match self {
            EvalStringFn::Lower => create::<{ STRICT }, _, _>(args, |s| s.to_lowercase()),
            EvalStringFn::Upper => create::<{ STRICT }, _, _>(args, |s| s.to_uppercase()),
            EvalStringFn::CharLength => create::<{ STRICT }, _, _>(args, |s| s.chars().count()),
            EvalStringFn::OctetLength => create::<{ STRICT }, _, _>(args, |s| s.len()),
            EvalStringFn::BitLength => create::<{ STRICT }, _, _>(args, |s| (s.len() * 8)),
        }
    }
}

impl<F, R> ExecuteEvalExpr<1> for EvalExprWrapper<EvalStringFn, F>
where
    F: Fn(&Box<String>) -> R,
    R: Into<Value>,
{
    #[inline]
    fn evaluate<'a>(
        &'a self,
        args: [Cow<'a, Value>; 1],
        _ctx: &'a dyn EvalContext,
    ) -> Cow<'a, Value> {
        let [value] = args;
        Cow::Owned(match value.borrow() {
            Value::String(s) => ((self.f)(s)).into(),
            _ => Missing,
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EvalTrimFn {
    /// Represents a built-in trim string function, e.g. `trim(both from ' foobar ')`.
    TrimBoth,
    /// Represents a built-in start trim string function.
    TrimStart,
    /// Represents a built-in end trim string function.
    TrimEnd,
}

impl BindEvalExpr for EvalTrimFn {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        let create = |f: for<'a> fn(&'a str, &'a str) -> &'a str| {
            BinaryValueExpr::create_typed::<{ STRICT }, _>(
                [TYPE_STRING, TYPE_STRING],
                args,
                move |to_trim, value| match (to_trim, value) {
                    (Value::String(to_trim), Value::String(value)) => {
                        Value::from(f(to_trim, value))
                    }
                    _ => Missing,
                },
            )
        };
        match self {
            EvalTrimFn::TrimBoth => create(|trim, value| {
                let to_trim = trim.chars().collect_vec();
                value.trim_matches(&to_trim[..])
            }),
            EvalTrimFn::TrimStart => create(|trim, value| {
                let to_trim = trim.chars().collect_vec();
                value.trim_start_matches(&to_trim[..])
            }),
            EvalTrimFn::TrimEnd => create(|trim, value| {
                let to_trim = trim.chars().collect_vec();
                value.trim_end_matches(&to_trim[..])
            }),
        }
    }
}

/// Represents a built-in position string function, e.g. `position('3' IN '123456789')`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalFnPosition {}

impl BindEvalExpr for EvalFnPosition {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        BinaryValueExpr::create_typed::<STRICT, _>(
            [TYPE_STRING, TYPE_STRING],
            args,
            |needle, haystack| match (needle, haystack) {
                (Value::String(needle), Value::String(haystack)) => haystack
                    .find(needle.as_ref())
                    .map(|l| l + 1)
                    .unwrap_or(0)
                    .into(),
                _ => Missing,
            },
        )
    }
}

/// Represents a built-in substring string function, e.g. `substring('123456789' FROM 2)`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalFnSubstring {}

impl BindEvalExpr for EvalFnSubstring {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        match args.len() {
            2 => BinaryValueExpr::create_typed::<STRICT, _>(
                [TYPE_STRING, TYPE_INT],
                args,
                |value, offset| match (value, offset) {
                    (Value::String(value), Value::Integer(offset)) => {
                        let offset = (std::cmp::max(offset, &1) - 1) as usize;
                        let substring = value.chars().skip(offset).collect::<String>();
                        Value::from(substring)
                    }
                    _ => Missing,
                },
            ),
            3 => TernaryValueExpr::create_typed::<STRICT, _>(
                [TYPE_STRING, TYPE_INT, TYPE_INT],
                args,
                |value, offset, length| match (value, offset, length) {
                    (Value::String(value), Value::Integer(offset), Value::Integer(length)) => {
                        let (offset, length) = if *length < 1 {
                            (0, 0)
                        } else if *offset < 1 {
                            let length = std::cmp::max(offset + (length - 1), 0) as usize;
                            let offset = std::cmp::max(*offset, 0) as usize;
                            (offset, length)
                        } else {
                            ((offset - 1) as usize, *length as usize)
                        };
                        let substring = value.chars().skip(offset).take(length).collect::<String>();
                        Value::from(substring)
                    }
                    _ => Missing,
                },
            ),
            n => Err(BindError::ArgNumMismatch {
                expected: 3,
                found: n,
            }),
        }
    }
}

/// Represents a built-in overlay string function, e.g. `OVERLAY('hello' PLACING 'XX' FROM 2 FOR 3)`.
#[derive(Debug, Default, Clone)]
pub(crate) struct EvalFnOverlay {}

impl BindEvalExpr for EvalFnOverlay {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        fn overlay(value: &str, replacement: &str, offset: i64, length: usize) -> Value {
            let mut value = value.to_string();
            let start = std::cmp::max(offset - 1, 0) as usize;
            if start > value.len() {
                value += replacement;
            } else {
                let end = std::cmp::min(start + length, value.len());
                value.replace_range(start..end, replacement);
            }

            Value::from(value)
        }

        match args.len() {
            3 => TernaryValueExpr::create_typed::<STRICT, _>(
                [TYPE_STRING, TYPE_STRING, TYPE_INT],
                args,
                |value, replacement, offset| match (value, replacement, offset) {
                    (Value::String(value), Value::String(replacement), Value::Integer(offset)) => {
                        let length = replacement.len();
                        overlay(value.as_ref(), replacement.as_ref(), *offset, length)
                    }
                    _ => Missing,
                },
            ),
            4 => QuaternaryValueExpr::create_typed::<STRICT, _>(
                [TYPE_STRING, TYPE_STRING, TYPE_INT, TYPE_INT],
                args,
                |value, replacement, offset, length| match (value, replacement, offset, length) {
                    (
                        Value::String(value),
                        Value::String(replacement),
                        Value::Integer(offset),
                        Value::Integer(length),
                    ) => {
                        let length = std::cmp::max(*length, 0) as usize;
                        overlay(value.as_ref(), replacement.as_ref(), *offset, length)
                    }
                    _ => Missing,
                },
            ),
            n => Err(BindError::ArgNumMismatch {
                expected: 4,
                found: n,
            }),
        }
    }
}
