use crate::expr::executor::ExecInput;

/// Scalar implementation: Vector / Vector
/// Note: i64 division is not available in SIMD for wide crate, so we use scalar
#[inline]
pub(crate) unsafe fn scalar_div_i64_vv(
    lhs: *const i64,
    rhs: *const i64,
    out: *mut i64,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = *lhs.add(i) / *rhs.add(i);
    }
}

/// Scalar implementation: Vector / Constant
#[inline]
pub(crate) unsafe fn scalar_div_i64_vc(vec: *const i64, constant: i64, out: *mut i64, len: usize) {
    for i in 0..len {
        *out.add(i) = *vec.add(i) / constant;
    }
}

/// Scalar implementation: Constant / Vector
#[inline]
pub(crate) unsafe fn scalar_div_i64_cv(constant: i64, vec: *const i64, out: *mut i64, len: usize) {
    for i in 0..len {
        *out.add(i) = constant / *vec.add(i);
    }
}

/// Scalar fallback: handles selection vectors and edge cases
#[inline]
pub(crate) unsafe fn scalar_div_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: *mut i64,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = lhs.get_unchecked(i) / rhs.get_unchecked(i);
    }
}

/// Int64 division kernel
/// Note: Uses scalar operations as i64 division is not available in SIMD
#[inline]
pub(crate) unsafe fn kernel_div_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: &mut [i64],
    len: usize,
) {
    let out_ptr = out.as_mut_ptr();

    match (
        lhs.is_constant,
        rhs.is_constant,
        lhs.selection.is_some(),
        rhs.selection.is_some(),
    ) {
        (false, false, false, false) => {
            scalar_div_i64_vv(lhs.data, rhs.data, out_ptr, len);
        }
        (false, true, false, _) => {
            scalar_div_i64_vc(lhs.data, *rhs.data, out_ptr, len);
        }
        (true, false, _, false) => {
            scalar_div_i64_cv(*lhs.data, rhs.data, out_ptr, len);
        }
        (true, true, _, _) => {
            let result = *lhs.data / *rhs.data;
            for i in 0..len {
                *out_ptr.add(i) = result;
            }
        }
        _ => {
            scalar_div_i64(lhs, rhs, out_ptr, len);
        }
    }
}
