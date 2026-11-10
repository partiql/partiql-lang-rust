use super::internal::ValueRef;
use super::shape::{FieldName, FieldShape, RowShape};
use super::value_view::ValueView;

pub struct RegisterReader<'a> {
    pub(crate) slots: &'a [ValueRef<'a>],
}

impl<'a> RegisterReader<'a> {
    pub(crate) fn new(slots: &'a [ValueRef<'a>]) -> Self {
        RegisterReader { slots }
    }

    pub fn get_i64(&self, col: usize) -> Option<i64> {
        self.slots.get(col).and_then(|v| v.as_i64().ok())
    }

    pub fn get_str(&self, col: usize) -> Option<&'a str> {
        self.slots.get(col).and_then(|v| match v {
            ValueRef::Str(s) => Some(*s),
            _ => None,
        })
    }

    /// Get a ValueView cursor for navigating the value at the specified column
    ///
    /// Returns None if the column index is out of bounds.
    ///
    /// # Example
    /// ```ignore
    /// let view = reader.get_value_view(0)?;
    /// if view.get_type() == ValueType::Tuple {
    ///     let mut cursor = view.step_in()?;
    ///     while let Some(next) = cursor.next()? {
    ///         let name = cursor.get_field_name()?;
    ///         let value = cursor.get_i64()?;
    ///         cursor = next;
    ///     }
    /// }
    /// ```
    pub fn get_value_view(&self, col: usize) -> Option<ValueView<'a>> {
        self.slots.get(col).map(|v| ValueView::from_ref(*v))
    }

    /// The attributes of a tuple row shaped `fields`: each one's name, and the shape of its
    /// value in this row. As in a tuple constructor, an attribute whose value is MISSING, or
    /// whose name (read from a register) is no string, is left out: `SELECT x.a, x.b` of a
    /// row without `b` is `{'a': ...}`.
    pub fn attributes<'s>(
        &'s self,
        fields: &'s [FieldShape],
    ) -> impl Iterator<Item = (&'s str, &'s RowShape)> + 's {
        fields.iter().filter_map(move |field| {
            let name = match &field.name {
                FieldName::Static(name) => name.as_str(),
                FieldName::Register(reg) => self.get_str(*reg)?,
            };
            match field.value {
                RowShape::Register(reg, _)
                    if matches!(self.slots.get(reg), Some(ValueRef::Missing)) =>
                {
                    None
                }
                _ => Some((name, &field.value)),
            }
        })
    }
}
