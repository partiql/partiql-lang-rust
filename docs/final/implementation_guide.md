# Implementation Guide: Detailed Specifications

This document provides detailed implementation specifications for the streaming-first execution engine. It is organized by component and includes complete code examples, struct definitions, and usage patterns.

## 1. Compiled Plan & Specs

### 1.1 CompiledPlan Structure

Compiled artifacts are immutable and reusable. A compiled plan contains the full operator graph, layouts, and scalar programs.

```rust
struct CompiledPlan {
    nodes: Vec<RelOpSpec>,
    bytecode: Vec<Program>,
    layouts: Vec<ScanLayout>,
    slot_count: usize,           // Number of slots in row frame
    max_registers: usize,        // Maximum registers across all programs
}

enum RelOpSpec {
    Pipeline(PipelineSpec),
    HashJoin(HashJoinSpec),
    HashAgg(HashAggSpec),
    Sort(SortSpec),
    Custom(Box<dyn BlockingOperatorSpec>),
}

struct PipelineSpec {
    steps: Vec<StepSpec>,
    reader_layout: ScanLayout,
}

enum StepSpec {
    Filter { program_idx: usize },
    Project { program_idx: usize },
    Limit { count: usize },
}
```

The compiled plan is shared across threads (`Arc<CompiledPlan>`, `Send + Sync`) and never mutated.

### 1.2 Thread Safety Model

```rust
// Compiled plan is immutable and thread-safe
let plan = Arc::new(compiled_plan);

// Multiple VMs can share the same plan concurrently
let handles: Vec<_> = (0..num_threads).map(|_| {
    let plan = Arc::clone(&plan);
    thread::spawn(move || {
        let mut vm = PartiQLVM::new((*plan).clone(), None)?;
        vm.execute()
    })
}).collect();
```

## 2. PartiQLVM Architecture

### 2.1 VM Structure

Each query execution instantiates a `PartiQLVM`, which serves as a single-threaded virtual machine owning all execution state.

```rust
pub struct PartiQLVM {
    compiled: Arc<CompiledPlan>,
    operators: Vec<RelOp>,
    arena: Arena,                          // VM-level memory for computed values
    scratch: RowFrameScratch,              // Reusable scratch space
    registers: Vec<ValueRef<'static>>,     // Unified register array
    slot_count: usize,                     // Number of slots in row frame
    root: usize,                           // Root operator index
}

impl PartiQLVM {
    /// Create VM with initial plan
    pub fn new(
        compiled: CompiledPlan,
        udf_registry: Option<Arc<dyn UdfRegistry>>
    ) -> Result<Self> {
        let slot_count = compiled.slot_count;
        let max_regs = compiled.max_register_count();
        
        // Allocate unified register array: slots + temporaries
        let registers = vec![ValueRef::Missing; slot_count + max_regs];
        
        // Instantiate operators from specs
        let operators = instantiate_operators(&compiled, udf_registry)?;
        
        Ok(PartiQLVM {
            compiled: Arc::new(compiled),
            operators,
            arena: Arena::new(16384),  // 16KB default
            scratch: RowFrameScratch::new(slot_count),
            registers,
            slot_count,
            root: 0,  // Root operator typically at index 0
        })
    }
    
    /// Execute loaded plan, returning polymorphic results
    pub fn execute(&mut self) -> Result<ExecutionResult<'_>> {
        // Return appropriate result type based on statement
        match self.compiled.statement_type() {
            StatementType::Query => {
                Ok(ExecutionResult::Query(QueryIterator::new(self)))
            }
            StatementType::Insert | StatementType::Update | StatementType::Delete => {
                let rows_affected = self.execute_mutation()?;
                Ok(ExecutionResult::Mutation(MutationSummary { rows_affected }))
            }
            StatementType::Create | StatementType::Drop => {
                let objects_created = self.execute_definition()?;
                Ok(ExecutionResult::Definition(DefinitionSummary { objects_created }))
            }
        }
    }
    
    /// Load new plan for execution (VM must not be executing)
    pub fn load_plan(
        &mut self,
        compiled: CompiledPlan,
        udf_registry: Option<Arc<dyn UdfRegistry>>
    ) -> Result<()> {
        let needed = compiled.slot_count + compiled.max_register_count();
        
        // Grow register array if needed, never shrink
        if needed > self.registers.len() {
            self.registers.resize(needed, ValueRef::Missing);
        } else {
            // Clear existing registers
            for reg in &mut self.registers[..needed] {
                *reg = ValueRef::Missing;
            }
        }
        
        // Re-instantiate operators from new plan specs
        self.operators = instantiate_operators(&compiled, udf_registry)?;
        self.compiled = Arc::new(compiled);
        self.slot_count = compiled.slot_count;
        
        Ok(())
    }
    
    // Internal execution helpers
    fn execute_mutation(&mut self) -> Result<usize> { /* ... */ }
    fn execute_definition(&mut self) -> Result<usize> { /* ... */ }
    fn open_operators(&mut self) -> Result<()> { /* ... */ }
    fn close_operators(&mut self) -> Result<()> { /* ... */ }
}
```

### 2.2 VM Reusability Patterns

**Pattern 1: Re-execution of same plan**
```rust
let plan_arc = Arc::new(compiled_plan);
let mut vm = PartiQLVM::new((*plan_arc).clone(), None)?;

// First execution
match vm.execute()? {
    ExecutionResult::Query(mut iter) => {
        while let Some(row) = iter.next()? {
            // Process row
        }
    }
    _ => {}
}

// Re-execute same plan
vm.load_plan((*plan_arc).clone(), None)?;
match vm.execute()? {
    ExecutionResult::Query(mut iter) => { /* ... */ }
    _ => {}
}
```

**Pattern 2: Multiple different plans (sequential)**
```rust
let mut vm = PartiQLVM::new(plan1, None)?;

// Execute first plan
match vm.execute()? {
    ExecutionResult::Query(mut iter) => {
        while let Some(row) = iter.next()? {
            // Process row
        }
    }
    _ => {}
}

// Load and execute different plan
vm.load_plan(plan2, None)?;
match vm.execute()? {
    ExecutionResult::Query(mut iter) => { /* ... */ }
    _ => {}
}
```

**Pattern 3: Multi-statement batch processing**
```rust
let mut vm = PartiQLVM::new(initial_plan, None)?;

for compiled_plan in compiled_statements {
    vm.load_plan(compiled_plan, None)?;
    match vm.execute()? {
        ExecutionResult::Query(mut iter) => {
            // Process SELECT results
            while let Some(row) = iter.next()? {
                println!("{:?}", row);
            }
        }
        ExecutionResult::Mutation(summary) => {
            println!("Modified {} rows", summary.rows_affected);
        }
        ExecutionResult::Definition(summary) => {
            println!("Created {} objects", summary.objects_created);
        }
    }
}
```

### 2.3 Unified Register Array

The VM uses a single contiguous register array where registers [0..slot_count] serve as row slots, and registers [slot_count..] are used for expression evaluation temporaries.

```rust
// During VM initialization
let slot_count = compiled.slot_count;
let max_regs = compiled.max_register_count();
let registers = vec![ValueRef::Missing; slot_count + max_regs];

// Readers populate slots [0..slot_count]
reader.next_row(&mut registers[0..slot_count])?;

// Programs use temporaries [slot_count..]
program.eval(&arena, &mut registers, udf_registry)?;
```

### 2.4 Memory Ownership Model

The VM owns two critical execution resources:

**1. Arena** (default 16KB): Bump allocator for computed values
```rust
pub struct Arena {
    buffer: Vec<u8>,          // Contiguous buffer
    offset: usize,            // Current allocation offset
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        Arena {
            buffer: Vec::with_capacity(capacity),
            offset: 0,
        }
    }
    
    pub fn alloc<T>(&mut self, value: T) -> &T {
        // Bump allocate into contiguous buffer
        // Returns reference valid until arena reset
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.offset) as *mut T;
            ptr.write(value);
            self.offset += std::mem::size_of::<T>();
            &*ptr
        }
    }
    
    pub fn reset(&mut self) {
        // O(1) reset via offset = 0
        self.offset = 0;
    }
}
```

**2. Registers**: Pre-allocated array sized to maximum register count
```rust
// Allocated once at VM creation
let registers = vec![ValueRef::Missing; slot_count + max_regs];

// Borrowed by Program::eval() during expression evaluation
program.eval(&arena, &mut registers, udf)?;
```

## 3. Relational Operators

### 3.1 RelOp Unified Enum

```rust
pub type SlotId = u16;

enum RelOp {
    Pipeline(PipelineOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>]
    ) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(arena, regs),
            RelOp::HashJoin(op) => op.next_row(arena, regs),
            RelOp::HashAgg(op) => op.next_row(arena, regs),
            RelOp::Sort(op) => op.next_row(arena, regs),
            RelOp::Custom(op) => op.next_row(arena, regs),
        }
    }
    
    fn open(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.open(),
            RelOp::HashJoin(op) => op.open(),
            RelOp::HashAgg(op) => op.open(),
            RelOp::Sort(op) => op.open(),
            RelOp::Custom(op) => op.open(),
        }
    }
    
    fn close(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.close(),
            RelOp::HashJoin(op) => op.close(),
            RelOp::HashAgg(op) => op.close(),
            RelOp::Sort(op) => op.close(),
            RelOp::Custom(op) => op.close(),
        }
    }
}
```

### 3.2 Streaming Pipeline Operator

Streaming operators are fused into a `PipelineOp`. The runner executes a tight loop over rows to minimize call overhead.

```rust
struct PipelineOp {
    steps: Vec<Step>,
    reader: Box<dyn RowReader>,
}

impl PipelineOp {
    fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>]
    ) -> Result<bool> {
        loop {
            // Read next row from source
            if !self.reader.next_row(regs)? {
                return Ok(false);
            }
            
            // Execute pipeline steps
            if self.run_steps(arena, regs)? {
                return Ok(true);
            }
        }
    }

    fn run_steps<'a>(
        &mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>]
    ) -> Result<bool> {
        for step in &mut self.steps {
            if !step.eval(arena, regs)? {
                return Ok(false);  // Row filtered out
            }
        }
        Ok(true)  // Row passes all steps
    }
    
    fn open(&mut self) -> Result<()> {
        self.reader.open()
    }
    
    fn close(&mut self) -> Result<()> {
        self.reader.close()
    }
}
```

### 3.3 Pipeline Steps

Pipeline steps are native Rust enums for performance.

```rust
enum Step {
    Filter { program: Program },
    Project { program: Program },
    Limit { remaining: usize },
}

impl Step {
    fn eval<'a>(
        &mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>
    ) -> Result<bool> {
        match self {
            Step::Filter { program } => {
                // Evaluate predicate
                program.eval(arena, regs, udf)?;
                // Check result in first slot (convention)
                Ok(regs[0].as_bool()?)
            }
            Step::Project { program } => {
                // Execute projection
                program.eval(arena, regs, udf)?;
                Ok(true)
            }
            Step::Limit { remaining } => {
                if *remaining == 0 {
                    return Ok(false);
                }
                *remaining -= 1;
                Ok(true)
            }
        }
    }
}
```

### 3.4 Blocking Operators

Blocking operators must respect buffer stability when storing rows.

```rust
struct HashJoinState {
    build_table: HashMap<JoinKey, Vec<RowOwned>>,
    probe_child: Box<RelOp>,
    reader_caps: ReaderCaps,
}

impl HashJoinState {
    fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>]
    ) -> Result<bool> {
        // Pull from probe side and join with build table
        // ...
    }
    
    fn store_build_row(&mut self, regs: &[ValueRef]) -> Result<()> {
        match self.reader_caps.stability {
            BufferStability::UntilClose => {
                // Safe to store borrowed references
                self.build_table.entry(key)
                    .or_default()
                    .push(RowOwned::from_borrowed(regs));
            }
            BufferStability::UntilNext => {
                // Must materialize to owned values
                self.build_table.entry(key)
                    .or_default()
                    .push(RowOwned::materialize(regs));
            }
        }
        Ok(())
    }
}
```

## 4. Data Source Readers

### 4.1 RowReader Trait

```rust
pub enum BufferStability {
    UntilNext,    // Reader may reuse buffers on next next_row() call
    UntilClose,   // Reader guarantees stability until close()
}

pub struct ReaderCaps {
    pub stability: BufferStability,
    pub can_project: bool,
    pub can_return_opaque: bool,
}

pub trait RowReader {
    fn caps(&self) -> ReaderCaps;
    fn set_projection(&mut self, layout: ScanLayout) -> Result<()>;
    fn open(&mut self) -> Result<()>;
    
    /// Populate row data into registers [0..slot_count]
    /// Readers manage their own memory; borrowed references must remain
    /// valid according to BufferStability contract
    fn next_row(&mut self, regs: &mut [ValueRef<'_>]) -> Result<bool>;
    
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
    fn close(&mut self) -> Result<()>;
}
```

### 4.2 Example Ion Reader Implementation

```rust
pub struct IonRowReader {
    reader: ion_rs::Reader,
    string_storage: Vec<String>,  // Reader-owned storage for strings
    projection: Option<ScanLayout>,
    caps: ReaderCaps,
}

impl IonRowReader {
    pub fn new(data: &[u8]) -> Self {
        IonRowReader {
            reader: ion_rs::Reader::new(data),
            string_storage: Vec::new(),
            projection: None,
            caps: ReaderCaps {
                stability: BufferStability::UntilNext,
                can_project: true,
                can_return_opaque: false,
            },
        }
    }
}

impl RowReader for IonRowReader {
    fn caps(&self) -> ReaderCaps {
        self.caps
    }
    
    fn set_projection(&mut self, layout: ScanLayout) -> Result<()> {
        self.projection = Some(layout);
        Ok(())
    }
    
    fn open(&mut self) -> Result<()> {
        // Open file handle, initialize reader state
        Ok(())
    }
    
    fn next_row(&mut self, regs: &mut [ValueRef<'_>]) -> Result<bool> {
        // Clear string storage for new row
        self.string_storage.clear();
        
        // Read next Ion value
        let value = match self.reader.next()? {
            Some(v) => v,
            None => return Ok(false),
        };
        
        // Populate slots based on projection
        if let Some(layout) = &self.projection {
            for proj in &layout.projections {
                let slot_value = match proj.source {
                    ScanSource::FieldPath(ref name) => {
                        self.extract_field(&value, name)?
                    }
                    ScanSource::BaseRow => {
                        ValueRef::from_ion(&value)
                    }
                    _ => ValueRef::Missing,
                };
                regs[proj.target_slot as usize] = slot_value;
            }
        }
        
        Ok(true)
    }
    
    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::FieldPath(field_name.to_string()))
    }
    
    fn close(&mut self) -> Result<()> {
        // Close file handle, cleanup resources
        Ok(())
    }
    
    // Helper to extract and store strings
    fn extract_field(&mut self, value: &IonValue, name: &str) -> Result<ValueRef<'_>> {
        match value.get_field(name) {
            Some(IonValue::String(s)) => {
                // Store string in reader-owned storage
                self.string_storage.push(s.to_string());
                let stored = self.string_storage.last().unwrap();
                Ok(ValueRef::Str(stored.as_str()))
            }
            Some(IonValue::Int(i)) => Ok(ValueRef::I64(*i)),
            // ... other types
            None => Ok(ValueRef::Missing),
        }
    }
}
```

## 5. Projection Pushdown

### 5.1 ScanLayout Structure

```rust
pub struct ScanLayout {
    pub projections: Vec<ScanProjection>,
}

pub struct ScanProjection {
    pub source: ScanSource,
    pub target_slot: SlotId,
    pub type_hint: TypeHint,
}

pub enum ScanSource {
    ColumnIndex(usize),        // For columnar formats
    FieldPath(String),         // For document formats
    BaseRow,                   // Entire row
}

pub enum TypeHint {
    Any,
    I64,
    F64,
    Bool,
    String,
    // ... other types
}
```

### 5.2 Projection Usage Example

```rust
// Compiler creates projection layout
let layout = ScanLayout {
    projections: vec![
        ScanProjection {
            source: ScanSource::FieldPath("id".to_string()),
            target_slot: 0,
            type_hint: TypeHint::I64,
        },
        ScanProjection {
            source: ScanSource::FieldPath("name".to_string()),
            target_slot: 1,
            type_hint: TypeHint::String,
        },
    ],
};

// Configure reader with projection
reader.set_projection(layout)?;

// Reader populates only requested fields
reader.next_row(&mut regs[0..2])?;
// regs[0] = id value
// regs[1] = name value
```

## 6. Execution Data Model

### 6.1 ValueRef and ValueOwned

```rust
pub enum ValueRef<'a> {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(&'a str),
    Bytes(&'a [u8]),
    Obj(ObjRef<'a>),
    Arr(ArrRef<'a>),
    Opaque(OpaqueRef<'a>),
}

pub enum ValueOwned {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Object(HashMap<String, ValueOwned>),
    Array(Vec<ValueOwned>),
}

impl<'a> ValueRef<'a> {
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            ValueRef::Bool(b) => Ok(*b),
            _ => Err(Error::type_mismatch("bool")),
        }
    }
    
    pub fn as_i64(&self) -> Result<i64> {
        match self {
            ValueRef::I64(i) => Ok(*i),
            _ => Err(Error::type_mismatch("i64")),
        }
    }
    
    pub fn to_owned(&self) -> ValueOwned {
        match self {
            ValueRef::Bool(b) => ValueOwned::Bool(*b),
            ValueRef::I64(i) => ValueOwned::I64(*i),
            ValueRef::Str(s) => ValueOwned::String(s.to_string()),
            // ... other variants
            _ => ValueOwned::Missing,
        }
    }
}
```

### 6.2 RowFrame and Slots

```rust
pub struct RowFrame<'a> {
    slots: &'a mut [ValueRef<'a>],
    arena: &'a Arena,
}

impl<'a> RowFrame<'a> {
    pub fn new(slots: &'a mut [ValueRef<'a>], arena: &'a Arena) -> Self {
        RowFrame { slots, arena }
    }
    
    pub fn get(&self, slot: SlotId) -> &ValueRef<'a> {
        &self.slots[slot as usize]
    }
    
    pub fn set(&mut self, slot: SlotId, value: ValueRef<'a>) {
        self.slots[slot as usize] = value;
    }
    
    pub fn alloc(&self, value: ValueOwned) -> &ValueOwned {
        self.arena.alloc(value)
    }
}
```

## 7. Scalar Bytecode VM

### 7.1 Expression AST

```rust
enum Expr {
    Literal(ValueOwned),
    SlotRef(SlotId),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    GetField(Box<Expr>, String),
    UdfCall { name: String, args: Vec<Expr> },
}
```

### 7.2 Bytecode Program

```rust
pub struct Program {
    pub insts: Vec<Inst>,
    pub consts: Vec<ValueOwned>,
    pub keys: Vec<String>,
    pub reg_count: u16,  // Maximum registers needed by this program
}

pub enum Inst {
    LoadConst { dst: u16, const_idx: u16 },
    LoadSlot { dst: u16, slot: SlotId },
    StoreSlot { slot: SlotId, src: u16 },
    
    // Arithmetic (i64)
    AddI64 { dst: u16, a: u16, b: u16 },
    SubI64 { dst: u16, a: u16, b: u16 },
    MulI64 { dst: u16, a: u16, b: u16 },
    DivI64 { dst: u16, a: u16, b: u16 },
    ModI64 { dst: u16, a: u16, b: u16 },
    
    // Arithmetic (f64)
    AddF64 { dst: u16, a: u16, b: u16 },
    SubF64 { dst: u16, a: u16, b: u16 },
    MulF64 { dst: u16, a: u16, b: u16 },
    DivF64 { dst: u16, a: u16, b: u16 },
    
    // Comparisons (i64)
    EqI64 { dst: u16, a: u16, b: u16 },
    NeI64 { dst: u16, a: u16, b: u16 },
    LtI64 { dst: u16, a: u16, b: u16 },
    LeI64 { dst: u16, a: u16, b: u16 },
    GtI64 { dst: u16, a: u16, b: u16 },
    GeI64 { dst: u16, a: u16, b: u16 },
    
    // Logical
    AndBool { dst: u16, a: u16, b: u16 },
    OrBool { dst: u16, a: u16, b: u16 },
    NotBool { dst: u16, src: u16 },
    
    // Object/field access
    GetField { dst: u16, base: u16, key_idx: u16 },
    
    // UDF calls
    CallUdf { dst: u16, func_idx: u16, args: Vec<u16> },
}
```

### 7.3 Program Evaluation

```rust
impl Program {
    pub fn eval<'a>(
        &self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>
    ) -> Result<()> {
        for inst in &self.insts {
            self.eval_inst(inst, arena, regs, udf)?;
        }
        Ok(())
    }
    
    fn eval_inst<'a>(
        &self,
        inst: &Inst,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>
    ) -> Result<()> {
        match *inst {
            Inst::LoadConst { dst, const_idx } => {
                let value = &self.consts[const_idx as usize];
                regs[dst as usize] = ValueRef::from_owned(value);
            }
            Inst::AddI64 { dst, a, b } => {
                let av = regs[a as usize].as_i64()?;
                let bv = regs[b as usize].as_i64()?;
                regs[dst as usize] = ValueRef::I64(av + bv);
            }
            Inst::EqI64 { dst, a, b } => {
                let av = regs[a as usize].as_i64()?;
                let bv = regs[b as usize].as_i64()?;
                regs[dst as usize] = ValueRef::Bool(av == bv);
            }
            Inst::AndBool { dst, a, b } => {
                let av = regs[a as usize].as_bool()?;
                let bv = regs[b as usize].as_bool()?;
                regs[dst as usize] = ValueRef::Bool(av && bv);
            }
            Inst::GetField { dst, base, key_idx } => {
                let obj = regs[base as usize].as_obj()?;
                let key = &self.keys[key_idx as usize];
                let value = obj.get(key)?;
                regs[dst as usize] = value;
            }
            Inst::StoreSlot { slot, src } => {
                regs[slot as usize] = regs[src as usize];
            }
            Inst::CallUdf { dst, func_idx, ref args } => {
                let func_name = &self.keys[func_idx as usize];
                let udf_registry = udf.ok_or(Error::no_udf_registry())?;
                let func = udf_registry.get(func_name)?;
                
                let arg_values: Vec<_> = args.iter()
                    .map(|&r| &regs[r as usize])
                    .collect();
                
                let result = func.call(&arg_values, arena)?;
                regs[dst as usize] = result;
            }
            _ => { /* ... other instructions */ }
        }
        Ok(())
    }
}
```

### 7.4 Compilation Example

```rust
// Compile: slot[1] + 42
fn compile_add_example(slot_count: usize) -> Program {
    Program {
        insts: vec![
            // Register 0 = slot[1] (direct access, no load needed)
            // Register 1 = constant 42
            Inst::LoadConst { dst: 1, const_idx: 0 },
            // Register 2 = register 0 + register 1
            Inst::AddI64 { dst: 2, a: 1, b: 1 },
            // Store result in slot 0
            Inst::StoreSlot { slot: 0, src: 2 },
        ],
        consts: vec![ValueOwned::I64(42)],
        keys: vec![],
        reg_count: 3,  // Need registers 0, 1, 2
    }
}
```

## 8. Result Streaming & APIs

### 8.1 Execution Result Types

```rust
pub enum ExecutionResult<'vm> {
    Query(QueryIterator<'vm>),       // SELECT - streaming results
    Mutation(MutationSummary),       // INSERT/UPDATE/DELETE - immediate summary
    Definition(DefinitionSummary),   // CREATE/DROP - immediate summary
}

pub struct MutationSummary {
    pub rows_affected: usize,
}

pub struct DefinitionSummary {
    pub objects_created: usize,
}
```

### 8.2 Query Iterator with RAII

```rust
pub struct QueryIterator<'vm> {
    vm: &'vm mut PartiQLVM,
    opened: bool,  // Track operator resource state
}

impl<'vm> QueryIterator<'vm> {
    pub fn new(vm: &'vm mut PartiQLVM) -> Self {
        QueryIterator { vm, opened: false }
    }
    
    pub fn next(&mut self) -> Result<Option<RowView<'_>>> {
        // Lazy open on first iteration
        if !self.opened {
            self.vm.open_operators()?;
            self.opened = true;
        }
        
        // Arena reset and row processing
        self.vm.arena.reset();
        
        let regs = &mut self.vm.registers[0..self.vm.slot_count];
        let op = &mut self.vm.operators[self.vm.root];
        
        if op.next_row(&self.vm.arena, regs)? {
            Ok(Some(RowView::new(regs)))
        } else {
            Ok(None)
        }
    }
}

impl Drop for QueryIterator<'_> {
    fn drop(&mut self) {
        if self.opened {
            // Best-effort close, ignore errors in Drop
            let _ = self.vm.close_operators();
        }
    }
}
```

### 8.3 RowView and ValueView

```rust
pub struct RowView<'a> {
    slots: &'a [ValueRef<'a>],
}

impl<'a> RowView<'a> {
    pub fn new(slots: &'a [ValueRef<'a>]) -> Self {
        RowView { slots }
    }
    
    pub fn get(&self, col: usize) -> ValueView<'a> {
        ValueView::new(&self.slots[col])
    }
    
    pub fn get_i64(&self, col: usize) -> Option<i64> {
        self.slots[col].as_i64().ok()
    }
    
    pub fn get_f64(&self, col: usize) -> Option<f64> {
        self.slots[col].as_f64().ok()
    }
    
    pub fn get_bool(&self, col: usize) -> Option<bool> {
        self.slots[col].as_bool().ok()
    }
    
    pub fn get_str(&self, col: usize) -> Option<&'a str> {
        match &self.slots[col] {
            ValueRef::Str(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn get_value(&self, col: usize) -> ValueOwned {
        self.slots[col].to_owned()
    }
}

pub struct ValueView<'a> {
    value: &'a ValueRef<'a>,
}

impl<'a> ValueView<'a> {
    pub fn new(value: &'a ValueRef<'a>) -> Self {
        ValueView { value }
    }
    
    pub fn is_missing(&self) -> bool {
        matches!(self.value, ValueRef::Missing)
    }
    
    pub fn is_null(&self) -> bool {
        matches!(self.value, ValueRef::Null)
    }
    
    pub fn as_i64(&self) -> Option<i64> {
        self.value.as_i64().ok()
    }
    
    pub fn as_str(&self) -> Option<&'a str> {
        match self.value {
            ValueRef::Str(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn to_owned(&self) -> ValueOwned {
        self.value.to_owned()
    }
}
```

### 8.4 Usage Examples

```rust
// Single query execution
let mut vm = PartiQLVM::new(compiled_select, None)?;
match vm.execute()? {
    ExecutionResult::Query(mut iter) => {
        while let Some(row) = iter.next()? {
            let id = row.get_i64(0).unwrap_or(0);
            let name = row.get_str(1).unwrap_or("");
            println!("id: {}, name: {}", id, name);
        }
        // Drop closes operators automatically
    }
    _ => {}
}
// VM immediately reusable for load_plan()

// Multi-statement batch
let mut vm = PartiQLVM::new(initial_plan, None)?;
for plan in compiled_statements {
    vm.load_plan(plan, None)?;
    match vm.execute()? {
        ExecutionResult::Query(mut iter) => {
            while let Some(row) = iter.next()? {
                // Process row
            }
        }
        ExecutionResult::Mutation(summary) => {
            println!("Modified {} rows", summary.rows_affected);
        }
        ExecutionResult::Definition(summary) => {
            println!("Created {} objects", summary.objects_created);
        }
    }
}
```

## 9. Public API Design

### 9.1 Public Surface

```rust
// Query compilation interface
pub struct PreparedQuery {
    compiled: Arc<CompiledPlan>,
}

impl PreparedQuery {
    pub fn compile(sql: &str) -> Result<Self> {
        let compiled = compile_query(sql)?;
        Ok(PreparedQuery {
            compiled: Arc::new(compiled),
        })
    }
    
    pub fn execute(&self) -> Result<ExecutionResult> {
        let mut vm = PartiQLVM::new((*self.compiled).clone(), None)?;
        vm.execute()
    }
    
    pub fn schema(&self) -> &Schema {
        self.compiled.result_schema()
    }
}

// Schema information
pub struct Schema {
    pub columns: Vec<ColumnInfo>,
}

pub struct ColumnInfo {
    pub name: String,
    pub type_hint: TypeHint,
}
```

### 9.2 Internal-Only Types

These types are NOT exposed in the public API:

- `ValueRef`, `ValueOwned` (may need to reconsider for extensibility)
- `RowFrame`, `SlotId`
- `RelOp`, `PipelineOp`, `Step`
- `Arena`, registers, bytecode structures (`Program`, `Inst`)
- `CompiledPlan`, `RelOpSpec`

### 9.3 Extensibility Considerations

**Open Question**: The visibility of `ValueRef` and `ValueOwned` may need to be reconsidered for extensibility:

1. **Custom Readers**: May require direct access to construct `ValueRef` instances
2. **User-Defined Functions**: Need to receive and return values in some format
3. **Custom Operators**: May need to interact with the row data model

**Possible Solutions**:
- Expose `ValueRef`/`ValueOwned` as part of extensibility APIs
- Provide conversion traits between public and internal types
- Design separate "plugin" APIs with controlled value representations

This trade-off between encapsulation and extensibility should be evaluated as implementation progresses and customer needs become clearer.

### 9.4 Evolution Guarantee

The public API design enables:
- Adding vectorized operators without changing user code
- Batch readers transparently integrated
- Adaptive execution strategies hidden behind `ExecutionResult`
- Internal optimizations without API breakage

Example: A query executing row-at-a-time today can transparently switch to vectorized execution when operating on columnar data—the public interface (`ExecutionResult::Query(iterator)`) remains identical while internal implementation changes.
