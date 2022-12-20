use partiql_value::{BindingsName, Tuple, Value};
use unicase::UniCase;

pub trait Bindings<T> {
    fn get(&self, name: &BindingsName) -> Option<&T>;
}

impl Bindings<Value> for Tuple {
    fn get(&self, name: &BindingsName) -> Option<&Value> {
        self.get(name)
    }
}

pub mod basic {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct MapBindings<T> {
        sensitive: HashMap<String, usize>,
        insensitive: HashMap<UniCase<String>, usize>,
        values: Vec<T>,
    }

    impl<T> Default for MapBindings<T> {
        fn default() -> Self {
            MapBindings {
                sensitive: HashMap::new(),
                insensitive: HashMap::new(),
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
                panic!("Cannot insert duplicate binding of name {}", name)
            }
        }
    }

    impl<T> Bindings<T> for MapBindings<T> {
        #[inline]
        fn get(&self, name: &BindingsName) -> Option<&T> {
            let idx = match name {
                BindingsName::CaseSensitive(s) => self.sensitive.get(s),
                BindingsName::CaseInsensitive(s) => {
                    self.insensitive.get(&UniCase::new(s.to_string()))
                }
            };
            idx.and_then(|idx| self.values.get(*idx))
        }
    }

    impl From<&Tuple> for MapBindings<Value> {
        fn from(t: &Tuple) -> Self {
            let mut bindings = MapBindings::default();
            t.pairs().for_each(|(k, v)| bindings.insert(k, v.clone()));

            bindings
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::basic::MapBindings;
    use partiql_value::partiql_tuple;

    #[test]
    fn test_bindings_from_tuple() {
        let t = partiql_tuple![("a", partiql_tuple![("p", 1)]), ("b", 2)];
        println!("{:?}", MapBindings::from(&t));
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
