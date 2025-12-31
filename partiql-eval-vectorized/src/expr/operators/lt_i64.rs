use crate::expr::executor::ExecInput;

/// Scalar implementation: Vector < Vector
///
/// Selection Vector Behavior (Approach 2):
/// - Inputs: `get_unchecked(i)` maps logical index to physical index via input selection
/// - Output: Writes to sparse physical indices if out_selection present, dense otherwise
#[inline]
pub(crate) unsafe fn scalar_lt_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: *mut bool,
    out_selection: Option<*const usize>,
    len: usize,
) {
    if let Some(sel_ptr) = out_selection {
        // Sparse output
        for i in 0..len {
            let out_idx = *sel_ptr.add(i);
            *out.add(out_idx) = lhs.get_unchecked(i) < rhs.get_unchecked(i);
        }
    } else {
        // Dense output
        for i in 0..len {
            *out.add(i) = lhs.get_unchecked(i) < rhs.get_unchecked(i);
        }
    }
}

/// Int64 less-than kernel
#[inline]
pub(crate) unsafe fn kernel_lt_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: &mut [bool],
    out_selection: Option<*const usize>,
    len: usize,
) {
    let out_ptr = out.as_mut_ptr();

    // Check if we can use optimized dense path
    if lhs.selection.is_none() && rhs.selection.is_none() && out_selection.is_none() {
        // Optimized dense path
        match (lhs.is_constant, rhs.is_constant) {
            (false, false) => {
                for i in 0..len {
                    *out_ptr.add(i) = *lhs.data.add(i) < *rhs.data.add(i);
                }
            }
            (false, true) => {
                let c = *rhs.data;
                for i in 0..len {
                    *out_ptr.add(i) = *lhs.data.add(i) < c;
                }
            }
            (true, false) => {
                let c = *lhs.data;
                for i in 0..len {
                    *out_ptr.add(i) = c < *rhs.data.add(i);
                }
            }
            (true, true) => {
                let result = *lhs.data < *rhs.data;
                for i in 0..len {
                    *out_ptr.add(i) = result;
                }
            }
        }
    } else {
        // Fallback for selection vectors
        scalar_lt_i64(lhs, rhs, out_ptr, out_selection, len);
    }
}
