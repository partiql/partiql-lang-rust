use crate::aggregate;
use crate::arena::Arena;
use crate::error::{EngineError, Result};
use crate::expr::{AggFunc, UdfRegistry};
use crate::value::ValueRef;

pub(crate) struct BuiltinFunctions {
    /// Strict mode: an argument a function can't take is an error, rather than MISSING.
    strict: bool,
}

impl BuiltinFunctions {
    pub fn new(strict: bool) -> Self {
        BuiltinFunctions { strict }
    }
}

impl UdfRegistry for BuiltinFunctions {
    fn call<'a>(
        &self,
        name: &str,
        args: &[ValueRef<'a>],
        arena: &'a Arena,
    ) -> Result<ValueRef<'a>> {
        match name {
            "CharLength" => builtin_char_length(args),
            "Lower" => builtin_lower(args, arena),
            "Upper" => builtin_upper(args, arena),
            "OctetLength" => builtin_octet_length(args),
            "BitLength" => builtin_bit_length(args),
            "Substring" => builtin_substring(args, arena),
            "Position" => builtin_position(args),
            "Overlay" => builtin_overlay(args, arena),
            "Abs" => builtin_abs(args),
            "Mod" => builtin_mod(args),
            "Cardinality" => builtin_cardinality(args),
            "LTrim" => builtin_ltrim(args),
            "BTrim" => builtin_btrim(args),
            "RTrim" => builtin_rtrim(args),
            "Exists" => builtin_exists(args),
            _ if name.starts_with("Extract") => match DateTimeField::named(name) {
                Some(field) => self.extract(field, args.first().copied()),
                None => Err(EngineError::UdfNotFound(name.to_string())),
            },
            _ => match coll_function(name) {
                Some((func, distinct)) => {
                    let collection = args.first().copied().unwrap_or(ValueRef::Missing);
                    aggregate::over_collection(func, distinct, collection, self.strict, arena)
                }
                None => Err(EngineError::UdfNotFound(name.to_string())),
            },
        }
    }
}

fn builtin_char_length<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::Str(s)) => Ok(ValueRef::I64(s.chars().count() as i64)),
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_lower<'a>(args: &[ValueRef<'a>], arena: &'a Arena) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::Str(s)) => {
            let lowered = s.to_lowercase();
            let bytes = arena.alloc_slice(lowered.as_bytes());
            let s = unsafe { std::str::from_utf8_unchecked(bytes) };
            Ok(ValueRef::Str(s))
        }
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_upper<'a>(args: &[ValueRef<'a>], arena: &'a Arena) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::Str(s)) => {
            let uppered = s.to_uppercase();
            let bytes = arena.alloc_slice(uppered.as_bytes());
            let s = unsafe { std::str::from_utf8_unchecked(bytes) };
            Ok(ValueRef::Str(s))
        }
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_octet_length<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::Str(s)) => Ok(ValueRef::I64(s.len() as i64)),
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_bit_length<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::Str(s)) => Ok(ValueRef::I64((s.len() * 8) as i64)),
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_substring<'a>(args: &[ValueRef<'a>], arena: &'a Arena) -> Result<ValueRef<'a>> {
    match args.len() {
        2 => match (&args[0], &args[1]) {
            (ValueRef::Str(s), ValueRef::I64(offset)) => {
                let offset = (std::cmp::max(*offset, 1) - 1) as usize;
                let substring: String = s.chars().skip(offset).collect();
                let bytes = arena.alloc_slice(substring.as_bytes());
                let result = unsafe { std::str::from_utf8_unchecked(bytes) };
                Ok(ValueRef::Str(result))
            }
            (ValueRef::Null, _) | (_, ValueRef::Null) => Ok(ValueRef::Null),
            (ValueRef::Missing, _) | (_, ValueRef::Missing) => Ok(ValueRef::Missing),
            _ => Ok(ValueRef::Missing),
        },
        3 => match (&args[0], &args[1], &args[2]) {
            (ValueRef::Str(s), ValueRef::I64(offset), ValueRef::I64(length)) => {
                let (skip, take) = if *length < 1 {
                    (0, 0)
                } else if *offset < 1 {
                    let take = std::cmp::max(offset + (length - 1), 0) as usize;
                    (0, take)
                } else {
                    ((*offset - 1) as usize, *length as usize)
                };
                let substring: String = s.chars().skip(skip).take(take).collect();
                let bytes = arena.alloc_slice(substring.as_bytes());
                let result = unsafe { std::str::from_utf8_unchecked(bytes) };
                Ok(ValueRef::Str(result))
            }
            _ if args.iter().any(|a| matches!(a, ValueRef::Null)) => Ok(ValueRef::Null),
            _ if args.iter().any(|a| matches!(a, ValueRef::Missing)) => Ok(ValueRef::Missing),
            _ => Ok(ValueRef::Missing),
        },
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_position<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match (args.first(), args.get(1)) {
        (Some(ValueRef::Str(needle)), Some(ValueRef::Str(haystack))) => {
            let pos = haystack.find(needle).map_or(0, |l| l + 1) as i64;
            Ok(ValueRef::I64(pos))
        }
        (Some(ValueRef::Null), _) | (_, Some(ValueRef::Null)) => Ok(ValueRef::Null),
        (Some(ValueRef::Missing), _) | (_, Some(ValueRef::Missing)) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_overlay<'a>(args: &[ValueRef<'a>], arena: &'a Arena) -> Result<ValueRef<'a>> {
    fn do_overlay(value: &str, replacement: &str, offset: i64, length: usize) -> String {
        let mut result = value.to_string();
        let start = std::cmp::max(offset - 1, 0) as usize;
        if start > result.len() {
            result += replacement;
        } else {
            let end = std::cmp::min(start + length, result.len());
            result.replace_range(start..end, replacement);
        }
        result
    }

    match args.len() {
        3 => match (&args[0], &args[1], &args[2]) {
            (ValueRef::Str(value), ValueRef::Str(replacement), ValueRef::I64(offset)) => {
                let length = replacement.len();
                let result = do_overlay(value, replacement, *offset, length);
                let bytes = arena.alloc_slice(result.as_bytes());
                let s = unsafe { std::str::from_utf8_unchecked(bytes) };
                Ok(ValueRef::Str(s))
            }
            _ if args.iter().any(|a| matches!(a, ValueRef::Null)) => Ok(ValueRef::Null),
            _ if args.iter().any(|a| matches!(a, ValueRef::Missing)) => Ok(ValueRef::Missing),
            _ => Ok(ValueRef::Missing),
        },
        4 => match (&args[0], &args[1], &args[2], &args[3]) {
            (
                ValueRef::Str(value),
                ValueRef::Str(replacement),
                ValueRef::I64(offset),
                ValueRef::I64(length),
            ) => {
                let length = std::cmp::max(*length, 0) as usize;
                let result = do_overlay(value, replacement, *offset, length);
                let bytes = arena.alloc_slice(result.as_bytes());
                let s = unsafe { std::str::from_utf8_unchecked(bytes) };
                Ok(ValueRef::Str(s))
            }
            _ if args.iter().any(|a| matches!(a, ValueRef::Null)) => Ok(ValueRef::Null),
            _ if args.iter().any(|a| matches!(a, ValueRef::Missing)) => Ok(ValueRef::Missing),
            _ => Ok(ValueRef::Missing),
        },
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_abs<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::I64(n)) => n
            .checked_abs()
            .map(ValueRef::I64)
            .ok_or_else(|| EngineError::Arithmetic("overflow".to_string())),
        Some(ValueRef::F64(n)) => Ok(ValueRef::F64(n.abs())),
        Some(ValueRef::Decimal(d)) => {
            let abs_val = if d.is_sign_negative() { -d } else { *d };
            Ok(ValueRef::Decimal(abs_val))
        }
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_mod<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match (args.first(), args.get(1)) {
        (Some(ValueRef::I64(a)), Some(ValueRef::I64(b))) => {
            if *b == 0 {
                Ok(ValueRef::Missing)
            } else {
                a.checked_rem(*b)
                    .map(ValueRef::I64)
                    .ok_or_else(|| EngineError::Arithmetic("overflow".to_string()))
            }
        }
        (Some(ValueRef::F64(a)), Some(ValueRef::F64(b))) => Ok(ValueRef::F64(a % b)),
        (Some(ValueRef::Null), _) | (_, Some(ValueRef::Null)) => Ok(ValueRef::Null),
        (Some(ValueRef::Missing), _) | (_, Some(ValueRef::Missing)) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_cardinality<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::List(items)) => Ok(ValueRef::I64(items.len() as i64)),
        Some(ValueRef::Bag(items)) => Ok(ValueRef::I64(items.len() as i64)),
        Some(ValueRef::Tuple(t)) => Ok(ValueRef::I64(t.fields.len() as i64)),
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_ltrim<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match (args.first(), args.get(1)) {
        (Some(ValueRef::Str(trim_chars)), Some(ValueRef::Str(source))) => {
            let chars: Vec<char> = trim_chars.chars().collect();
            Ok(ValueRef::Str(source.trim_start_matches(&chars[..])))
        }
        (Some(ValueRef::Null), _) | (_, Some(ValueRef::Null)) => Ok(ValueRef::Null),
        (Some(ValueRef::Missing), _) | (_, Some(ValueRef::Missing)) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_btrim<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match (args.first(), args.get(1)) {
        (Some(ValueRef::Str(trim_chars)), Some(ValueRef::Str(source))) => {
            let chars: Vec<char> = trim_chars.chars().collect();
            Ok(ValueRef::Str(source.trim_matches(&chars[..])))
        }
        (Some(ValueRef::Null), _) | (_, Some(ValueRef::Null)) => Ok(ValueRef::Null),
        (Some(ValueRef::Missing), _) | (_, Some(ValueRef::Missing)) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_rtrim<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match (args.first(), args.get(1)) {
        (Some(ValueRef::Str(trim_chars)), Some(ValueRef::Str(source))) => {
            let chars: Vec<char> = trim_chars.chars().collect();
            Ok(ValueRef::Str(source.trim_end_matches(&chars[..])))
        }
        (Some(ValueRef::Null), _) | (_, Some(ValueRef::Null)) => Ok(ValueRef::Null),
        (Some(ValueRef::Missing), _) | (_, Some(ValueRef::Missing)) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Missing),
    }
}

fn builtin_exists<'a>(args: &[ValueRef<'a>]) -> Result<ValueRef<'a>> {
    match args.first() {
        Some(ValueRef::Tuple(t)) => Ok(ValueRef::Bool(!t.fields.is_empty())),
        Some(ValueRef::List(items)) => Ok(ValueRef::Bool(!items.is_empty())),
        Some(ValueRef::Bag(items)) => Ok(ValueRef::Bool(!items.is_empty())),
        Some(ValueRef::Null) => Ok(ValueRef::Null),
        Some(ValueRef::Missing) => Ok(ValueRef::Missing),
        _ => Ok(ValueRef::Bool(false)),
    }
}

/// A field `EXTRACT` reads of a datetime.
#[derive(Clone, Copy, Debug)]
enum DateTimeField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    TimezoneHour,
    TimezoneMinute,
}

impl DateTimeField {
    /// The field of an `EXTRACT` function, by the name the planner calls it.
    fn named(name: &str) -> Option<DateTimeField> {
        Some(match name {
            "ExtractYear" => DateTimeField::Year,
            "ExtractMonth" => DateTimeField::Month,
            "ExtractDay" => DateTimeField::Day,
            "ExtractHour" => DateTimeField::Hour,
            "ExtractMinute" => DateTimeField::Minute,
            "ExtractSecond" => DateTimeField::Second,
            "ExtractTimezoneHour" => DateTimeField::TimezoneHour,
            "ExtractTimezoneMinute" => DateTimeField::TimezoneMinute,
            _ => return None,
        })
    }
}

impl BuiltinFunctions {
    /// `EXTRACT(field FROM value)`, as in the legacy evaluator: NULL for NULL, MISSING for
    /// MISSING, and MISSING for a field the value has none of (the hour of a date, the time
    /// zone of a timestamp without one). A value that is no datetime is MISSING too, or an
    /// error in strict mode. Fields are local, as written; SECOND is a decimal with the
    /// fractional digits the value was written with.
    fn extract<'a>(
        &self,
        field: DateTimeField,
        value: Option<ValueRef<'a>>,
    ) -> Result<ValueRef<'a>> {
        use DateTimeField as F;
        let int = |n: i64| ValueRef::I64(n);
        let (date, time) = match value.unwrap_or(ValueRef::Missing) {
            ValueRef::Null => return Ok(ValueRef::Null),
            ValueRef::Missing => return Ok(ValueRef::Missing),
            ValueRef::Date(date) => (Some(date), None),
            ValueRef::Time(time) => (None, Some(time)),
            ValueRef::Timestamp(ts) => {
                let (date, time) = ts.local();
                (Some(date), Some(time))
            }
            other if self.strict => {
                return Err(EngineError::TypeError(format!(
                    "EXTRACT({field:?}) of a value that is no datetime: {}",
                    crate::compare::type_name(&other)
                )))
            }
            _ => return Ok(ValueRef::Missing),
        };
        let ymd = date.map(|d| d.ymd());
        let hms = time.map(|t| t.hms_micro());
        let offset = time.and_then(|t| t.offset().get()).map(i64::from);
        Ok(match field {
            F::Year => ymd.map_or(ValueRef::Missing, |(y, _, _)| int(i64::from(y))),
            F::Month => ymd.map_or(ValueRef::Missing, |(_, m, _)| int(i64::from(m))),
            F::Day => ymd.map_or(ValueRef::Missing, |(_, _, d)| int(i64::from(d))),
            F::Hour => hms.map_or(ValueRef::Missing, |(h, _, _, _)| int(i64::from(h))),
            F::Minute => hms.map_or(ValueRef::Missing, |(_, m, _, _)| int(i64::from(m))),
            F::Second => match (hms, time) {
                (Some((_, _, s, micros)), Some(time)) => {
                    ValueRef::Decimal(seconds(s, micros, time.precision()))
                }
                _ => ValueRef::Missing,
            },
            // An offset's hours and minutes both have its sign (-08:09 is -8 and -9)
            F::TimezoneHour => offset.map_or(ValueRef::Missing, |m| int(m / 60)),
            F::TimezoneMinute => offset.map_or(ValueRef::Missing, |m| int(m % 60)),
        })
    }
}

/// `second.micros` as a decimal with `precision` fractional digits, or 6 where the
/// microseconds need them (a value rounded from more digits than it was written with).
fn seconds(second: u8, micros: u32, precision: u8) -> rust_decimal::Decimal {
    let mut seconds =
        rust_decimal::Decimal::new(i64::from(second) * 1_000_000 + i64::from(micros), 6);
    let unit = 10u32.pow(u32::from(6 - precision.min(6)));
    if micros / unit * unit == micros {
        seconds.rescale(u32::from(precision));
    }
    seconds
}

/// The aggregate a `COLL_*` function name (as the planner calls it) computes, and whether
/// over distinct values.
fn coll_function(name: &str) -> Option<(AggFunc, bool)> {
    let (func, quantifier) = name.strip_suffix(')')?.split_once('(')?;
    let func = match func {
        "CollCount" => AggFunc::Count,
        "CollSum" => AggFunc::Sum,
        "CollAvg" => AggFunc::Avg,
        "CollMin" => AggFunc::Min,
        "CollMax" => AggFunc::Max,
        "CollAny" => AggFunc::Any,
        "CollEvery" => AggFunc::Every,
        _ => return None,
    };
    match quantifier {
        "All" => Some((func, false)),
        "Distinct" => Some((func, true)),
        _ => None,
    }
}
