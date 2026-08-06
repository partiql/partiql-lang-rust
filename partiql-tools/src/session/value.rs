//! Conversion of VM register rows into owned `partiql_value::Value`, and thin
//! adapter over the canonical `partiql-extension-ion` encoder for the ion
//! render path. `value_to_element` is a delegation — spec-complete
//! PartiQL-encoded-Ion (bag / missing / date / time / timestamp annotations)
//! lives in the extension, not here.

use partiql_extension_ion::encode::{IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding;
use partiql_value::{Tuple, Value};

/// A `partiql_value::Value` that has no PartiQL-encoded-Ion representation.
#[derive(Debug)]
pub(super) struct RowConvertError(pub(super) String);

impl std::fmt::Display for RowConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot encode value as Ion: {}", self.0)
    }
}

impl std::error::Error for RowConvertError {}

/// Convert an owned PartiQL value into a PartiQL-encoded-Ion `Element` by
/// running it through the canonical extension encoder into an
/// `ElementStreamWriter`, then popping the single top-level element out.
pub(super) fn value_to_element(value: &Value) -> Result<ion_rs::element::Element, RowConvertError> {
    let mut out: Vec<ion_rs::element::Element> = Vec::with_capacity(1);
    {
        let mut writer = ion_rs::element::element_stream_writer::ElementStreamWriter::new(&mut out);
        let mut encoder = IonEncoderBuilder::new(
            IonEncoderConfig::default().with_mode(Encoding::PartiqlEncodedAsIon),
        )
        .build(&mut writer)
        .map_err(|e| RowConvertError(format!("build encoder: {e:?}")))?;
        encoder
            .write_value(value)
            .map_err(|e| RowConvertError(format!("encode value: {e:?}")))?;
    }
    out.pop()
        .ok_or_else(|| RowConvertError("encoder produced no element".to_string()))
}

/// Convert one VM register row into an owned `Value`. Errors are propagated
/// rather than fabricated (no `"?"` field names, no silent `continue`, no
/// `unwrap`s over reader state).
pub(super) fn row_to_value(
    row: &partiql_eval::value::RegisterReader<'_>,
    shape: &partiql_eval::value::Shape,
) -> Result<Value, RowConvertError> {
    use partiql_eval::value::{FieldName, RowShape};

    match shape.row_shape() {
        RowShape::Struct(fields) => {
            let mut tuple = Tuple::new();
            for field in fields.iter() {
                let name = match &field.name {
                    FieldName::Static(s) => (*s).to_string(),
                    FieldName::Register(reg) => row
                        .get_str(*reg)
                        .ok_or_else(|| {
                            RowConvertError(format!("field-name register {reg} unreadable"))
                        })?
                        .to_string(),
                };
                let reg_idx = match &field.value {
                    RowShape::Register(idx, _) => *idx,
                    RowShape::Struct(_) => {
                        return Err(RowConvertError(format!(
                            "nested struct row-shape not supported for field {name:?}"
                        )));
                    }
                };
                let mut view = row
                    .get_value_view(reg_idx)
                    .ok_or_else(|| RowConvertError(format!("register {reg_idx} unreadable")))?;
                tuple.insert(&name, value_view_to_value(&mut view)?);
            }
            Ok(Value::Tuple(Box::new(tuple)))
        }
        RowShape::Register(idx, _) => {
            let mut view = row
                .get_value_view(*idx)
                .ok_or_else(|| RowConvertError(format!("register {idx} unreadable")))?;
            value_view_to_value(&mut view)
        }
    }
}

fn value_view_to_value(
    view: &mut partiql_eval::value::ValueView<'_>,
) -> Result<Value, RowConvertError> {
    use partiql_eval::value::ValueType;

    Ok(match view.get_type() {
        ValueType::Missing => Value::Missing,
        ValueType::Null => Value::Null,
        ValueType::Bool => Value::Boolean(view.get_bool().map_err(scalar_err)?),
        ValueType::Integer => Value::Integer(view.get_i64().map_err(scalar_err)?),
        ValueType::Decimal => Value::Decimal(Box::new(view.get_decimal().map_err(scalar_err)?)),
        ValueType::Float => Value::Real(view.get_f64().map_err(scalar_err)?.into()),
        ValueType::String => {
            Value::String(Box::new(view.get_str().map_err(scalar_err)?.to_string()))
        }
        ValueType::Bytes => Value::Blob(Box::new(view.get_bytes().map_err(scalar_err)?.to_vec())),
        ValueType::Tuple => Value::Tuple(Box::new(walk_struct(view)?)),
        ValueType::List => Value::List(Box::new(walk_sequence(view)?.into())),
        ValueType::Bag => Value::Bag(Box::new(walk_sequence(view)?.into())),
    })
}

fn scalar_err<E: std::fmt::Debug>(e: E) -> RowConvertError {
    RowConvertError(format!("scalar read: {e:?}"))
}

fn walk_err<E: std::fmt::Debug>(op: &'static str) -> impl FnOnce(E) -> RowConvertError {
    move |e| RowConvertError(format!("{op}: {e:?}"))
}

/// `ValueView::step_in` returns `IllegalState("cannot step into empty X")` for
/// an empty container rather than an Ok+empty walk. That is a legitimate case
/// for us — an empty bag is a valid result. Distinguish it from real errors
/// on the message so we can yield an empty sequence instead of propagating.
fn is_empty_container_error(e: &partiql_eval::EngineError) -> bool {
    matches!(e, partiql_eval::EngineError::IllegalState(msg)
        if msg.starts_with("cannot step into empty"))
}

/// Drive a `step_in` / `loop { advance }` / `step_out` walk over a container,
/// yielding each child's decoded `Value`. Shared by `List` and `Bag`. Struct
/// walks differ (they also read a field name per iteration), so they have
/// their own helper.
fn walk_sequence(
    view: &mut partiql_eval::value::ValueView<'_>,
) -> Result<Vec<Value>, RowConvertError> {
    if let Err(e) = view.step_in() {
        if is_empty_container_error(&e) {
            return Ok(Vec::new());
        }
        return Err(walk_err("step_in")(e));
    }
    let mut items = Vec::new();
    loop {
        items.push(value_view_to_value(view)?);
        if !view.advance().map_err(walk_err("advance"))? {
            break;
        }
    }
    view.step_out().map_err(walk_err("step_out"))?;
    Ok(items)
}

fn walk_struct(view: &mut partiql_eval::value::ValueView<'_>) -> Result<Tuple, RowConvertError> {
    if let Err(e) = view.step_in() {
        if is_empty_container_error(&e) {
            return Ok(Tuple::new());
        }
        return Err(walk_err("step_in")(e));
    }
    let mut tuple = Tuple::new();
    loop {
        let name = view
            .get_field_name()
            .map_err(walk_err("get_field_name"))?
            .to_string();
        let value = value_view_to_value(view)?;
        tuple.insert(&name, value);
        if !view.advance().map_err(walk_err("advance"))? {
            break;
        }
    }
    view.step_out().map_err(walk_err("step_out"))?;
    Ok(tuple)
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_value::DateTime;
    use std::num::NonZeroU8;

    /// Locks in the encoder adapter's DateTime handling. This is NOT
    /// end-to-end coverage — engine `ValueType` has no `DateTime` variant,
    /// so VM rows currently cannot produce `Value::DateTime`. Test guards
    /// regression the day the engine adds `DateTime` support.
    #[test]
    fn datetime_encodes_as_ion_timestamp() {
        let dt = DateTime::from_ymdhms_nano_offset_minutes(
            2020,
            NonZeroU8::new(3).unwrap(),
            14,
            12,
            30,
            45,
            0,
            Some(0),
        );
        let value = Value::DateTime(Box::new(dt));
        let element = value_to_element(&value).expect("DateTime must encode via canonical encoder");
        assert_eq!(element.ion_type(), ion_rs::IonType::Timestamp);
    }
}
