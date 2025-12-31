use crate::expr::executor::ExecInput;
use wide::i64x4;

/// SIMD helper: Vector * Vector (flat, no selection)
#[inline]
pub(crate) unsafe fn simd_mul_i64_vv(lhs: *const i64, rhs: *const i64, out: *mut i64, len: usize) {
    const LANES: usize = 4;
    let chunks = len / LANES;

    for i in 0..chunks {
        let offset = i * LANES;

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

        let result = lhs_vec * rhs_vec;
        let result_array = result.to_array();
        *out.add(offset) = result_array[0];
        *out.add(offset + 1) = result_array[1];
        *out.add(offset + 2) = result_array[2];
        *out.add(offset + 3) = result_array[3];
    }

    for i in (chunks * LANES)..len {
        *out.add(i) = *lhs.add(i) * *rhs.add(i);
    }
}

/// SIMD helper: Vector * Constant (broadcast)
#[inline]
pub(crate) unsafe fn simd_mul_i64_vc(vec: *const i64, constant: i64, out: *mut i64, len: usize) {
    const LANES: usize = 4;
    let chunks = len / LANES;
    let constant_vec = i64x4::splat(constant);

    for i in 0..chunks {
        let offset = i * LANES;

        let vec_simd = i64x4::new([
            *vec.add(offset),
            *vec.add(offset + 1),
            *vec.add(offset + 2),
            *vec.add(offset + 3),
        ]);

        let result = vec_simd * constant_vec;
        let result_array = result.to_array();
        *out.add(offset) = result_array[0];
        *out.add(offset + 1) = result_array[1];
        *out.add(offset + 2) = result_array[2];
        *out.add(offset + 3) = result_array[3];
    }

    for i in (chunks * LANES)..len {
        *out.add(i) = *vec.add(i) * constant;
    }
}

/// Scalar fallback: handles selection vectors and edge cases
#[inline]
pub(crate) unsafe fn scalar_mul_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: *mut i64,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = lhs.get_unchecked(i) * rhs.get_unchecked(i);
    }
}

/// Int64 multiplication kernel with SIMD
#[inline]
pub(crate) unsafe fn kernel_mul_i64(
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
            simd_mul_i64_vv(lhs.data, rhs.data, out_ptr, len);
        }
        (false, true, false, _) => {
            simd_mul_i64_vc(lhs.data, *rhs.data, out_ptr, len);
        }
        (true, false, _, false) => {
            simd_mul_i64_vc(rhs.data, *lhs.data, out_ptr, len);
        }
        (true, true, _, _) => {
            let result = *lhs.data * *rhs.data;
            for i in 0..len {
                *out_ptr.add(i) = result;
            }
        }
        _ => {
            scalar_mul_i64(lhs, rhs, out_ptr, len);
        }
    }
}
