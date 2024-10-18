use crate::Value;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub fn coerce_f64_to_decimal(real_value: &f64) -> Option<Decimal> {
    if !real_value.is_finite() {
        None
    } else {
        Decimal::from_f64(*real_value)
    }
}

pub fn coerce_int_or_real_to_decimal(value: &Value) -> Value {
    match value {
        Value::Integer(int_value) => Value::from(rust_decimal::Decimal::from(*int_value)),
        Value::Real(real_value) => {
            if !real_value.is_finite() {
                Value::Missing
            } else {
                match Decimal::from_f64(real_value.0) {
                    Some(d_from_r) => Value::from(d_from_r),
                    None => Value::Missing, // TODO: decide on behavior when float cannot be coerced to Decimal
                }
            }
        }
        _ => todo!("Unsupported coercion to Decimal"),
    }
}

pub fn coerce_int_to_real(value: &Value) -> Value {
    match value {
        Value::Integer(int_value) => Value::Real(OrderedFloat(*int_value as f64)),
        _ => todo!("Unsupported coercion to Real"),
    }
}
