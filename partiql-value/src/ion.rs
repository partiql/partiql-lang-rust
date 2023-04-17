use crate::{Bag, DateTime, List, Tuple, Value};
use ion_rs::{Int, IonReader, IonType, Reader, StreamItem};
use once_cell::sync::Lazy;
use regex::RegexSet;
use rust_decimal::prelude::ToPrimitive;
use std::num::NonZeroU8;
use std::str::FromStr;

const BAG_ANNOT: &str = "$bag";
const TIME_ANNOT: &str = "$time";
const DATE_ANNOT: &str = "$date";
const MISSING_ANNOT: &str = "$missing";

// TODO handle errors more gracefully than `expect`/`unwrap`
/// For a given `contents` as Ion text produces a PartiQL Value
pub fn parse_ion(contents: &str) -> Value {
    let mut reader = ion_rs::ReaderBuilder::new()
        .build(contents)
        .expect("reading contents");

    // expecting a single top-level value
    let item = reader.next().expect("test value");
    let val = match item {
        StreamItem::Value(typ) => parse_value(&mut reader, typ),
        StreamItem::Null(_) => Value::Null,
        StreamItem::Nothing => panic!("expecting a test value"),
    };

    assert_eq!(reader.next().expect("test end"), StreamItem::Nothing);

    val
}

fn parse_value(reader: &mut Reader, typ: IonType) -> Value {
    match typ {
        IonType::Null => parse_null(reader),
        IonType::Bool => Value::Boolean(reader.read_bool().unwrap()),
        IonType::Int => match reader.read_int().unwrap() {
            Int::I64(i) => Value::Integer(i),
            Int::BigInt(_) => todo!("bigint"),
        },
        IonType::Float => Value::Real(reader.read_f64().unwrap().into()),
        IonType::Decimal => {
            // TODO ion Decimal doesn't give a lot of functionality to get at the data currently
            // TODO    and it's not clear whether we'll continue with rust decimal or switch to big decimal
            let ion_dec = reader.read_decimal().unwrap();
            let ion_dec_str = format!("{ion_dec}").replace('d', "e");
            let dec = rust_decimal::Decimal::from_str(&ion_dec_str)
                .or_else(|_| rust_decimal::Decimal::from_scientific(&ion_dec_str));
            Value::Decimal(dec.unwrap())
        }
        IonType::Timestamp => {
            if has_annotation(reader, DATE_ANNOT) {
                parse_date(reader).into()
            } else {
                parse_datetime(reader).into()
            }
        }
        IonType::Symbol => Value::String(Box::new(
            reader
                .read_symbol()
                .unwrap()
                .text()
                .unwrap_or("")
                .to_string(),
        )),
        IonType::String => Value::String(Box::new(reader.read_string().unwrap())),
        IonType::Clob => Value::Blob(Box::new(reader.read_clob().unwrap())),
        IonType::Blob => Value::Blob(Box::new(reader.read_blob().unwrap())),
        IonType::List => {
            if has_annotation(reader, BAG_ANNOT) {
                Bag::from(parse_sequence(reader)).into()
            } else {
                List::from(parse_sequence(reader)).into()
            }
        }
        IonType::SExp => todo!("sexp"),
        IonType::Struct => {
            if has_annotation(reader, TIME_ANNOT) {
                parse_time(reader).into()
            } else {
                parse_tuple(reader).into()
            }
        }
    }
}

#[inline]
fn has_annotation(reader: &Reader, annot: &str) -> bool {
    reader.annotations().any(|a| a.unwrap().eq(&annot))
}

#[inline]
fn parse_null(reader: &Reader) -> Value {
    if has_annotation(reader, MISSING_ANNOT) {
        Value::Missing
    } else {
        Value::Null
    }
}

const RE_SET_TIME_PARTS: [&str; 5] = [
    "^hour$",
    "^minute$",
    "^second$",
    "^timezone_hour$",
    "^timezone_minute$",
];
const TIME_PARTS_HOUR: usize = 0;
const TIME_PARTS_MINUTE: usize = 1;
const TIME_PARTS_SECOND: usize = 2;
const TIME_PARTS_TZ_HOUR: usize = 3;
const TIME_PARTS_TZ_MINUTE: usize = 4;
static TIME_PARTS_PATTERN_SET: Lazy<RegexSet> =
    Lazy::new(|| RegexSet::new(RE_SET_TIME_PARTS).unwrap());

fn parse_time(reader: &mut Reader) -> DateTime {
    fn expect_u8(reader: &mut Reader, typ: Option<IonType>) -> u8 {
        match typ {
            Some(IonType::Int) => match reader.read_int().unwrap() {
                Int::I64(i) => i as u8, // TODO check range
                Int::BigInt(_) => todo!("bigint"),
            },
            _ => {
                todo!("error; not a u8")
            }
        }
    }
    fn maybe_i8(reader: &mut Reader, typ: Option<IonType>) -> Option<i8> {
        match typ {
            Some(IonType::Int) => match reader.read_int().unwrap() {
                Int::I64(i) => Some(i as i8), // TODO check range
                Int::BigInt(_) => todo!("bigint"),
            },
            _ => None,
        }
    }
    fn expect_f64(reader: &mut Reader, typ: Option<IonType>) -> f64 {
        match typ {
            Some(IonType::Decimal) => {
                // TODO ion Decimal doesn't give a lot of functionality to get at the data currently
                // TODO    and it's not clear whether we'll continue with rust decimal or switch to big decimal
                let ion_dec = reader.read_decimal().unwrap();
                let ion_dec_str = format!("{ion_dec}").replace('d', "e");
                let dec = rust_decimal::Decimal::from_str(&ion_dec_str)
                    .or_else(|_| rust_decimal::Decimal::from_scientific(&ion_dec_str));
                let dec = dec.unwrap();
                dec.to_f64().unwrap()
            }
            Some(IonType::Float) => reader.read_f64().unwrap(),
            _ => {
                todo!("error; not a f64: {:?}", typ)
            }
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

    reader.step_in().expect("step into struct");
    #[allow(irrefutable_let_patterns)]
    while let item = reader.next().expect("struct value") {
        let (key, typ) = match item {
            StreamItem::Value(typ) => (reader.field_name().expect("field name"), Some(typ)),
            StreamItem::Null(_) => (reader.field_name().expect("field name"), None),
            StreamItem::Nothing => break,
        };
        let matches = patterns.matches(key.text().unwrap());
        match matches.into_iter().next() {
            Some(TIME_PARTS_HOUR) => time.hour = Some(expect_u8(reader, typ)),
            Some(TIME_PARTS_MINUTE) => time.minute = Some(expect_u8(reader, typ)),
            Some(TIME_PARTS_SECOND) => time.second = Some(expect_f64(reader, typ)),
            Some(TIME_PARTS_TZ_HOUR) => time.tz_hour = maybe_i8(reader, typ),
            Some(TIME_PARTS_TZ_MINUTE) => time.tz_minute = maybe_i8(reader, typ),
            _ => {
                todo!("error: unexpected time field name")
            }
        }
    }
    reader.step_out().expect("step out of struct");

    DateTime::from_hms_nano_tz(
        time.hour.expect("hour"),
        time.minute.expect("minute"),
        time.second.expect("second").trunc() as u8,
        time.second.expect("second").fract() as u32,
        time.tz_hour,
        time.tz_minute,
    )
}

fn parse_datetime(reader: &mut Reader) -> DateTime {
    let ts = reader.read_timestamp().unwrap();
    let offset = ts.offset();
    DateTime::from_ymdhms_nano_offset_minutes(
        ts.year(),
        NonZeroU8::new(ts.month() as u8).unwrap(),
        ts.day() as u8,
        ts.hour() as u8,
        ts.minute() as u8,
        ts.second() as u8,
        ts.nanoseconds(),
        offset,
    )
}

fn parse_date(reader: &mut Reader) -> DateTime {
    let ts = reader.read_timestamp().unwrap();
    DateTime::from_ymd(
        ts.year(),
        NonZeroU8::new(ts.month() as u8).unwrap(),
        ts.day() as u8,
    )
}

fn parse_tuple(reader: &mut Reader) -> Tuple {
    let mut tuple = Tuple::new();
    reader.step_in().expect("step into struct");
    loop {
        let item = reader.next().expect("struct value");
        let (key, value) = match item {
            StreamItem::Value(typ) => (
                reader.field_name().expect("field name"),
                parse_value(reader, typ),
            ),
            StreamItem::Null(_) => (reader.field_name().expect("field name"), Value::Null),
            StreamItem::Nothing => break,
        };
        tuple.insert(key.text().unwrap(), value);
    }
    reader.step_out().expect("step out of struct");
    tuple
}

fn parse_sequence(reader: &mut Reader) -> Vec<Value> {
    reader.step_in().expect("step into sequence");
    let mut values = vec![];
    loop {
        let item = reader.next().expect("test value");
        let val = match item {
            StreamItem::Value(typ) => parse_value(reader, typ),
            StreamItem::Null(_) => Value::Null,
            StreamItem::Nothing => break,
        };
        values.push(val);
    }
    reader.step_out().expect("step out of sequence");
    values
}
