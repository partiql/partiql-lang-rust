use delegate::delegate;
use ion_rs::{Decimal, Int, IonError, IonReader, IonType, StreamItem, Symbol};
use once_cell::sync::Lazy;
use partiql_value::{Bag, DateTime, List, Tuple, Value};
use regex::RegexSet;
use rust_decimal::prelude::ToPrimitive;

use std::num::NonZeroU8;
use std::str::FromStr;

use thiserror::Error;
use time::Duration;

use crate::common::*;

/// Errors in ion decoding.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum IonDecodeError {
    /// Ion Reader error.
    #[error("Ion read error: `{}`", .0)]
    IonReaderError(IonError),

    /// Unsupported type error.
    #[error("Ion read error: unsupported value of type `{}`", .0)]
    UnsupportedType(&'static str),

    /// Conversion error.
    #[error("Ion read error: conversion error `{}`", .0)]
    ConversionError(String),

    /// Stream error.
    #[error("Ion read error: stream error `{}`", .0)]
    StreamError(String),

    /// Any other reading error.
    #[error("Ion read error: unknown error")]
    Unknown,
}

impl From<IonError> for IonDecodeError {
    fn from(value: IonError) -> Self {
        IonDecodeError::IonReaderError(value)
    }
}

impl From<rust_decimal::Error> for IonDecodeError {
    fn from(value: rust_decimal::Error) -> Self {
        IonDecodeError::ConversionError(format!("bad decimal conversion: `{value}`"))
    }
}

/// Result of attempts to decode a [`Value`] from Ion.
pub type IonDecodeResult = Result<Value, IonDecodeError>;

/// Config for construction an Ion decoder.
pub struct IonDecoderConfig {
    mode: Encoding,
}

impl IonDecoderConfig {
    /// Set the mode to `mode`
    pub fn with_mode(mut self, mode: crate::Encoding) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for IonDecoderConfig {
    fn default() -> Self {
        IonDecoderConfig {
            mode: crate::Encoding::Ion,
        }
    }
}

/// Builder for creating a decoder.
pub struct IonDecoderBuilder {
    config: IonDecoderConfig,
}

impl IonDecoderBuilder {
    /// Create the builder from 'config'
    pub fn new(config: IonDecoderConfig) -> Self {
        Self { config }
    }

    /// Create a decoder given the previously specified config and an ion [`Reader`].
    pub fn build<'a>(
        self,
        reader: impl 'a + IonReader<Item = StreamItem, Symbol = Symbol>,
    ) -> Result<IonValueIter<'a>, IonDecodeError> {
        let decoder = SimpleIonValueDecoder {};
        let inner: Box<dyn Iterator<Item = IonDecodeResult>> = match self.config.mode {
            crate::Encoding::Ion => Box::new(IonValueIterInner { reader, decoder }),
            crate::Encoding::PartiqlEncodedAsIon => {
                let decoder = PartiqlEncodedIonValueDecoder { inner: decoder };
                Box::new(IonValueIterInner { reader, decoder })
            }
        };

        Ok(IonValueIter { inner })
    }
}

impl Default for IonDecoderBuilder {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

/// An Iterator over [`IonDecodeResult`] corresponding to the decoded top-level Ion stream values.
pub struct IonValueIter<'a> {
    inner: Box<dyn Iterator<Item = IonDecodeResult> + 'a>,
}

impl<'a> Iterator for IonValueIter<'a> {
    type Item = IonDecodeResult;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

struct IonValueIterInner<D, R>
where
    D: IonValueDecoder<R>,
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    reader: R,
    decoder: D,
}

impl<D, R> Iterator for IonValueIterInner<D, R>
where
    D: IonValueDecoder<R>,
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    type Item = IonDecodeResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.next() {
            Ok(StreamItem::Value(typ)) => Some(self.decoder.decode_value(&mut self.reader, typ)),
            Ok(StreamItem::Null(_)) => Some(self.decoder.decode_null(&mut self.reader)),
            Ok(StreamItem::Nothing) => None,
            Err(e) => Some(Err(e.into())),
        }
    }
}

trait IonValueDecoder<R>
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    #[inline]
    fn decode_value(&self, reader: &mut R, typ: IonType) -> IonDecodeResult {
        match typ {
            IonType::Null => self.decode_null(reader),
            IonType::Bool => self.decode_bool(reader),
            IonType::Int => self.decode_int(reader),
            IonType::Float => self.decode_float(reader),
            IonType::Decimal => self.decode_decimal(reader),
            IonType::Timestamp => self.decode_timestamp(reader),
            IonType::Symbol => self.decode_symbol(reader),
            IonType::String => self.decode_string(reader),
            IonType::Clob => self.decode_clob(reader),
            IonType::Blob => self.decode_blob(reader),
            IonType::List => self.decode_list(reader),
            IonType::SExp => self.decode_sexp(reader),
            IonType::Struct => self.decode_struct(reader),
        }
    }

    fn decode_null(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_bool(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_int(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_float(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_decimal(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_timestamp(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_symbol(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_string(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_clob(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_blob(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_list(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_sexp(&self, reader: &mut R) -> IonDecodeResult;
    fn decode_struct(&self, reader: &mut R) -> IonDecodeResult;
}

fn ion_decimal_to_decimal(ion_dec: Decimal) -> Result<rust_decimal::Decimal, rust_decimal::Error> {
    // TODO ion Decimal doesn't give a lot of functionality to get at the data currently
    // TODO    and it's not clear whether we'll continue with rust decimal or switch to big decimal
    let ion_dec_str = format!("{ion_dec}").replace('d', "e");
    rust_decimal::Decimal::from_str(&ion_dec_str)
        .or_else(|_| rust_decimal::Decimal::from_scientific(&ion_dec_str))
}

struct SimpleIonValueDecoder {}

impl<R> IonValueDecoder<R> for SimpleIonValueDecoder
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    #[inline]
    fn decode_null(&self, _: &mut R) -> IonDecodeResult {
        Ok(Value::Null)
    }

    #[inline]
    fn decode_bool(&self, reader: &mut R) -> IonDecodeResult {
        Ok(Value::Boolean(reader.read_bool()?))
    }

    #[inline]
    fn decode_int(&self, reader: &mut R) -> IonDecodeResult {
        match reader.read_int()? {
            Int::I64(i) => Ok(Value::Integer(i)),
            Int::BigInt(_) => Err(IonDecodeError::UnsupportedType("bigint")),
        }
    }

    #[inline]
    fn decode_float(&self, reader: &mut R) -> IonDecodeResult {
        Ok(Value::Real(reader.read_f64()?.into()))
    }

    #[inline]
    fn decode_decimal(&self, reader: &mut R) -> IonDecodeResult {
        let dec = ion_decimal_to_decimal(reader.read_decimal()?);
        Ok(Value::Decimal(Box::new(dec?)))
    }

    #[inline]
    fn decode_timestamp(&self, reader: &mut R) -> IonDecodeResult {
        let ts = reader.read_timestamp()?;
        let offset = ts.offset();
        let datetime = DateTime::from_ymdhms_nano_offset_minutes(
            ts.year(),
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

    #[inline]
    fn decode_symbol(&self, reader: &mut R) -> IonDecodeResult {
        Ok(Value::String(Box::new(
            reader.read_symbol()?.text_or_error()?.to_string(),
        )))
    }

    #[inline]
    fn decode_string(&self, reader: &mut R) -> IonDecodeResult {
        Ok(Value::String(Box::new(
            reader.read_string()?.text().to_string(),
        )))
    }

    #[inline]
    fn decode_clob(&self, reader: &mut R) -> IonDecodeResult {
        Ok(Value::Blob(Box::new(reader.read_clob()?.as_slice().into())))
    }

    #[inline]
    fn decode_blob(&self, reader: &mut R) -> IonDecodeResult {
        Ok(Value::Blob(Box::new(reader.read_blob()?.as_slice().into())))
    }

    #[inline]
    fn decode_list(&self, reader: &mut R) -> IonDecodeResult {
        decode_list(self, reader)
    }

    #[inline]
    fn decode_sexp(&self, _: &mut R) -> IonDecodeResult {
        Err(IonDecodeError::UnsupportedType("sexp"))
    }

    #[inline]
    fn decode_struct(&self, reader: &mut R) -> IonDecodeResult {
        decode_struct(self, reader)
    }
}

#[inline]
fn decode_list<R>(decoder: &impl IonValueDecoder<R>, reader: &mut R) -> IonDecodeResult
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    reader.step_in()?;
    let mut values = vec![];
    'values: loop {
        let item = reader.next()?;
        let val = match item {
            StreamItem::Value(typ) => decoder.decode_value(reader, typ)?,
            StreamItem::Null(_) => decoder.decode_null(reader)?,
            StreamItem::Nothing => break 'values,
        };
        values.push(val);
    }
    reader.step_out()?;
    Ok(List::from(values).into())
}

#[inline]
fn decode_struct<R>(decoder: &impl IonValueDecoder<R>, reader: &mut R) -> IonDecodeResult
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    let mut tuple = Tuple::new();
    reader.step_in()?;
    'kv: loop {
        let item = reader.next()?;
        let (key, value) = match item {
            StreamItem::Value(typ) => {
                let field_name = reader.field_name()?;
                (field_name, decoder.decode_value(reader, typ)?)
            }
            StreamItem::Null(_) => (reader.field_name()?, decoder.decode_null(reader)?),
            StreamItem::Nothing => break 'kv,
        };
        tuple.insert(key.text_or_error()?, value);
    }
    reader.step_out()?;
    Ok(tuple.into())
}

struct PartiqlEncodedIonValueDecoder {
    inner: SimpleIonValueDecoder,
}

#[inline]
fn has_annotation(
    reader: &impl IonReader<Item = StreamItem, Symbol = Symbol>,
    annot: &str,
) -> bool {
    reader
        .annotations()
        .any(|a| a.map_or(false, |a| a == annot))
}

static TIME_PARTS_PATTERN_SET: Lazy<RegexSet> =
    Lazy::new(|| RegexSet::new(RE_SET_TIME_PARTS).unwrap());

impl PartiqlEncodedIonValueDecoder {
    fn decode_date<R>(&self, reader: &mut R) -> IonDecodeResult
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let ts = reader.read_timestamp()?;
        let datetime = DateTime::from_ymd(
            ts.year(),
            NonZeroU8::new(ts.month() as u8).ok_or(IonDecodeError::ConversionError(
                "month outside of range".into(),
            ))?,
            ts.day() as u8,
        );
        Ok(datetime.into())
    }

    fn decode_time<R>(&self, reader: &mut R) -> IonDecodeResult
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        fn expect_u8<R>(
            reader: &mut R,
            typ: Option<IonType>,
            unit: &'static str,
        ) -> Result<u8, IonDecodeError>
        where
            R: IonReader<Item = StreamItem, Symbol = Symbol>,
        {
            match typ {
                Some(IonType::Int) => match reader.read_int()? {
                    Int::I64(i) => Ok(i as u8), // TODO check range
                    Int::BigInt(_) => Err(IonDecodeError::ConversionError(format!(
                        "value for {unit} outside of range"
                    ))),
                },
                _ => Err(IonDecodeError::ConversionError(format!(
                    "value for {unit} unexpected type"
                ))),
            }
        }
        fn maybe_i8<R>(
            reader: &mut R,
            typ: Option<IonType>,
            unit: &'static str,
        ) -> Result<Option<i8>, IonDecodeError>
        where
            R: IonReader<Item = StreamItem, Symbol = Symbol>,
        {
            match typ {
                Some(IonType::Int) => match reader.read_int()? {
                    Int::I64(i) => Ok(Some(i as i8)), // TODO check range
                    Int::BigInt(_) => Err(IonDecodeError::ConversionError(format!(
                        "value for {unit} outside of range"
                    ))),
                },
                None => Ok(None),
                Some(IonType::Null) => Ok(None),
                _ => Err(IonDecodeError::ConversionError(format!(
                    "value for {unit} unexpected type {typ:?}"
                ))),
            }
        }
        fn expect_f64<R>(
            reader: &mut R,
            typ: Option<IonType>,
            unit: &'static str,
        ) -> Result<f64, IonDecodeError>
        where
            R: IonReader<Item = StreamItem, Symbol = Symbol>,
        {
            match typ {
                Some(IonType::Decimal) => {
                    let dec = ion_decimal_to_decimal(reader.read_decimal()?);
                    Ok(dec?.to_f64().unwrap_or(0f64))
                }
                Some(IonType::Float) => Ok(reader.read_f64()?),
                _ => Err(IonDecodeError::ConversionError(format!(
                    "value for {unit} unexpected type"
                ))),
            }
        }

        #[derive(Default)]
        struct TimeParts {
            pub hour: Option<u8>,
            pub minute: Option<u8>,
            pub second: Option<f64>,
            pub tz_hour: Option<i8>,
            pub tz_minute: Option<i8>,
        }

        let mut time = TimeParts::default();
        let patterns: &RegexSet = &TIME_PARTS_PATTERN_SET;

        reader.step_in()?;
        #[allow(irrefutable_let_patterns)]
        while let item = reader.next()? {
            let (key, typ) = match item {
                StreamItem::Value(typ) => (reader.field_name()?, Some(typ)),
                StreamItem::Null(_) => (reader.field_name()?, None),
                StreamItem::Nothing => break,
            };
            let matches = patterns.matches(key.text_or_error()?);
            match matches.into_iter().next() {
                Some(TIME_PARTS_HOUR) => time.hour = Some(expect_u8(reader, typ, "hour")?),
                Some(TIME_PARTS_MINUTE) => time.minute = Some(expect_u8(reader, typ, "minute")?),
                Some(TIME_PARTS_SECOND) => time.second = Some(expect_f64(reader, typ, "second")?),
                Some(TIME_PARTS_TZ_HOUR) => time.tz_hour = maybe_i8(reader, typ, "tz_hour")?,
                Some(TIME_PARTS_TZ_MINUTE) => time.tz_minute = maybe_i8(reader, typ, "tz_minute")?,
                _ => {
                    return Err(IonDecodeError::ConversionError(
                        "unexpected field name for time".to_string(),
                    ))
                }
            }
        }
        reader.step_out()?;

        let hour = time.hour.ok_or_else(|| {
            IonDecodeError::ConversionError("expected `hour` key for DateTime".into())
        })?;
        let minute = time.minute.ok_or_else(|| {
            IonDecodeError::ConversionError("expected `minute` key for DateTime".into())
        })?;
        let second = time.second.ok_or_else(|| {
            IonDecodeError::ConversionError("expected `second` key for DateTime".into())
        })?;
        let seconds = Duration::seconds_f64(second);
        let datetime = DateTime::from_hms_nano_tz(
            hour,
            minute,
            seconds.whole_seconds() as u8,
            seconds.subsec_nanoseconds() as u32,
            time.tz_hour,
            time.tz_minute,
        );
        Ok(datetime.into())
    }
}

impl<R> IonValueDecoder<R> for PartiqlEncodedIonValueDecoder
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    #[inline]
    fn decode_null(&self, reader: &mut R) -> IonDecodeResult {
        if has_annotation(reader, MISSING_ANNOT) {
            Ok(Value::Missing)
        } else {
            Ok(Value::Null)
        }
    }

    #[inline]
    fn decode_timestamp(&self, reader: &mut R) -> IonDecodeResult {
        if has_annotation(reader, DATE_ANNOT) {
            self.decode_date(reader)
        } else {
            self.inner.decode_timestamp(reader)
        }
    }

    #[inline]
    fn decode_list(&self, reader: &mut R) -> IonDecodeResult {
        let is_bag = has_annotation(reader, BAG_ANNOT);
        let list = decode_list(self, reader);
        if is_bag {
            Ok(Bag::from(list?.coerce_into_list()).into())
        } else {
            list
        }
    }

    #[inline]
    fn decode_struct(&self, reader: &mut R) -> IonDecodeResult {
        if has_annotation(reader, TIME_ANNOT) {
            self.decode_time(reader)
        } else {
            decode_struct(self, reader)
        }
    }

    delegate! {
        to self.inner {
            fn decode_bool(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_int(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_float(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_decimal(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_symbol(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_string(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_clob(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_blob(&self, reader: &mut R) -> IonDecodeResult;
            fn decode_sexp(&self, reader: &mut R) -> IonDecodeResult;
        }
    }
}
