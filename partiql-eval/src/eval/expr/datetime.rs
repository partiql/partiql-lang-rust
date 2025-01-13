use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};

use partiql_types::{type_datetime, PartiqlNoIdShapeBuilder};
use partiql_value::Value::Missing;
use partiql_value::{DateTime, Value};

use rust_decimal::Decimal;
use std::fmt::Debug;

use crate::eval::eval_expr_wrapper::UnaryValueExpr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EvalExtractFn {
    /// Represents a year `EXTRACT` function, e.g. `extract(YEAR FROM t)`.
    Year,
    /// Represents a month `EXTRACT` function, e.g. `extract(MONTH FROM t)`.
    Month,
    /// Represents a day `EXTRACT` function, e.g. `extract(DAY FROM t)`.
    Day,
    /// Represents an hour `EXTRACT` function, e.g. `extract(HOUR FROM t)`.
    Hour,
    /// Represents a minute `EXTRACT` function, e.g. `extract(MINUTE FROM t)`.
    Minute,
    /// Represents a second `EXTRACT` function, e.g. `extract(SECOND FROM t)`.
    Second,
    /// Represents a timezone hour `EXTRACT` function, e.g. `extract(TIMEZONE_HOUR FROM t)`.
    TzHour,
    /// Represents a timezone minute `EXTRACT` function, e.g. `extract(TIMEZONE_MINUTE FROM t)`.
    TzMinute,
}

impl BindEvalExpr for EvalExtractFn {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        #[inline]
        fn total_seconds(second: u8, nanosecond: u32) -> Value {
            const NANOSECOND_SCALE: u32 = 9;
            let total = Duration::new(u64::from(second), nanosecond).as_nanos() as i128;
            Decimal::from_i128_with_scale(total, NANOSECOND_SCALE).into()
        }
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();

        let create = |f: fn(&DateTime) -> Value| {
            UnaryValueExpr::create_typed::<{ STRICT }, _>(
                [type_datetime!(bld)],
                args,
                move |value| match value {
                    Value::DateTime(dt) => f(dt.as_ref()),
                    _ => Missing,
                },
            )
        };

        match self {
            EvalExtractFn::Year => create(|dt: &DateTime| match dt {
                DateTime::Date(d) => Value::from(d.year()),
                DateTime::Timestamp(ts) => Value::from(ts.year()),
                DateTime::TimestampWithTz(ts) => Value::from(ts.year()),
                _ => Missing,
            }),
            EvalExtractFn::Month => create(|dt: &DateTime| match dt {
                DateTime::Date(d) => Value::from(d.month() as u8),
                DateTime::Timestamp(ts) => Value::from(ts.month() as u8),
                DateTime::TimestampWithTz(ts) => Value::from(ts.month() as u8),
                _ => Missing,
            }),
            EvalExtractFn::Day => create(|dt: &DateTime| match dt {
                DateTime::Date(d) => Value::from(d.day()),
                DateTime::Timestamp(ts) => Value::from(ts.day()),
                DateTime::TimestampWithTz(ts) => Value::from(ts.day()),
                _ => Missing,
            }),
            EvalExtractFn::Hour => create(|dt: &DateTime| match dt {
                DateTime::Time(t) => Value::from(t.hour()),
                DateTime::TimeWithTz(t, _) => Value::from(t.hour()),
                DateTime::Timestamp(ts) => Value::from(ts.hour()),
                DateTime::TimestampWithTz(ts) => Value::from(ts.hour()),
                _ => Missing,
            }),
            EvalExtractFn::Minute => create(|dt: &DateTime| match dt {
                DateTime::Time(t) => Value::from(t.minute()),
                DateTime::TimeWithTz(t, _) => Value::from(t.minute()),
                DateTime::Timestamp(ts) => Value::from(ts.minute()),
                DateTime::TimestampWithTz(ts) => Value::from(ts.minute()),
                _ => Missing,
            }),
            EvalExtractFn::Second => create(|dt: &DateTime| match dt {
                DateTime::Time(t) => total_seconds(t.second(), t.nanosecond()),
                DateTime::TimeWithTz(t, _) => total_seconds(t.second(), t.nanosecond()),
                DateTime::Timestamp(ts) => total_seconds(ts.second(), ts.nanosecond()),
                DateTime::TimestampWithTz(ts) => total_seconds(ts.second(), ts.nanosecond()),
                _ => Missing,
            }),
            EvalExtractFn::TzHour => create(|dt: &DateTime| match dt {
                DateTime::TimeWithTz(_, tz) => Value::from(tz.whole_hours()),
                DateTime::TimestampWithTz(ts) => Value::from(ts.offset().whole_hours()),

                _ => Missing,
            }),
            EvalExtractFn::TzMinute => create(|dt: &DateTime| match dt {
                DateTime::TimeWithTz(_, tz) => Value::from(tz.minutes_past_hour()),
                DateTime::TimestampWithTz(ts) => Value::from(ts.offset().minutes_past_hour()),
                _ => Missing,
            }),
        }
    }
}
