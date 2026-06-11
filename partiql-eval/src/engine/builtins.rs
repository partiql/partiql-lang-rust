use crate::engine::arena::Arena;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::UdfRegistry;
use crate::engine::value::ValueRef;

pub(crate) struct BuiltinFunctions;

impl BuiltinFunctions {
    pub fn new() -> Self {
        BuiltinFunctions
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
            _ => Err(EngineError::UdfNotFound(name.to_string())),
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
        Some(ValueRef::I64(n)) => Ok(ValueRef::I64(n.abs())),
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
                Ok(ValueRef::I64(a % b))
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
