use itertools::Itertools;
use partiql_extension_ion::boxed_ion::BoxedIonType;
use partiql_value::datum::{
    Datum, DatumCategory, DatumCategoryOwned, DatumCategoryRef, OwnedFieldView, SequenceDatum,
    TupleDatum,
};
use partiql_value::{EqualityValue, NullableEq, Value};
use std::io::{BufReader, Cursor};

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

fn read() -> Value {
    let data = include_bytes!("../resources/test/test.ion");
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
    result += &value.dump_datum_stats(indent + 2);

    match value.into_category() {
        DatumCategoryOwned::Null => {
            result += &format!("{:indent$}↳ NULL\n", "", indent = indent + 2)
        }
        DatumCategoryOwned::Missing => {
            result += &format!("{:indent$}↳ MISSING\n", "", indent = indent + 2)
        }
        DatumCategoryOwned::Tuple(tuple) => {
            result += &tuple.dump_tuple_stats(indent + 2);
            for OwnedFieldView { name, value } in tuple {
                result += &flatten_dump_owned(&format!("{name}: "), value, indent + 2);
            }
        }
        DatumCategoryOwned::Sequence(seq) => {
            result += &seq.dump_seq_stats(indent + 2);
            for child in seq {
                result += &flatten_dump_owned("", child, indent + 2);
            }
        }
        DatumCategoryOwned::Scalar(_) => {
            // N/A
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
    insta::assert_snapshot!(dump_owned(read()));
}

fn flatten_dump_ref(prefix: &str, value: Value, indent: usize) -> String {
    let mut result = if indent > 0 {
        format!("{:indent$}↳ {prefix}{value}\n", "")
    } else {
        format!("{prefix}{value}\n")
    };
    result += &value.dump_datum_stats(indent + 2);

    match value.category() {
        DatumCategoryRef::Null => result += &format!("{:indent$}↳ NULL\n", "", indent = indent + 2),
        DatumCategoryRef::Missing => {
            result += &format!("{:indent$}↳ MISSING\n", "", indent = indent + 2)
        }
        DatumCategoryRef::Tuple(tuple) => {
            result += &tuple.dump_tuple_stats(indent + 2);
            match value.into_category() {
                DatumCategoryOwned::Tuple(tuple) => {
                    for OwnedFieldView { name, value } in tuple {
                        result += &flatten_dump_ref(&format!("{name}: "), value, indent + 2);
                    }
                }
                _ => unreachable!(),
            }
        }
        DatumCategoryRef::Sequence(seq) => {
            result += &seq.dump_seq_stats(indent + 2);
            match value.into_category() {
                DatumCategoryOwned::Sequence(seq) => {
                    for child in seq {
                        result += &flatten_dump_ref("", child, indent + 2);
                    }
                }
                _ => unreachable!(),
            }
        }
        DatumCategoryRef::Scalar(_) => {
            // N/A
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
    insta::assert_snapshot!(dump_ref(read()));
}

fn dump_eq<const NULLS_EQUAL: bool, const NAN_EQUAL: bool>() -> String {
    let l: Vec<_> = match read().into_category() {
        DatumCategoryOwned::Sequence(seq) => seq.into_iter().collect(),
        _ => panic!("expected top level sequence"),
    };
    let r = l.clone();

    let mut result = String::new();
    let cartesian = l.into_iter().cartesian_product(r);
    for (left, right) in cartesian {
        let leq = EqualityValue::<'_, NULLS_EQUAL, NAN_EQUAL, _>(&left);

        let req = EqualityValue::<'_, NULLS_EQUAL, NAN_EQUAL, _>(&right);
        let eq_res = NullableEq::eq(&leq, &req);

        // Only print when equal
        if eq_res == Value::Boolean(true) {
            result += &format!("{left} = {right} → {eq_res}\n")
        }
    }

    result
}

#[test]
fn all_types_eq_nulls_eq_nans_eq() {
    insta::assert_snapshot!(dump_eq::<true, true>());
}
#[test]
fn all_types_eq_nulls_eq_nans_neq() {
    insta::assert_snapshot!(dump_eq::<true, false>());
}
#[test]
fn all_types_eq_nulls_neq_nans_eq() {
    insta::assert_snapshot!(dump_eq::<false, true>());
}
#[test]
fn all_types_eq_nulls_neq_nans_neq() {
    insta::assert_snapshot!(dump_eq::<false, false>());
}
