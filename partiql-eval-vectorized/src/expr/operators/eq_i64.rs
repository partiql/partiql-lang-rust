use crate::expr::executor::ExecInput;
use wide::{i64x4, CmpEq};

/// SIMD helper: Vector == Vector (flat, no selection)
/// 
/// Uses `wide::i64x4` for 4-way SIMD comparison.
/// On x86 AVX2: processes 4 i64s per instruction
/// On ARM NEON: processes 2 i64s per instruction (auto-adapted by wide crate)
///
/// # Safety
/// - lhs and rhs must point to valid memory with at least `len` elements
/// - out must point to valid memory with at least `len` elements
#[inline]
pub(crate) unsafe fn simd_eq_i64_vv(
    lhs: *const i64,
    rhs: *const i64,
    out: *mut bool,
    len: usize,
) {
    const LANES: usize = 4; // Process 4 i64s at once
    let chunks = len / LANES;
    
    // SIMD path: process 4 elements at a time
    for i in 0..chunks {
        let offset = i * LANES;
        
        // Load 4 i64s from each vector
        let lhs_vec = i64x4::new([
            *lhs.add(offset),
            *lhs.add(offset + 1),
            *lhs.add(offset + 2),
            *lhs.add(offset + 3),
        ]);
        let rhs_vec = i64x4::new([
            *rhs.add(offset),
            *rhs.add(offset + 1),
            *rhs.add(offset + 2),
            *rhs.add(offset + 3),
        ]);
        
        // Compare: returns i64x4 with all bits set (-1) for true, 0 for false
        let cmp = lhs_vec.cmp_eq(rhs_vec);
        
        // Extract results as bitmask and write to output
        // move_mask() extracts the sign bit from each lane into a u16
        let mask = cmp.move_mask();
        *out.add(offset) = (mask & 1) != 0;
        *out.add(offset + 1) = (mask & 2) != 0;
        *out.add(offset + 2) = (mask & 4) != 0;
        *out.add(offset + 3) = (mask & 8) != 0;
    }
    
    // Scalar tail: handle remaining elements (< 4)
    for i in (chunks * LANES)..len {
        *out.add(i) = *lhs.add(i) == *rhs.add(i);
    }
}

/// SIMD helper: Vector == Constant (broadcast)
/// 
/// Broadcasts the constant across SIMD lanes and compares against vector.
///
/// # Safety
/// - vec must point to valid memory with at least `len` elements
/// - out must point to valid memory with at least `len` elements
#[inline]
pub(crate) unsafe fn simd_eq_i64_vc(
    vec: *const i64,
    constant: i64,
    out: *mut bool,
    len: usize,
) {
    const LANES: usize = 4;
    let chunks = len / LANES;
    
    // Broadcast constant across all SIMD lanes
    let constant_vec = i64x4::splat(constant);
    
    // SIMD path: process 4 elements at a time
    for i in 0..chunks {
        let offset = i * LANES;
        
        let vec_simd = i64x4::new([
            *vec.add(offset),
            *vec.add(offset + 1),
            *vec.add(offset + 2),
            *vec.add(offset + 3),
        ]);
        
        let cmp = vec_simd.cmp_eq(constant_vec);
        let mask = cmp.move_mask();
        *out.add(offset) = (mask & 1) != 0;
        *out.add(offset + 1) = (mask & 2) != 0;
        *out.add(offset + 2) = (mask & 4) != 0;
        *out.add(offset + 3) = (mask & 8) != 0;
    }
    
    // Scalar tail
    for i in (chunks * LANES)..len {
        *out.add(i) = *vec.add(i) == constant;
    }
}

/// Scalar fallback: handles selection vectors and edge cases
/// 
/// Used when:
/// - Either input has a selection vector (sparse access)
/// - For very small batches where SIMD overhead isn't worth it
///
/// # Safety
/// - Must respect the ExecInput contracts (selection vector validity, data pointer)
#[inline]
pub(crate) unsafe fn scalar_eq_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: *mut bool,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = lhs.get_unchecked(i) == rhs.get_unchecked(i);
    }
}

/// Int64 equality kernel with SIMD - handles all input combinations
/// 
/// Dispatches to optimal implementation based on input characteristics:
/// - SIMD fast path: flat vectors without selection (common in filters)
/// - SIMD broadcast: one vector, one constant (also common)
/// - Scalar fallback: selection vectors, constants, edge cases
///
/// # Safety
/// Caller must ensure len <= out.len()
#[inline]
pub(crate) unsafe fn kernel_eq_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: &mut [bool],
    len: usize,
) {
    let out_ptr = out.as_mut_ptr();
    
    match (lhs.is_constant, rhs.is_constant, lhs.selection.is_some(), rhs.selection.is_some()) {
        // Fast path: Vector == Vector, both flat (no selection) → SIMD
        (false, false, false, false) => {
            simd_eq_i64_vv(lhs.data, rhs.data, out_ptr, len);
        }
        
        // Medium path: Vector == Constant, vector is flat → SIMD broadcast
        (false, true, false, _) => {
            simd_eq_i64_vc(lhs.data, *rhs.data, out_ptr, len);
        }
        
        // Medium path: Constant == Vector, vector is flat → SIMD broadcast
        (true, false, _, false) => {
            simd_eq_i64_vc(rhs.data, *lhs.data, out_ptr, len);
        }
        
        // Special case: Constant == Constant → single comparison, fill all
        (true, true, _, _) => {
            let result = *lhs.data == *rhs.data;
            for i in 0..len {
                *out_ptr.add(i) = result;
            }
        }
        
        // Fallback: Selection vectors or other complex cases → scalar
        _ => {
            scalar_eq_i64(lhs, rhs, out_ptr, len);
        }
    }
}
