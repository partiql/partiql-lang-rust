//! Ion literals (`` `...` ``), held as variants: the literal's Ion text, decoded to the VM
//! value it denotes when an operator reads it.

use crate::arena::Arena;
use crate::error::{EngineError, Result};
use crate::value::{value_ref_to_owned, TupleField, TupleRef, ValueOwned, ValueRef};
use ion_rs::{Element, IonType};
use rust_decimal::Decimal;

/// The type name of an Ion variant.
pub(crate) const ION: &str = "Ion";

/// Check that `bytes` hold one Ion value.
pub(crate) fn validate(bytes: &[u8]) -> Result<()> {
    Element::read_one(bytes)
        .map(|_| ())
        .map_err(|e| EngineError::InvalidIon(e.to_string()))
}

/// `value`, with a variant decoded into `arena` as the VM value it holds. The value's own
/// Ion annotations are dropped (`a::1` is 1), as in the legacy evaluator. A variant the VM
/// has no type for (a timestamp) stays as it is; so does one nested in a decoded collection
/// (or an annotated one), as a variant of its own.
pub(crate) fn lower<'a>(value: ValueRef<'a>, arena: &'a Arena) -> Result<ValueRef<'a>> {
    let ValueRef::Variant(bytes, _) = value else {
        return Ok(value);
    };
    let element = Element::read_one(bytes).map_err(|e| EngineError::InvalidIon(e.to_string()))?;
    Ok(decode(&element, arena).unwrap_or(value))
}

/// The canonical text of the Ion value in `bytes`, without its annotations: the same for
/// every way of writing one value (` 2000-01-01T00:00+00:00` and `2000-01-01T00:00Z`).
pub(crate) fn canonical<'a>(bytes: &[u8], arena: &'a Arena) -> Result<&'a [u8]> {
    let element = Element::read_one(bytes).map_err(|e| EngineError::InvalidIon(e.to_string()))?;
    let bare = Element::from(element.value().clone());
    Ok(arena.alloc_slice(bare.to_string().as_bytes()))
}

/// `element` as a VM value, or `None` when the VM has no type for it.
fn decode<'a>(element: &Element, arena: &'a Arena) -> Option<ValueRef<'a>> {
    if element.is_null() {
        return Some(ValueRef::Null);
    }
    Some(match element.ion_type() {
        IonType::Null => ValueRef::Null,
        IonType::Bool => ValueRef::Bool(element.as_bool()?),
        IonType::Int => {
            let int = element.as_int()?;
            match int.as_i64() {
                Some(n) => ValueRef::I64(n),
                None => {
                    ValueRef::Decimal(Decimal::try_from_i128_with_scale(int.as_i128()?, 0).ok()?)
                }
            }
        }
        IonType::Float => ValueRef::F64(element.as_float()?),
        IonType::Decimal => ValueRef::Decimal(decimal(&element.as_decimal()?)?),
        IonType::String | IonType::Symbol => ValueRef::Str(alloc_str(arena, element.as_text()?)),
        IonType::Blob | IonType::Clob => ValueRef::Bytes(arena.alloc_slice(element.as_lob()?)),
        IonType::List | IonType::SExp => {
            let items: Vec<ValueRef<'a>> = element
                .as_sequence()?
                .elements()
                .map(|child| nested(child, arena))
                .collect();
            ValueRef::List(arena.alloc_slice(&items))
        }
        IonType::Struct => {
            let mut fields = Vec::new();
            for (name, child) in element.as_struct()?.fields() {
                fields.push(TupleField {
                    name: alloc_str(arena, name.text()?),
                    value: nested(child, arena),
                });
            }
            ValueRef::Tuple(arena.alloc_tuple_ref(TupleRef {
                fields: arena.alloc_slice(&fields),
            }))
        }
        IonType::Timestamp => return None,
    })
}

/// An Ion literal's value, decoded when the VM has a type for it.
pub(crate) fn decode_owned(bytes: &[u8]) -> Result<ValueOwned> {
    let arena = Arena::default();
    Ok(match lower(ValueRef::Variant(bytes, ION), &arena)? {
        ValueRef::Variant(..) => ValueOwned::Variant(bytes.to_vec(), ION.to_string()),
        value => value_ref_to_owned(value),
    })
}

/// A collection's element: decoded, or else a variant of its own. An annotated element stays
/// a variant, so a result that holds it keeps its annotations; an operator reading it drops
/// them when it decodes it in turn.
fn nested<'a>(element: &Element, arena: &'a Arena) -> ValueRef<'a> {
    let decoded = match element.annotations().is_empty() {
        true => decode(element, arena),
        false => None,
    };
    decoded.unwrap_or_else(|| {
        ValueRef::Variant(arena.alloc_slice(element.to_string().as_bytes()), ION)
    })
}

/// An Ion decimal, when it fits a `Decimal` (28 digits of scale).
fn decimal(d: &ion_rs::Decimal) -> Option<Decimal> {
    // Negative zero has no integer coefficient; its sign is set below
    let coefficient = match d.coefficient().is_negative_zero() {
        true => 0,
        false => ion_rs::Int::try_from(d.coefficient()).ok()?.as_i128()?,
    };
    let exponent = d.exponent();
    let mut value = if exponent <= 0 {
        Decimal::try_from_i128_with_scale(coefficient, u32::try_from(-exponent).ok()?).ok()?
    } else {
        let scaled = 10i128
            .checked_pow(u32::try_from(exponent).ok()?)
            .and_then(|p| coefficient.checked_mul(p))?;
        Decimal::try_from_i128_with_scale(scaled, 0).ok()?
    };
    if d.coefficient().is_negative_zero() {
        value.set_sign_negative(true);
    }
    Some(value)
}

fn alloc_str<'a>(arena: &'a Arena, s: &str) -> &'a str {
    let bytes = arena.alloc_slice(s.as_bytes());
    // Safety: copied from a `str`
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lowered(text: &str) -> String {
        let arena = Arena::default();
        let value = lower(ValueRef::Variant(text.as_bytes(), ION), &arena).expect("valid Ion");
        format!("{value:?}")
    }

    #[test]
    fn scalars_decode_to_vm_values() {
        assert_eq!(lowered("1"), "I64(1)");
        assert_eq!(lowered("1.50"), "Decimal(1.50)");
        assert_eq!(lowered("15d1"), "Decimal(150)");
        assert_eq!(lowered("-0.0"), "Decimal(-0.0)");
        assert_eq!(lowered("1e0"), "F64(1.0)");
        assert_eq!(lowered("\"a\""), "Str(\"a\")");
        assert_eq!(lowered("abc"), "Str(\"abc\")");
        assert_eq!(lowered(" null.int "), "Null");
        assert_eq!(lowered("true"), "Bool(true)");
        // Past i64, an int is a decimal
        assert_eq!(
            lowered("9223372036854775808"),
            "Decimal(9223372036854775808)"
        );
    }

    #[test]
    fn what_the_vm_cannot_hold_stays_a_variant() {
        assert!(lowered("2000-01-02T03:04:05Z").starts_with("Variant("));
        // Past a decimal's 96 bits
        assert!(lowered("170141183460469231731687303715884105727").starts_with("Variant("));
        // A nested timestamp is a variant of its own in a decoded list
        let list = lowered("[1, 2000T]");
        assert!(list.starts_with("List([I64(1), Variant("), "{list}");
    }

    #[test]
    fn annotations_are_dropped() {
        assert_eq!(lowered("a::1"), "I64(1)");
        assert_eq!(lowered("a::b::null.int"), "Null");
        // A nested annotated value stays a variant, annotations and all, until read
        let nested = lowered("[a::1, {b: c::\"x\", d: 2}]");
        assert!(nested.starts_with("List([Variant("), "{nested}");
        assert!(nested.contains("value: I64(2)"), "{nested}");
        let arena = Arena::default();
        let child = lower(ValueRef::Variant(b"a::1", ION), &arena).expect("valid Ion");
        assert_eq!(format!("{child:?}"), "I64(1)");
    }

    #[test]
    fn invalid_ion_is_an_error() {
        assert!(validate(b"{{ bad").is_err());
        assert!(validate(b"{a: 1}").is_ok());
    }
}
