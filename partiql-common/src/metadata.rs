use indexmap::map::Entry;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use std::borrow::Borrow;
use std::fmt::Result;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub struct PartiqlMetadata<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    inner: IndexMap<T, PartiqlMetaValue<T>>,
}

#[allow(dead_code)]
impl<T> PartiqlMetadata<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: T, value: PartiqlMetaValue<T>) {
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &T) -> Option<&PartiqlMetaValue<T>> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &T) -> Option<&mut PartiqlMetaValue<T>> {
        self.inner.get_mut(key)
    }

    pub fn contains_key(&self, key: &T) -> bool {
        self.inner.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &T> {
        self.inner.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &PartiqlMetaValue<T>> {
        self.inner.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut PartiqlMetaValue<T>> {
        self.inner.values_mut()
    }

    pub fn entry(&mut self, key: T) -> Entry<'_, T, PartiqlMetaValue<T>> {
        self.inner.entry(key)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn remove(&mut self, key: &T) -> Option<PartiqlMetaValue<T>> {
        self.inner.swap_remove(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, &PartiqlMetaValue<T>)> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&T, &mut PartiqlMetaValue<T>)> {
        self.inner.iter_mut()
    }

    pub fn vec_value(&self, key: &str) -> Option<Vec<PartiqlMetaValue<T>>> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Array(v)) = value {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn bool_value(&self, key: &str) -> Option<bool> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Bool(v)) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn f32_value(&self, key: &str) -> Option<f32> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Float32(v)) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn f64_value(&self, key: &str) -> Option<f64> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Float64(v)) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn decimal_value(&self, key: &str) -> Option<Decimal> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Decimal(v)) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn i32_value(&self, key: &str) -> Option<i32> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Int32(v)) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn i64_value(&self, key: &str) -> Option<i64> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Int64(v)) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn map_value(&self, key: &str) -> Option<PartiqlMetadata<T>> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::Map(v)) = value {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn string_value(&self, key: &str) -> Option<String> {
        let value = self.inner.get(key);
        if let Some(PartiqlMetaValue::String(v)) = value {
            Some(v.clone())
        } else {
            None
        }
    }
}

impl<T> Default for PartiqlMetadata<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn default() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }
}

impl<T> FromIterator<(T, PartiqlMetaValue<T>)> for PartiqlMetadata<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from_iter<I: IntoIterator<Item = (T, PartiqlMetaValue<T>)>>(iter: I) -> Self {
        let inner = iter.into_iter().collect();
        Self { inner }
    }
}

impl<T> IntoIterator for PartiqlMetadata<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    type Item = (T, PartiqlMetaValue<T>);
    type IntoIter = indexmap::map::IntoIter<T, PartiqlMetaValue<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    Array(Vec<PartiqlMetaValue<T>>),
    Bool(bool),
    Float32(f32),
    Float64(f64),
    Decimal(Decimal),
    Int32(i32),
    Int64(i64),
    Map(PartiqlMetadata<T>),
    String(String),
}

impl<T> From<bool> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: bool) -> Self {
        PartiqlMetaValue::Bool(value)
    }
}

impl<T> From<i32> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: i32) -> Self {
        PartiqlMetaValue::Int32(value)
    }
}
impl<T> From<i64> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: i64) -> Self {
        PartiqlMetaValue::Int64(value)
    }
}

impl<T> From<f64> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: f64) -> Self {
        PartiqlMetaValue::Float64(value)
    }
}

impl<T> From<String> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: String) -> Self {
        PartiqlMetaValue::String(value)
    }
}
impl<T> From<&'static str> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: &'static str) -> Self {
        PartiqlMetaValue::String(value.to_owned())
    }
}

impl<T> From<Vec<PartiqlMetaValue<T>>> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: Vec<PartiqlMetaValue<T>>) -> Self {
        PartiqlMetaValue::Array(value)
    }
}

impl<T> From<&[PartiqlMetaValue<T>]> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(slice: &[PartiqlMetaValue<T>]) -> Self {
        PartiqlMetaValue::Array(slice.to_vec())
    }
}

impl<T> From<PartiqlMetadata<T>> for PartiqlMetaValue<T>
where
    T: Eq + Clone + Hash + Borrow<str>,
{
    fn from(value: PartiqlMetadata<T>) -> Self {
        PartiqlMetaValue::Map(value)
    }
}

impl<T> Display for PartiqlMetaValue<T>
where
    T: Eq + Hash + Display + Clone + Borrow<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PartiqlMetaValue::Array(arr) => {
                write!(f, "[")?;
                for (idx, item) in arr.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            PartiqlMetaValue::Bool(v) => write!(f, "{}", v),
            PartiqlMetaValue::Decimal(v) => write!(f, "{}", v),
            PartiqlMetaValue::Float64(v) => write!(f, "{}", v),
            PartiqlMetaValue::Float32(v) => write!(f, "{}", v),
            PartiqlMetaValue::Int32(v) => write!(f, "{}", v),
            PartiqlMetaValue::Int64(v) => write!(f, "{}", v),
            PartiqlMetaValue::Map(map) => {
                write!(f, "{{")?;
                for (t, v) in map.iter() {
                    write!(f, "{}: {} , ", t, v)?;
                }
                write!(f, "}}")
            }
            PartiqlMetaValue::String(v) => write!(f, "{}", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::{PartiqlMetaValue, PartiqlMetadata};

    #[test]
    fn test_metadata() {
        let foo_val = PartiqlMetaValue::String("foo".to_string());
        let i64_val = PartiqlMetaValue::Int64(2);

        let expected_vec_val = vec![foo_val, i64_val];
        let expected_bool_val = true;
        let expected_int_val = 2;
        let expected_float_val = 2.5;
        let expected_str_val = "foo";

        let mut expected_map = PartiqlMetadata::new();
        expected_map.insert("bool value", expected_bool_val.into());
        expected_map.insert("integer value", expected_int_val.into());

        let mut metas = PartiqlMetadata::new();
        metas.insert("vec value", expected_vec_val.clone().into());
        metas.insert("bool value", expected_bool_val.into());
        metas.insert("integer value", expected_int_val.into());
        metas.insert("float value", expected_float_val.into());
        metas.insert("string value", expected_str_val.into());
        metas.insert("map value", expected_map.clone().into());

        let vec_val = metas.vec_value("vec value").expect("vec meta value");
        let bool_val = metas.bool_value("bool value").expect("bool meta value");
        let int_val = metas.i32_value("integer value").expect("i32 meta value");
        let float_val = metas.f64_value("float value").expect("f64 meta value");
        let string_val = metas
            .string_value("string value")
            .expect("string meta value");
        let map_val = metas.map_value("map value").expect("map meta value");

        assert_eq!(vec_val, expected_vec_val.clone());
        assert_eq!(bool_val, expected_bool_val.clone());
        assert_eq!(int_val, expected_int_val.clone());
        assert_eq!(float_val, expected_float_val.clone());
        assert_eq!(string_val, expected_str_val);
        assert_eq!(map_val, expected_map.clone());
    }
}
