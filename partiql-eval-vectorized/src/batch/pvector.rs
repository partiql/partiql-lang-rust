use crate::error::EvalError;
use std::marker::PhantomData;
use std::sync::Arc;

/// Type information for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    Int64,
    Float64,
    Boolean,
    String,
}

/// Memory-mapped buffer wrapper
/// 
/// Provides a typed view over memory-mapped data. The mmap is read-only
/// and any attempt to mutate it will result in a panic or copy-on-write.
#[derive(Debug)]
pub struct MmapSlice<T> {
    mmap: memmap2::Mmap,
    _marker: PhantomData<T>,
}

impl<T> MmapSlice<T> {
    /// Create from mmap with safety checks
    /// 
    /// # Safety
    /// - The mmap must contain valid data of type T
    /// - The mmap must be properly aligned for type T
    /// - The mmap's lifetime must exceed this MmapSlice's lifetime
    pub unsafe fn new(mmap: memmap2::Mmap) -> Result<Self, EvalError> {
        // Verify alignment
        if mmap.as_ptr() as usize % std::mem::align_of::<T>() != 0 {
            return Err(EvalError::General(
                "Memory-mapped data is not properly aligned for type".to_string(),
            ));
        }
        Ok(Self {
            mmap,
            _marker: PhantomData,
        })
    }

    /// Get a slice view of the memory-mapped data
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        let ptr = self.mmap.as_ptr() as *const T;
        let len = self.mmap.len() / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// Buffer owns the actual data storage for a column.
/// 
/// Buffers can be:
/// - Owned: Heap-allocated immutable shared data via Arc<[T]>
/// - Mmap: Memory-mapped read-only data
/// - Constant: Single value logically repeated across all rows
/// 
/// The Arc wrapping happens at this level, enabling multiple TypedVector
/// views to share the same underlying data without copying.
#[derive(Debug, Clone)]
pub enum Buffer<T> {
    /// Owned heap-allocated immutable shared slice
    Owned(Arc<[T]>),
    
    /// Memory-mapped read-only data
    Mmap(Arc<MmapSlice<T>>),
    
    /// Single constant value (logically repeated across rows)
    /// Useful for literal values in expressions
    Constant(T),
}

impl<T> Buffer<T> {
    /// Get a slice view of the buffer's data
    /// 
    /// For Constant buffers, returns a slice of length 1 pointing to the constant value.
    /// Callers should use TypedVector's offset/len to handle the logical repetition.
    #[inline]
    fn as_slice(&self) -> &[T] {
        match self {
            Buffer::Owned(arc) => arc.as_ref(),
            Buffer::Mmap(mmap) => mmap.as_slice(),
            Buffer::Constant(val) => {
                // Return single-element slice
                unsafe { std::slice::from_raw_parts(val, 1) }
            }
        }
    }
}

/// TypedVector is a view into a Buffer with a specific offset and length.
/// 
/// The Buffer enum owns the Arc internally, so TypedVector no longer needs
/// to wrap Buffer in Arc. This simplifies the ownership model:
/// - Buffer owns the Arc<[T]> or Arc<MmapSlice<T>>
/// - TypedVector owns a Buffer and defines a view (offset, len)
/// 
/// # Memory Model
/// ```text
/// Buffer::Owned: Arc<[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]>
///                         ↑←────────────→↑
///                    TypedVector { offset: 2, len: 6 }
///                    Represents: [2, 3, 4, 5, 6, 7]
/// 
/// Buffer::Constant: Single value 42
///                   TypedVector { offset: 0, len: 1000 }
///                   Logically represents: [42, 42, ..., 42] (1000 times)
/// ```
#[derive(Debug, Clone)]
pub struct TypedVector<T> {
    /// The underlying buffer (owns Arc internally)
    buffer: Buffer<T>,
    /// Starting offset into the buffer
    offset: usize,
    /// Number of elements in this view
    len: usize,
}

impl<T: Clone> TypedVector<T> {
    /// Create a new vector with a fresh owned buffer
    pub fn new(capacity: usize) -> Self {
        let data: Arc<[T]> = Arc::from(Vec::with_capacity(capacity));
        Self {
            buffer: Buffer::Owned(data),
            offset: 0,
            len: 0,
        }
    }

    /// Create a vector from existing data (converts Vec to Arc<[T]>)
    pub fn from_vec(data: Vec<T>) -> Self {
        let len = data.len();
        Self {
            buffer: Buffer::Owned(Arc::from(data)),
            offset: 0,
            len,
        }
    }

    /// Create a vector from a constant value
    /// 
    /// The constant value is stored once and logically repeated `len` times.
    /// This is memory-efficient for literal expressions.
    pub fn from_constant(value: T, len: usize) -> Self {
        Self {
            buffer: Buffer::Constant(value),
            offset: 0,
            len,
        }
    }

    /// Create a vector from memory-mapped data
    /// 
    /// # Safety
    /// - The mmap must contain valid data of type T
    /// - The mmap must be properly aligned for type T
    /// - The mmap's lifetime must exceed this vector's lifetime
    pub unsafe fn from_mmap(mmap: memmap2::Mmap) -> Result<Self, EvalError> {
        let mmap_slice = MmapSlice::new(mmap)?;
        let len = mmap_slice.as_slice().len();
        Ok(Self {
            buffer: Buffer::Mmap(Arc::new(mmap_slice)),
            offset: 0,
            len,
        })
    }

    /// Get a read-only slice view of the data
    /// 
    /// Note: For Constant buffers, this will materialize the constant into an owned buffer
    /// on first call. This trades memory for safe access.
    pub fn as_slice(&self) -> &[T] {
        match &self.buffer {
            Buffer::Owned(arc) => &arc[self.offset..self.offset + self.len],
            Buffer::Mmap(mmap) => {
                let full_slice = mmap.as_slice();
                &full_slice[self.offset..self.offset + self.len]
            }
            Buffer::Constant(_) => {
                // Constant buffers cannot provide a safe slice without materialization
                // This is a limitation of the current API
                panic!("Cannot get slice from constant buffer - use as_mut_slice() to materialize or handle at higher level");
            }
        }
    }
    
    /// Get a read-only slice view, materializing constant buffers if needed
    /// 
    /// This is a workaround for constant buffers. It will convert the buffer
    /// to owned if it's a constant, then return the slice.
    fn as_slice_materialized(&mut self) -> &[T] {
        // Materialize constant buffers
        if matches!(&self.buffer, Buffer::Constant(_)) {
            let data: Vec<T> = vec![
                match &self.buffer {
                    Buffer::Constant(val) => val.clone(),
                    _ => unreachable!(),
                };
                self.len
            ];
            self.buffer = Buffer::Owned(Arc::from(data));
            self.offset = 0;
        }
        
        // Now we can safely call as_slice
        match &self.buffer {
            Buffer::Owned(arc) => &arc[self.offset..self.offset + self.len],
            Buffer::Mmap(mmap) => {
                let full_slice = mmap.as_slice();
                &full_slice[self.offset..self.offset + self.len]
            }
            Buffer::Constant(_) => unreachable!(),
        }
    }

    /// Create a new view into a subset of this vector (zero-copy)
    /// 
    /// # Panics
    /// Panics if `start + len` exceeds the vector's length
    pub fn slice(&self, start: usize, len: usize) -> Self {
        assert!(start + len <= self.len, "Slice out of bounds");
        Self {
            buffer: self.buffer.clone(),
            offset: self.offset + start,
            len,
        }
    }

    /// Get mutable access to the data
    /// 
    /// # Panics
    /// - Panics if the buffer is memory-mapped (mmap buffers are read-only)
    /// 
    /// # Copy-on-Write
    /// - For Owned buffers: If the buffer is shared (Arc strong count > 1), creates a copy
    /// - For Constant buffers: Always creates an owned copy
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match &mut self.buffer {
            Buffer::Owned(arc) => {
                // Copy-on-write: if buffer is shared, make our own copy
                if Arc::strong_count(arc) > 1 {
                    let data: Vec<T> = self.as_slice().to_vec();
                    self.buffer = Buffer::Owned(Arc::from(data));
                    self.offset = 0;
                }
                
                // Now we have exclusive access
                let slice = Arc::get_mut(match &mut self.buffer {
                    Buffer::Owned(a) => a,
                    _ => unreachable!(),
                })
                .expect("Buffer should not be shared after COW");
                
                &mut slice[self.offset..self.offset + self.len]
            }
            Buffer::Mmap(_) => {
                panic!("Cannot get mutable access to memory-mapped buffer");
            }
            Buffer::Constant(val) => {
                // Copy-on-write: expand constant to owned buffer
                let data: Vec<T> = vec![val.clone(); self.len];
                self.buffer = Buffer::Owned(Arc::from(data));
                self.offset = 0;
                // Recurse to get mutable slice from newly created owned buffer
                self.as_mut_slice()
            }
        }
    }

    /// Get the length of this vector view
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if this vector view is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear the vector, setting length to 0
    /// 
    /// For shared or mmap buffers, this creates a new empty owned buffer.
    pub fn clear(&mut self) {
        match &mut self.buffer {
            Buffer::Owned(arc) => {
                if Arc::strong_count(arc) > 1 {
                    // Buffer is shared, create new empty buffer
                    self.buffer = Buffer::Owned(Arc::from(Vec::<T>::new()));
                } else {
                    // We have exclusive access, can clear in place
                    if let Some(slice_mut) = Arc::get_mut(arc) {
                        // Arc<[T]> cannot be resized, so create new empty one
                        self.buffer = Buffer::Owned(Arc::from(Vec::<T>::new()));
                    }
                }
            }
            Buffer::Mmap(_) => {
                // Cannot clear mmap, create new empty buffer
                self.buffer = Buffer::Owned(Arc::from(Vec::<T>::new()));
            }
            Buffer::Constant(_) => {
                // Cannot clear constant, create new empty buffer
                self.buffer = Buffer::Owned(Arc::from(Vec::<T>::new()));
            }
        }
        self.offset = 0;
        self.len = 0;
    }
}

impl<T: Clone + Default> TypedVector<T> {
    /// Create a new vector with default-initialized elements
    pub fn with_default(size: usize) -> Self {
        Self::from_vec(vec![T::default(); size])
    }
}

/// Physical vector - type-erased columnar storage
/// 
/// `PVector` is an enum that wraps `TypedVector<T>` for different data types,
/// providing a type-erased interface for working with columns of different types.
/// 
/// The underlying data is stored in `Buffer` instances which handle Arc sharing,
/// allowing for efficient zero-copy operations between operators.
#[derive(Debug, Clone)]
pub enum PVector {
    Int64(TypedVector<i64>),
    Float64(TypedVector<f64>),
    Boolean(TypedVector<bool>),
    String(TypedVector<String>),
}

impl PVector {
    /// Create new vector of given type pre-allocated with size
    /// All elements initialized to default values (0, false, empty string)
    pub fn new(type_info: TypeInfo, size: usize) -> Self {
        match type_info {
            TypeInfo::Int64 => PVector::Int64(TypedVector::with_default(size)),
            TypeInfo::Float64 => PVector::Float64(TypedVector::with_default(size)),
            TypeInfo::Boolean => PVector::Boolean(TypedVector::with_default(size)),
            TypeInfo::String => PVector::String(TypedVector::with_default(size)),
        }
    }

    /// Get number of elements
    pub fn len(&self) -> usize {
        match self {
            PVector::Int64(v) => v.len(),
            PVector::Float64(v) => v.len(),
            PVector::Boolean(v) => v.len(),
            PVector::String(v) => v.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Copy data from another vector
    pub fn copy_from(&mut self, other: &PVector) -> Result<(), EvalError> {
        match (self, other) {
            (PVector::Int64(dst), PVector::Int64(src)) => {
                let dst_slice = dst.as_mut_slice();
                let src_slice = src.as_slice();
                dst_slice.copy_from_slice(src_slice);
                Ok(())
            }
            (PVector::Float64(dst), PVector::Float64(src)) => {
                let dst_slice = dst.as_mut_slice();
                let src_slice = src.as_slice();
                dst_slice.copy_from_slice(src_slice);
                Ok(())
            }
            (PVector::Boolean(dst), PVector::Boolean(src)) => {
                let dst_slice = dst.as_mut_slice();
                let src_slice = src.as_slice();
                dst_slice.copy_from_slice(src_slice);
                Ok(())
            }
            (PVector::String(dst), PVector::String(src)) => {
                let dst_slice = dst.as_mut_slice();
                let src_slice = src.as_slice();
                dst_slice.clone_from_slice(src_slice);
                Ok(())
            }
            _ => Err(EvalError::TypeMismatch),
        }
    }

    /// Get type information
    pub fn type_info(&self) -> TypeInfo {
        match self {
            PVector::Int64(_) => TypeInfo::Int64,
            PVector::Float64(_) => TypeInfo::Float64,
            PVector::Boolean(_) => TypeInfo::Boolean,
            PVector::String(_) => TypeInfo::String,
        }
    }

    /// Get as Int64 vector if possible
    pub fn as_int64(&self) -> Option<&TypedVector<i64>> {
        match self {
            PVector::Int64(v) => Some(v),
            _ => None,
        }
    }

    /// Get as Float64 vector if possible
    pub fn as_float64(&self) -> Option<&TypedVector<f64>> {
        match self {
            PVector::Float64(v) => Some(v),
            _ => None,
        }
    }

    /// Get as Boolean vector if possible
    pub fn as_boolean(&self) -> Option<&TypedVector<bool>> {
        match self {
            PVector::Boolean(v) => Some(v),
            _ => None,
        }
    }

    /// Get as String vector if possible
    pub fn as_string(&self) -> Option<&TypedVector<String>> {
        match self {
            PVector::String(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable Int64 vector if possible
    pub fn as_int64_mut(&mut self) -> Option<&mut TypedVector<i64>> {
        match self {
            PVector::Int64(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable Float64 vector if possible
    pub fn as_float64_mut(&mut self) -> Option<&mut TypedVector<f64>> {
        match self {
            PVector::Float64(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable Boolean vector if possible
    pub fn as_boolean_mut(&mut self) -> Option<&mut TypedVector<bool>> {
        match self {
            PVector::Boolean(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable String vector if possible
    pub fn as_string_mut(&mut self) -> Option<&mut TypedVector<String>> {
        match self {
            PVector::String(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_buffer() {
        let mut vec = TypedVector::from_constant(42, 100);
        assert_eq!(vec.len(), 100);
        // Materialize constant before reading
        let slice = vec.as_slice_materialized();
        assert_eq!(slice[0], 42);
        assert_eq!(slice[99], 42);
    }

    #[test]
    fn test_constant_cow() {
        let mut vec = TypedVector::from_constant(42, 10);
        // as_mut_slice will materialize the constant
        let slice = vec.as_mut_slice();
        slice[0] = 100;
        
        // After COW (materialization), first element should be changed
        assert_eq!(vec.as_slice()[0], 100);
        // Rest should still be 42
        assert_eq!(vec.as_slice()[1], 42);
        assert_eq!(vec.as_slice()[9], 42);
    }

    #[test]
    fn test_owned_buffer_sharing() {
        let vec1 = TypedVector::from_vec(vec![1, 2, 3, 4, 5]);
        let vec2 = vec1.clone();
        
        // Both should see same data
        assert_eq!(vec1.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(vec2.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_owned_buffer_cow() {
        let vec1 = TypedVector::from_vec(vec![1, 2, 3, 4, 5]);
        let mut vec2 = vec1.clone();
        
        // Mutating vec2 should trigger COW
        vec2.as_mut_slice()[0] = 100;
        
        // vec1 unchanged
        assert_eq!(vec1.as_slice()[0], 1);
        // vec2 changed
        assert_eq!(vec2.as_slice()[0], 100);
    }

    #[test]
    fn test_slice_operation() {
        let vec = TypedVector::from_vec(vec![1, 2, 3, 4, 5]);
        let sliced = vec.slice(1, 3);
        
        assert_eq!(sliced.as_slice(), &[2, 3, 4]);
    }
}
