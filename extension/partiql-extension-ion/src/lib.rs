#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

mod common;
pub mod decode;
pub mod encode;

pub use common::Encoding;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::{IonDecodeResult, IonDecoderBuilder, IonDecoderConfig};
    use crate::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
    use itertools::Itertools;

    use ion_rs_old::element::writer::TextKind;
    use ion_rs_old::element::{Element, IntoAnnotatedElement};
    use ion_rs_old::types::{Bytes, Sequence, Struct};
    use ion_rs_old::{Decimal, Int, IonType, Str, Timestamp};

    use partiql_value::{bag, list, tuple, DateTime, Value};
    use rust_decimal_macros::dec;
    use std::num::NonZeroU8;

    fn decode_ion_text(contents: &str, encoding: Encoding) -> IonDecodeResult {
        let reader = ion_rs_old::ReaderBuilder::new().build(contents)?;
        let mut iter = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(encoding))
            .build(reader)?;

        let val = iter.next();

        val.unwrap()
    }

    fn encode_ion_text(value: &Value, encoding: Encoding) -> Result<String, IonEncodeError> {
        let mut buff = vec![];
        let mut writer = ion_rs_old::TextWriterBuilder::new(TextKind::Compact)
            .build(&mut buff)
            .expect("writer");
        let mut encoder = IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(encoding))
            .build(&mut writer)?;

        encoder.write_value(value)?;

        drop(encoder);
        drop(writer);

        Ok(String::from_utf8(buff).expect("string"))
    }

    fn decode_ion_element(
        contents: ion_rs_old::element::Element,
        encoding: Encoding,
    ) -> IonDecodeResult {
        let reader = ion_rs_old::element::element_stream_reader::ElementStreamReader::new(contents);
        let mut iter = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(encoding))
            .build(reader)?;

        let val = iter.next();

        val.unwrap()
    }

    fn encode_ion_element(
        value: &Value,
        encoding: Encoding,
    ) -> Result<Vec<ion_rs_old::element::Element>, IonEncodeError> {
        let mut out = vec![];
        let mut writer =
            ion_rs_old::element::element_stream_writer::ElementStreamWriter::new(&mut out);
        let mut encoder = IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(encoding))
            .build(&mut writer)?;
        encoder.write_value(value)?;

        drop(encoder);
        drop(writer);

        Ok(out)
    }

    #[track_caller]
    fn assert_decode_encode(
        ion: &str,
        element: impl Into<ion_rs_old::element::Element>,
        val: impl Into<Value>,
        encoding: Encoding,
    ) {
        let expected_value = val.into();

        // decode text
        let decoded_ion_text = decode_ion_text(ion, encoding).expect("decode text expected");
        assert_eq!(decoded_ion_text, expected_value);

        // decode element
        let expected_element = element.into();
        let decoded_ion_element =
            decode_ion_element(expected_element, encoding).expect("decode element encoded");
        assert_eq!(decoded_ion_element, expected_value);

        // round-trip value through text
        let encoded_text_value =
            encode_ion_text(&expected_value, encoding).expect("encode to text");
        let decoded_encoded_text_value =
            decode_ion_text(&encoded_text_value, encoding).expect("decode of encode to text");
        assert_eq!(decoded_encoded_text_value, expected_value);

        // round-trip value through element
        let mut encoded_element_value =
            encode_ion_element(&expected_value, encoding).expect("encode to element");
        assert_eq!(encoded_element_value.len(), 1);
        let decoded_encoded_element_value =
            decode_ion_element(encoded_element_value.pop().unwrap(), encoding)
                .expect("decode of encode to element");
        assert_eq!(decoded_encoded_element_value, expected_value);
    }

    #[track_caller]
    fn assert_ion(
        ion: &str,
        element: impl Into<ion_rs_old::element::Element>,
        val: impl Into<Value>,
    ) {
        assert_decode_encode(ion, element, val, Encoding::Ion);
    }

    #[track_caller]
    fn assert_partiql_encoded_ion(
        ion: &str,
        element: impl Into<ion_rs_old::element::Element>,
        val: impl Into<Value>,
    ) {
        assert_decode_encode(ion, element, val, Encoding::PartiqlEncodedAsIon);
    }

    #[test]
    fn partiql_value_from_ion() {
        assert_ion(
            "null",
            ion_rs_old::element::Value::Null(IonType::Null),
            Value::Null,
        );

        // bool
        assert_ion("true", ion_rs_old::element::Value::Bool(true), true);
        assert_ion("false", ion_rs_old::element::Value::Bool(false), false);

        // int
        assert_ion("42", ion_rs_old::element::Value::Int(Int::I64(42)), 42);
        assert_ion("-5", ion_rs_old::element::Value::Int(Int::I64(-5)), -5);

        // float
        assert_ion("1.1e0", ion_rs_old::element::Value::Float(1.1), 1.1);

        // decimal
        assert_ion(
            "1.",
            ion_rs_old::element::Value::Decimal(Decimal::new(1, 0)),
            dec!(1),
        );

        // text
        assert_ion(
            "'foo'",
            ion_rs_old::element::Value::String(Str::from("foo")),
            "foo",
        );
        assert_ion(
            "\"foo\"",
            ion_rs_old::element::Value::String(Str::from("foo")),
            "foo",
        );

        // datetime
        assert_ion(
            "2017-01-01T01:02:03.4+00:30",
            ion_rs_old::element::Value::Timestamp(
                Timestamp::with_ymd_hms_millis(2017, 1, 1, 1, 2, 3, 400)
                    .build_at_offset(30)
                    .expect("ion timestamp"),
            ),
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400_000_000,
                Some(30),
            ),
        );
        assert_ion(
            "2017-01-01T01:02:03.4-00:00",
            ion_rs_old::element::Value::Timestamp(
                Timestamp::with_ymd_hms_millis(2017, 1, 1, 1, 2, 3, 400)
                    .build_at_unknown_offset()
                    .expect("ion timestamp"),
            ),
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400_000_000,
                None,
            ),
        );

        // lob
        assert_ion(
            "{{ +AB/ }}",
            ion_rs_old::element::Value::Blob(Bytes::from(vec![248, 0, 127])),
            Value::Blob(Box::new(vec![248, 0, 127])),
        );
        assert_ion(
            "{{ \"CLOB of text.\" }}",
            ion_rs_old::element::Value::Clob(Bytes::from("CLOB of text.")),
            Value::Blob(Box::new("CLOB of text.".bytes().collect_vec())),
        );

        // list
        assert_ion(
            "[1,2,\"3\"]",
            ion_rs_old::element::Value::List(Sequence::new([
                ion_rs_old::element::Value::Int(Int::I64(1)),
                ion_rs_old::element::Value::Int(Int::I64(2)),
                ion_rs_old::element::Value::String(Str::from("3")),
            ])),
            list![1, 2, "3"],
        );

        // struct
        assert_ion(
            "{\"k\": [1,2,3]}",
            ion_rs_old::element::Value::Struct(
                Struct::builder()
                    .with_field("k", ion_rs_old::element::List(Sequence::new([1, 2, 3])))
                    .build(),
            ),
            tuple![("k", list![1, 2, 3])],
        );
    }

    #[test]
    fn partiql_value_from_partiql_encoded_ion() {
        assert_partiql_encoded_ion(
            "null",
            ion_rs_old::element::Value::Null(IonType::Null),
            Value::Null,
        );
        assert_partiql_encoded_ion(
            "$missing::null",
            ion_rs_old::element::Value::Null(IonType::Null).with_annotations(["$missing"]),
            Value::Missing,
        );

        // bool
        assert_partiql_encoded_ion("true", ion_rs_old::element::Value::Bool(true), true);
        assert_partiql_encoded_ion("false", ion_rs_old::element::Value::Bool(false), false);

        // int
        assert_partiql_encoded_ion("42", ion_rs_old::element::Value::Int(Int::I64(42)), 42);
        assert_partiql_encoded_ion("-5", ion_rs_old::element::Value::Int(Int::I64(-5)), -5);

        // float
        assert_partiql_encoded_ion("1.1e0", ion_rs_old::element::Value::Float(1.1), 1.1);

        // decimal
        assert_partiql_encoded_ion(
            "1.",
            ion_rs_old::element::Value::Decimal(Decimal::new(1, 0)),
            dec!(1),
        );

        // text
        assert_partiql_encoded_ion(
            "'foo'",
            ion_rs_old::element::Value::String(Str::from("foo")),
            "foo",
        );
        assert_partiql_encoded_ion(
            "\"foo\"",
            ion_rs_old::element::Value::String(Str::from("foo")),
            "foo",
        );

        // datetime
        assert_partiql_encoded_ion(
            "2017-01-01T01:02:03.4+00:30",
            ion_rs_old::element::Value::Timestamp(
                Timestamp::with_ymd_hms_millis(2017, 1, 1, 1, 2, 3, 400)
                    .build_at_offset(30)
                    .expect("ion timestamp"),
            ),
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400_000_000,
                Some(30),
            ),
        );
        assert_partiql_encoded_ion(
            "2017-01-01T01:02:03.4-00:00",
            ion_rs_old::element::Value::Timestamp(
                Timestamp::with_ymd_hms_millis(2017, 1, 1, 1, 2, 3, 400)
                    .build_at_unknown_offset()
                    .expect("ion timestamp"),
            ),
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400_000_000,
                None,
            ),
        );
        assert_partiql_encoded_ion(
            "$time::{ hour: 12, minute: 11, second: 10.08}",
            ion_rs_old::element::Value::Struct(
                Struct::builder()
                    .with_fields([
                        ("hour", ion_rs_old::element::Value::Int(Int::I64(12))),
                        ("minute", ion_rs_old::element::Value::Int(Int::I64(11))),
                        ("second", ion_rs_old::element::Value::Float(10.08)),
                    ])
                    .build(),
            )
            .with_annotations(["$time"]),
            DateTime::from_hms_nano(12, 11, 10, 80_000_000),
        );
        assert_partiql_encoded_ion(
            "$time::{ hour: 12, minute: 11, second: 10.08, timezone_hour: 0, timezone_minute: 30}",
            ion_rs_old::element::Value::Struct(
                Struct::builder()
                    .with_fields([
                        ("hour", ion_rs_old::element::Value::Int(Int::I64(12))),
                        ("minute", ion_rs_old::element::Value::Int(Int::I64(11))),
                        ("second", ion_rs_old::element::Value::Float(10.08)),
                        (
                            "timezone_hour",
                            ion_rs_old::element::Value::Int(Int::I64(0)),
                        ),
                        (
                            "timezone_minute",
                            ion_rs_old::element::Value::Int(Int::I64(30)),
                        ),
                    ])
                    .build(),
            )
            .with_annotations(["$time"]),
            DateTime::from_hms_nano_tz(12, 11, 10, 80_000_000, None, Some(30)),
        );
        assert_partiql_encoded_ion(
            "$date::1957-05-25T",
            ion_rs_old::element::Value::Timestamp(
                Timestamp::with_ymd(1957, 5, 25)
                    .build()
                    .expect("ion timestamp"),
            )
            .with_annotations(["$date"]),
            DateTime::from_ymd(1957, NonZeroU8::new(5).unwrap(), 25),
        );

        // lob
        assert_partiql_encoded_ion(
            "{{ +AB/ }}",
            ion_rs_old::element::Value::Blob(Bytes::from(vec![248, 0, 127])),
            Value::Blob(Box::new(vec![248, 0, 127])),
        );
        assert_partiql_encoded_ion(
            "{{ \"CLOB of text.\" }}",
            ion_rs_old::element::Value::Clob(Bytes::from("CLOB of text.")),
            Value::Blob(Box::new("CLOB of text.".bytes().collect_vec())),
        );

        // list
        assert_partiql_encoded_ion(
            "[1,2,\"3\"]",
            ion_rs_old::element::Value::List(Sequence::new([
                ion_rs_old::element::Value::Int(Int::I64(1)),
                ion_rs_old::element::Value::Int(Int::I64(2)),
                ion_rs_old::element::Value::String(Str::from("3")),
            ])),
            list![1, 2, "3"],
        );

        // bag
        assert_partiql_encoded_ion(
            "$bag::[1,2,\"3\", null, $missing::null]",
            ion_rs_old::element::Value::List(Sequence::new::<Element, _>([
                ion_rs_old::element::Value::Int(Int::I64(1)).into(),
                ion_rs_old::element::Value::Int(Int::I64(2)).into(),
                ion_rs_old::element::Value::String(Str::from("3")).into(),
                ion_rs_old::element::Value::Null(IonType::Null).into(),
                ion_rs_old::element::Value::Null(IonType::Null).with_annotations(["$missing"]),
            ]))
            .with_annotations(["$bag"]),
            bag![1, 2, "3", Value::Null, Value::Missing],
        );

        // struct
        assert_partiql_encoded_ion(
            "{\"k\": []}",
            ion_rs_old::element::Value::Struct(
                Struct::builder()
                    .with_field(
                        "k",
                        ion_rs_old::element::List(Sequence::new::<Element, _>([])),
                    )
                    .build(),
            ),
            tuple![("k", list![])],
        );
        assert_partiql_encoded_ion(
            "{\"k\": [1,2,3]}",
            ion_rs_old::element::Value::Struct(
                Struct::builder()
                    .with_field("k", ion_rs_old::element::List(Sequence::new([1, 2, 3])))
                    .build(),
            ),
            tuple![("k", list![1, 2, 3])],
        );
    }
}
