use partiql_value::datum::RefTupleView;
use partiql_value::{BindingsName, DateTime, Tuple, Value};
use std::any::Any;
use std::fmt::Debug;

pub trait Bindings<T>: Debug
where
    T: Debug,
{
    fn get<'a>(&'a self, name: &BindingsName<'a>) -> Option<&'a T>;
}

impl Bindings<Value> for Tuple {
    fn get<'a>(&'a self, name: &BindingsName<'a>) -> Option<&'a Value> {
        self.get(name)
    }
}

impl<'x, T> Bindings<Value> for &T
where
    T: RefTupleView<'x, Value>,
{
    fn get<'a>(&'a self, name: &BindingsName<'a>) -> Option<&'a Value> {
        self.get_val(name).map(|c|c.as_ref())
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
