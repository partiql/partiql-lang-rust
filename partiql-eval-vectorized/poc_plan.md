# PartiQL Vectorized Evaluator - Proof of Concept Plan

## High-Level Overview

### Goal
Create a vectorized evaluation engine for PartiQL that processes data in batches using columnar storage, demonstrating significant performance improvements over the current tuple-at-a-time evaluator.

### Scope - PoC Query
```sql
SELECT a, b FROM data WHERE a > 10 AND b < 100
```

This query exercises:
- **Scan**: Read data in batches
- **Filter**: Apply predicate with comparison and logical operations
- **Project**: Select specific columns

### Key Design Decisions
1. **No NULL/MISSING handling** - Simplified for PoC
2. **Columnar storage** - Data stored in type-specific vectors (`PVector`)
3. **Batch processing** - Process 1024 rows at a time
4. **Type specialization** - Separate implementations per type (Int64, Float64, Boolean, String)
5. **Output parameters** - Functions write to pre-allocated buffers
6. **Inline type checking** - Type resolution during plan conversion
7. **Volcano-style interface** - Iterator-based `next_batch()` pattern

### Architecture Diagram
```
LogicalPlan<BindingsOp>
         ↓
    [VectorizedPlanBuilder]
         ↓ (with SourceTypeDef + FnRegistry)
    VectorizedPlan
         ↓
    [VectorizedExecutor]
         ↓
    VectorizedBatch (columnar)
         ↓
    Results
```

---

## Phase 1: Core Abstractions & Type System

### Phase 1.1: PVector (Physical Vector)

**Purpose**: Type-specific columnar storage without null handling.

**Interface**:
```rust
/// Physical vector - type-specific columnar storage
#[derive(Debug, Clone)]
pub enum PVector {
    Int64(Vec<i64>),
    Float64(Vec<f64>),
    Boolean(Vec<bool>),
    String(Vec<String>),
}

impl PVector {
    /// Create new vector of given type pre-allocated with size
    /// All elements initialized to default values (0, false, empty string)
    pub fn new(type_info: TypeInfo, size: usize) -> Self;
    
    /// Get number of elements
    pub fn len(&self) -> usize;
    
    /// Copy data from another vector
    pub fn copy_from(&mut self, other: &PVector) -> Result<(), EvalError>;
    
    /// Get type information
    pub fn type_info(&self) -> TypeInfo;
}
```

**Pseudocode**:
```rust
// Create pre-allocated vector
let mut vec = PVector::new(TypeInfo::Int64, 1024);

// Write to indices directly (no push needed)
if let PVector::Int64(ref mut data) = vec {
    data[0] = 42;
    data[100] = 100;
}

// Length
assert_eq!(vec.len(), 1024);  // Pre-allocated size

// Implementation of new():
impl PVector {
    pub fn new(type_info: TypeInfo, size: usize) -> Self {
        match type_info {
            TypeInfo::Int64 => PVector::Int64(vec![0; size]),
            TypeInfo::Float64 => PVector::Float64(vec![0.0; size]),
            TypeInfo::Boolean => PVector::Boolean(vec![false; size]),
            TypeInfo::String => PVector::String(vec![String::new(); size]),
        }
    }
}
```

---

### Phase 1.2: TypeInfo and SourceTypeDef

**Purpose**: Type system and schema definition.

**Interface**:
```rust
/// Type information for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    Int64,
    Float64,
    Boolean,
    String,
}

/// Source schema definition
#[derive(Debug, Clone)]
pub struct SourceTypeDef {
    fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_info: TypeInfo,
}

impl SourceTypeDef {
    pub fn new(fields: Vec<Field>) -> Self;
    
    /// Get type for a column by name
    pub fn get_type(&self, name: &str) -> Result<TypeInfo, PlanError>;
    
    /// Get column index by name
    pub fn get_column_index(&self, name: &str) -> Result<usize, PlanError>;
    
    /// Get field by index
    pub fn get_field(&self, idx: usize) -> Option<&Field>;
    
    /// Number of fields
    pub fn field_count(&self) -> usize;
}
```

**Pseudocode**:
```rust
// Define schema
let schema = SourceTypeDef::new(vec![
    Field { name: "a".to_string(), type_info: TypeInfo::Int64 },
    Field { name: "b".to_string(), type_info: TypeInfo::Int64 },
    Field { name: "name".to_string(), type_info: TypeInfo::String },
]);

// Lookup
let a_type = schema.get_type("a")?; // TypeInfo::Int64
let a_idx = schema.get_column_index("a")?; // 0
```

---

### Phase 1.3: VectorizedFn Trait

**Purpose**: Interface for vectorized functions with output parameter pattern.

**Interface**:
```rust
/// Vectorized function that operates on column vectors
/// 
/// Contract: Output must be pre-allocated by caller with correct size and type.
/// Functions write directly to output indices without resizing.
pub trait VectorizedFn: Debug {
    /// Execute function, writing results to pre-allocated output
    /// 
    /// Preconditions:
    /// - output.len() == inputs[0].len() (all inputs same length)
    /// - output type matches expected output type
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) 
        -> Result<(), EvalError>;
    
    /// Get function identifier
    fn fn_id(&self) -> FnId;
    
    /// Get output type given input types
    fn output_type(&self, input_types: &[TypeInfo]) -> TypeInfo;
}

/// Function identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnId {
    pub name: &'static str,
    pub id: u32,
    pub signature: Vec<TypeInfo>,
}
```

**Pseudocode (Scalar Version)**:
```rust
// Example: Greater-than for Int64 (scalar loop)
struct VecGtInt64;

impl VectorizedFn for VecGtInt64 {
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        // Extract input vectors
        let left = match inputs[0] {
            PVector::Int64(v) => v,
            _ => return Err(EvalError::TypeMismatch),
        };
        let right = match inputs[1] {
            PVector::Int64(v) => v,
            _ => return Err(EvalError::TypeMismatch),
        };
        
        // Get output vector (pre-allocated by caller)
        let result = match output {
            PVector::Boolean(ref mut v) => v,
            _ => return Err(EvalError::TypeMismatch),
        };
        
        // Direct write to indices (no push/resize)
        for i in 0..left.len() {
            result[i] = left[i] > right[i];
        }
        
        Ok(())
    }
    
    fn fn_id(&self) -> FnId {
        FnId {
            name: "gt",
            id: 1,
            signature: vec![TypeInfo::Int64, TypeInfo::Int64],
        }
    }
    
    fn output_type(&self, _input_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
}
```

---

### Phase 1.4: VectorizedFnRegistry

**Purpose**: Register and resolve vectorized functions by operation and types.

**Interface**:
```rust
pub struct VectorizedFnRegistry {
    functions: HashMap<FnKey, Box<dyn VectorizedFn>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FnKey {
    op_type: OpType,
    signature: Vec<TypeInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OpType {
    Binary(BinaryOp),
    Unary(UnaryOp),
}

impl VectorizedFnRegistry {
    pub fn new() -> Self;
    
    /// Create registry with all default functions
    pub fn default() -> Self;
    
    /// Register a function
    pub fn register(
        &mut self, 
        op: OpType, 
        signature: Vec<TypeInfo>,
        func: Box<dyn VectorizedFn>
    );
    
    /// Resolve binary operator to vectorized function
    pub fn resolve_binary_op(
        &self,
        op: BinaryOp,
        lhs_type: TypeInfo,
        rhs_type: TypeInfo,
    ) -> Option<Box<dyn VectorizedFn>>;
}
```

**Pseudocode**:
```rust
// Build registry
let mut registry = VectorizedFnRegistry::new();

// Register functions
registry.register(
    OpType::Binary(BinaryOp::Gt),
    vec![TypeInfo::Int64, TypeInfo::Int64],
    Box::new(VecGtInt64),
);

registry.register(
    OpType::Binary(BinaryOp::Lt),
    vec![TypeInfo::Int64, TypeInfo::Int64],
    Box::new(VecLtInt64),
);

registry.register(
    OpType::Binary(BinaryOp::And),
    vec![TypeInfo::Boolean, TypeInfo::Boolean],
    Box::new(VecAnd),
);

// Resolve function
let gt_fn = registry.resolve_binary_op(
    BinaryOp::Gt,
    TypeInfo::Int64,
    TypeInfo::Int64,
).expect("function exists");
```

---

## Phase 2: Data Ingestion

### Phase 2.1: BatchReader Trait

**Purpose**: Iterator interface for reading batches of data.

**Interface**:
```rust
/// Reads data in batches
pub trait BatchReader {
    /// Get next batch, returns None when exhausted
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;
    
    /// Get schema
    fn schema(&self) -> &SourceTypeDef;
}
```

---

### Phase 2.2: VectorizedBatch

**Purpose**: Container for a batch of columnar data.

**Interface**:
```rust
/// Batch of columnar data
#[derive(Debug, Clone)]
pub struct VectorizedBatch {
    columns: Vec<PVector>,
    row_count: usize,
    schema: SourceTypeDef,
}

impl VectorizedBatch {
    pub fn new(schema: SourceTypeDef, capacity: usize) -> Self;
    
    /// Get number of rows in batch
    pub fn row_count(&self) -> usize;
    
    /// Get column by index
    pub fn column(&self, idx: usize) -> Result<&PVector, EvalError>;
    
    /// Get mutable column by index
    pub fn column_mut(&mut self, idx: usize) -> Result<&mut PVector, EvalError>;
    
    /// Get schema
    pub fn schema(&self) -> &SourceTypeDef;
    
    /// Clear all columns, retaining capacity
    pub fn clear(&mut self);
}
```

**Pseudocode**:
```rust
// Create batch
let schema = SourceTypeDef::new(vec![
    Field { name: "a".to_string(), type_info: TypeInfo::Int64 },
    Field { name: "b".to_string(), type_info: TypeInfo::Int64 },
]);

let batch = VectorizedBatch::new(schema, 1024);

// Access columns
let col_a = batch.column(0)?;
assert_eq!(batch.row_count(), 0);
```

---

### Phase 2.3: TupleIteratorReader

**Purpose**: Convert PartiQL Value/Tuple stream to columnar batches.

**Interface**:
```rust
pub struct TupleIteratorReader {
    iter: Box<dyn Iterator<Item = Tuple>>,
    schema: SourceTypeDef,
    batch_size: usize,
}

impl TupleIteratorReader {
    pub fn new(
        iter: Box<dyn Iterator<Item = Tuple>>,
        schema: SourceTypeDef,
        batch_size: usize,
    ) -> Self;
}

impl BatchReader for TupleIteratorReader {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Create empty batch
        let mut batch = VectorizedBatch::new(
            self.schema.clone(), 
            self.batch_size
        );
        
        let mut count = 0;
        
        // Fill batch
        while count < self.batch_size {
            match self.iter.next() {
                None => break,
                Some(tuple) => {
                    // Extract values and append to columns
                    for (idx, field) in self.schema.fields.iter().enumerate() {
                        let value = tuple.get(&field.name)?;
                        self.append_value(batch.column_mut(idx)?, value)?;
                    }
                    count += 1;
                }
            }
        }
        
        if count == 0 {
            Ok(None)
        } else {
            batch.row_count = count;
            Ok(Some(batch))
        }
    }
    
    fn schema(&self) -> &SourceTypeDef {
        &self.schema
    }
}
```

---

## Phase 3: Vectorized Operations

### Phase 3.1: Comparison Operations

**Functions to implement (scalar versions first)**:
- `VecGtInt64`: a > b
- `VecLtInt64`: a < b
- `VecGteInt64`: a >= b
- `VecLteInt64`: a <= b
- `VecEqInt64`: a == b
- `VecNeqInt64`: a != b

**Implementation Pattern (Scalar)**:
```rust
struct VecLtInt64;

impl VectorizedFn for VecLtInt64 {
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        let left = extract_int64(inputs[0])?;
        let right = extract_int64(inputs[1])?;
        let result = extract_bool_mut(output)?;
        
        // Direct write to pre-allocated output
        for i in 0..left.len() {
            result[i] = left[i] < right[i];
        }
        
        Ok(())
    }
    
    fn fn_id(&self) -> FnId {
        FnId { name: "lt", id: 2, signature: vec![TypeInfo::Int64, TypeInfo::Int64] }
    }
    
    fn output_type(&self, _: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
}
```

---

### Phase 3.2: Logical Operations

**Functions to implement**:
- `VecAnd`: a AND b
- `VecOr`: a OR b
- `VecNot`: NOT a

**Implementation Pattern**:
```rust
struct VecAnd;

impl VectorizedFn for VecAnd {
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        let left = extract_bool(inputs[0])?;
        let right = extract_bool(inputs[1])?;
        let result = extract_bool_mut(output)?;
        
        result.clear();
        result.reserve(left.len());
        
        for i in 0..left.len() {
            result.push(left[i] && right[i]);
        }
        
        Ok(())
    }
    
    fn fn_id(&self) -> FnId {
        FnId { name: "and", id: 10, signature: vec![TypeInfo::Boolean, TypeInfo::Boolean] }
    }
    
    fn output_type(&self, _: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
}
```

---

### Phase 3.3: Arithmetic Operations

**Functions to implement**:
- `VecAddInt64`: a + b
- `VecSubInt64`: a - b
- `VecMulInt64`: a * b
- `VecDivInt64`: a / b

**Implementation Pattern (Scalar)**:
```rust
struct VecAddInt64;

impl VectorizedFn for VecAddInt64 {
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        let left = extract_int64(inputs[0])?;
        let right = extract_int64(inputs[1])?;
        let result = extract_int64_mut(output)?;
        
        // Direct write to pre-allocated output
        for i in 0..left.len() {
            result[i] = left[i] + right[i];
        }
        
        Ok(())
    }
    
    fn fn_id(&self) -> FnId {
        FnId { name: "add", id: 20, signature: vec![TypeInfo::Int64, TypeInfo::Int64] }
    }
    
    fn output_type(&self, _: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Int64
    }
}
```

---

### Phase 3.4: SIMD Optimization (Optional)

**Purpose**: Add SIMD versions of hot functions after scalar versions proven correct.

**SIMD with Vec works perfectly** - Vec provides contiguous memory just like arrays.

**Example: SIMD Greater-Than using `std::simd`**:
```rust
use std::simd::{i64x4, mask64x4, SimdPartialOrd};

struct VecGtInt64Simd;

impl VectorizedFn for VecGtInt64Simd {
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        let left = extract_int64(inputs[0])?;
        let right = extract_int64(inputs[1])?;
        let result = extract_bool_mut(output)?;
        
        const LANES: usize = 4;  // AVX: 4 x i64 = 256 bits
        let chunks = left.len() / LANES;
        
        // SIMD loop: process 4 elements at once
        for i in 0..chunks {
            let offset = i * LANES;
            
            // Load into SIMD registers
            let l = i64x4::from_slice(&left[offset..]);
            let r = i64x4::from_slice(&right[offset..]);
            
            // Single vectorized comparison instruction
            let mask = l.simd_gt(r);
            
            // Store results
            for lane in 0..LANES {
                result[offset + lane] = mask.test(lane);
            }
        }
        
        // Handle remainder with scalar code
        for i in (chunks * LANES)..left.len() {
            result[i] = left[i] > right[i];
        }
        
        Ok(())
    }
    
    fn fn_id(&self) -> FnId {
        FnId { name: "gt_simd", id: 101, signature: vec![TypeInfo::Int64, TypeInfo::Int64] }
    }
    
    fn output_type(&self, _: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
}
```

**When to add SIMD**:
1. After scalar versions work correctly
2. Benchmark to verify speedup (3-4x typical)
3. Focus on hot functions (comparisons, arithmetic)
4. Consider auto-vectorization first (Rust compiler may do it)

**Platform-specific SIMD**:
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;  // For AVX2/AVX-512 intrinsics

// Can mix portable std::simd with platform-specific intrinsics
```

---

## Phase 4: Expression System & Planning

### Phase 4.1: VectorizedExpr Trait

**Purpose**: Expression evaluation interface.

**Interface**:
```rust
/// Expression that can be evaluated on a batch
pub trait VectorizedExpr: Debug {
    /// Evaluate expression on batch, writing result to output
    fn eval(&self, batch: &VectorizedBatch, output: &mut PVector) 
        -> Result<(), EvalError>;
    
    /// Get the output type of this expression
    fn output_type(&self) -> TypeInfo;
}
```

**Expression Types**:

```rust
// 1. Column reference
#[derive(Debug)]
pub struct ColumnRef {
    column_idx: usize,
    type_info: TypeInfo,
}

impl VectorizedExpr for ColumnRef {
    fn eval(&self, batch: &VectorizedBatch, output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        let source = batch.column(self.column_idx)?;
        output.copy_from(source)?;
        Ok(())
    }
    
    fn output_type(&self) -> TypeInfo {
        self.type_info
    }
}

// 2. Literal value
#[derive(Debug)]
pub struct LiteralExpr {
    value: PVector,  // Single element vector
    type_info: TypeInfo,
}

impl VectorizedExpr for LiteralExpr {
    fn eval(&self, batch: &VectorizedBatch, output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        // Broadcast literal to batch size
        let row_count = batch.row_count();
        broadcast_literal(&self.value, row_count, output)?;
        Ok(())
    }
    
    fn output_type(&self) -> TypeInfo {
        self.type_info
    }
}

// 3. Function call
#[derive(Debug)]
pub struct FnCallExpr {
    function: Box<dyn VectorizedFn>,
    inputs: Vec<Box<dyn VectorizedExpr>>,
    output_type: TypeInfo,
}

impl VectorizedExpr for FnCallExpr {
    fn eval(&self, batch: &VectorizedBatch, output: &mut PVector) 
        -> Result<(), EvalError> 
    {
        // Evaluate input expressions into pre-allocated temporaries
        let mut input_vecs: Vec<PVector> = Vec::with_capacity(self.inputs.len());
        for input_expr in &self.inputs {
            // Pre-allocate with correct size
            let mut temp = PVector::new(input_expr.output_type(), batch.row_count());
            input_expr.eval(batch, &mut temp)?;
            input_vecs.push(temp);
        }
        
        // Call function (output already pre-allocated by caller)
        let input_refs: Vec<&PVector> = input_vecs.iter().collect();
        self.function.execute(&input_refs, output)?;
        
        Ok(())
    }
    
    fn output_type(&self) -> TypeInfo {
        self.output_type
    }
}
```

---

### Phase 4.2: VectorizedPlanBuilder

**Purpose**: Convert LogicalPlan to VectorizedPlan with inline type checking.

**Interface**:
```rust
pub struct VectorizedPlanBuilder<'a> {
    fn_registry: &'a VectorizedFnRegistry,
    source_types: &'a SourceTypeDef,
}

impl<'a> VectorizedPlanBuilder<'a> {
    pub fn new(
        source_types: &'a SourceTypeDef,
        fn_registry: &'a VectorizedFnRegistry,
    ) -> Self;
    
    /// Build vectorized plan from logical plan
    pub fn build_plan(
        &self, 
        logical: &LogicalPlan<BindingsOp>
    ) -> Result<VectorizedPlan, PlanError>;
    
    /// Build expression tree with type inference
    fn build_expr(&self, expr: &ValueExpr) 
        -> Result<Box<dyn VectorizedExpr>, PlanError>;
}
```

**Pseudocode**:
```rust
impl VectorizedPlanBuilder<'_> {
    fn build_expr(&self, expr: &ValueExpr) 
        -> Result<Box<dyn VectorizedExpr>, PlanError> 
    {
        match expr {
            // Leaf: Variable reference
            ValueExpr::VarRef(name, _) => {
                let type_info = self.source_types.get_type(name)?;
                let col_idx = self.source_types.get_column_index(name)?;
                Ok(Box::new(ColumnRef { 
                    column_idx: col_idx, 
                    type_info 
                }))
            }
            
            // Leaf: Literal
            ValueExpr::Lit(lit) => {
                let (value, type_info) = self.literal_to_pvector(lit)?;
                Ok(Box::new(LiteralExpr { value, type_info }))
            }
            
            // Recursive: Binary operation
            ValueExpr::BinaryExpr(op, lhs, rhs) => {
                // Recursively build children
                let lhs_expr = self.build_expr(lhs)?;
                let rhs_expr = self.build_expr(rhs)?;
                
                // Get types from children (bottom-up inference)
                let lhs_type = lhs_expr.output_type();
                let rhs_type = rhs_expr.output_type();
                
                // Resolve function (inline type checking)
                let function = self.fn_registry
                    .resolve_binary_op(*op, lhs_type, rhs_type)
                    .ok_or_else(|| PlanError::NoFunctionMatch {
                        op: format!("{:?}", op),
                        lhs_type,
                        rhs_type,
                    })?;
                
                let output_type = function.output_type(&[lhs_type, rhs_type]);
                
                Ok(Box::new(FnCallExpr {
                    function,
                    inputs: vec![lhs_expr, rhs_expr],
                    output_type,
                }))
            }
            
            _ => Err(PlanError::UnsupportedExpr),
        }
    }
}
```

---

## Phase 5: Execution Engine

### Phase 5.1: VectorizedOperator Trait

**Purpose**: Volcano-style operator interface.

**Interface**:
```rust
/// Physical operator that produces batches
pub trait VectorizedOperator {
    /// Get next batch of results
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;
}
```

---

### Phase 5.2: VectorizedScan

**Purpose**: Read data from source.

**Interface**:
```rust
pub struct VectorizedScan {
    reader: Box<dyn BatchReader>,
}

impl VectorizedScan {
    pub fn new(reader: Box<dyn BatchReader>) -> Self;
}

impl VectorizedOperator for VectorizedScan {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        self.reader.next_batch()
    }
}
```

---

### Phase 5.3: VectorizedFilter

**Purpose**: Apply predicate to filter rows.

**Interface**:
```rust
pub struct VectorizedFilter {
    input: Box<dyn VectorizedOperator>,
    predicate: Box<dyn VectorizedExpr>,
    // Pre-allocated buffers
    predicate_result: PVector,
}

impl VectorizedFilter {
    pub fn new(
        input: Box<dyn VectorizedOperator>,
        predicate: Box<dyn VectorizedExpr>,
    ) -> Self;
}

impl VectorizedOperator for VectorizedFilter {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Get input batch
        let input_batch = match self.input.next_batch()? {
            Some(batch) => batch,
            None => return Ok(None),
        };
        
        // Evaluate predicate
        self.predicate.eval(&input_batch, &mut self.predicate_result)?;
        
        // Extract boolean vector
        let selection = match &self.predicate_result {
            PVector::Boolean(v) => v,
            _ => return Err(EvalError::TypeMismatch),
        };
        
        // Filter batch based on selection
        let filtered = self.filter_batch(&input_batch, selection)?;
        
        Ok(Some(filtered))
    }
}
```

**Pseudocode for filter_batch**:
```rust
impl VectorizedFilter {
    fn filter_batch(
        &self, 
        batch: &VectorizedBatch, 
        selection: &[bool]
    ) -> Result<VectorizedBatch, EvalError> {
        let mut result = VectorizedBatch::new(
            batch.schema().clone(),
            batch.row_count(),
        );
        
        // For each column
        for col_idx in 0..batch.schema().field_count() {
            let src_col = batch.column(col_idx)?;
            let dst_col = result.column_mut(col_idx)?;
            
            // Copy selected rows
            match (src_col, dst_col) {
                (PVector::Int64(src), PVector::Int64(dst)) => {
                    for (i, &selected) in selection.iter().enumerate() {
                        if selected {
                            dst.push(src[i]);
                        }
                    }
                }
                // ... other types
            }
        }
        
        result.row_count = selection.iter().filter(|&&x| x).count();
        Ok(result)
    }
}
```

---

### Phase 5.4: VectorizedProject

**Purpose**: Project columns.

**Interface**:
```rust
pub struct VectorizedProject {
    input: Box<dyn VectorizedOperator>,
    projections: Vec<(String, Box<dyn VectorizedExpr>)>,
    output_schema: SourceTypeDef,
}

impl VectorizedProject {
    pub fn new(
        input: Box<dyn VectorizedOperator>,
        projections: Vec<(String, Box<dyn VectorizedExpr>)>,
    ) -> Self;
}

impl VectorizedOperator for VectorizedProject {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Get input batch
        let input_batch = match self.input.next_batch()? {
            Some(batch) => batch,
            None => return Ok(None),
        };
        
        // Create output batch
        let mut output = VectorizedBatch::new(
            self.output_schema.clone(),
            input_batch.row_count(),
        );
        
        // Evaluate each projection
        for (col_idx, (_name, expr)) in self.projections.iter().enumerate() {
            let output_col = output.column_mut(col_idx)?;
            expr.eval(&input_batch, output_col)?;
        }
        
        output.row_count = input_batch.row_count();
        Ok(Some(output))
    }
}
```

---

## Phase 6: Testing & Validation

### Phase 6.1: Unit Tests

**Test each component in isolation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vec_gt_int64() {
        let left = PVector::Int64(vec![1, 5, 10, 15]);
        let right = PVector::Int64(vec![5, 5, 5, 5]);
        let mut output = PVector::Boolean(Vec::new());
        
        let func = VecGtInt64;
        func.execute(&[&left, &right], &mut output).unwrap();
        
        match output {
            PVector::Boolean(v) => {
                assert_eq!(v, vec![false, false, true, true]);
            }
            _ => panic!("Wrong output type"),
        }
    }
    
    #[test]
    fn test_vec_and() {
        let left = PVector::Boolean(vec![true, true, false, false]);
        let right = PVector::Boolean(vec![true, false, true, false]);
        let mut output = PVector::Boolean(Vec::new());
        
        let func = VecAnd;
        func.execute(&[&left, &right], &mut output).unwrap();
        
        match output {
            PVector::Boolean(v) => {
                assert_eq!(v, vec![true, false, false, false]);
            }
            _ => panic!("Wrong output type"),
        }
    }
    
    #[test]
    fn test_column_ref_expr() {
        let schema = SourceTypeDef::new(vec![
            Field { name: "a".to_string(), type_info: TypeInfo::Int64 },
        ]);
        
        let mut batch = VectorizedBatch::new(schema.clone(), 4);
        // Fill batch with data...
        
        let expr = ColumnRef { column_idx: 0, type_info: TypeInfo::Int64 };
        let mut output = PVector::Int64(Vec::new());
        
        expr.eval(&batch, &mut output).unwrap();
        // Verify output matches column 0
    }
}
```

---

### Phase 6.2: Integration Tests

**Test PoC query end-to-end**:

```rust
#[test]
fn test_poc_query() {
    // Query: SELECT a, b FROM data WHERE a > 10 AND b < 100
    
    // 1. Setup data
    let data = vec![
        tuple![("a", 5), ("b", 50)],    // filtered out (a <= 10)
        tuple![("a", 15), ("b", 80)],   // included
        tuple![("a", 20), ("b", 150)],  // filtered out (b >= 100)
        tuple![("a", 25), ("b", 90)],   // included
    ];
    
    let schema = SourceTypeDef::new(vec![
        Field { name: "a".to_string(), type_info: TypeInfo::Int64 },
        Field { name: "b".to_string(), type_info: TypeInfo::Int64 },
    ]);
    
    // 2. Build reader
    let reader = TupleIteratorReader::new(
        Box::new(data.into_iter()),
        schema.clone(),
        1024,
    );
    
    // 3. Build plan
    let fn_registry = VectorizedFnRegistry::default();
    let builder = VectorizedPlanBuilder::new(&schema, &fn_registry);
    
    // Build predicate: a > 10 AND b < 100
    let predicate = /* build from logical plan */;
    
    // Build operators
    let scan = VectorizedScan::new(Box::new(reader));
    let filter = VectorizedFilter::new(Box::new(scan), predicate);
    let project = VectorizedProject::new(
        Box::new(filter),
        vec![
            ("a".to_string(), /* column ref a */),
            ("b".to_string(), /* column ref b */),
        ],
    );
    
    // 4. Execute
    let mut pipeline = project;
    let mut result_count = 0;
    
    while let Some(batch) = pipeline.next_batch().unwrap() {
        result_count += batch.row_count();
        // Verify batch contents
    }
    
    // Should return 2 rows: (15, 80) and (25, 90)
    assert_eq!(result_count, 2);
}
```

---

### Phase 6.3: Benchmarking

**Compare performance with current evaluator**:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_vectorized(c: &mut Criterion) {
    // Generate test data
    let data: Vec<Tuple> = (0..100_000)
        .map(|i| tuple![("a", i), ("b", i % 200)])
        .collect();
    
    c.bench_function("vectorized_poc_query", |b| {
        b.iter(|| {
            let schema = SourceTypeDef::new(vec![
                Field { name: "a".to_string(), type_info: TypeInfo::Int64 },
                Field { name: "b".to_string(), type_info: TypeInfo::Int64 },
            ]);
            
            let reader = TupleIteratorReader::new(
                Box::new(data.clone().into_iter()),
                schema.clone(),
                1024,
            );
            
            // Build and execute plan
            let mut pipeline = build_poc_pipeline(reader);
            
            let mut total = 0;
            while let Some(batch) = pipeline.next_batch().unwrap() {
                total += batch.row_count();
            }
            
            black_box(total)
        });
    });
}

fn benchmark_current_eval(c: &mut Criterion) {
    // Same query using current evaluator
    let data: Vec<Tuple> = (0..100_000)
        .map(|i| tuple![("a", i), ("b", i % 200)])
        .collect();
    
    c.bench_function("current_poc_query", |b| {
        b.iter(|| {
            // Execute using partiql-eval
            let result = execute_with_current_eval(&data);
            black_box(result.len())
        });
    });
}

criterion_group!(benches, benchmark_vectorized, benchmark_current_eval);
criterion_main!(benches);
```

**Metrics to track**:
- Execution time (ms)
- Throughput (rows/sec)
- Memory usage (MB)
- Speedup factor (current vs vectorized)

---

## Project Structure

```
partiql-eval-vectorized/
├── Cargo.toml
├── poc_plan.md              # This document
├── src/
│   ├── lib.rs
│   ├── error.rs             # Error types
│   │
│   ├── batch/
│   │   ├── mod.rs
│   │   ├── pvector.rs       # PVector enum
│   │   ├── batch.rs         # VectorizedBatch
│   │   └── source_type.rs   # SourceTypeDef, TypeInfo, Field
│   │
│   ├── reader/
│   │   ├── mod.rs
│   │   ├── batch_reader.rs  # BatchReader trait
│   │   └── tuple_reader.rs  # TupleIteratorReader
│   │
│   ├── functions/
│   │   ├── mod.rs
│   │   ├── fn_trait.rs      # VectorizedFn trait, FnId
│   │   ├── registry.rs      # VectorizedFnRegistry
│   │   ├── comparison.rs    # VecGtInt64, VecLtInt64, etc.
│   │   ├── logical.rs       # VecAnd, VecOr, VecNot
│   │   └── arithmetic.rs    # VecAddInt64, VecSubInt64, etc.
│   │
│   ├── expr/
│   │   ├── mod.rs
│   │   ├── expr_trait.rs    # VectorizedExpr trait
│   │   ├── column_ref.rs    # ColumnRef expression
│   │   ├── literal.rs       # LiteralExpr
│   │   └── fn_call.rs       # FnCallExpr
│   │
│   ├── planner/
│   │   ├── mod.rs
│   │   └── builder.rs       # VectorizedPlanBuilder
│   │
│   └── operators/
│       ├── mod.rs
│       ├── operator_trait.rs # VectorizedOperator trait
│       ├── scan.rs           # VectorizedScan
│       ├── filter.rs         # VectorizedFilter
│       └── project.rs        # VectorizedProject
│
├── tests/
│   ├── unit_tests.rs
│   └── integration_tests.rs
│
└── benches/
    └── poc_benchmark.rs
```

---

## Success Criteria

### Functional Requirements
✅ **Correctness**: PoC query produces same results as current evaluator
✅ **Type Safety**: Type errors caught during planning (not at runtime)
✅ **Extensibility**: Easy to add new operations following established patterns

### Performance Requirements
✅ **Speedup**: Minimum 2-3x faster than current evaluator on PoC query
✅ **Throughput**: Process at least 1M rows/second on simple predicates
✅ **Memory**: No more than 2x memory overhead vs current evaluator

### Code Quality
✅ **Test Coverage**: >80% coverage on core components
✅ **Documentation**: All public APIs documented
✅ **Clean Abstractions**: Clear separation between layers

---

## Implementation Timeline

### Week 1: Foundation
- Phase 1: Core abstractions (PVector, TypeInfo, traits)
- Set up project structure
- Write basic unit tests

### Week 2: Data & Functions
- Phase 2: Data ingestion (BatchReader, VectorizedBatch)
- Phase 3: Implement comparison, logical, and arithmetic operations
- Unit tests for all functions

### Week 3: Planning & Execution
- Phase 4: Expression system and plan builder
- Phase 5: Operators (Scan, Filter, Project)
- Wire up end-to-end pipeline

### Week 4: Testing & Optimization
- Phase 6: Integration tests
- Benchmarking suite
- Performance tuning
- Documentation

---

## Next Steps

1. **Review this plan** - Validate approach and make adjustments
2. **Set up project** - Create `partiql-eval-vectorized` crate
3. **Start with Phase 1.1** - Implement `PVector` and basic types
4. **Iterate rapidly** - Get something working end-to-end quickly
5. **Measure early** - Set up benchmarks from the start

---

## Future Extensions (Post-PoC)

Once PoC is validated, consider:
- NULL/MISSING handling
- More data types (Decimal, DateTime, Blob)
- More operators (Join, Aggregation, Sort)
- SIMD optimizations (via `std::simd` or platform intrinsics)
- Adaptive batch sizing
- Code generation for hot paths
- Integration with Apache Arrow

---

## References

- **Current evaluator**: `partiql-eval/src/eval/evaluable.rs`
- **Logical plan**: `partiql-logical/src/lib.rs`
- **Values**: `partiql-value/src/value.rs`
- **DuckDB vectorized execution**: https://duckdb.org/docs/internals/vector
- **Arrow columnar format**: https://arrow.apache.org/docs/format/Columnar.html
