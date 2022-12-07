use ion_rs::external::bigdecimal::BigDecimal;
use ion_rs::{Integer, IonReader, IonType, Reader, StreamItem};
use partiql_value::{Bag, List, Tuple, Value};
use std::fs::read;
use std::str::FromStr;

pub(crate) struct TestValue {
    value: Value,
}

impl From<&str> for TestValue {
    fn from(contents: &str) -> Self {
        TestValue {
            value: parse_test_value_str(contents),
        }
    }
}

fn parse_test_value_str(contents: &str) -> Value {
    let mut reader = ion_rs::ReaderBuilder::new()
        .build(contents)
        .expect("reading contents");

    // expecting a single top-level value
    let item = reader.next().expect("test value");
    let val = match item {
        StreamItem::Value(typ) => parse_test_value(&mut reader, typ),
        StreamItem::Null(_) => parse_null(&reader),
        StreamItem::Nothing => panic!("expecting a test value"),
    };

    assert_eq!(reader.next().expect("test end"), StreamItem::Nothing);

    val
}

#[inline]
fn parse_null(reader: &Reader) -> Value {
    if has_annotation(reader, "$missing") {
        Value::Missing
    } else {
        Value::Null
    }
}

#[inline]
fn has_annotation(reader: &Reader, annot: &str) -> bool {
    reader.annotations().any(|a| a.unwrap().eq(&annot))
}

// TODO handle errors more gracefully than `expect`/`unwrap`
fn parse_test_value(reader: &mut Reader, typ: IonType) -> Value {
    match typ {
        IonType::Null => parse_null(reader),
        IonType::Boolean => Value::Boolean(reader.read_bool().unwrap()),
        IonType::Integer => match reader.read_integer().unwrap() {
            Integer::I64(i) => Value::Integer(i),
            Integer::BigInt(_) => todo!("bigint"),
        },
        IonType::Float => Value::Real(reader.read_f64().unwrap().into()),
        IonType::Decimal => {
            // TODO ion Decimal doesn't give a lot of functionality to get at the data currently
            // TODO    and it's not clear whether we'll continue with rust decimal or switch to big decimal
            let ion_dec = reader.read_decimal().unwrap();
            let ion_dec_str = format!("{}", ion_dec).replace('d', "e");
            Value::Decimal(rust_decimal::Decimal::from_scientific(&ion_dec_str).unwrap())
        }
        IonType::Timestamp => todo!("timestamp"),
        IonType::Symbol => Value::String(Box::new(reader.read_symbol().unwrap().to_string())),
        IonType::String => Value::String(Box::new(reader.read_string().unwrap())),
        IonType::Clob => todo!("clob"),
        IonType::Blob => Value::Blob(Box::new(reader.read_blob().unwrap())),
        IonType::List => {
            if has_annotation(reader, "$bag") {
                Bag::from(parse_test_value_sequence(reader)).into()
            } else {
                List::from(parse_test_value_sequence(reader)).into()
            }
        }
        IonType::SExpression => todo!("sexp"),
        IonType::Struct => parse_test_value_tuple(reader).into(),
    }
}

fn parse_test_value_tuple(reader: &mut Reader) -> Tuple {
    let mut tuple = Tuple::new();
    reader.step_in().expect("step into struct");
    while let item = reader.next().expect("struct value") {
        let (key, value) = match item {
            StreamItem::Value(typ) => (
                reader.field_name().expect("field name"),
                parse_test_value(reader, typ),
            ),
            StreamItem::Null(_) => (
                reader.field_name().expect("field name"),
                parse_null(&reader),
            ),
            StreamItem::Nothing => break,
        };
        tuple.insert(key.text().unwrap(), value);
    }
    reader.step_out().expect("step out of struct");
    tuple
}

fn parse_test_value_sequence(reader: &mut Reader) -> Vec<Value> {
    reader.step_in().expect("step into sequence");
    let mut values = vec![];
    loop {
        let item = reader.next().expect("test value");
        let val = match item {
            StreamItem::Value(typ) => parse_test_value(reader, typ),
            StreamItem::Null(_) => parse_null(&reader),
            StreamItem::Nothing => break,
        };
        values.push(val);
    }
    reader.step_out().expect("step out of sequence");
    values
}

#[cfg(test)]
#[cfg(not(feature = "conformance_test"))]
mod tests {
    use super::parse_test_value_str;
    use ion_rs::Decimal;
    use partiql_value::{partiql_bag, partiql_list, partiql_tuple, Bag, List, Tuple, Value};

    #[track_caller]
    fn parse(test: &str, expected: Value) {
        let val = parse_test_value_str(test);
        assert_eq!(val, expected);
    }

    #[test]
    fn simple() {
        parse("null", Value::Null);
        parse("$missing::null", Value::Missing);
        parse("9", Value::Integer(9));
        parse("true", Value::Boolean(true));
        parse("false", Value::Boolean(false));
        parse("\"str\"", Value::String(Box::new("str".into())));
    }

    #[test]
    fn bag() {
        let test = "$bag::[
            {
                f: 1,
                d: 2e0,
                s: 1
            }
        ]";
        let expected = Value::from(partiql_bag![partiql_tuple![
            ("f", 1),
            ("d", Value::Real(2.0.into())),
            ("s", 1)
        ]]);
    }

    #[test]
    fn tuple() {
        parse(
            "{
                    sensor: 1,
                    reading: 42
                  }",
            Value::Tuple(Box::new(partiql_tuple![("sensor", 1), ("reading", 42)])),
        );
    }

    #[test]
    fn tt2() {
        parse(
            "{
                    sensors: [
                        {
                            sensor: 1
                        },
                        {
                            sensor: 2
                        }
                    ],
                    logs: [
                        {
                            sensor: 1,
                            co: 4d-1
                        },
                        {
                            sensor: 1,
                            co: 2d-1
                        },
                        {
                            sensor: 2,
                            co: 3d-1
                        }
                    ]
                }",
            Value::Tuple(Box::new(partiql_tuple![
                (
                    "sensors",
                    partiql_list![partiql_tuple![("sensor", 1)], partiql_tuple![("sensor", 2)]]
                ),
                (
                    "logs",
                    partiql_list![
                        partiql_tuple![("sensor", 1), ("co", rust_decimal::Decimal::new(4, 1))],
                        partiql_tuple![("sensor", 1), ("co", rust_decimal::Decimal::new(2, 1))],
                        partiql_tuple![("sensor", 2), ("co", rust_decimal::Decimal::new(3, 1))]
                    ]
                )
            ])),
        );
    }

    #[test]
    fn list() {
        let test = "[
            {
                f: 1,
                d: 2e0,
                s: 1
            }
        ]";
        let expected = Value::from(partiql_list![partiql_tuple![
            ("f", 1),
            ("d", Value::Real(2.0.into())),
            ("s", 1)
        ]]);
    }
}
