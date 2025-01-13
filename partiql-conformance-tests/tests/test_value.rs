use partiql_eval::eval::Evaluated;
use partiql_value::{EqualityValue, NullableEq, Value};

use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;

#[allow(dead_code)]
#[derive(Debug, Ord, PartialOrd)]
pub(crate) struct TestValue {
    pub value: Value,
}

impl Eq for TestValue {}

impl PartialEq for TestValue {
    fn eq(&self, other: &Self) -> bool {
        let wrap_value = EqualityValue::<'_, true, true, Value>;
        NullableEq::eq(&wrap_value(&self.value), &wrap_value(&other.value)) == Value::Boolean(true)
    }
}

impl From<Value> for TestValue {
    fn from(value: Value) -> Self {
        TestValue { value }
    }
}

impl From<Evaluated> for TestValue {
    fn from(value: Evaluated) -> Self {
        value.result.into()
    }
}

impl From<&str> for TestValue {
    fn from(contents: &str) -> Self {
        parse_test_value_str(contents).into()
    }
}

fn parse_test_value_str(contents: &str) -> Value {
    let reader = ion_rs_old::ReaderBuilder::new()
        .build(contents)
        .expect("reading contents");
    let mut iter = IonDecoderBuilder::new(
        IonDecoderConfig::default().with_mode(Encoding::PartiqlEncodedAsIon),
    )
    .build(reader)
    .expect("building decoder");

    let val = iter.next();

    val.expect("test value to exist")
        .expect("value decode to succeed")
}

#[cfg(test)]
#[cfg(not(feature = "conformance_test"))]
mod tests {
    use super::parse_test_value_str;

    use partiql_value::{bag, list, tuple, Value};

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
        let expected = Value::from(bag![tuple![
            ("f", 1),
            ("d", Value::Real(2.0.into())),
            ("s", 1)
        ]]);
        parse(test, expected);
    }

    #[test]
    fn tuple() {
        parse(
            "{
                    sensor: 1,
                    reading: 42
                  }",
            Value::Tuple(Box::new(tuple![("sensor", 1), ("reading", 42)])),
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
            Value::Tuple(Box::new(tuple![
                (
                    "sensors",
                    list![tuple![("sensor", 1)], tuple![("sensor", 2)]]
                ),
                (
                    "logs",
                    list![
                        tuple![("sensor", 1), ("co", rust_decimal::Decimal::new(4, 1))],
                        tuple![("sensor", 1), ("co", rust_decimal::Decimal::new(2, 1))],
                        tuple![("sensor", 2), ("co", rust_decimal::Decimal::new(3, 1))]
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
        let expected = Value::from(list![tuple![
            ("f", 1),
            ("d", Value::Real(2.0.into())),
            ("s", 1)
        ]]);
        parse(test, expected);
    }
}
