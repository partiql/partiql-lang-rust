use delegate::delegate;
use ion_rs_old::{Decimal, Int, IonError, IonReader, IonType, StreamItem, Symbol};
use once_cell::sync::Lazy;
use partiql_value::{Bag, DateTime, EdgeSpec, Graph, List, SimpleGraph, Tuple, Value, Variant};
use regex::RegexSet;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashSet;

use crate::boxed_ion::BoxedIonType;
use crate::common::{
    Encoding, BAG_ANNOT, BOXED_ION_ANNOT, DATE_ANNOT, GRAPH_ANNOT, MISSING_ANNOT,
    RE_SET_TIME_PARTS, TIME_ANNOT, TIME_PARTS_HOUR, TIME_PARTS_MINUTE, TIME_PARTS_SECOND,
    TIME_PARTS_TZ_HOUR, TIME_PARTS_TZ_MINUTE,
};
use std::num::NonZeroU8;
use std::rc::Rc;
use std::str::FromStr;
use thiserror::Error;
use time::Duration;

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
    #[must_use]
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
    #[must_use]
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

impl Iterator for IonValueIter<'_> {
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
#[inline]
fn dispatch_decode_value<R, D>(decoder: &D, reader: &mut R, typ: IonType) -> IonDecodeResult
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
    D: IonValueDecoder<R> + ?Sized,
{
    match typ {
        IonType::Null => decoder.decode_null(reader),
        IonType::Bool => decoder.decode_bool(reader),
        IonType::Int => decoder.decode_int(reader),
        IonType::Float => decoder.decode_float(reader),
        IonType::Decimal => decoder.decode_decimal(reader),
        IonType::Timestamp => decoder.decode_timestamp(reader),
        IonType::Symbol => decoder.decode_symbol(reader),
        IonType::String => decoder.decode_string(reader),
        IonType::Clob => decoder.decode_clob(reader),
        IonType::Blob => decoder.decode_blob(reader),
        IonType::List => decoder.decode_list(reader),
        IonType::SExp => decoder.decode_sexp(reader),
        IonType::Struct => decoder.decode_struct(reader),
    }
}

trait IonValueDecoder<R>
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    #[inline]
    fn decode_value(&self, reader: &mut R, typ: IonType) -> IonDecodeResult {
        dispatch_decode_value(self, reader, typ)
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
    reader.annotations().any(|a| a.is_ok_and(|a| a == annot))
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

    fn decode_boxed<R>(&self, reader: &mut R) -> IonDecodeResult
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let annot: Vec<_> = reader
            .annotations()
            .skip(1) // skip the `$ion` boxing annotation
            .filter_map(|s| s.ok().and_then(|s| s.text().map(|s| s.to_string())))
            .collect();
        let mut loader = ion_elt::ElementLoader::for_reader(reader);
        let elt = loader.materialize_current()?.unwrap();
        let elt = elt.with_annotations(annot);

        let ion_ctor = Box::new(BoxedIonType {});
        let contents = elt.to_string();
        Ok(Value::from(
            Variant::new(contents, ion_ctor)
                .map_err(|e| IonDecodeError::StreamError(e.to_string()))?,
        ))
    }

    fn decode_graph<R>(&self, reader: &mut R) -> IonDecodeResult
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let err = || IonDecodeError::ConversionError("Invalid graph specified".into());
        let mut nodes = None;
        let mut edges = None;
        reader.step_in()?;
        'kv: loop {
            match reader.next()? {
                StreamItem::Value(typ) => match typ {
                    IonType::List => match reader.field_name()?.text_or_error()? {
                        "nodes" => nodes = Some(self.decode_nodes(reader)?),
                        "edges" => edges = Some(self.decode_edges(reader)?),
                        _ => return Err(err()),
                    },
                    _ => return Err(err()),
                },
                StreamItem::Null(_) => return Err(err()),
                StreamItem::Nothing => break 'kv,
            }
        }
        reader.step_out()?;

        let nodes = nodes.ok_or_else(err)?;
        let (ids, labels, ends, payloads) = edges.ok_or_else(err)?;
        let edge_specs = ends
            .into_iter()
            .map(|(l, dir, r)| match dir.as_str() {
                "->" => Ok(EdgeSpec::Directed(l, r)),
                "<-" => Ok(EdgeSpec::Directed(r, l)),
                "--" => Ok(EdgeSpec::Undirected(l, r)),
                _ => Err(err()),
            })
            .collect::<Result<Vec<EdgeSpec>, _>>()?;
        Ok(Value::Graph(Box::new(Graph::Simple(Rc::new(
            SimpleGraph::from_spec(nodes, (ids, labels, edge_specs, payloads)),
        )))))
    }

    fn decode_nodes<R>(
        &self,
        reader: &mut R,
    ) -> Result<(Vec<String>, Vec<HashSet<String>>, Vec<Option<Value>>), IonDecodeError>
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let err = || IonDecodeError::ConversionError("Invalid graph specified".into());
        reader.step_in()?;
        let mut ids = vec![];
        let mut labels = vec![];
        let mut payloads = vec![];
        'values: loop {
            let item = reader.next()?;
            match item {
                StreamItem::Nothing => break 'values,
                StreamItem::Value(IonType::Struct) => {
                    let (id, labelset, payload) = self.decode_node(reader)?;
                    ids.push(id);
                    labels.push(labelset);
                    payloads.push(payload);
                }
                _ => return Err(err()),
            }
        }
        reader.step_out()?;
        Ok((ids, labels, payloads))
    }

    fn decode_node<R>(
        &self,
        reader: &mut R,
    ) -> Result<(String, HashSet<String>, Option<Value>), IonDecodeError>
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let err = || IonDecodeError::ConversionError("Invalid graph specified".into());
        let mut id = None;
        let mut labels = None;
        let mut payload = None;
        reader.step_in()?;
        'kv: loop {
            let item = reader.next()?;
            if item == StreamItem::Nothing {
                break 'kv;
            }
            let fname = reader.field_name()?;
            let fname = fname.text_or_error()?;
            match (fname, item) {
                ("id", StreamItem::Value(IonType::Symbol)) => {
                    id = Some(reader.read_symbol()?.text_or_error()?.to_string());
                }
                ("labels", StreamItem::Value(IonType::List)) => {
                    let mut labelset = HashSet::new();
                    reader.step_in()?;
                    while let item = reader.next()? {
                        match item {
                            StreamItem::Value(IonType::String) => {
                                labelset.insert(reader.read_string()?.to_string());
                            }
                            StreamItem::Nothing => break,
                            _ => return Err(err()),
                        }
                    }
                    reader.step_out()?;
                    labels = Some(labelset);
                }
                ("payload", StreamItem::Value(typ)) => {
                    payload = Some(self.decode_value(reader, typ)?);
                }
                _ => return Err(err()),
            }
        }
        reader.step_out()?;

        let id = id.ok_or_else(err)?;
        let labels = labels.unwrap_or_else(Default::default);
        Ok((id, labels, payload))
    }

    fn decode_edges<R>(
        &self,
        reader: &mut R,
    ) -> Result<
        (
            Vec<String>,
            Vec<HashSet<String>>,
            Vec<(String, String, String)>,
            Vec<Option<Value>>,
        ),
        IonDecodeError,
    >
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let err = || IonDecodeError::ConversionError("Invalid graph specified".into());
        reader.step_in()?;
        let mut ids = vec![];
        let mut labels = vec![];
        let mut ends = vec![];
        let mut payloads = vec![];
        'values: loop {
            let item = reader.next()?;
            match item {
                StreamItem::Nothing => break 'values,
                StreamItem::Value(IonType::Struct) => {
                    let (id, labelset, end, payload) = self.decode_edge(reader)?;
                    ids.push(id);
                    labels.push(labelset);
                    ends.push(end);
                    payloads.push(payload);
                }
                _ => return Err(err()),
            }
        }
        reader.step_out()?;
        Ok((ids, labels, ends, payloads))
    }

    fn decode_edge<R>(
        &self,
        reader: &mut R,
    ) -> Result<
        (
            String,
            HashSet<String>,
            (String, String, String),
            Option<Value>,
        ),
        IonDecodeError,
    >
    where
        R: IonReader<Item = StreamItem, Symbol = Symbol>,
    {
        let err = || IonDecodeError::ConversionError("Invalid graph specified".into());
        let mut id = None;
        let mut labels = None;
        let mut ends = None;
        let mut payload = None;
        reader.step_in()?;
        'kv: loop {
            let item = reader.next()?;
            if item == StreamItem::Nothing {
                break 'kv;
            }
            let fname = reader.field_name()?;
            let fname = fname.text_or_error()?;
            match (fname, item) {
                ("id", StreamItem::Value(IonType::Symbol)) => {
                    id = Some(reader.read_symbol()?.text_or_error()?.to_string());
                }
                ("labels", StreamItem::Value(IonType::List)) => {
                    let mut labelset = HashSet::new();
                    reader.step_in()?;
                    while let item = reader.next()? {
                        match item {
                            StreamItem::Value(IonType::String) => {
                                labelset.insert(reader.read_string()?.to_string());
                            }
                            StreamItem::Nothing => break,
                            _ => return Err(err()),
                        }
                    }
                    reader.step_out()?;
                    labels = Some(labelset);
                }
                ("ends", StreamItem::Value(IonType::SExp)) => {
                    reader.step_in()?;
                    reader.next()?;
                    let l = reader.read_symbol()?.text_or_error()?.to_string();
                    reader.next()?;
                    let dir = reader.read_symbol()?.text_or_error()?.to_string();
                    reader.next()?;
                    let r = reader.read_symbol()?.text_or_error()?.to_string();
                    reader.step_out()?;
                    ends = Some((l, dir, r));
                }
                ("payload", StreamItem::Value(typ)) => {
                    payload = Some(self.decode_value(reader, typ)?);
                }
                _ => return Err(err()),
            }
        }
        reader.step_out()?;

        let id = id.ok_or_else(err)?;
        let labels = labels.unwrap_or_else(Default::default);
        let ends = ends.ok_or_else(err)?;
        Ok((id, labels, ends, payload))
    }
}

impl<R> IonValueDecoder<R> for PartiqlEncodedIonValueDecoder
where
    R: IonReader<Item = StreamItem, Symbol = Symbol>,
{
    fn decode_value(&self, reader: &mut R, typ: IonType) -> IonDecodeResult {
        if has_annotation(reader, BOXED_ION_ANNOT) {
            self.decode_boxed(reader)
        } else {
            dispatch_decode_value(self, reader, typ)
        }
    }

    #[inline]
    fn decode_null(&self, reader: &mut R) -> IonDecodeResult {
        if has_annotation(reader, BOXED_ION_ANNOT) {
            self.decode_boxed(reader)
        } else if has_annotation(reader, MISSING_ANNOT) {
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
        } else if has_annotation(reader, GRAPH_ANNOT) {
            self.decode_graph(reader)
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

// Code in this module is copied from ion-rs v0.18, in order to make use of `materialize_current`,
// which is not exposed there.
mod ion_elt {
    use ion_rs_old::element::{Element, IntoAnnotatedElement, Sequence, Struct, Value};
    use ion_rs_old::{IonReader, IonResult, StreamItem, Symbol};

    /// Helper type; wraps an [ElementReader] and recursively materializes the next value in the
    /// reader's input, reporting any errors that might occur along the way.
    pub(crate) struct ElementLoader<'a, R: ?Sized> {
        reader: &'a mut R,
    }

    impl<'a, R: IonReader<Item = StreamItem, Symbol = Symbol> + ?Sized> ElementLoader<'a, R> {
        pub(crate) fn for_reader(reader: &'a mut R) -> ElementLoader<'a, R> {
            ElementLoader { reader }
        }

        /// Advances the reader to the next value in the stream and uses [Self::materialize_current]
        /// to materialize it.
        pub(crate) fn materialize_next(&mut self) -> IonResult<Option<Element>> {
            // Advance the reader to the next value
            let _ = self.reader.next()?;
            self.materialize_current()
        }

        /// Recursively materialize the reader's current Ion value and returns it as `Ok(Some(value))`.
        /// If there are no more values at this level, returns `Ok(None)`.
        /// If an error occurs while materializing the value, returns an `Err`.
        /// Calling this method advances the reader and consumes the current value.
        pub(crate) fn materialize_current(&mut self) -> IonResult<Option<Element>> {
            // Collect this item's annotations into a Vec. We have to do this before materializing the
            // value itself because materializing a collection requires advancing the reader further.
            let mut annotations = Vec::new();
            // Current API limitations require `self.reader.annotations()` to heap allocate its
            // iterator even if there aren't annotations. `self.reader.has_annotations()` is trivial
            // and allows us to skip the heap allocation in the common case.
            if self.reader.has_annotations() {
                for annotation in self.reader.annotations() {
                    annotations.push(annotation?);
                }
            }

            let value = match self.reader.current() {
                // No more values at this level of the stream
                StreamItem::Nothing => return Ok(None),
                // This is a typed null
                StreamItem::Null(ion_type) => Value::Null(ion_type),
                // This is a non-null value
                StreamItem::Value(ion_type) => {
                    use ion_rs_old::IonType::*;
                    match ion_type {
                        Null => unreachable!("non-null value had IonType::Null"),
                        Bool => Value::Bool(self.reader.read_bool()?),
                        Int => Value::Int(self.reader.read_int()?),
                        Float => Value::Float(self.reader.read_f64()?),
                        Decimal => Value::Decimal(self.reader.read_decimal()?),
                        Timestamp => Value::Timestamp(self.reader.read_timestamp()?),
                        Symbol => Value::Symbol(self.reader.read_symbol()?),
                        String => Value::String(self.reader.read_string()?),
                        Clob => Value::Clob(self.reader.read_clob()?.into()),
                        Blob => Value::Blob(self.reader.read_blob()?.into()),
                        // It's a collection; recursively materialize all of this value's children
                        List => Value::List(self.materialize_sequence()?),
                        SExp => Value::SExp(self.materialize_sequence()?),
                        Struct => Value::Struct(self.materialize_struct()?),
                    }
                }
            };
            Ok(Some(value.with_annotations(annotations)))
        }

        /// Steps into the current sequence and materializes each of its children to construct
        /// an [`Vec<Element>`]. When all of the children have been materialized, steps out.
        /// The reader MUST be positioned over a list or s-expression when this is called.
        fn materialize_sequence(&mut self) -> IonResult<Sequence> {
            let mut child_elements = Vec::new();
            self.reader.step_in()?;
            while let Some(element) = self.materialize_next()? {
                child_elements.push(element);
            }
            self.reader.step_out()?;
            Ok(child_elements.into())
        }

        /// Steps into the current struct and materializes each of its fields to construct
        /// an [`Struct`]. When all of the fields have been materialized, steps out.
        /// The reader MUST be positioned over a struct when this is called.
        fn materialize_struct(&mut self) -> IonResult<Struct> {
            let mut child_elements = Vec::new();
            self.reader.step_in()?;
            while let StreamItem::Value(_) | StreamItem::Null(_) = self.reader.next()? {
                let field_name = self.reader.field_name()?;
                let value = self
                    .materialize_current()?
                    .expect("materialize_current() returned None for user data");
                child_elements.push((field_name, value));
            }
            self.reader.step_out()?;
            Ok(Struct::from_iter(child_elements))
        }
    }
}
