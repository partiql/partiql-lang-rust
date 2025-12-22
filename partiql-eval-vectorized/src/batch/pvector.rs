use crate::error::EvalError;
use std::marker::PhantomData;
use std::sync::Arc;

/// Logical type information for SQL-like types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalType {
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
/// 
/// Constants are now handled at the PhysicalVector level.
#[derive(Debug, Clone)]
pub enum Buffer<T> {
    /// Owned heap-allocated immutable shared slice
    Owned(Arc<[T]>),
    
    /// Memory-mapped read-only data
    Mmap(Arc<MmapSlice<T>>),
}

/// PhysicalVector represents the physical storage of columnar data.
/// 
/// Can be either:
/// - Flat: Traditional columnar storage with buffer + offset + len
/// - Constant: Single value logically repeated across all rows
/// 
/// This is the generic representation that works with any type T.
#[derive(Debug, Clone)]
pub enum PhysicalVector<T> {
    /// Flat vector with buffer, offset, and length
    Flat {
        buffer: Buffer<T>,
        offset: usize,
        len: usize,
    },
    
    /// Constant vector with a single value repeated logically
    Constant {
        value: T,
        len: usize,
    },
}

impl<T: Clone> PhysicalVector<T> {
    /// Create a flat vector from existing data
    pub fn from_vec(data: Vec<T>) -> Self {
        let len = data.len();
        PhysicalVector::Flat {
            buffer: Buffer::Owned(Arc::from(data)),
            offset: 0,
            len,
        }
    }

    /// Create a constant vector
    pub fn from_constant(value: T, len: usize) -> Self {
        PhysicalVector::Constant { value, len }
    }

    /// Create a flat vector from memory-mapped data
    /// 
    /// # Safety
    /// - The mmap must contain valid data of type T
    /// - The mmap must be properly aligned for type T
    /// - The mmap's lifetime must exceed this vector's lifetime
    pub unsafe fn from_mmap(mmap: memmap2::Mmap) -> Result<Self, EvalError> {
        let mmap_slice = MmapSlice::new(mmap)?;
        let len = mmap_slice.as_slice().len();
        Ok(PhysicalVector::Flat {
            buffer: Buffer::Mmap(Arc::new(mmap_slice)),
            offset: 0,
            len,
        })
    }

    /// Get the length of this vector
    pub fn len(&self) -> usize {
        match self {
            PhysicalVector::Flat { len, .. } => *len,
            PhysicalVector::Constant { len, .. } => *len,
        }
    }

    /// Check if this vector is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a read-only slice view of the data
    /// 
    /// For Constant vectors, this will materialize the constant into a flat vector.
    pub fn as_slice(&self) -> &[T] {
        match self {
            PhysicalVector::Flat { buffer, offset, len } => {
                match buffer {
                    Buffer::Owned(arc) => &arc[*offset..*offset + *len],
                    Buffer::Mmap(mmap) => {
                        let full_slice = mmap.as_slice();
                        &full_slice[*offset..*offset + *len]
                    }
                }
            }
            PhysicalVector::Constant { .. } => {
                panic!("Cannot get slice from constant vector - use as_mut_slice() to materialize");
            }
        }
    }

    /// Get mutable access to the data
    /// 
    /// # Panics
    /// - Panics if the buffer is memory-mapped (mmap buffers are read-only)
    /// 
    /// # Copy-on-Write
    /// - For Flat with Owned: If shared (Arc strong count > 1), creates a copy
    /// - For Constant: Always materializes to Flat with owned buffer
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // Check if we need to materialize or COW
        let needs_transform = match self {
            PhysicalVector::Flat { buffer, .. } => {
                matches!(buffer, Buffer::Owned(arc) if Arc::strong_count(arc) > 1)
            }
            PhysicalVector::Constant { .. } => true,
        };

        if needs_transform {
            // Get the data we need before the transform
            let (new_data, new_len) = match self {
                PhysicalVector::Flat { buffer, offset, len } => {
                    if let Buffer::Owned(arc) = buffer {
                        let data = arc[*offset..*offset + *len].to_vec();
                        (data, *len)
                    } else {
                        unreachable!()
                    }
                }
                PhysicalVector::Constant { value, len } => {
                    (vec![value.clone(); *len], *len)
                }
            };

            // Transform to owned flat vector
            *self = PhysicalVector::Flat {
                buffer: Buffer::Owned(Arc::from(new_data)),
                offset: 0,
                len: new_len,
            };
        }

        // Now we can safely get mutable access
        match self {
            PhysicalVector::Flat { buffer, offset, len } => {
                match buffer {
                    Buffer::Owned(arc) => {
                        let slice = Arc::get_mut(arc)
                            .expect("Buffer should not be shared after COW");
                        &mut slice[*offset..*offset + *len]
                    }
                    Buffer::Mmap(_) => {
                        panic!("Cannot get mutable access to memory-mapped buffer");
                    }
                }
            }
            PhysicalVector::Constant { .. } => {
                unreachable!("Should have been transformed to Flat above")
            }
        }
    }

    /// Create a new view into a subset of this vector (zero-copy for Flat)
    /// 
    /// # Panics
    /// Panics if `start + len` exceeds the vector's length
    pub fn slice(&self, start: usize, len: usize) -> Self {
        assert!(start + len <= self.len(), "Slice out of bounds");
        
        match self {
            PhysicalVector::Flat { buffer, offset, .. } => {
                PhysicalVector::Flat {
                    buffer: buffer.clone(),
                    offset: offset + start,
                    len,
                }
            }
            PhysicalVector::Constant { value, .. } => {
                PhysicalVector::Constant {
                    value: value.clone(),
                    len,
                }
            }
        }
    }

    /// Clear the vector by setting its length to 0
    /// 
    /// For Flat vectors with owned buffers, this maintains the capacity.
    /// For Constant vectors, just sets len to 0.
    pub fn clear(&mut self) {
        match self {
            PhysicalVector::Flat { len, .. } => {
                *len = 0;
            }
            PhysicalVector::Constant { len, .. } => {
                *len = 0;
            }
        }
    }
}

impl<T: Clone + Default> PhysicalVector<T> {
    /// Create a flat vector with default-initialized elements
    pub fn with_default(size: usize) -> Self {
        Self::from_vec(vec![T::default(); size])
    }
}

/// Type-erased physical vector
/// 
/// Wraps PhysicalVector<T> for different concrete types, providing
/// a type-erased interface for working with columns of different types.
#[derive(Debug, Clone)]
pub enum PhysicalVectorEnum {
    Int64(PhysicalVector<i64>),
    Float64(PhysicalVector<f64>),
    Boolean(PhysicalVector<bool>),
    String(PhysicalVector<String>),
}

impl PhysicalVectorEnum {
    /// Create a new vector of the given logical type
    pub fn new(ty: LogicalType, size: usize) -> Self {
        match ty {
            LogicalType::Int64 => PhysicalVectorEnum::Int64(PhysicalVector::with_default(size)),
            LogicalType::Float64 => PhysicalVectorEnum::Float64(PhysicalVector::with_default(size)),
            LogicalType::Boolean => PhysicalVectorEnum::Boolean(PhysicalVector::with_default(size)),
            LogicalType::String => PhysicalVectorEnum::String(PhysicalVector::with_default(size)),
        }
    }

    /// Get the logical type of this vector
    pub fn logical_type(&self) -> LogicalType {
        match self {
            PhysicalVectorEnum::Int64(_) => LogicalType::Int64,
            PhysicalVectorEnum::Float64(_) => LogicalType::Float64,
            PhysicalVectorEnum::Boolean(_) => LogicalType::Boolean,
            PhysicalVectorEnum::String(_) => LogicalType::String,
        }
    }

    /// Get the length of this vector
    pub fn len(&self) -> usize {
        match self {
            PhysicalVectorEnum::Int64(v) => v.len(),
            PhysicalVectorEnum::Float64(v) => v.len(),
            PhysicalVectorEnum::Boolean(v) => v.len(),
            PhysicalVectorEnum::String(v) => v.len(),
        }
    }

    /// Check if this vector is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get as Int64 vector if possible
    pub fn as_int64(&self) -> Option<&PhysicalVector<i64>> {
        match self {
            PhysicalVectorEnum::Int64(v) => Some(v),
            _ => None,
        }
    }

    /// Get as Float64 vector if possible
    pub fn as_float64(&self) -> Option<&PhysicalVector<f64>> {
        match self {
            PhysicalVectorEnum::Float64(v) => Some(v),
            _ => None,
        }
    }

    /// Get as Boolean vector if possible
    pub fn as_boolean(&self) -> Option<&PhysicalVector<bool>> {
        match self {
            PhysicalVectorEnum::Boolean(v) => Some(v),
            _ => None,
        }
    }

    /// Get as String vector if possible
    pub fn as_string(&self) -> Option<&PhysicalVector<String>> {
        match self {
            PhysicalVectorEnum::String(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable Int64 vector if possible
    pub fn as_int64_mut(&mut self) -> Option<&mut PhysicalVector<i64>> {
        match self {
            PhysicalVectorEnum::Int64(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable Float64 vector if possible
    pub fn as_float64_mut(&mut self) -> Option<&mut PhysicalVector<f64>> {
        match self {
            PhysicalVectorEnum::Float64(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable Boolean vector if possible
    pub fn as_boolean_mut(&mut self) -> Option<&mut PhysicalVector<bool>> {
        match self {
            PhysicalVectorEnum::Boolean(v) => Some(v),
            _ => None,
        }
    }

    /// Get mutable String vector if possible
    pub fn as_string_mut(&mut self) -> Option<&mut PhysicalVector<String>> {
        match self {
            PhysicalVectorEnum::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Vector is the primary public API for columnar data
/// 
/// Combines logical type information with physical storage representation.
/// This is what operators and expressions work with.
#[derive(Debug, Clone)]
pub struct Vector {
    /// Logical type (SQL-like type)
    pub ty: LogicalType,
    
    /// Physical representation
    pub physical: PhysicalVectorEnum,
}

impl Vector {
    /// Create a new vector with the given logical type and size
    pub fn new(ty: LogicalType, size: usize) -> Self {
        Self {
            ty,
            physical: PhysicalVectorEnum::new(ty, size),
        }
    }

    /// Create an Int64 vector from data
    pub fn from_i64(data: Vec<i64>) -> Self {
        Self {
            ty: LogicalType::Int64,
            physical: PhysicalVectorEnum::Int64(PhysicalVector::from_vec(data)),
        }
    }

    /// Create a Float64 vector from data
    pub fn from_f64(data: Vec<f64>) -> Self {
        Self {
            ty: LogicalType::Float64,
            physical: PhysicalVectorEnum::Float64(PhysicalVector::from_vec(data)),
        }
    }

    /// Create a Boolean vector from data
    pub fn from_bool(data: Vec<bool>) -> Self {
        Self {
            ty: LogicalType::Boolean,
            physical: PhysicalVectorEnum::Boolean(PhysicalVector::from_vec(data)),
        }
    }

    /// Create a String vector from data
    pub fn from_string(data: Vec<String>) -> Self {
        Self {
            ty: LogicalType::String,
            physical: PhysicalVectorEnum::String(PhysicalVector::from_vec(data)),
        }
    }

    /// Create a constant Int64 vector
    pub fn constant_i64(value: i64, len: usize) -> Self {
        Self {
            ty: LogicalType::Int64,
            physical: PhysicalVectorEnum::Int64(PhysicalVector::from_constant(value, len)),
        }
    }

    /// Create a constant Float64 vector
    pub fn constant_f64(value: f64, len: usize) -> Self {
        Self {
            ty: LogicalType::Float64,
            physical: PhysicalVectorEnum::Float64(PhysicalVector::from_constant(value, len)),
        }
    }

    /// Create a constant Boolean vector
    pub fn constant_bool(value: bool, len: usize) -> Self {
        Self {
            ty: LogicalType::Boolean,
            physical: PhysicalVectorEnum::Boolean(PhysicalVector::from_constant(value, len)),
        }
    }

    /// Create a constant String vector
    pub fn constant_string(value: String, len: usize) -> Self {
        Self {
            ty: LogicalType::String,
            physical: PhysicalVectorEnum::String(PhysicalVector::from_constant(value, len)),
        }
    }

    /// Get the length of this vector
    pub fn len(&self) -> usize {
        self.physical.len()
    }

    /// Check if this vector is empty
    pub fn is_empty(&self) -> bool {
        self.physical.is_empty()
    }

    /// Get the logical type
    pub fn logical_type(&self) -> LogicalType {
        self.ty
    }

    /// Copy data from another vector into this one
    /// 
    /// # Errors
    /// Returns an error if the types don't match or lengths differ
    pub fn copy_from(&mut self, other: &Vector) -> Result<(), EvalError> {
        if self.ty != other.ty {
            return Err(EvalError::General(format!(
                "Type mismatch: cannot copy {:?} into {:?}",
                other.ty, self.ty
            )));
        }

        if self.len() != other.len() {
            return Err(EvalError::General(format!(
                "Length mismatch: cannot copy {} elements into vector of length {}",
                other.len(),
                self.len()
            )));
        }

        // Copy data based on type
        match (&mut self.physical, &other.physical) {
            (PhysicalVectorEnum::Int64(dest), PhysicalVectorEnum::Int64(src)) => {
                let dest_slice = dest.as_mut_slice();
                let src_slice = src.as_slice();
                dest_slice.copy_from_slice(src_slice);
            }
            (PhysicalVectorEnum::Float64(dest), PhysicalVectorEnum::Float64(src)) => {
                let dest_slice = dest.as_mut_slice();
                let src_slice = src.as_slice();
                dest_slice.copy_from_slice(src_slice);
            }
            (PhysicalVectorEnum::Boolean(dest), PhysicalVectorEnum::Boolean(src)) => {
                let dest_slice = dest.as_mut_slice();
                let src_slice = src.as_slice();
                dest_slice.copy_from_slice(src_slice);
            }
            (PhysicalVectorEnum::String(dest), PhysicalVectorEnum::String(src)) => {
                let dest_slice = dest.as_mut_slice();
                let src_slice = src.as_slice();
                dest_slice.clone_from_slice(src_slice);
            }
            _ => unreachable!("Type mismatch should have been caught earlier"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_vector() {
        let vec = Vector::from_i64(vec![1, 2, 3, 4, 5]);
        assert_eq!(vec.len(), 5);
        assert_eq!(vec.logical_type(), LogicalType::Int64);
        
        if let PhysicalVectorEnum::Int64(pv) = &vec.physical {
            assert_eq!(pv.as_slice(), &[1, 2, 3, 4, 5]);
        } else {
            panic!("Expected Int64 vector");
        }
    }

    #[test]
    fn test_constant_vector() {
        let vec = Vector::constant_i64(42, 100);
        assert_eq!(vec.len(), 100);
        assert_eq!(vec.logical_type(), LogicalType::Int64);
        
        if let PhysicalVectorEnum::Int64(pv) = &vec.physical {
            assert!(matches!(pv, PhysicalVector::Constant { value: 42, len: 100 }));
        } else {
            panic!("Expected Int64 vector");
        }
    }

    #[test]
    fn test_constant_materialization() {
        let mut vec = Vector::constant_i64(42, 10);
        
        if let PhysicalVectorEnum::Int64(pv) = &mut vec.physical {
            let slice = pv.as_mut_slice();
            slice[0] = 100;
            
            assert_eq!(slice[0], 100);
            assert_eq!(slice[1], 42);
        } else {
            panic!("Expected Int64 vector");
        }
    }

    #[test]
    fn test_slice_operation() {
        let vec = Vector::from_i64(vec![1, 2, 3, 4, 5]);
        
        if let PhysicalVectorEnum::Int64(pv) = &vec.physical {
            let sliced = pv.slice(1, 3);
            assert_eq!(sliced.as_slice(), &[2, 3, 4]);
        } else {
            panic!("Expected Int64 vector");
        }
    }

    #[test]
    fn test_cow_on_shared() {
        let vec1 = Vector::from_i64(vec![1, 2, 3, 4, 5]);
        let mut vec2 = vec1.clone();
        
        if let PhysicalVectorEnum::Int64(pv) = &mut vec2.physical {
            pv.as_mut_slice()[0] = 100;
        }
        
        // vec1 should be unchanged
        if let PhysicalVectorEnum::Int64(pv) = &vec1.physical {
            assert_eq!(pv.as_slice()[0], 1);
        }
        
        // vec2 should be changed
        if let PhysicalVectorEnum::Int64(pv) = &vec2.physical {
            assert_eq!(pv.as_slice()[0], 100);
        }
    }
}
