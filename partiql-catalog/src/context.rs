use partiql_value::datum::RefTupleView;
use partiql_value::{BindingsName, DateTime, Tuple, Value};
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;

pub trait Bindings<T>: Debug
where
    T: Clone + Debug,
{
    fn get<'a>(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, T>>;
}

impl Bindings<Value> for Tuple {
    fn get<'a>(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        self.get(name).map(Cow::Borrowed)
    }
}

impl<T> Bindings<Value> for &T
where
    for<'a> T: RefTupleView<'a, Value>,
{
    fn get<'a>(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        self.get_val(name)
    }
}

#[derive(Debug)]
pub struct SystemContext {
    pub now: DateTime,
}

/// Represents a session context that is used during evaluation of a plan.
pub trait SessionContext: Debug {
    fn system_context(&self) -> &SystemContext;

    fn user_context(&self, name: &str) -> Option<&(dyn Any)>;
}
