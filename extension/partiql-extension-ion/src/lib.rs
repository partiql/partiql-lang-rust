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

    use partiql_value::{partiql_bag, partiql_list, partiql_tuple, DateTime, Value};
    use rust_decimal_macros::dec;
    use std::num::NonZeroU8;

    fn decode_ion(contents: &str, encoding: Encoding) -> IonDecodeResult {
        let reader = ion_rs::ReaderBuilder::new().build(contents)?;
        let mut iter = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(encoding))
            .build(reader)?;

        let val = iter.next();

        val.unwrap()
    }

    fn encode_ion(value: &Value, encoding: Encoding) -> Result<String, IonEncodeError> {
        let mut buff = vec![];
        let mut writer = ion_rs::TextWriterBuilder::new()
            .build(&mut buff)
            .expect("writer");
        let mut encoder = IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(encoding))
            .build(&mut writer)?;

        encoder.write_value(value)?;

        drop(encoder);
        drop(writer);

        Ok(String::from_utf8(buff).expect("string"))
    }

    #[track_caller]
    fn assert_decode_encode(ion: &str, val: impl Into<Value>, encoding: Encoding) {
        let expected_value = val.into();

        let decoded_ion = decode_ion(ion, encoding).expect("decode expected");
        let encoded_ion = encode_ion(&expected_value, encoding).expect("encode expected");
        let decoded_encoded = decode_ion(&encoded_ion, encoding).expect("decode encoded");
        assert_eq!(decoded_ion, expected_value);
        assert_eq!(decoded_encoded, expected_value);
    }

    #[track_caller]
    fn assert_ion(ion: &str, val: impl Into<Value>) {
        assert_decode_encode(ion, val, Encoding::Ion);
    }

    #[track_caller]
    fn assert_partiql_encoded_ion(ion: &str, val: impl Into<Value>) {
        assert_decode_encode(ion, val, Encoding::PartiqlEncodedAsIon);
    }

    #[test]
    fn partiql_value_from_ion() {
        assert_ion("null", Value::Null);

        // bool
        assert_ion("true", true);
        assert_ion("false", false);

        // int
        assert_ion("42", 42);
        assert_ion("-5", -5);

        // float
        assert_ion("1.1e0", 1.1);

        // decimal
        assert_ion("1.", dec!(1));

        // text
        assert_ion("'foo'", "foo");
        assert_ion("\"foo\"", "foo");

        // datetime
        assert_ion(
            "2017-01-01T01:02:03.4+00:30",
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400000000,
                Some(30),
            ),
        );
        assert_ion(
            "2017-01-01T01:02:03.4-00:00",
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400000000,
                None,
            ),
        );

        // lob
        assert_ion("{{ +AB/ }}", Value::Blob(Box::new(vec![248, 0, 127])));
        assert_ion(
            "{{ \"CLOB of text.\" }}",
            Value::Blob(Box::new("CLOB of text.".bytes().collect_vec())),
        );

        // list
        assert_ion("[1,2,\"3\"]", partiql_list![1, 2, "3"]);

        // struct
        assert_ion("{\"k\": []}", partiql_tuple![("k", partiql_list![])]);
    }

    #[test]
    fn partiql_value_from_partiql_encoded_ion() {
        assert_partiql_encoded_ion("null", Value::Null);
        assert_partiql_encoded_ion("$missing::null", Value::Missing);

        // bool
        assert_partiql_encoded_ion("true", true);
        assert_partiql_encoded_ion("false", false);

        // int
        assert_partiql_encoded_ion("42", 42);
        assert_partiql_encoded_ion("-5", -5);

        // float
        assert_partiql_encoded_ion("1.1e0", 1.1);

        // decimal
        assert_partiql_encoded_ion("1.", dec!(1));

        // text
        assert_partiql_encoded_ion("'foo'", "foo");
        assert_partiql_encoded_ion("\"foo\"", "foo");

        // datetime
        assert_partiql_encoded_ion(
            "2017-01-01T01:02:03.4+00:30",
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400000000,
                Some(30),
            ),
        );
        assert_partiql_encoded_ion(
            "2017-01-01T01:02:03.4-00:00",
            DateTime::from_ymdhms_nano_offset_minutes(
                2017,
                NonZeroU8::new(1).unwrap(),
                1,
                1,
                2,
                3,
                400000000,
                None,
            ),
        );
        assert_partiql_encoded_ion(
            "$time::{ hour: 12, minute: 11, second: 10.08}",
            DateTime::from_hms_nano(12, 11, 10, 80000000),
        );
        assert_partiql_encoded_ion(
            "$time::{ hour: 12, minute: 11, second: 10.08, timezone_hour: 0, timezone_minute: 30}",
            DateTime::from_hms_nano_tz(12, 11, 10, 80000000, None, Some(30)),
        );
        assert_partiql_encoded_ion(
            "$date::1957-05-25T",
            DateTime::from_ymd(1957, NonZeroU8::new(5).unwrap(), 25),
        );

        // lob
        assert_partiql_encoded_ion("{{ +AB/ }}", Value::Blob(Box::new(vec![248, 0, 127])));
        assert_partiql_encoded_ion(
            "{{ \"CLOB of text.\" }}",
            Value::Blob(Box::new("CLOB of text.".bytes().collect_vec())),
        );

        // list
        assert_partiql_encoded_ion("[1,2,\"3\"]", partiql_list![1, 2, "3"]);

        // bag
        assert_partiql_encoded_ion(
            "$bag::[1,2,\"3\", null, $missing::null]",
            partiql_bag![1, 2, "3", Value::Null, Value::Missing],
        );

        // struct
        assert_partiql_encoded_ion("{\"k\": []}", partiql_tuple![("k", partiql_list![])]);
    }
}
