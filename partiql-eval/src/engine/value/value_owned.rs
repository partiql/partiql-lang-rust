use partiql_value::Value;
use std::ops::Deref;

pub(super) use super::internal::ValueRef;

/// Newtype wrapper around Value that implements Send + Sync.
///
/// # Safety
/// TODO: TEMPORARY - ValueOwned wraps Value which contains Rc<SimpleGraph> in Graph variant.
/// This impl allows CompiledPlan to be Send + Sync for thread-safe query compilation.
/// MUST replace with a thread-safe value type before using in actual multi-threaded execution.
/// DO NOT send ValueOwned containing Value::Graph across threads - will cause undefined behavior.
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ValueOwned(Value);

// Safety: See struct-level safety comment.
// This is a temporary workaround until Value is replaced with a thread-safe alternative.
unsafe impl Send for ValueOwned {}
unsafe impl Sync for ValueOwned {}

impl Deref for ValueOwned {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Value> for ValueOwned {
    fn from(v: Value) -> Self {
        ValueOwned(v)
    }
}

impl From<ValueOwned> for Value {
    fn from(v: ValueOwned) -> Self {
        v.0
    }
}

impl AsRef<Value> for ValueOwned {
    fn as_ref(&self) -> &Value {
        &self.0
    }
}

use crate::engine::error::{EngineError, Result};

/// Type identifier for PartiQL values
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    Missing,
    Null,
    Bool,
    Integer,
    Float,
    String,
    Bytes,
    Tuple,
    List,
    Bag,
}

/// Cursor for single-pass traversal of hierarchical PartiQL values
///
/// ValueView provides a cursor-based API for navigating through nested data structures
/// without copying. Navigation methods consume self to enforce single-pass semantics.
///
/// # Example
/// ```ignore
/// let view = reader.get_value_view(0)?;
///
/// if view.get_type() == ValueType::Tuple {
///     let mut cursor = view.step_in()?;
///     while let Some(next) = cursor.next()? {
///         let name = cursor.get_field_name()?;
///         let value = cursor.get_i64()?;
///         cursor = next;
///     }
/// }
/// ```
pub struct ValueView<'a> {
    current: ValueRef<'a>,
    context: Option<NavigationContext<'a>>,
}

struct NavigationContext<'a> {
    kind: ContainerKind,
    data: ContainerData<'a>,
    parent: ValueRef<'a>, // Original container reference for step_out()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerKind {
    Tuple,
    #[allow(dead_code)]
    List,
    #[allow(dead_code)]
    Bag,
}

enum ContainerData<'a> {
    Tuple {
        fields: &'a [super::internal::TupleField<'a>],
        index: usize,
    },
    #[allow(dead_code)]
    List {
        items: Vec<ValueRef<'a>>,
        index: usize,
    },
    #[allow(dead_code)]
    Bag {
        items: Vec<ValueRef<'a>>,
        index: usize,
    },
}

impl<'a> ValueView<'a> {
    /// Create a new ValueView from a ValueRef
    pub(crate) fn from_ref(value: ValueRef<'a>) -> Self {
        ValueView {
            current: value,
            context: None,
        }
    }

    /// Get the type of the current value
    pub fn get_type(&self) -> ValueType {
        // Always return the type of the current value, regardless of navigation context
        match self.current {
            ValueRef::Missing => ValueType::Missing,
            ValueRef::Null => ValueType::Null,
            ValueRef::Bool(_) => ValueType::Bool,
            ValueRef::I64(_) => ValueType::Integer,
            ValueRef::F64(_) => ValueType::Float,
            ValueRef::Str(_) => ValueType::String,
            ValueRef::Bytes(_) => ValueType::Bytes,
            ValueRef::Tuple(_) => ValueType::Tuple,
            ValueRef::Owned(_) => {
                // ValueRef::Owned will be removed in the future
                // For now, treat as opaque
                ValueType::Null
            }
        }
    }

    /// Check if the current value is a container (Tuple, List, or Bag)
    pub fn is_container(&self) -> bool {
        matches!(
            self.get_type(),
            ValueType::Tuple | ValueType::List | ValueType::Bag
        )
    }

    /// Check if the current value is a scalar
    pub fn is_scalar(&self) -> bool {
        !self.is_container()
    }

    /// Get integer value
    pub fn get_i64(&self) -> Result<i64> {
        match self.current {
            ValueRef::I64(v) => Ok(v),
            _ => Err(EngineError::TypeError(format!(
                "expected Integer, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get boolean value
    pub fn get_bool(&self) -> Result<bool> {
        match self.current {
            ValueRef::Bool(v) => Ok(v),
            _ => Err(EngineError::TypeError(format!(
                "expected Bool, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get float value
    pub fn get_f64(&self) -> Result<f64> {
        match self.current {
            ValueRef::F64(v) => Ok(v),
            _ => Err(EngineError::TypeError(format!(
                "expected Float, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get string value
    pub fn get_str(&self) -> Result<&'a str> {
        match self.current {
            ValueRef::Str(s) => Ok(s),
            _ => Err(EngineError::TypeError(format!(
                "expected String, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get bytes value
    pub fn get_bytes(&self) -> Result<&'a [u8]> {
        match self.current {
            ValueRef::Bytes(b) => Ok(b),
            _ => Err(EngineError::TypeError(format!(
                "expected Bytes, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get the name of the current tuple field
    ///
    /// Only valid when navigating inside a tuple via step_in()
    pub fn get_field_name(&self) -> Result<&'a str> {
        match &self.context {
            Some(NavigationContext {
                kind: ContainerKind::Tuple,
                data: ContainerData::Tuple { fields, index },
                ..
            }) => {
                if *index < fields.len() {
                    Ok(fields[*index].name)
                } else {
                    Err(EngineError::IllegalState(
                        "cursor past end of tuple".to_string(),
                    ))
                }
            }
            _ => Err(EngineError::TypeError(
                "get_field_name only valid inside tuple".to_string(),
            )),
        }
    }

    /// Get the number of fields/elements in the current container
    pub fn container_size(&self) -> Result<usize> {
        match &self.context {
            Some(ctx) => Ok(match &ctx.data {
                ContainerData::Tuple { fields, .. } => fields.len(),
                ContainerData::List { items, .. } => items.len(),
                ContainerData::Bag { items, .. } => items.len(),
            }),
            None => match self.current {
                ValueRef::Tuple(tuple_ref) => Ok(tuple_ref.fields.len()),
                _ => Err(EngineError::TypeError(
                    "container_size only valid on containers".to_string(),
                )),
            },
        }
    }

    /// Get the number of fields in a tuple (alias for container_size for tuples)
    pub fn field_count(&self) -> Result<usize> {
        if self.get_type() != ValueType::Tuple {
            return Err(EngineError::TypeError(
                "field_count only valid on tuples".to_string(),
            ));
        }
        self.container_size()
    }

    /// Step into a Tuple container, positioning at the first field
    ///
    /// Consumes self and returns a new cursor positioned at the first field.
    /// Only valid on Tuple types (ValueRef::Tuple).
    ///
    /// # Errors
    /// - Returns error if current value is not a Tuple
    /// - Returns error if already inside a container
    /// - Returns error if tuple is empty
    pub fn step_in(self) -> Result<Self> {
        if self.context.is_some() {
            return Err(EngineError::IllegalState(
                "already inside a container".to_string(),
            ));
        }

        match self.current {
            ValueRef::Tuple(tuple_ref) => {
                if tuple_ref.fields.is_empty() {
                    return Err(EngineError::IllegalState(
                        "cannot step into empty tuple".to_string(),
                    ));
                }
                let parent = self.current; // Store the tuple for step_out()
                Ok(ValueView {
                    current: tuple_ref.fields[0].value,
                    context: Some(NavigationContext {
                        kind: ContainerKind::Tuple,
                        parent,
                        data: ContainerData::Tuple {
                            fields: tuple_ref.fields,
                            index: 0,
                        },
                    }),
                })
            }
            _ => Err(EngineError::TypeError(format!(
                "cannot step into {:?}, only Tuple supported",
                self.get_type()
            ))),
        }
    }

    /// Advance to the next field in the tuple
    ///
    /// Consumes self and returns Some(cursor) if there is a next field,
    /// or None if at the end of the tuple.
    ///
    /// # Errors
    /// - Returns error if not currently inside a container
    pub fn next(mut self) -> Result<Option<Self>> {
        let ctx = self.context.take().ok_or_else(|| {
            EngineError::IllegalState("next() called outside container".to_string())
        })?;

        let parent = ctx.parent; // Preserve parent for step_out()

        match ctx.data {
            ContainerData::Tuple { fields, mut index } => {
                index += 1;
                if index >= fields.len() {
                    return Ok(None);
                }
                Ok(Some(ValueView {
                    current: fields[index].value,
                    context: Some(NavigationContext {
                        kind: ctx.kind,
                        parent,
                        data: ContainerData::Tuple { fields, index },
                    }),
                }))
            }
            ContainerData::List { items, mut index } => {
                index += 1;
                if index >= items.len() {
                    return Ok(None);
                }
                Ok(Some(ValueView {
                    current: items[index],
                    context: Some(NavigationContext {
                        kind: ctx.kind,
                        parent,
                        data: ContainerData::List { items, index },
                    }),
                }))
            }
            ContainerData::Bag { items, mut index } => {
                index += 1;
                if index >= items.len() {
                    return Ok(None);
                }
                Ok(Some(ValueView {
                    current: items[index],
                    context: Some(NavigationContext {
                        kind: ctx.kind,
                        parent,
                        data: ContainerData::Bag { items, index },
                    }),
                }))
            }
        }
    }

    /// Exit the current container and return to the parent level
    ///
    /// Consumes self and returns a cursor positioned at the container itself.
    ///
    /// # Errors
    /// - Returns error if not currently inside a container
    pub fn step_out(mut self) -> Result<Self> {
        let ctx = self.context.take().ok_or_else(|| {
            EngineError::IllegalState("step_out() called outside container".to_string())
        })?;

        // Return the stored parent container directly - no temporary creation
        Ok(ValueView {
            current: ctx.parent,
            context: None,
        })
    }
}
