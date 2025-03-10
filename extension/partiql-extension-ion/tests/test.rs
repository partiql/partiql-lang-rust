use ion_rs::Element;
use ion_rs_old::element::writer::TextKind;
use itertools::Itertools;
use partiql_common::pretty::ToPretty;
use partiql_extension_ion::boxed_ion::BoxedIonType;

use partiql_extension_ion::decode::{IonDecodeResult, IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding;
use partiql_value::datum::{
    Datum, DatumCategory, DatumCategoryOwned, DatumCategoryRef, DatumLower, OwnedFieldView,
    OwnedSequenceView, OwnedTupleView, RefSequenceView, RefTupleView, SequenceDatum, TupleDatum,
};
use partiql_value::{Bag, BindingsName, EqualityValue, NullableEq, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::ops::Not;
use std::path::PathBuf;
use walkdir::WalkDir;

trait DumpDatumStats {
    fn dump_datum_stats(&self, indent: usize) -> String;
}

trait DumpSeqStats {
    fn dump_seq_stats(&self, indent: usize) -> String;
}

trait DumpTupleStats {
    fn dump_tuple_stats(&self, indent: usize) -> String;
}

impl<D: Datum<Value>> DumpDatumStats for D {
    fn dump_datum_stats(&self, indent: usize) -> String {
        format!(
            "{:indent$}「is_null:{is_null}, is_missing:{is_missing}, is_absent:{is_absent}, is_present:{is_present}, is_sequence:{is_sequence}, is_ordered:{is_ordered}」\n",
            "",
            is_null = self.is_null(),
            is_missing = self.is_missing(),
            is_absent = self.is_absent(),
            is_present = self.is_present(),
            is_sequence = self.is_sequence(),
            is_ordered = self.is_ordered(),
        )
    }
}

impl<T: TupleDatum> DumpTupleStats for T {
    fn dump_tuple_stats(&self, indent: usize) -> String {
        format!(
            "{:indent$}「len:{len}, is_empty:{is_empty}」\n",
            "",
            len = self.len(),
            is_empty = self.is_empty()
        )
    }
}

impl<S: SequenceDatum> DumpSeqStats for S {
    fn dump_seq_stats(&self, indent: usize) -> String {
        format!(
            "{:indent$}「is_ordered:{is_ordered}, len:{len}, is_empty:{is_empty}」\n",
            "",
            is_ordered = self.is_ordered(),
            len = self.len(),
            is_empty = self.is_empty()
        )
    }
}

#[derive(Debug)]
enum IonDataFormat {
    StreamText,
    StreamBinary,
    SingleTlvText,
    SingleTlvBinary,
}

fn read(fmt: IonDataFormat) -> Value {
    let data: Vec<u8> = include_bytes!("../resources/test/test.ion").into();
    let data = match fmt {
        IonDataFormat::StreamText => data,
        IonDataFormat::StreamBinary => Element::read_all(data.as_slice())
            .expect("read all")
            .encode_as(ion_rs::v1_0::Binary)
            .expect("encode"),
        IonDataFormat::SingleTlvText => {
            let seq = Element::read_all(data.as_slice()).expect("read all");
            let list: Element = ion_rs::Value::List(seq).into();

            list.encode_as(ion_rs::v1_0::Text).expect("encode").into()
        }
        IonDataFormat::SingleTlvBinary => {
            let seq = Element::read_all(data.as_slice()).expect("read all");
            let list: Element = ion_rs::Value::List(seq).into();

            list.encode_as(ion_rs::v1_0::Binary).expect("encode")
        }
    };
    let buf = BufReader::new(Cursor::new(data));
    let ion = BoxedIonType {}
        .stream_from_read(buf)
        .expect("Failed to read");
    ion.into_value()
}

fn flatten_dump_owned(prefix: &str, value: Value, indent: usize) -> String {
    let mut result = if indent > 0 {
        format!("{:indent$}↳ {prefix}{value}\n", "")
    } else {
        format!("{prefix}{value}\n")
    };

    let value_datum_stats = value.dump_datum_stats(indent + 2);

    let value2 = value.clone();
    match value.into_category() {
        DatumCategoryOwned::Null => {
            result += &value_datum_stats;
            result += &format!("{:indent$}↳ NULL\n", "", indent = indent + 2)
        }
        DatumCategoryOwned::Missing => {
            result += &value_datum_stats;
            result += &format!("{:indent$}↳ MISSING\n", "", indent = indent + 2)
        }
        DatumCategoryOwned::Tuple(tuple) => {
            result += &value_datum_stats;
            result += &tuple.dump_tuple_stats(indent + 2);

            let mut found: HashMap<String, [Vec<_>; 2]> = HashMap::default();
            for OwnedFieldView { name, value } in tuple {
                let entry = found.entry(name.clone()).or_default();

                entry[0].push(value.clone());
                result += &flatten_dump_owned(&format!("{name}: "), value, indent + 2);

                let taken_value = match value2.clone().into_category() {
                    DatumCategoryOwned::Tuple(tuple) => tuple
                        .take_val(&BindingsName::CaseSensitive(name.clone().into()))
                        .expect("value"),
                    _ => unreachable!(),
                };
                entry[1].push(taken_value);
            }

            // assert that all 'taken' values (e.g. those reached via `.key` pathing) are also
            // found via iteration of fields. Note that iteration of fields may find more values
            // in the case that a key is duplicated in the struct/tuple
            for (_, [iterated, taken]) in found {
                if iterated.len() == 1 {
                    // no duplicate keys
                    assert_eq!(iterated[0], taken[0]);
                } else {
                    // the tuple had duplicated keys, iteration order is not specified
                    // assert that 'taken' values are a subset of iterated values
                    for v in taken {
                        assert!(iterated.contains(&v));
                    }
                }
            }
        }
        DatumCategoryOwned::Sequence(seq) => {
            result += &value_datum_stats;
            result += &seq.dump_seq_stats(indent + 2);
            for (idx, child) in seq.into_iter().enumerate() {
                result += &flatten_dump_owned("", child.clone(), indent + 2);

                let taken_value = match value2.clone().into_category() {
                    DatumCategoryOwned::Sequence(seq) => seq.take_val(idx as i64).expect("value"),
                    _ => unreachable!(),
                };
                assert_eq!(child, taken_value);
            }
        }
        DatumCategoryOwned::Scalar(v) => {
            let datum_owned_value_stats = v.dump_datum_stats(indent + 2);
            assert_eq!(value_datum_stats, datum_owned_value_stats);
            let datum_lowered_value_stats = v.lower().unwrap().dump_datum_stats(indent + 2);
            assert_eq!(datum_lowered_value_stats, datum_owned_value_stats);
            result += &datum_owned_value_stats;
        }
    }
    result
}
fn dump_owned(ion: Value) -> String {
    let cat = ion.into_category();
    match cat {
        DatumCategoryOwned::Sequence(seq) => seq
            .into_iter()
            .map(|v| flatten_dump_owned("", v, 0))
            .collect(),
        _ => panic!("expected top level sequence"),
    }
}

#[test]
fn all_types_owned() {
    insta::assert_snapshot!(dump_owned(read(IonDataFormat::StreamText)));
}

fn flatten_dump_ref(prefix: &str, value: Value, indent: usize) -> String {
    let mut result = if indent > 0 {
        format!("{:indent$}↳ {prefix}{value}\n", "")
    } else {
        format!("{prefix}{value}\n")
    };
    let value_datum_stats = value.dump_datum_stats(indent + 2);

    match value.category() {
        DatumCategoryRef::Null => {
            result += &value_datum_stats;
            result += &format!("{:indent$}↳ NULL\n", "", indent = indent + 2)
        }
        DatumCategoryRef::Missing => {
            result += &value_datum_stats;
            result += &format!("{:indent$}↳ MISSING\n", "", indent = indent + 2)
        }
        DatumCategoryRef::Tuple(tuple) => {
            result += &value_datum_stats;
            result += &tuple.dump_tuple_stats(indent + 2);
            match value.clone().into_category() {
                DatumCategoryOwned::Tuple(tuple_owned) => {
                    let mut found: HashMap<String, (Vec<_>, Vec<_>)> = HashMap::default();
                    for OwnedFieldView { name, value } in tuple_owned {
                        let entry = found.entry(name.clone()).or_default();

                        entry.0.push(value.clone());
                        result += &flatten_dump_ref(&format!("{name}: "), value, indent + 2);

                        let get_val = tuple
                            .get_val(&BindingsName::CaseSensitive(name.clone().into()))
                            .expect("value");
                        entry.1.push(get_val);
                    }

                    // assert that all 'taken' values (e.g. those reached via `.key` pathing) are also
                    // found via iteration of fields. Note that iteration of fields may find more values
                    // in the case that a key is duplicated in the struct/tuple
                    for (_, (iterated, taken)) in found {
                        if iterated.len() == 1 {
                            // no duplicate keys
                            assert_eq!(&iterated[0], taken[0].as_ref());
                        } else {
                            // the tuple had duplicated keys, iteration order is not specified
                            // assert that 'taken' values are a subset of iterated values
                            for v in taken {
                                assert!(iterated.contains(&v));
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        DatumCategoryRef::Sequence(seq) => {
            result += &value_datum_stats;
            result += &seq.dump_seq_stats(indent + 2);

            match value.clone().into_category() {
                DatumCategoryOwned::Sequence(seq_owned) => {
                    for (idx, child) in seq_owned.into_iter().enumerate() {
                        let get_value = seq.get_val(idx as i64).expect("get_val");
                        assert_eq!(&child, get_value.as_ref());
                        result += &flatten_dump_ref("", child, indent + 2);
                    }
                }
                _ => unreachable!(),
            }
        }
        DatumCategoryRef::Scalar(v) => {
            let lowered = match value.clone().into_category() {
                DatumCategoryOwned::Scalar(v) => v.into_lower(),
                _ => unreachable!(),
            };
            let datum_owned_value_stats = v.dump_datum_stats(indent + 2);
            assert_eq!(value_datum_stats, datum_owned_value_stats);
            let datum_lowered_value_stats = lowered.unwrap().dump_datum_stats(indent + 2);
            assert_eq!(datum_lowered_value_stats, datum_owned_value_stats);
            result += &datum_owned_value_stats;
        }
    }
    result
}
fn dump_ref(ion: Value) -> String {
    let cat = ion.into_category();
    match cat {
        DatumCategoryOwned::Sequence(seq) => seq
            .into_iter()
            .map(|v| crate::flatten_dump_ref("", v, 0))
            .collect(),
        _ => panic!("expected top level sequence"),
    }
}

#[test]
fn all_types_ref() {
    insta::assert_snapshot!(dump_ref(read(IonDataFormat::StreamText)));
}

fn dump_eq<const NULLS_EQUAL: bool, const NAN_EQUAL: bool>(fmt: IonDataFormat) -> String {
    let l: Vec<_> = match read(fmt).into_category() {
        DatumCategoryOwned::Sequence(seq) => seq.into_iter().collect(),
        _ => panic!("expected top level sequence"),
    };
    let r = l.clone();

    let mut result = String::new();
    let cartesian = l.into_iter().cartesian_product(r);
    for (left, right) in cartesian {
        let leq = EqualityValue::<'_, NULLS_EQUAL, NAN_EQUAL, _>(&left);
        let req = EqualityValue::<'_, NULLS_EQUAL, NAN_EQUAL, _>(&right);

        // eq
        let eq_res = NullableEq::eq(&leq, &req);
        let neq_res = NullableEq::neq(&leq, &req);
        assert_eq!(eq_res, neq_res.not());

        //eqg
        let eqg_res = NullableEq::eqg(&leq, &req);
        let neqg_res = NullableEq::neqg(&leq, &req);
        assert_eq!(eqg_res, neqg_res.not());

        // eqg always allows NULL=NULL
        if NULLS_EQUAL {
            assert_eq!(eq_res, eqg_res);
        }

        // Only print when equal
        if eq_res == Value::Boolean(true) {
            result += &format!("{left} = {right} → {eq_res}\n")
        }
    }

    result
}

#[test]
fn all_types_eq_nulls_eq_nans_eq() {
    // There are some slight unexpected behavior in this equality
    // Check https://github.com/amazon-ion/ion-rust/issues/903 for fixes
    for fmt in [
        IonDataFormat::StreamText,
        IonDataFormat::StreamBinary,
        IonDataFormat::SingleTlvText,
        IonDataFormat::SingleTlvBinary,
    ] {
        insta::assert_snapshot!(
            format!("nulls_eq_nans_eq_{fmt:?}"),
            dump_eq::<true, true>(fmt)
        );
    }
}
#[test]
fn all_types_eq_nulls_eq_nans_neq() {
    // There are some slight unexpected behavior in this equality
    // Check https://github.com/amazon-ion/ion-rust/issues/903 for fixes
    for fmt in [
        IonDataFormat::StreamText,
        IonDataFormat::StreamBinary,
        IonDataFormat::SingleTlvText,
        IonDataFormat::SingleTlvBinary,
    ] {
        insta::assert_snapshot!(
            format!("nulls_eq_nans_neq_{fmt:?}"),
            dump_eq::<true, false>(fmt)
        );
    }
}
#[test]
fn all_types_eq_nulls_neq_nans_eq() {
    // There are some slight unexpected behavior in this equality
    // Check https://github.com/amazon-ion/ion-rust/issues/903 for fixes
    for fmt in [
        IonDataFormat::StreamText,
        IonDataFormat::StreamBinary,
        IonDataFormat::SingleTlvText,
        IonDataFormat::SingleTlvBinary,
    ] {
        insta::assert_snapshot!(
            format!("nulls_neq_nans_eq_{fmt:?}"),
            dump_eq::<false, true>(fmt)
        );
    }
}
#[test]
fn all_types_eq_nulls_neq_nans_neq() {
    // There are some slight unexpected behavior in this equality
    // Check https://github.com/amazon-ion/ion-rust/issues/903 for fixes
    for fmt in [
        IonDataFormat::StreamText,
        IonDataFormat::StreamBinary,
        IonDataFormat::SingleTlvText,
        IonDataFormat::SingleTlvBinary,
    ] {
        insta::assert_snapshot!(
            format!("nulls_neq_nans_neq_{fmt:?}"),
            dump_eq::<false, false>(fmt)
        );
    }
}

fn decode_ion_text(contents: &str, encoding: Encoding) -> IonDecodeResult {
    let reader = ion_rs_old::ReaderBuilder::new().build(contents)?;
    let mut iter =
        IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(encoding)).build(reader)?;

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

#[test]
fn roundtrip() {
    let data = read(IonDataFormat::StreamText);
    let bag: Bag = data.into_iter().collect();
    let val = Value::from(bag);

    let encoded = encode_ion_text(&val, Encoding::PartiqlEncodedAsIon).unwrap();
    let round_tripped = decode_ion_text(&encoded, Encoding::PartiqlEncodedAsIon).unwrap();

    let original = val.to_pretty_string(80).unwrap();
    let round_tripped = round_tripped.to_pretty_string(80).unwrap();
    assert_eq!(original, round_tripped);
}

#[test]
fn verify_ion_tests() {
    let mut result = String::new();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/ion-tests/iontestdata/good");
    let root = path.display().to_string();
    for entry in WalkDir::new(path).sort_by_file_name() {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let epath = entry.path().display().to_string();
            let epath = epath.strip_prefix(&root).unwrap().replace("\\", "/");
            result += "\n=====================\n";
            result += epath.as_str();
            result += "\n---------------------\n";

            let mut data = Vec::new();
            File::open(entry.path())
                .unwrap()
                .read_to_end(&mut data)
                .unwrap();
            let ion_type = Box::new(BoxedIonType {});
            let boxed = partiql_value::Variant::new(data, ion_type);

            let boxed = match boxed {
                Ok(boxed) => boxed.to_pretty_string(80).unwrap(),
                Err(e) => {
                    format!("Err: `{e}`")
                }
            };

            result += &boxed;
            result += "\n=====================\n";
        }
    }

    insta::assert_snapshot!(result);
}
