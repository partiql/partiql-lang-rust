use crate::{List, Tuple, Value};
use ion_rs::{Integer, IonReader, IonType, Reader, StreamItem};

// TODO handle errors more gracefully than `expect`/`unwrap`

pub(crate) fn parse_ion(contents: &str) -> Value {
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
        IonType::Null => Value::Null,
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
        IonType::Symbol => Value::String(Box::new(
            reader
                .read_symbol()
                .unwrap()
                .text()
                .unwrap_or("")
                .to_string(),
        )),
        IonType::String => Value::String(Box::new(reader.read_string().unwrap())),
        IonType::Clob => todo!("clob"),
        IonType::Blob => Value::Blob(Box::new(reader.read_blob().unwrap())),
        IonType::List => List::from(parse_sequence(reader)).into(),
        IonType::SExpression => todo!("sexp"),
        IonType::Struct => parse_tuple(reader).into(),
    }
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
