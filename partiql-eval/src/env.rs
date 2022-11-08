use partiql_value::{BindingsName, Tuple, Value};
use unicase::UniCase;

pub trait Bindings<T> {
    fn get_binding(&self, name: &BindingsName) -> Option<&T>;
}

impl Bindings<Value> for Tuple {
    fn get_binding(&self, name: &BindingsName) -> Option<&Value> {
        match name {
            BindingsName::CaseSensitive(s) => self.get(s),
            BindingsName::CaseInsensitive(s) => {
                //TODO
                self.get(s)
            }
        }
    }
}

pub mod basic {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
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
            // TODO error on duplicate insensitive
            let idx = self.values.len();
            self.values.push(value);
            self.sensitive.insert(name.to_string(), idx);
            self.insensitive.insert(UniCase::new(name.to_string()), idx);
        }
    }

    impl<T> Bindings<T> for MapBindings<T> {
        #[inline]
        fn get_binding(&self, name: &BindingsName) -> Option<&T> {
            let idx = match name {
                BindingsName::CaseSensitive(s) => self.sensitive.get(s),
                BindingsName::CaseInsensitive(s) => {
                    self.insensitive.get(&UniCase::new(s.to_string()))
                }
            };
            idx.and_then(|idx| self.values.get(*idx))
        }
    }
}
