use crate::expr::executor::ExecInput;

/// Scalar implementation: NOT Vector (always dense output)
#[inline]
pub(crate) unsafe fn scalar_not_bool_v(
    vec: *const bool,
    out: *mut bool,
    len: usize,
) {
    for i in 0..len {
        *out.add(i) = !*vec.add(i);
    }
}

/// Scalar fallback: handles selection vectors and edge cases
///
/// Selection Vector Behavior (Approach 2):
/// - Input: `get_unchecked(i)` maps logical index to physical index via input selection
/// - Output: Writes to sparse physical indices if out_selection present, dense otherwise
#[inline]
pub(crate) unsafe fn scalar_not_bool(
    input: ExecInput<bool>,
    out: *mut bool,
    out_selection: Option<*const usize>,
    len: usize,
) {
    if let Some(sel_ptr) = out_selection {
        // Sparse output
        for i in 0..len {
            let out_idx = *sel_ptr.add(i);
            *out.add(out_idx) = !input.get_unchecked(i);
        }
    } else {
        // Dense output
        for i in 0..len {
            *out.add(i) = !input.get_unchecked(i);
        }
    }
}

/// Boolean NOT kernel (unary operation)
#[inline]
pub(crate) unsafe fn kernel_not_bool(
    input: ExecInput<bool>,
    out: &mut [bool],
    out_selection: Option<*const usize>,
    len: usize,
) {
    let out_ptr = out.as_mut_ptr();

    match (input.is_constant, input.selection.is_some()) {
        (false, false) => {
            // No input selection = write densely
            scalar_not_bool_v(input.data, out_ptr, len);
        }
        (true, _) => {
            // Constant input = fill all with negated constant
            let result = !*input.data;
            for i in 0..len {
                *out_ptr.add(i) = result;
            }
        }
        _ => {
            // Fallback handles selection - pass through out_selection
            scalar_not_bool(input, out_ptr, out_selection, len);
        }
    }
}
