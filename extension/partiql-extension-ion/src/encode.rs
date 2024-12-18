use delegate::delegate;

use ion_rs_old::{IonError, IonType, IonWriter};
use ordered_float::OrderedFloat;
use partiql_value::{Bag, DateTime, List, Tuple, Value};
use rust_decimal::Decimal;

use crate::common::{
    BAG_ANNOT, DATE_ANNOT, MISSING_ANNOT, TIME_ANNOT, TIME_PART_HOUR_KEY, TIME_PART_MINUTE_KEY,
    TIME_PART_SECOND_KEY, TIME_PART_TZ_HOUR_KEY, TIME_PART_TZ_MINUTE_KEY,
};
use crate::Encoding;
use thiserror::Error;
use time::{Date, Duration, Time, UtcOffset};

/// Errors in ion encoding.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum IonEncodeError {
    /// Ion Writer error.
    #[error("Ion write error: `{}`", .0)]
    IonWriterError(IonError),

    /// Unsupported type error.
    #[error("Ion write error: unsupported value of type `{}`", .0)]
    UnsupportedType(&'static str),
}

impl From<IonError> for IonEncodeError {
    fn from(value: IonError) -> Self {
        IonEncodeError::IonWriterError(value)
    }
}

/// Result of attempts to encode a [`Value`] to Ion.
pub type IonEncodeResult = Result<(), IonEncodeError>;

/// Config for construction an Ion encoder.
pub struct IonEncoderConfig {
    mode: Encoding,
}

impl IonEncoderConfig {
    /// Set the mode to `mode`
    #[must_use]
    pub fn with_mode(mut self, mode: crate::Encoding) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for IonEncoderConfig {
    fn default() -> Self {
        IonEncoderConfig {
            mode: crate::Encoding::Ion,
        }
    }
}

/// Builder for creating an encoder.
pub struct IonEncoderBuilder {
    config: IonEncoderConfig,
}

impl IonEncoderBuilder {
    /// Create the builder from 'config'
    #[must_use]
    pub fn new(config: IonEncoderConfig) -> Self {
        Self { config }
    }

    /// Create a encoder given the previously specified config and an ion [`IonWriter`].
    pub fn build<'a, W, I>(
        self,
        writer: &'a mut I,
    ) -> Result<Box<dyn ValueEncoder<W, I> + 'a>, IonEncodeError>
    where
        W: 'a,
        I: IonWriter<Output = W> + 'a,
    {
        let encoder = SimpleIonValueEncoder { writer };
        let encoder: Box<dyn ValueEncoder<W, I>> = match self.config.mode {
            crate::Encoding::Ion => Box::new(encoder),
            crate::Encoding::PartiqlEncodedAsIon => {
                Box::new(PartiqlEncodedIonValueEncoder { inner: encoder })
            }
        };

        Ok(encoder)
    }
}

impl Default for IonEncoderBuilder {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

/// An encoder which will write [`Value`]s as Ion stream values.
pub trait ValueEncoder<W, I>
where
    I: IonWriter<Output = W>,
{
    /// A reference to the writer used by this encoder.
    fn writer(&mut self) -> &mut I;

    /// Write an Ion stream value from the given [`Value`]
    fn write_value(&mut self, value: &Value) -> IonEncodeResult;
}

trait IonValueEncoder<W, I>: ValueEncoder<W, I>
where
    I: IonWriter<Output = W>,
{
    fn get_writer(&mut self) -> &mut I;

    #[inline]
    fn encode_value(&mut self, value: &Value) -> IonEncodeResult {
        match value {
            Value::Null => self.encode_null(),
            Value::Missing => self.encode_missing(),
            Value::Boolean(b) => self.encode_bool(b),
            Value::Integer(i) => self.encode_int(i),
            Value::Real(r) => self.encode_real(r),
            Value::Decimal(d) => self.encode_decimal(d),
            Value::String(s) => self.encode_string(s.as_ref()),
            Value::Blob(b) => self.encode_blob(b.as_ref()),
            Value::DateTime(dt) => self.encode_datetime(dt.as_ref()),
            Value::List(l) => self.encode_list(l.as_ref()),
            Value::Bag(b) => self.encode_bag(b.as_ref()),
            Value::Tuple(t) => self.encode_tuple(t.as_ref()),
            Value::Variant(_) => {
                todo!("ion encode embedded doc")
            }
        }
    }

    fn encode_null(&mut self) -> IonEncodeResult;
    fn encode_missing(&mut self) -> IonEncodeResult;
    fn encode_bool(&mut self, val: &bool) -> IonEncodeResult;
    fn encode_int(&mut self, val: &i64) -> IonEncodeResult;
    fn encode_real(&mut self, val: &OrderedFloat<f64>) -> IonEncodeResult;
    fn encode_decimal(&mut self, val: &Decimal) -> IonEncodeResult;
    fn encode_string(&mut self, val: &str) -> IonEncodeResult;
    fn encode_blob(&mut self, val: &[u8]) -> IonEncodeResult;
    fn encode_datetime(&mut self, val: &DateTime) -> IonEncodeResult;
    fn encode_list(&mut self, val: &List) -> IonEncodeResult;
    fn encode_bag(&mut self, val: &Bag) -> IonEncodeResult;
    fn encode_tuple(&mut self, val: &Tuple) -> IonEncodeResult;
}

impl<'a, W, I> ValueEncoder<W, I> for SimpleIonValueEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn writer(&mut self) -> &mut I {
        self.get_writer()
    }

    #[inline]
    fn write_value(&mut self, value: &Value) -> IonEncodeResult {
        self.encode_value(value)
    }
}

impl<'a, W, I> ValueEncoder<W, I> for PartiqlEncodedIonValueEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn writer(&mut self) -> &mut I {
        self.get_writer()
    }

    #[inline]
    fn write_value(&mut self, value: &Value) -> IonEncodeResult {
        self.encode_value(value)
    }
}

struct SimpleIonValueEncoder<'a, W, I>
where
    I: IonWriter<Output = W>,
{
    pub(crate) writer: &'a mut I,
}

impl<'a, W, I> IonValueEncoder<W, I> for SimpleIonValueEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn get_writer(&mut self) -> &mut I {
        self.writer
    }

    fn encode_null(&mut self) -> IonEncodeResult {
        Ok(self.writer.write_null(IonType::Null)?)
    }

    fn encode_missing(&mut self) -> IonEncodeResult {
        Err(IonEncodeError::UnsupportedType("missing"))
    }

    fn encode_bool(&mut self, val: &bool) -> IonEncodeResult {
        Ok(self.writer.write_bool(*val)?)
    }

    fn encode_int(&mut self, val: &i64) -> IonEncodeResult {
        Ok(self.writer.write_i64(*val)?)
    }

    fn encode_real(&mut self, val: &OrderedFloat<f64>) -> IonEncodeResult {
        Ok(self.writer.write_f64(val.0)?)
    }

    fn encode_decimal(&mut self, val: &Decimal) -> IonEncodeResult {
        let scale = i64::from(val.scale());
        let mantissa = val.mantissa();
        let dec = ion_rs_old::Decimal::new(mantissa, -scale);
        Ok(self.writer.write_decimal(&dec)?)
    }

    fn encode_string(&mut self, val: &str) -> IonEncodeResult {
        Ok(self.writer.write_string(val)?)
    }

    fn encode_blob(&mut self, val: &[u8]) -> IonEncodeResult {
        Ok(self.writer.write_blob(val)?)
    }

    fn encode_datetime(&mut self, val: &DateTime) -> IonEncodeResult {
        match val {
            DateTime::Timestamp(ts) => {
                let ts = ion_rs_old::Timestamp::with_ymd(
                    ts.year() as u32,
                    ts.month() as u32,
                    u32::from(ts.day()),
                )
                .with_hms(
                    u32::from(ts.hour()),
                    u32::from(ts.minute()),
                    u32::from(ts.second()),
                )
                .with_nanoseconds(ts.nanosecond())
                .build_at_unknown_offset()?;

                Ok(self.writer.write_timestamp(&ts)?)
            }
            DateTime::TimestampWithTz(ts) => {
                let ts = ion_rs_old::Timestamp::with_ymd(
                    ts.year() as u32,
                    ts.month() as u32,
                    u32::from(ts.day()),
                )
                .with_hms(
                    u32::from(ts.hour()),
                    u32::from(ts.minute()),
                    u32::from(ts.second()),
                )
                .with_nanoseconds(ts.nanosecond())
                .build_at_offset(i32::from(ts.offset().whole_minutes()))?;

                Ok(self.writer.write_timestamp(&ts)?)
            }
            _ => Err(IonEncodeError::UnsupportedType(
                "unsupported date time variant",
            )),
        }
    }

    fn encode_list(&mut self, val: &List) -> IonEncodeResult {
        encode_list(self, val.iter())
    }

    fn encode_bag(&mut self, _val: &Bag) -> IonEncodeResult {
        Err(IonEncodeError::UnsupportedType("bag"))
    }

    fn encode_tuple(&mut self, val: &Tuple) -> IonEncodeResult {
        encode_tuple(self, val)
    }
}

#[inline]
fn encode_list<'a, W, I, V>(encoder: &'a mut impl IonValueEncoder<W, I>, vals: V) -> IonEncodeResult
where
    W: 'a,
    I: IonWriter<Output = W> + 'a,
    V: Iterator<Item = &'a Value> + 'a,
{
    encoder.writer().step_in(IonType::List)?;
    for v in vals {
        encoder.encode_value(v)?;
    }
    encoder.writer().step_out()?;

    Ok(())
}

#[inline]
fn encode_tuple<'a, W, I>(
    encoder: &'a mut impl IonValueEncoder<W, I>,
    val: &'a Tuple,
) -> IonEncodeResult
where
    W: 'a,
    I: IonWriter<Output = W> + 'a,
{
    encoder.writer().step_in(IonType::Struct)?;
    for (k, v) in val.pairs() {
        encoder.writer().set_field_name(k);
        encoder.encode_value(v)?;
    }
    encoder.writer().step_out()?;

    Ok(())
}

struct PartiqlEncodedIonValueEncoder<'a, W, I>
where
    I: IonWriter<Output = W>,
{
    inner: SimpleIonValueEncoder<'a, W, I>,
}

impl<'a, W, I> PartiqlEncodedIonValueEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn write_date(&mut self, date: &Date) -> IonEncodeResult {
        self.inner
            .writer
            .set_annotations(std::iter::once(DATE_ANNOT));
        let ts = ion_rs_old::Timestamp::with_ymd(
            date.year() as u32,
            date.month() as u32,
            u32::from(date.day()),
        )
        .build()?;

        Ok(self.inner.writer.write_timestamp(&ts)?)
    }

    fn write_timestamp(&mut self, time: &Time, offset: Option<&UtcOffset>) -> IonEncodeResult {
        let writer = &mut self.inner.writer;
        writer.set_annotations(std::iter::once(TIME_ANNOT));
        writer.step_in(IonType::Struct)?;
        writer.set_field_name(TIME_PART_HOUR_KEY);
        writer.write_i64(i64::from(time.hour()))?;
        writer.set_field_name(TIME_PART_MINUTE_KEY);
        writer.write_i64(i64::from(time.minute()))?;
        writer.set_field_name(TIME_PART_SECOND_KEY);

        let seconds = Duration::new(i64::from(time.second()), time.nanosecond() as i32);
        writer.write_f64(seconds.as_seconds_f64())?;

        if let Some(offset) = offset {
            writer.set_field_name(TIME_PART_TZ_HOUR_KEY);
            writer.write_i64(i64::from(offset.whole_hours()))?;
            writer.set_field_name(TIME_PART_TZ_MINUTE_KEY);
            writer.write_i64(i64::from(offset.minutes_past_hour()))?;
        }

        writer.step_out()?;

        Ok(())
    }
}

impl<'a, W, I> IonValueEncoder<W, I> for PartiqlEncodedIonValueEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn get_writer(&mut self) -> &mut I {
        self.inner.writer
    }

    fn encode_missing(&mut self) -> IonEncodeResult {
        self.inner
            .writer
            .set_annotations(std::iter::once(MISSING_ANNOT));
        self.inner.writer.write_null(IonType::Null)?;
        Ok(())
    }

    fn encode_datetime(&mut self, val: &DateTime) -> IonEncodeResult {
        match val {
            DateTime::Date(date) => self.write_date(date),
            DateTime::Time(time) => self.write_timestamp(time, None),
            DateTime::TimeWithTz(time, offset) => self.write_timestamp(time, Some(offset)),
            _ => self.inner.encode_datetime(val),
        }
    }

    fn encode_list(&mut self, val: &List) -> IonEncodeResult {
        encode_list(self, val.iter())
    }

    fn encode_bag(&mut self, val: &Bag) -> IonEncodeResult {
        self.inner
            .writer
            .set_annotations(std::iter::once(BAG_ANNOT));
        encode_list(self, val.iter())
    }

    fn encode_tuple(&mut self, val: &Tuple) -> IonEncodeResult {
        encode_tuple(self, val)
    }

    delegate! {
        to self.inner {
            fn encode_null(& mut self) -> IonEncodeResult;
            fn encode_bool(& mut self, val: & bool) -> IonEncodeResult;
            fn encode_int(& mut self, val: & i64) -> IonEncodeResult;
            fn encode_real(& mut self, val: & OrderedFloat<f64>) -> IonEncodeResult;
            fn encode_decimal(& mut self, val: & Decimal) -> IonEncodeResult;
            fn encode_string(& mut self, val: & str) -> IonEncodeResult;
            fn encode_blob(& mut self, val: & [u8]) -> IonEncodeResult;
        }
    }
}
