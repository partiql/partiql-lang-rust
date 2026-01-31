use super::internal::ValueRef;
use super::value_owned::ValueOwned;

pub struct RegisterReader<'a> {
    pub(crate) slots: &'a [ValueRef<'a>],
}

impl<'a> RegisterReader<'a> {
    pub(crate) fn new(slots: &'a [ValueRef<'a>]) -> Self {
        RegisterReader { slots }
    }

    pub fn get_i64(&self, col: usize) -> Option<i64> {
        self.slots.get(col).and_then(|v| v.as_i64().ok())
    }

    pub fn get_str(&self, col: usize) -> Option<&'a str> {
        self.slots.get(col).and_then(|v| match v {
            ValueRef::Str(s) => Some(*s),
            _ => None,
        })
    }

    pub fn get_value(&self, col: usize) -> ValueOwned {
        self.slots
            .get(col)
            .map(|v| match v {
                ValueRef::Missing => ValueOwned::from(partiql_value::Value::Missing),
                ValueRef::Null => ValueOwned::from(partiql_value::Value::Null),
                ValueRef::Bool(b) => ValueOwned::from(partiql_value::Value::Boolean(*b)),
                ValueRef::I64(i) => ValueOwned::from(partiql_value::Value::Integer(*i)),
                ValueRef::F64(f) => ValueOwned::from(partiql_value::Value::Real((*f).into())),
                ValueRef::Str(s) => {
                    ValueOwned::from(partiql_value::Value::String(Box::new(s.to_string())))
                }
                ValueRef::Bytes(b) => {
                    ValueOwned::from(partiql_value::Value::Blob(Box::new(b.to_vec())))
                }
                ValueRef::Owned(o) => (*o).clone(),
            })
            .unwrap_or(ValueOwned::from(partiql_value::Value::Missing))
    }
}
