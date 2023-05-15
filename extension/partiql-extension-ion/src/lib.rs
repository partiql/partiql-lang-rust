use ordered_float::OrderedFloat;
use std::cmp::Ordering;

use std::borrow::Cow;

use std::fmt::{Debug, Formatter};
use std::hash::Hash;

use std::{ops, vec};

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal as RustDecimal, Decimal};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod common;
pub mod decode;
pub mod encode;

pub use common::Encoding;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::{IonDecodeError, IonDecodeResult, IonDecoderBuilder, IonDecoderConfig};
    use crate::encode::{
        IonEncodeError, IonEncodeResult, IonEncoderBuilder, IonEncoderConfig, IonValueEncoder,
    };
    use partiql_value::{partiql_bag, partiql_list, partiql_tuple, Value};
    use rust_decimal_macros::dec;
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::mem;
    use std::rc::Rc;

    pub fn decode_ion(contents: &str, encoding: Encoding) -> IonDecodeResult {
        let mut reader = ion_rs::ReaderBuilder::new().build(contents)?;
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

        encoder.encode_value(&value)?;

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
        assert_decode_encode(&ion, val, Encoding::Ion);
    }

    #[track_caller]
    fn assert_partiql_encoded_ion(ion: &str, val: impl Into<Value>) {
        assert_decode_encode(&ion, val, Encoding::PartiqlEncodedAsIon);
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

        // decimal
        assert_ion("1.", dec!(1));

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

        // decimal
        assert_partiql_encoded_ion("1.", dec!(1));

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
