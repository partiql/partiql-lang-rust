# Approach 2: Selection-Aware Output - Changes Required

## What Would Change in add_i64.rs

### Current (Approach 1 - Dense Output)

```rust
/// Scalar fallback: handles selection vectors and edge cases
#[inline]
pub(crate) unsafe fn scalar_add_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: *mut i64,
    len: usize,
) {
    for i in 0..len {
        // Read from potentially sparse physical indices (via selection)
        // Write to dense output at logical index i
        *out.add(i) = lhs.get_unchecked(i) + rhs.get_unchecked(i);
    }
}
```

### Approach 2 (Selection-Aware Output)

```rust
/// Scalar fallback: handles selection vectors for both input AND output
#[inline]
pub(crate) unsafe fn scalar_add_i64(
    lhs: ExecInput<i64>,
    rhs: ExecInput<i64>,
    out: *mut i64,
    out_selection: Option<*const usize>,  // NEW PARAMETER
    len: usize,
) {
    for i in 0..len {
        // Read from potentially sparse physical indices (via selection)
        let result = lhs.get_unchecked(i) + rhs.get_unchecked(i);
        
        // Write to output - either dense OR sparse based on out_selection
        let out_idx = if let Some(sel_ptr) = out_selection {
            *sel_ptr.add(i)  // Write to sparse physical index
        } else {
            i  // Write densely
        };
        *out.add(out_idx) = result;
    }
}
```

## Example Execution Comparison

**Input:**
- lhs: [10, 20, 30, 40, 50] with selection [0, 2, 4]
- rhs: [1, 2, 3, 4, 5] with selection [0, 2, 4]

### Approach 1 (Dense Output) - What We Implemented
```rust
Output buffer: [?, ?, ?]  // 3 elements allocated
Loop iterations:
  i=0: read lhs[0]=10, rhs[0]=1, write out[0]=11
  i=1: read lhs[2]=30, rhs[2]=3, write out[1]=33
  i=2: read lhs[4]=50, rhs[4]=5, write out[2]=55
Result: [11, 33, 55]  // Dense, 3 elements
```

### Approach 2 (Selection-Aware Output) - The Alternative
```rust
Output buffer: [?, ?, ?, ?, ?]  // 5 elements allocated (full size)
out_selection: [0, 2, 4]  // Same as input selection

Loop iterations:
  i=0: read lhs[0]=10, rhs[0]=1, write out[0]=11
  i=1: read lhs[2]=30, rhs[2]=3, write out[2]=33
  i=2: read lhs[4]=50, rhs[4]=5, write out[4]=55
Result: [11, ?, 33, ?, 55]  // Sparse, 5 slots with holes
```

## Is It a Simple Change?

**In add_i64.rs alone: YES, very simple!**
- Add one parameter: `out_selection: Option<*const usize>`
- Add 4 lines in the loop to handle output index mapping

**But the complexity is elsewhere:**

### 1. Every Kernel Must Change
All operators need the same signature change:
- scalar_add_i64_vv, scalar_add_i64_vc, scalar_add_i64_cv
- sub_i64, mul_i64, div_i64, gt_i64, lt_i64, eq_i64, etc.
- **ALL comparison operators, ALL arithmetic operators**

### 2. Optimized Paths Break
The fast SIMD paths assume dense output:
```rust
(false, false, false, false) => {
    scalar_add_i64_vv(lhs.data, rhs.data, out_ptr, len);
    // ^^^ This assumes dense writes - breaks with selection output!
}
```
You'd need to either:
- Disable SIMD when output has selection (slow!)
- OR write selection-aware SIMD kernels (complex!)

### 3. ExecInput Needs Output Selection
```rust
pub struct ExecInput<'a, T: Copy> {
    pub data: *const T,
    pub selection: Option<*const usize>,  // Input selection
    pub out_selection: Option<*const usize>,  // NEW: Output selection
    pub len: usize,
    pub is_constant: bool,
    _marker: PhantomData<&'a T>,
}
```

### 4. Executor Complexity Increases
```rust
fn execute_binary_i64_to_i64(...) {
    // Need to determine output selection
    let out_selection = match selection {
        Some(sel) => Some(sel.indices.as_ptr()),  // Propagate?
        None => None,  // Or create new?
    };
    
    // Pass to every kernel
    kernel(lhs, rhs, out, out_selection, len);
}
```

### 5. Selection Combination Problem
**Can two inputs have different selections?**

**Answer: NO, not in the current architecture!**

Here's why:
1. **One selection per batch**: `VectorizedBatch` has a single selection vector for the entire batch
   ```rust
   pub struct VectorizedBatch {
       columns: Vec<Vector>,
       selection: Option<SelectionVector>,  // ONE for all columns
   }
   ```

2. **Selection is row-level, not column-level**: Selection represents "which rows are active"
   - All columns in a batch share the same row selection
   - Example: Selection [0, 2, 4] means "rows 0, 2, 4 are valid for ALL columns"

3. **Executor passes same selection to all inputs**:
   ```rust
   fn execute_binary_i64_to_i64(..., selection: Option<&SelectionVector>) {
       let lhs = self.decode_input_i64(&inputs[0], batch, selection)?;
       let rhs = self.decode_input_i64(&inputs[1], batch, selection)?;
       // ^^^ Both get the SAME selection
   }
   ```

**What about scratch registers from previous operations?**
- With Approach 1 (current): Scratch output is dense, no selection → operates on all rows
- With Approach 2: Scratch would maintain selection → but still same selection as batch

**So Approach 2 wouldn't have this problem!** 
All inputs to an operation always share the same selection vector (or none).

### 6. Memory Management
- Must allocate full-size buffers even when 99% filtered
- Scratch vectors need to track both physical size and logical length
- More complex memory patterns hurt cache performance

## The Verdict

**Simple in add_i64.rs? Yes - about 10 lines of code.**

**Simple for the system? Still No - but less complex than initially thought:**

**Actual complexities:**
- ❌ Every operator signature changes (all kernels need `out_selection` parameter)
- ❌ SIMD optimizations break or need rewriting (scatter stores needed)
- ❌ Memory overhead for sparse storage (allocate full batch size even when 1% selected)
- ❌ Cache performance degrades (scattered writes, poor locality)
- ❌ Testing complexity increases dramatically

**Non-issue (eliminated):**
- ✅ Selection combination is NOT a problem (batch-level selection, same for all columns)

**So Approach 2 is simpler than initially described, but still more complex than Approach 1.**

The main reasons industry chose Approach 1:
1. **Better cache performance** - dense writes, better locality
2. **SIMD-friendly** - contiguous writes enable vectorization
3. **Memory efficiency** - allocate only what's needed
4. **Simpler testing** - output is always predictable dense format

Approach 2's main benefit (zero-copy filtering) only helps for pure filter operations, not arithmetic/compute operations where you're creating new data anyway.
