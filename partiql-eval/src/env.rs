use partiql_catalog::context::Bindings;
use partiql_value::{BindingsName, Tuple, Value};
use std::fmt::Debug;
use unicase::UniCase;

pub mod basic {
    use super::{Bindings, BindingsName, Debug, Tuple, UniCase, Value};
    use partiql_value::datum::{RefFieldView, RefTupleView};
    use std::borrow::Cow;

    use rustc_hash::FxHashMap;

    #[derive(Debug, Clone)]
    pub struct MapBindings<T> {
        sensitive: FxHashMap<String, usize>,
        insensitive: FxHashMap<UniCase<String>, usize>,
        values: Vec<T>,
    }

    impl<T> Default for MapBindings<T> {
        fn default() -> Self {
            MapBindings {
                sensitive: FxHashMap::default(),
                insensitive: FxHashMap::default(),
                values: vec![],
            }
        }
    }

    impl<T> MapBindings<T> {
        pub fn insert(&mut self, name: &str, value: T) {
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.insensitive.entry(UniCase::new(name.to_string()))
            {
                let idx = self.values.len();
                self.values.push(value);
                self.sensitive.insert(name.to_string(), idx);
                e.insert(idx);
            } else {
                panic!("Cannot insert duplicate binding of name {name}")
            }
        }
    }

    impl<T> Bindings<T> for MapBindings<T>
    where
        T: Clone + Debug,
    {
        #[inline]
        fn get<'a>(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, T>> {
            let idx = match name {
                BindingsName::CaseSensitive(s) => self.sensitive.get(s.as_ref()),
                BindingsName::CaseInsensitive(s) => {
                    self.insensitive.get(&UniCase::new(s.to_string()))
                }
            };
            idx.and_then(|idx| self.values.get(*idx).map(Cow::Borrowed))
        }
    }

    impl<'a> From<&'a dyn RefTupleView<'a, Value>> for MapBindings<Value> {
        fn from(value: &'a dyn RefTupleView<'a, Value>) -> Self {
            let mut bindings = MapBindings::default();
            for RefFieldView { name, value } in value.tuple_fields_iter() {
                bindings.insert(name.unwrap_or("_1"), value.into_owned());
            }
            bindings
        }
    }

    impl From<&Tuple> for MapBindings<Value> {
        fn from(t: &Tuple) -> Self {
            let mut bindings = MapBindings::default();
            for (k, v) in t.pairs() {
                bindings.insert(k, v.clone());
            }
            bindings
        }
    }

    impl From<Tuple> for MapBindings<Value> {
        fn from(t: Tuple) -> Self {
            let mut bindings = MapBindings::default();
            for (k, v) in t.into_pairs() {
                bindings.insert(&k, v);
            }
            bindings
        }
    }

    impl From<Value> for MapBindings<Value> {
        fn from(val: Value) -> Self {
            match val {
                Value::Null => MapBindings::default(),
                Value::Missing => MapBindings::default(),
                Value::Tuple(t) => (*t).into(),
                _ => todo!(),
            }
        }
    }

    impl From<&Value> for MapBindings<Value> {
        fn from(val: &Value) -> Self {
            match val {
                Value::Null => MapBindings::default(),
                Value::Missing => MapBindings::default(),
                Value::Tuple(t) => t.as_ref().into(),
                _ => todo!(),
            }
        }
    }

    #[derive(Debug)]
    pub struct NestedBindings<'a, T>
    where
        T: Debug,
    {
        bindings: MapBindings<T>,
        parent: &'a dyn Bindings<T>,
    }

    impl<'a, T> NestedBindings<'a, T>
    where
        T: Debug,
    {
        pub fn new(bindings: MapBindings<T>, parent: &'a dyn Bindings<T>) -> Self {
            Self { bindings, parent }
        }
    }

    impl<'b, T> Bindings<T> for NestedBindings<'b, T>
    where
        T: Clone + Debug + 'b,
    {
        fn get<'a>(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, T>> {
            match self.bindings.get(name) {
                Some(v) => Some(v),
                None => self.parent.get(name),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::basic::MapBindings;
    use partiql_value::tuple;

    #[test]
    fn test_bindings_from_tuple() {
        let t = tuple![("a", tuple![("p", 1)]), ("b", 2)];

        // by ref
        let bindings = MapBindings::from(&t);
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("a".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(tuple![("p", 1)]))
        );
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("b".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(2))
        );

        // by ownership
        let bindings = MapBindings::from(t);
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("a".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(tuple![("p", 1)]))
        );
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("b".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(2))
        );
    }

    #[test]
    fn test_bindings_from_value() {
        let bindings = MapBindings::from(Value::Null);
        assert_eq!(
            bindings.get(&BindingsName::CaseInsensitive("a".to_string().into())),
            None
        );
        let bindings = MapBindings::from(&Value::Null);
        assert_eq!(
            bindings.get(&BindingsName::CaseInsensitive("a".to_string().into())),
            None
        );
        let bindings = MapBindings::from(Value::Missing);
        assert_eq!(
            bindings.get(&BindingsName::CaseInsensitive("a".to_string().into())),
            None
        );
        let bindings = MapBindings::from(&Value::Missing);
        assert_eq!(
            bindings.get(&BindingsName::CaseInsensitive("a".to_string().into())),
            None
        );

        let t = Value::from(tuple![("a", tuple![("p", 1)]), ("b", 2)]);

        // by ref
        let bindings = MapBindings::from(&t);
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("a".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(tuple![("p", 1)]))
        );
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("b".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(2))
        );

        // by ownership
        let bindings = MapBindings::from(t);
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("a".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(tuple![("p", 1)]))
        );
        assert_eq!(
            bindings
                .get(&BindingsName::CaseInsensitive("b".to_string().into()))
                .as_ref()
                .map(AsRef::as_ref),
            Some(&Value::from(2))
        );
    }

    #[test]
    #[should_panic]
    fn test_bindings_insert_panics_same_string() {
        let mut bindings = MapBindings::default();
        bindings.insert("foo", Value::from(1));
        bindings.insert("foo", Value::from(2));
    }

    #[test]
    #[should_panic]
    fn test_bindings_insert_panics_case_insensitive_string() {
        let mut bindings = MapBindings::default();
        bindings.insert("foo", Value::from(1));
        bindings.insert("FOO", Value::from(2));
    }
}
