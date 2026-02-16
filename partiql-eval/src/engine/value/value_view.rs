pub(super) use super::internal::ValueRef;
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
    parent: ValueRef<'a>, // Original container reference
    parent_context: Option<Box<NavigationContext<'a>>>, // Parent's navigation state for nested step_out()
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
            ValueRef::List(_) => ValueType::List,
            ValueRef::Bag(_) => ValueType::Bag,
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

    /// Step into a container (Tuple, List, or Bag), positioning at the first element
    ///
    /// Mutates self to position at the first field/element of the container.
    /// Valid on Tuple, List, and Bag types.
    ///
    /// When called on a cursor that's already inside a container (from a previous step_in),
    /// this allows stepping into nested containers while preserving the parent's iteration state.
    ///
    /// # Errors
    /// - Returns error if current value is not a container
    /// - Returns error if container is empty
    pub fn step_in(&mut self) -> Result<()> {
        match self.current {
            ValueRef::Tuple(tuple_ref) => {
                if tuple_ref.fields.is_empty() {
                    return Err(EngineError::IllegalState(
                        "cannot step into empty tuple".to_string(),
                    ));
                }
                let parent = self.current;
                let parent_context = self.context.take().map(Box::new);

                self.current = tuple_ref.fields[0].value;
                self.context = Some(NavigationContext {
                    kind: ContainerKind::Tuple,
                    parent,
                    parent_context,
                    data: ContainerData::Tuple {
                        fields: tuple_ref.fields,
                        index: 0,
                    },
                });
                Ok(())
            }
            ValueRef::List(items) => {
                if items.is_empty() {
                    return Err(EngineError::IllegalState(
                        "cannot step into empty list".to_string(),
                    ));
                }
                let parent = self.current;
                let parent_context = self.context.take().map(Box::new);

                self.current = items[0];
                self.context = Some(NavigationContext {
                    kind: ContainerKind::List,
                    parent,
                    parent_context,
                    data: ContainerData::List {
                        items: items.to_vec(),
                        index: 0,
                    },
                });
                Ok(())
            }
            ValueRef::Bag(items) => {
                if items.is_empty() {
                    return Err(EngineError::IllegalState(
                        "cannot step into empty bag".to_string(),
                    ));
                }
                let parent = self.current;
                let parent_context = self.context.take().map(Box::new);

                self.current = items[0];
                self.context = Some(NavigationContext {
                    kind: ContainerKind::Bag,
                    parent,
                    parent_context,
                    data: ContainerData::Bag {
                        items: items.to_vec(),
                        index: 0,
                    },
                });
                Ok(())
            }
            _ => Err(EngineError::TypeError(format!(
                "cannot step into {:?}, only containers supported",
                self.get_type()
            ))),
        }
    }

    /// Advance to the next field in the tuple
    ///
    /// Mutates self to position at the next field.
    /// Returns true if there is a next field, false if at the end.
    ///
    /// # Errors
    /// - Returns error if not currently inside a container
    pub fn advance(&mut self) -> Result<bool> {
        let ctx = self.context.take().ok_or_else(|| {
            EngineError::IllegalState("advance() called outside container".to_string())
        })?;

        let parent = ctx.parent; // Preserve parent for step_out()
        let parent_context = ctx.parent_context; // Preserve parent's navigation state

        match ctx.data {
            ContainerData::Tuple { fields, mut index } => {
                index += 1;
                if index >= fields.len() {
                    // Restore context before returning false
                    self.context = Some(NavigationContext {
                        kind: ctx.kind,
                        parent,
                        parent_context,
                        data: ContainerData::Tuple {
                            fields,
                            index: index - 1,
                        },
                    });
                    return Ok(false);
                }
                // Mutate self to next field
                self.current = fields[index].value;
                self.context = Some(NavigationContext {
                    kind: ctx.kind,
                    parent,
                    parent_context,
                    data: ContainerData::Tuple { fields, index },
                });
                Ok(true)
            }
            ContainerData::List { items, mut index } => {
                index += 1;
                if index >= items.len() {
                    self.context = Some(NavigationContext {
                        kind: ctx.kind,
                        parent,
                        parent_context,
                        data: ContainerData::List {
                            items,
                            index: index - 1,
                        },
                    });
                    return Ok(false);
                }
                self.current = items[index];
                self.context = Some(NavigationContext {
                    kind: ctx.kind,
                    parent,
                    parent_context,
                    data: ContainerData::List { items, index },
                });
                Ok(true)
            }
            ContainerData::Bag { items, mut index } => {
                index += 1;
                if index >= items.len() {
                    self.context = Some(NavigationContext {
                        kind: ctx.kind,
                        parent,
                        parent_context,
                        data: ContainerData::Bag {
                            items,
                            index: index - 1,
                        },
                    });
                    return Ok(false);
                }
                self.current = items[index];
                self.context = Some(NavigationContext {
                    kind: ctx.kind,
                    parent,
                    parent_context,
                    data: ContainerData::Bag { items, index },
                });
                Ok(true)
            }
        }
    }

    /// Exit the current container and return to the parent level
    ///
    /// Mutates self to position at the parent container field.
    /// This allows continuing iteration at the parent level after exploring nested containers.
    ///
    /// # Errors
    /// - Returns error if not currently inside a container
    pub fn step_out(&mut self) -> Result<()> {
        let ctx = self.context.take().ok_or_else(|| {
            EngineError::IllegalState("step_out() called outside container".to_string())
        })?;

        // Restore parent's navigation state
        // If parent_context is Some, we were navigating inside a parent container
        // and should restore to that position. Otherwise, return to top level.
        self.current = ctx.parent;
        self.context = ctx.parent_context.map(|boxed| *boxed);
        Ok(())
    }
}
