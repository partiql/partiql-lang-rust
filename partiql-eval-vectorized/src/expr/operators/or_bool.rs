use crate::expr::executor::ExecInput;

/// Scalar implementation: Vector OR Vector (always dense output)
#[inline]
pub(crate) unsafe fn scalar_or_bool_vv(
    lhs: *const bool,
    rhs: *const bool,
    out: *mut bool,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = *lhs.add(i) || *rhs.add(i);
    }
}

/// Scalar implementation: Vector OR Constant (always dense output)
#[inline]
pub(crate) unsafe fn scalar_or_bool_vc(
    vec: *const bool,
    constant: bool,
    out: *mut bool,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = *vec.add(i) || constant;
    }
}

/// Scalar implementation: Constant OR Vector (always dense output)
#[inline]
pub(crate) unsafe fn scalar_or_bool_cv(
    constant: bool,
    vec: *const bool,
    out: *mut bool,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = constant || *vec.add(i);
    }
}

/// Scalar fallback: handles selection vectors and edge cases
///
/// Selection Vector Behavior (Approach 2):
/// - Inputs: `get_unchecked(i)` maps logical index to physical index via input selection
/// - Output: Writes to sparse physical indices if out_selection present, dense otherwise
#[inline]
pub(crate) unsafe fn scalar_or_bool(
    lhs: ExecInput<bool>,
    rhs: ExecInput<bool>,
    out: *mut bool,
    out_selection: Option<*const usize>,
    len: usize,
) {
    if let Some(sel_ptr) = out_selection {
        // Sparse output
        for i in 0..len {
            let out_idx = *sel_ptr.add(i);
            *out.add(out_idx) = lhs.get_unchecked(i) || rhs.get_unchecked(i);
        }
    } else {
        // Dense output
        for i in 0..len {
            *out.add(i) = lhs.get_unchecked(i) || rhs.get_unchecked(i);
        }
    }
}

/// Boolean OR kernel
#[inline]
pub(crate) unsafe fn kernel_or_bool(
    lhs: ExecInput<bool>,
    rhs: ExecInput<bool>,
    out: &mut [bool],
    out_selection: Option<*const usize>,
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
            // No input selections = write densely
            scalar_or_bool_vv(lhs.data, rhs.data, out_ptr, len);
        }
        (false, true, false, _) => {
            // No input selections = write densely
            scalar_or_bool_vc(lhs.data, *rhs.data, out_ptr, len);
        }
        (true, false, _, false) => {
            // No input selections = write densely
            scalar_or_bool_cv(*lhs.data, rhs.data, out_ptr, len);
        }
        (true, true, _, _) => {
            // Both constants = no selection needed
            let result = *lhs.data || *rhs.data;
            for i in 0..len {
                *out_ptr.add(i) = result;
            }
        }
        _ => {
            // Fallback handles selections - pass through out_selection
            scalar_or_bool(lhs, rhs, out_ptr, out_selection, len);
        }
    }
}
