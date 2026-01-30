# Database Object Reference Planning

## Overview

This document captures the design for adding database catalog support to PartiQL, allowing consumers to register catalogs that provide table access via ReaderFactory instances.

## Goals

1. Allow consumers to register multiple `DataCatalog` implementations with `partiql-eval`
2. Enable qualified table references (e.g., `my_catalog.schema.table`) in queries
3. Resolve table references to `ReaderFactory` instances at compile time
4. Support case-sensitive and case-insensitive table/catalog names via `BindingsName`
5. Eventually update `BindingsOp` to support scanning database objects

## Design Evolution

### Initial Approach: ScanDB Variant

**Original idea:** Add a new `BindingsOp::ScanDB` variant specifically for database object scans.

**Problem identified:** This would duplicate functionality with the existing `Scan` operator and limit flexibility.

### Refined Approach: DBRef Expression

**Better idea:** Add `DBRef` as a new `ValueExpr` variant, allowing database references to be used anywhere expressions are valid.

**Key insight:** The existing `Scan` operator can handle both `VarRef` and `DBRef` expressions. The compiler just needs to check the expression type and route accordingly.

### Final Design: DBRef with BindingsName

**Final refinement:** Use `BindingsName` for all name components to preserve case sensitivity information.

**Rationale:** Consistent with `VarRef`, provides proper SQL semantics, enables catalog implementations to handle case-sensitive vs case-insensitive lookups correctly.

## Architecture

### 1. DBRef Expression Type (partiql-logical)

Add a new expression variant for database object references:

```rust
use partiql_value::BindingsName;

pub struct DBRef {
    /// Name of the catalog (with case sensitivity info)
    pub catalog: BindingsName<'static>,
    
    /// Path to the database object - each component with case sensitivity
    /// Examples:
    /// - Single table: vec![BindingsName("users")]
    /// - Schema.table: vec![BindingsName("public"), BindingsName("users")]
    /// - Schema.schema.table: vec![BindingsName("db"), BindingsName("public"), BindingsName("users")]
    pub path: Vec<BindingsName<'static>>,
}

pub enum ValueExpr {
    VarRef(BindingsName<'static>, VarRefType),
    DBRef(DBRef),  // NEW: Catalog table reference
    // ... other variants
}
```

**Location:** `partiql-logical/src/lib.rs`

### 2. DataCatalog Trait (partiql-eval)

Define the catalog interface that users implement:

```rust
use partiql_value::BindingsName;

/// A data catalog provides table access by name/path
pub trait DataCatalog: Send + Sync {
    /// Returns the catalog's name
    fn name(&self) -> &BindingsName<'_>;
    
    /// Gets a ReaderFactory for a table by path
    /// Each path component has case sensitivity information
    /// 
    /// Examples:
    /// - path = [BindingsName("users")] -> simple table lookup
    /// - path = [BindingsName("public"), BindingsName("users")] -> schema.table
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<ReaderFactory>;
}
```

**Location:** `partiql-eval/src/engine/catalog.rs` (new file)

### 3. CatalogRegistry (partiql-eval)

Manages multiple registered catalogs:

```rust
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub struct CatalogRegistry {
    catalogs: FxHashMap<String, Arc<dyn DataCatalog>>,
}

impl CatalogRegistry {
    pub fn new() -> Self {
        Self {
            catalogs: FxHashMap::default(),
        }
    }
    
    /// Register a catalog. Uses normalized name for lookup.
    pub fn register_catalog(&mut self, catalog: Arc<dyn DataCatalog>) {
        let name = match catalog.name() {
            BindingsName::CaseSensitive(n) => n.to_string(),
            BindingsName::CaseInsensitive(n) => n.to_lowercase(),
        };
        self.catalogs.insert(name, catalog);
    }
    
    /// Get a catalog by name, respecting case sensitivity
    pub fn get_catalog(&self, name: &BindingsName<'_>) -> Option<&Arc<dyn DataCatalog>> {
        let lookup_key = match name {
            BindingsName::CaseSensitive(n) => n.as_ref().to_string(),
            BindingsName::CaseInsensitive(n) => n.to_lowercase(),
        };
        self.catalogs.get(&lookup_key)
    }
}
```

**Location:** `partiql-eval/src/engine/catalog.rs`

### 4. PlanCompiler Integration (partiql-eval)

Update the compiler to accept and use catalogs:

```rust
pub struct PlanCompiler {
    catalog_registry: Option<CatalogRegistry>,
    // ... existing fields
}

impl PlanCompiler {
    pub fn new(catalog_registry: Option<CatalogRegistry>) -> Self {
        Self {
            catalog_registry,
            // ... existing initialization
        }
    }
    
    fn compile_scan(&self, scan: &Scan) -> Result<CompiledScan> {
        match &scan.expr {
            // Existing path: VarRef reads from bindings/environment
            ValueExpr::VarRef(name, ref_type) => {
                self.compile_varref_scan(name, ref_type, scan)
            }
            
            // NEW path: DBRef reads from catalog
            ValueExpr::DBRef(db_ref) => {
                let registry = self.catalog_registry.as_ref()
                    .ok_or_else(|| EngineError::InvalidPlan(
                        "No catalog registry configured".to_string()
                    ))?;
                
                let catalog = registry.get_catalog(&db_ref.catalog)
                    .ok_or_else(|| EngineError::InvalidPlan(
                        format!("Catalog '{:?}' not found", db_ref.catalog)
                    ))?;
                
                let reader_factory = catalog.get_table(&db_ref.path)
                    .ok_or_else(|| EngineError::InvalidPlan(
                        format!("Table '{:?}' not found in catalog '{:?}'",
                                db_ref.path, db_ref.catalog)
                    ))?;
                
                self.compile_catalog_scan(reader_factory, scan)
            }
            
            // Other expression types
            expr => self.compile_dynamic_scan(expr, scan)
        }
    }
}
```

**Location:** `partiql-eval/src/engine/compiler.rs`

### 5. Planner Integration (partiql-logical-planner)

Update the planner to recognize qualified names and create DBRef:

```rust
// In partiql-logical-planner/src/lower.rs

// When processing FROM clause:
fn process_table_reference(&mut self, table_ref: &TableRef) -> BindingsOp {
    match table_ref {
        // Simple name: "FROM users"
        TableRef::Simple(name) => {
            // Use existing Scan behavior (backwards compatible)
            BindingsOp::Scan(Scan {
                expr: ValueExpr::VarRef(
                    BindingsName::from(name),
                    VarRefType::Global
                ),
                as_key: name.clone(),
                at_key: None,
            })
        }
        
        // Qualified name: "FROM my_catalog.users" or "FROM my_catalog.schema.users"
        TableRef::Qualified { catalog, path } => {
            let catalog_name = BindingsName::from(catalog);
            let object_path = path.iter()
                .map(|segment| BindingsName::from(segment))
                .collect();
            
            BindingsOp::Scan(Scan {
                expr: ValueExpr::DBRef(DBRef {
                    catalog: catalog_name,
                    path: object_path,
                }),
                as_key: path.last().unwrap().clone(),
                at_key: None,
            })
        }
    }
}
```

**Location:** `partiql-logical-planner/src/lower.rs`

## Example Usage

### Implementing a Custom Catalog

```rust
use partiql_value::BindingsName;
use partiql_eval::engine::catalog::DataCatalog;
use std::borrow::Cow;

struct MyCatalog {
    name: BindingsName<'static>,
    tables: FxHashMap<Vec<String>, ReaderFactory>,  // normalized keys
}

impl MyCatalog {
    pub fn new(name: &str, case_sensitive: bool) -> Self {
        let name = if case_sensitive {
            BindingsName::CaseSensitive(Cow::Owned(name.to_string()))
        } else {
            BindingsName::CaseInsensitive(Cow::Owned(name.to_string()))
        };
        
        Self {
            name,
            tables: FxHashMap::default(),
        }
    }
    
    pub fn register_table(&mut self, path: Vec<&str>, factory: ReaderFactory) {
        // Normalize path for lookup
        let normalized: Vec<String> = path.iter()
            .map(|s| s.to_lowercase())
            .collect();
        self.tables.insert(normalized, factory);
    }
}

impl DataCatalog for MyCatalog {
    fn name(&self) -> &BindingsName<'_> {
        &self.name
    }
    
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<ReaderFactory> {
        // Normalize path for lookup, respecting case sensitivity
        let normalized: Vec<String> = path.iter()
            .map(|name| match name {
                BindingsName::CaseSensitive(n) => n.to_string(),
                BindingsName::CaseInsensitive(n) => n.to_lowercase(),
            })
            .collect();
        
        self.tables.get(&normalized).cloned()
    }
}
```

### Using Catalogs with PlanCompiler

```rust
use partiql_eval::engine::catalog::{CatalogRegistry, DataCatalog};
use std::sync::Arc;

// Create and configure catalogs
let mut my_catalog = MyCatalog::new("my_catalog", false);
my_catalog.register_table(vec!["users"], create_users_reader_factory());
my_catalog.register_table(vec!["public", "orders"], create_orders_reader_factory());

// Create registry and register catalogs
let mut registry = CatalogRegistry::new();
registry.register_catalog(Arc::new(my_catalog));

// Create compiler with registry
let compiler = PlanCompiler::new(Some(registry));
let compiled = compiler.compile(&logical_plan)?;

// Execute query that references catalog tables
let mut vm = PartiQLVM::new(compiled, None)?;
let result = vm.execute()?;
```

### Query Examples

```sql
-- Simple unqualified reference (uses VarRef, backwards compatible)
SELECT * FROM users;

-- Qualified catalog reference (uses DBRef)
SELECT * FROM my_catalog.users;

-- Schema-qualified reference (uses DBRef)
SELECT * FROM my_catalog.public.orders;

-- Case-sensitive reference (if catalog configured for case sensitivity)
SELECT * FROM "MySchema"."Users";
```

## Benefits

### 1. Flexibility
- DBRef can be used anywhere ValueExpr is accepted
- Natural extension point for subqueries, computed expressions, etc.

### 2. Simplicity
- No new BindingsOp variant needed
- Reuses existing Scan operator infrastructure
- Clean separation between logical plan and execution

### 3. Case Sensitivity
- Proper SQL semantics through BindingsName
- Catalog implementations control matching behavior
- Type-safe at compile time

### 4. Backwards Compatibility
- Existing VarRef usage unchanged
- Simple unqualified names still work as before
- Opt-in catalog functionality

### 5. Performance
- Catalog lookup happens at compile time (not per row)
- ReaderFactory resolved once during compilation
- No runtime overhead for name resolution

### 6. Future-Proofing
- Natural migration path from VarRef(Global) to DBRef
- Can deprecate VarRefType::Global eventually
- Foundation for advanced catalog features (views, stored procedures, etc.)

## Implementation Plan

### Phase 1: Core Infrastructure
1. Add DBRef struct and ValueExpr::DBRef variant to partiql-logical
2. Create catalog.rs module in partiql-eval with DataCatalog trait
3. Implement CatalogRegistry

### Phase 2: Compiler Integration
1. Update PlanCompiler to accept CatalogRegistry
2. Add compile_catalog_scan method
3. Update compile_scan to handle DBRef case

### Phase 3: Planner Integration
1. Update AST to support qualified names (if needed)
2. Update planner to detect qualified names
3. Generate DBRef expressions for qualified references

### Phase 4: Public API
1. Export catalog types in partiql-eval/src/lib.rs
2. Update documentation
3. Add examples

### Phase 5: Testing
1. Unit tests for CatalogRegistry
2. Integration tests with custom catalogs
3. Case sensitivity tests
4. Backwards compatibility tests

## Migration Path

### For Existing Code (No Catalogs)
No changes required. Simple unqualified names continue to use VarRef and work as before.

### For New Code (With Catalogs)
```rust
// Before: Only bindings-based lookup
let compiler = PlanCompiler::new(None);

// After: With catalog support
let mut registry = CatalogRegistry::new();
registry.register_catalog(Arc::new(my_catalog));
let compiler = PlanCompiler::new(Some(registry));
```

### Future: Deprecate Global VarRef
Once catalogs are mature, could deprecate VarRefType::Global in favor of explicit catalog references with a "default" catalog.

## Open Questions

1. **Default Catalog**: Should there be a concept of a default catalog for unqualified names?
   - Pro: Easier migration from VarRef(Global)
   - Con: Adds complexity, potential ambiguity

2. **Catalog Chaining**: Should catalogs be able to delegate to other catalogs?
   - Pro: Enables composite catalogs, federation
   - Con: Increases complexity, potential for cycles

3. **Mutable Catalogs**: Should catalogs support runtime updates (adding/removing tables)?
   - Pro: Dynamic environments, catalog refresh
   - Con: Thread-safety complexity, cache invalidation

4. **View Support**: Should catalogs provide views (computed tables)?
   - Pro: Complete SQL semantics
   - Con: Requires query execution within catalog

5. **Schema Discovery**: Should catalogs support listing available tables/schemas?
   - Pro: IDE support, introspection
   - Con: Not needed for query execution

## Design Decisions Log

### Decision 1: Expression vs Operator
**Decision:** Make DBRef a ValueExpr, not a separate BindingsOp variant.

**Rationale:**
- More flexible (can use in any expression context)
- Simpler (reuses existing Scan operator)
- Future-proof (enables subqueries, computed expressions, etc.)

### Decision 2: Use BindingsName
**Decision:** Use BindingsName for all name components (catalog and path).

**Rationale:**
- Consistent with existing VarRef
- Preserves case sensitivity information
- Type-safe, compiler-enforced
- Enables correct SQL semantics

### Decision 3: Compile-Time Resolution
**Decision:** Resolve catalogs and tables at compile time (not runtime).

**Rationale:**
- Better error messages (fail at compile, not execution)
- Better performance (no per-row lookup)
- Simpler runtime (ReaderFactory already resolved)
- Aligns with existing planner/compiler architecture

### Decision 4: No ScanDB Variant
**Decision:** Don't add a separate ScanDB variant to BindingsOp.

**Rationale:**
- Existing Scan can handle both VarRef and DBRef
- Avoids duplication
- Keeps BindingsOp focused on relational operations
- Expressions are the right level of abstraction for "what to scan"

## Future Enhancements

### Short-term
- Add schema introspection API to DataCatalog
- Support table aliasing in FROM clauses
- Add catalog-level metadata (version, description, etc.)

### Medium-term
- Virtual tables / views computed from sub-queries
- Catalog-provided UDFs/UDAFs
- Cross-catalog joins optimization

### Long-term
- Federated query optimization
- Catalog-based statistics for query planning
- Catalog versioning and schema evolution

## References

- PartiQL Specification: https://partiql.org/spec.html
- SQL Standard (ISO/IEC 9075): Qualified names and catalogs
- Design discussions: This document evolution section

## Approval

This design has been discussed and refined through multiple iterations:
1. Initial ScanDB variant approach
2. Refined to DBRef expression approach
3. Final refinement with BindingsName

Ready for implementation once approved.
