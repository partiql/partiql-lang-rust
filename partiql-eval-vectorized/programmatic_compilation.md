# Programmatic Compilation Design

## Overview

Convert LogicalPlan (graph-based) to VectorizedPlan (tree-based physical operators) using a two-pass compilation strategy with integrated projection pushdown.

## Scope

- **Operators**: Scan, Filter, Project
- **Expressions**: Deferred to Phase 2 (stub executors for now)
- **Schema Tracking**: Each operator exposes its output schema

## Architecture

### Two-Pass Strategy

**Pass 1: Column Requirements Analysis**
- Walk backwards from Sink to Scan nodes
- Propagate required columns through operators
- Result: Map of `(data_source_name, scan_op_id) -> Vec<column_names>`

**Pass 2: Physical Operator Construction**
- Walk LogicalPlan building VectorizedOperators
- For Scans: Resolve column names to ProjectionSources
- For other ops: Build with proper schema propagation

### Key Design Decisions

1. Each LogicalPlan node consumed by exactly one other node (simplifies traversal)
2. VectorizedOperator trait exposes `output_schema()` for expression compilation
3. Projection pushdown integrated into Pass 1 (not a separate optimization)

## Components

### 1. VectorizedOperator Trait

```rust
trait VectorizedOperator {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;
    fn output_schema(&self) -> &SourceTypeDef;  // NEW
}
```

**All operators implement this trait:**
- VectorizedScan: Schema from projected columns
- VectorizedFilter: Schema from input (unchanged)
- VectorizedProject: Schema from projection expressions

### 2. Updated VectorizedScan

```rust
struct VectorizedScan {
    reader: Box<dyn BatchReader>,
    projections: Vec<ProjectionSource>,  // NEW: columns to read
    output_schema: SourceTypeDef,        // NEW: schema of projected columns
}
```

### 3. Pass 1: ColumnRequirements Analyzer

```rust
struct ColumnRequirements {
    scan_requirements: HashMap<(String, OpId), HashSet<String>>,
}

impl ColumnRequirements {
    fn analyze(&mut self, plan: &LogicalPlan<BindingsOp>) {
        let sink_id = find_sink(plan);
        analyze_operator(sink_id, plan, empty_set);
    }
    
    fn analyze_operator(op_id, plan, required_cols) {
        match plan.operator(op_id) {
            Scan(scan) => {
                data_source = extract_data_source_name(scan.expr);
                store_requirements((data_source, op_id), required_cols);
            }
            
            Filter(filter) => {
                needed = required_cols + extract_column_refs(filter.expr);
                input_id = find_predecessor(op_id, plan);
                analyze_operator(input_id, plan, needed);
            }
            
            Project(project) => {
                needed = empty_set;
                for (alias, expr) in project.exprs {
                    needed += extract_column_refs(expr);
                }
                input_id = find_predecessor(op_id, plan);
                analyze_operator(input_id, plan, needed);
            }
            
            Sink => {
                input_id = find_predecessor(op_id, plan);
                analyze_operator(input_id, plan, required_cols);
            }
        }
    }
}
```

**Helper: extract_column_refs**
```rust
fn extract_column_refs(expr: &ValueExpr, binding: &str) -> HashSet<String> {
    match expr {
        Path(VarRef(name, _), [Key(col_name)]) if name == binding => {
            return {col_name};
        }
        VarRef(name, _) => {
            return {name};
        }
        BinaryExpr(_, left, right) => {
            return extract_column_refs(left) + extract_column_refs(right);
        }
        // ... other cases
    }
}
```

### 4. Pass 2: LogicalToPhysical Translator

```rust
struct LogicalToPhysical {
    context: CompilerContext,
    column_requirements: ColumnRequirements,
}

impl LogicalToPhysical {
    fn translate(plan) -> Box<dyn VectorizedOperator> {
        sink_id = find_sink(plan);
        input_id = find_predecessor(sink_id, plan);
        return translate_node(input_id, plan);
    }
    
    fn translate_node(op_id, plan) -> Box<dyn VectorizedOperator> {
        match plan.operator(op_id) {
            Scan(scan) => build_scan(op_id, scan),
            Filter(filter) => build_filter(op_id, filter, plan),
            Project(project) => build_project(op_id, project, plan),
        }
    }
}
```

**build_scan**
```rust
fn build_scan(op_id, scan) -> Box<dyn VectorizedOperator> {
    // 1. Get data source
    data_source_name = extract_data_source_name(scan.expr);
    reader = context.get_data_source(data_source_name);
    
    // 2. Get required columns
    required_cols = column_requirements.get((data_source_name, op_id));
    
    // 3. Resolve to ProjectionSources
    projections = [];
    output_fields = [];
    for col_name in required_cols {
        proj_source = reader.resolve(col_name);
        projections.push(proj_source);
        
        field_type = infer_column_type(reader, col_name);
        output_fields.push(Field { name: col_name, type_info: field_type });
    }
    
    output_schema = SourceTypeDef::new(output_fields);
    
    // 4. Create operator
    return VectorizedScan::new(reader, projections, output_schema);
}
```

**build_filter**
```rust
fn build_filter(op_id, filter, plan) -> Box<dyn VectorizedOperator> {
    // 1. Build input
    input_id = find_predecessor(op_id, plan);
    input_op = translate_node(input_id, plan);
    
    // 2. Create stub executor (Phase 1: pass-through)
    stub_executor = create_stub_filter_executor();
    
    // 3. Create operator (caches input schema internally)
    return VectorizedFilter::new(input_op, stub_executor);
}
```

**build_project**
```rust
fn build_project(op_id, project, plan) -> Box<dyn VectorizedOperator> {
    // 1. Build input
    input_id = find_predecessor(op_id, plan);
    input_op = translate_node(input_id, plan);
    input_schema = input_op.output_schema();
    
    // 2. Build output schema and stub executor
    output_fields = [];
    compiled_exprs = [];
    output_types = [];
    
    for (idx, (alias, expr)) in project.exprs.enumerate() {
        col_name = try_extract_simple_column_ref(expr)?;
        col_idx = input_schema.get_column_index(col_name);
        col_type = input_schema.get_type(col_name);
        
        // Identity operation: pass through input column
        compiled_exprs.push(CompiledExpr {
            op: ExprOp::Identity,
            inputs: [ExprInput::InputCol(col_idx)],
            output: idx,
        });
        
        output_fields.push(Field { name: alias, type_info: col_type });
        output_types.push(col_type);
    }
    
    output_schema = SourceTypeDef::new(output_fields);
    output_indices = [0..project.exprs.len()];
    executor = ExpressionExecutor::new(compiled_exprs, output_types, output_indices);
    
    // 3. Create operator
    return VectorizedProject::new(input_op, executor, output_schema);
}
```

### 5. Compiler Entry Point

```rust
impl Compiler {
    fn compile(logical: &LogicalPlan<BindingsOp>) -> VectorizedPlan {
        // Pass 1: Analyze column requirements
        col_reqs = ColumnRequirements::new();
        col_reqs.analyze(logical);
        
        // Pass 2: Build physical operators
        translator = LogicalToPhysical {
            context: self.context,
            column_requirements: col_reqs,
        };
        root_op = translator.translate(logical);
        output_schema = root_op.output_schema().clone();
        
        return VectorizedPlan::new(root_op, output_schema);
    }
}
```

## Helper Functions

```rust
// Extract data source name from Scan.expr
fn extract_data_source_name(expr: &ValueExpr) -> String {
    match expr {
        VarRef(name, VarRefType::Global) => name.to_string(),
        _ => error("Scan must reference global table"),
    }
}

// Extract simple column reference like "v.a" -> "a"
fn try_extract_simple_column_ref(expr: &ValueExpr) -> Option<String> {
    match expr {
        Path(VarRef(_, _), [Key(col_name)]) => Some(col_name.to_string()),
        VarRef(name, _) => Some(name.to_string()),
        _ => None,
    }
}

// Graph traversal
fn find_sink(plan) -> OpId {
    plan.operators_by_id()
        .find(|(_, op)| matches!(op, BindingsOp::Sink))
        .map(|(id, _)| id)
}

fn find_predecessor(op_id, plan) -> OpId {
    plan.flows()
        .find(|(_, dst, _)| dst == op_id)
        .map(|(src, _, _)| src)
}
```

## Phase 2: Expression Compilation (Future)

With `output_schema()` on operators, expression compilation becomes straightforward:

```rust
fn build_filter(op_id, filter, plan) -> Box<dyn VectorizedOperator> {
    input_op = translate_node(input_id, plan);
    
    // Compile filter predicate using input schema
    predicate_executor = compile_predicate(
        filter.expr,
        input_op.output_schema()  // Schema available from operator!
    );
    
    return VectorizedFilter::new(input_op, predicate_executor);
}

fn compile_predicate(expr, input_schema) -> ExpressionExecutor {
    // Use input_schema to resolve column references
    // Generate bytecode for expression evaluation
}
```

## Example Compilation

**LogicalPlan**: `SELECT a, b FROM data WHERE a = 1000`

**AST**:
```
Sink
  ↑
Project(a, b)
  ↑
Filter(a = 1000)
  ↑
Scan(data, as "x")
```

**Pass 1 Analysis**:
1. Sink needs all columns from Project
2. Project(a, b) needs columns {a, b} from Filter
3. Filter(a = 1000) needs columns {a, b} + {a} = {a, b} from Scan
4. Scan("data") stores requirement: ("data", scan_op_id) → {a, b}

**Pass 2 Construction**:
1. Scan: Resolve {a, b} → [ProjectionSource::ColumnIndex(0), ColumnIndex(1)]
   - Output schema: {a: Int64, b: Int64}
2. Filter: Input schema from scan, stub executor
   - Output schema: {a: Int64, b: Int64} (unchanged)
3. Project: Extract columns a, b from input
   - Output schema: {a: Int64, b: Int64}

**Result**: VectorizedPlan with Project(Filter(Scan)) tree
