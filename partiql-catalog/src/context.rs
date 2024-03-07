use partiql_value::{BindingsName, DateTime, Tuple, Value};
use std::any::Any;
use std::fmt::Debug;

pub trait Bindings<T>: Debug {
    fn get(&self, name: &BindingsName) -> Option<&T>;
}

impl Bindings<Value> for Tuple {
    fn get(&self, name: &BindingsName) -> Option<&Value> {
        self.get(name)
    }
}

#[derive(Debug)]
pub struct SystemContext {
    pub now: DateTime,
}

/// Represents a session context that is used during evaluation of a plan.
pub trait SessionContext<'a>: Debug {
    fn bindings(&self) -> &dyn Bindings<Value>;

    fn system_context(&self) -> &SystemContext;

    fn user_context(&self, name: &str) -> Option<&(dyn Any)>;
}
