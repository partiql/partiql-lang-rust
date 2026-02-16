//! Shape types for describing the structure of query results.
//!
//! PartiQL is a gradually typed query language supporting nested data. The `Shape`
//! type captures the structure of query results at compile time, enabling consumers
//! to understand how to interpret the rows returned by `QueryIterator`.

// Re-export PhysicalType for convenience
pub use crate::engine::source::PhysicalType;

/// Shape of query results, describing both iteration behavior and row structure.
///
/// # Examples
///
/// ```ignore
/// // Bag of structs with known fields: SELECT id, name FROM users
/// Shape::Bag(RowShape::Struct(vec![
///     FieldShape { name: FieldName::Static("id".into()), value: RowShape::Register(0, PhysicalType::I64) },
///     FieldShape { name: FieldName::Static("name".into()), value: RowShape::Register(1, PhysicalType::Str) },
/// ]))
///
/// // Bag of integers: SELECT x FROM numbers
/// Shape::Bag(RowShape::Register(0, PhysicalType::I64))
///
/// // Single scalar: SELECT 42
/// Shape::Single(RowShape::Register(0, PhysicalType::I64))
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Unordered collection - iterator yields 0 or more rows
    Bag(RowShape),
    /// Ordered collection - iterator yields 0 or more rows
    List(RowShape),
    /// Single result - iterator yields exactly one row
    Single(RowShape),
}

impl Default for Shape {
    fn default() -> Self {
        // Default to a single dynamic value for backwards compatibility
        Shape::Single(RowShape::Register(0, PhysicalType::Dynamic))
    }
}

impl Shape {
    /// Create a bag of structs with the given fields.
    pub fn bag_of_structs(fields: Vec<FieldShape>) -> Self {
        Shape::Bag(RowShape::Struct(fields))
    }

    /// Create a bag of scalar values with the given type.
    pub fn bag_of_scalars(register: usize, ty: PhysicalType) -> Self {
        Shape::Bag(RowShape::Register(register, ty))
    }

    /// Create a list of structs with the given fields.
    pub fn list_of_structs(fields: Vec<FieldShape>) -> Self {
        Shape::List(RowShape::Struct(fields))
    }

    /// Create a list of scalar values with the given type.
    pub fn list_of_scalars(register: usize, ty: PhysicalType) -> Self {
        Shape::List(RowShape::Register(register, ty))
    }

    /// Create a single scalar result.
    pub fn single_scalar(register: usize, ty: PhysicalType) -> Self {
        Shape::Single(RowShape::Register(register, ty))
    }

    /// Create a single struct result.
    pub fn single_struct(fields: Vec<FieldShape>) -> Self {
        Shape::Single(RowShape::Struct(fields))
    }

    /// Check if this shape represents a collection (bag or list).
    pub fn is_collection(&self) -> bool {
        matches!(self, Shape::Bag(_) | Shape::List(_))
    }

    /// Check if this shape represents an ordered collection (list).
    pub fn is_ordered(&self) -> bool {
        matches!(self, Shape::List(_))
    }

    /// Get the row shape for this result.
    pub fn row_shape(&self) -> &RowShape {
        match self {
            Shape::Bag(row) | Shape::List(row) | Shape::Single(row) => row,
        }
    }
}

/// Shape of a single row in the query result.
///
/// Each row can either be a struct with named fields, or a single scalar value
/// stored in a register.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowShape {
    /// Row is a struct with named fields
    Struct(Vec<FieldShape>),
    /// Row is a single value in a register (register index, physical type)
    Register(usize, PhysicalType),
}

impl RowShape {
    /// Create a struct row shape with the given fields.
    pub fn with_fields(fields: Vec<FieldShape>) -> Self {
        RowShape::Struct(fields)
    }

    /// Create a scalar row shape with the given register and type.
    pub fn scalar(register: usize, ty: PhysicalType) -> Self {
        RowShape::Register(register, ty)
    }

    /// Check if this row shape is a struct.
    pub fn is_struct(&self) -> bool {
        matches!(self, RowShape::Struct(_))
    }

    /// Get the fields if this is a struct row shape.
    pub fn fields(&self) -> Option<&[FieldShape]> {
        match self {
            RowShape::Struct(fields) => Some(fields),
            RowShape::Register(_, _) => None,
        }
    }
}

/// A field within a struct row.
///
/// Fields have a name (which can be statically known or determined at runtime
/// from a register) and a value shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldShape {
    /// How the field's name is determined
    pub name: FieldName,
    /// The shape of the field's value
    pub value: RowShape,
}

impl FieldShape {
    /// Create a field with a static name and scalar value.
    pub fn static_scalar(name: impl Into<String>, register: usize, ty: PhysicalType) -> Self {
        FieldShape {
            name: FieldName::Static(name.into()),
            value: RowShape::Register(register, ty),
        }
    }

    /// Create a field with a static name and struct value.
    pub fn static_struct(name: impl Into<String>, fields: Vec<FieldShape>) -> Self {
        FieldShape {
            name: FieldName::Static(name.into()),
            value: RowShape::Struct(fields),
        }
    }

    /// Create a field with a dynamic name (read from a register) and scalar value.
    pub fn dynamic_scalar(name_register: usize, value_register: usize, ty: PhysicalType) -> Self {
        FieldShape {
            name: FieldName::Register(name_register),
            value: RowShape::Register(value_register, ty),
        }
    }
}

/// How a field's name is determined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldName {
    /// Statically known name
    Static(String),
    /// Name read from a register at runtime
    Register(usize),
}
