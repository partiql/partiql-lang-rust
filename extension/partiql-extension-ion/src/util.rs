use crate::decode::IonDecodeError;
use ion_rs::{Decimal, Element, IonResult};
use partiql_value::datum::DatumLowerResult;
use partiql_value::{DateTime, Value};
use std::num::NonZeroU8;
use std::str::FromStr;

pub enum PartiqlValueTarget<T> {
    Atom(Value),
    List(Vec<T>),
    Bag(Vec<T>),
    Struct(Vec<(String, T)>),
}

impl<T, V> From<V> for PartiqlValueTarget<T>
where
    V: Into<Value>,
{
    fn from(value: V) -> Self {
        PartiqlValueTarget::Atom(value.into())
    }
}

pub trait ToPartiqlValue<T> {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<T>>;
}

impl ToPartiqlValue<Element> for Element {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        let value = self.into_value();
        match value {
            ion_rs::Value::Null(_) => Ok(Value::Null.into()),
            ion_rs::Value::Bool(inner) => Ok(inner.into()),
            ion_rs::Value::Int(inner) => inner.into_partiql_value(),
            ion_rs::Value::Float(inner) => Ok(inner.into()),
            ion_rs::Value::Decimal(inner) => inner.into_partiql_value(),
            ion_rs::Value::Timestamp(inner) => inner.into_partiql_value(),
            ion_rs::Value::Symbol(inner) => inner.into_partiql_value(),
            ion_rs::Value::String(inner) => inner.into_partiql_value(),
            ion_rs::Value::Clob(inner) => inner.into_partiql_value(),
            ion_rs::Value::Blob(inner) => inner.into_partiql_value(),
            ion_rs::Value::List(inner) => inner.into_partiql_value(),
            ion_rs::Value::SExp(inner) => inner.into_partiql_value(),
            ion_rs::Value::Struct(inner) => inner.into_partiql_value(),
        }
    }
}

impl ToPartiqlValue<Element> for ion_rs::Int {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        if let Some(n) = self.as_i64() {
            Ok(Value::from(n).into())
        } else {
            let large = self.as_i128().expect("ion int i128");
            Ok(Value::from(large).into())
        }
    }
}

impl ToPartiqlValue<Element> for ion_rs::Decimal {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        let dec = ion_decimal_to_decimal(&self);
        Ok(dec.expect("ion decimal").into())
    }
}

impl ToPartiqlValue<Element> for ion_rs::Timestamp {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        let ts = self;
        let offset = ts.offset();
        let datetime = DateTime::from_ymdhms_nano_offset_minutes(
            ts.year() as i32,
            NonZeroU8::new(ts.month() as u8).ok_or(IonDecodeError::ConversionError(
                "month outside of range".into(),
            ))?,
            ts.day() as u8,
            ts.hour() as u8,
            ts.minute() as u8,
            ts.second() as u8,
            ts.nanoseconds(),
            offset,
        );
        Ok(datetime.into())
    }
}

impl ToPartiqlValue<Element> for ion_rs::Symbol {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        Ok(self.expect_text()?.into())
    }
}

impl ToPartiqlValue<Element> for ion_rs::Str {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        Ok(self.text().into())
    }
}

impl ToPartiqlValue<Element> for ion_rs::Bytes {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        Ok(Value::Blob(Box::new(self.into())).into())
    }
}

impl ToPartiqlValue<Element> for ion_rs::Sequence {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        Ok(PartiqlValueTarget::List(
            self.into_iter().collect::<Vec<_>>(),
        ))
    }
}

impl ToPartiqlValue<Element> for ion_rs::Struct {
    fn into_partiql_value(self) -> DatumLowerResult<PartiqlValueTarget<Element>> {
        let data: IonResult<Vec<_>> = self
            .into_iter()
            .map(|(sym, elt)| sym.expect_text().map(String::from).map(|s| (s, elt)))
            .collect();
        Ok(PartiqlValueTarget::Struct(data?))
    }
}

fn ion_decimal_to_decimal(ion_dec: &Decimal) -> Result<rust_decimal::Decimal, rust_decimal::Error> {
    // TODO ion Decimal doesn't give a lot of functionality to get at the data currently
    // TODO    and it's not clear whether we'll continue with rust decimal or switch to big decimal
    let ion_dec_str = format!("{ion_dec}").replace('d', "e");
    rust_decimal::Decimal::from_str(&ion_dec_str)
        .or_else(|_| rust_decimal::Decimal::from_scientific(&ion_dec_str))
}
