use crate::expr::executor::ExecInput;

/// Int64 not-equal kernel
#[inline]
pub(crate) unsafe fn kernel_ne_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: &mut [bool],
    len: usize,
) {
    let out_ptr = out.as_mut_ptr();

    match (lhs.is_constant, rhs.is_constant) {
        (false, false) => {
            for i in 0..len {
                *out_ptr.add(i) = lhs.get_unchecked(i) != rhs.get_unchecked(i);
            }
        }
        (false, true) => {
            let c = *rhs.data;
            for i in 0..len {
                *out_ptr.add(i) = lhs.get_unchecked(i) != c;
            }
        }
        (true, false) => {
            let c = *lhs.data;
            for i in 0..len {
                *out_ptr.add(i) = c != rhs.get_unchecked(i);
            }
        }
        (true, true) => {
            let result = *lhs.data != *rhs.data;
            for i in 0..len {
                *out_ptr.add(i) = result;
            }
        }
    }
}
