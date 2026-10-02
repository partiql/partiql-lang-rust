use super::internal::ValueRef;
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
}
