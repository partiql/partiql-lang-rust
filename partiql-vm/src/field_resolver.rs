//! Field resolution for projection pushdown.
//!
//! This module provides the mechanism for extracting field requirements from
//! expressions (top-down) and resolving them at scan nodes (bottom-up).
//!
//! # Flow
//! 1. Parent operators extract field refs from expressions → `CompileContext`
//! 2. `CompileContext` is passed down the DFS tree
//! 3. Scan nodes resolve requests via `DataSourceHandle::resolve()`
//! 4. Scan nodes populate column_slots in the resolver
//! 5. Expression compiler uses resolver to emit SlotRef instead of GetField

use partiql_logical::{PathComponent, ValueExpr};
use partiql_value::BindingsName;
use rustc_hash::{FxHashMap, FxHashSet};

/// Unique identifier for a field request (used for deduplication).
type FieldRequestId = u32;

/// A request from a parent node to resolve a field from a specific scan.
#[derive(Debug, Clone)]
pub(crate) struct FieldRequest {
    /// The scan alias this field belongs to (e.g. "x", "_1", "data")
    pub alias: String,
    /// The field name to resolve (e.g. "a", "b")
    pub field_name: String,
}

/// Context passed DOWN the DFS tree, accumulating field requests.
#[derive(Debug, Default)]
pub(crate) struct CompileContext {
    /// Field requests from ancestor nodes
    pub requests: Vec<FieldRequest>,
    /// Counter for generating unique FieldRequestIds
    next_id: FieldRequestId,
    /// Deduplication: (alias, field) → existing request ID
    seen: FxHashMap<(String, String), FieldRequestId>,
    /// Aliases whose whole row value is referenced (e.g. a bare `VarRef(alias)`), so their
    /// scans must load the whole value rather than only the requested fields.
    whole_values: FxHashSet<String>,
    /// Some expression may reference any scan's whole value (`SELECT *`, or an expression
    /// the extractor does not walk), so no scan may rely on field pushdown alone.
    whole_values_all: bool,
}

impl CompileContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request resolution of a field. Deduplicates: same (alias, field) is
    /// only stored once.
    pub fn request_field(&mut self, alias: &str, field_name: &str) {
        let key = (alias.to_string(), field_name.to_string());
        if self.seen.contains_key(&key) {
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.requests.push(FieldRequest {
            alias: alias.to_string(),
            field_name: field_name.to_string(),
        });
        self.seen.insert(key, id);
    }

    /// Get all requests targeting a specific alias.
    /// Request the whole value of the scan bound to `alias`.
    pub fn request_whole_value(&mut self, alias: &str) {
        self.whole_values.insert(alias.to_string());
    }

    /// Request the whole value of every scan.
    pub fn request_all_whole_values(&mut self) {
        self.whole_values_all = true;
    }

    /// Whether the scan bound to `alias` (or named `table_name`) must load its whole value.
    pub fn needs_whole_value(&self, alias: &str, table_name: Option<&str>) -> bool {
        self.whole_values_all
            || self.whole_values.iter().any(|a| {
                a.eq_ignore_ascii_case(alias)
                    || table_name.is_some_and(|t| a.eq_ignore_ascii_case(t))
            })
    }

    pub fn requests_for_alias<'a>(
        &'a self,
        alias: &str,
        table_name: Option<&str>,
    ) -> Vec<&'a FieldRequest> {
        self.requests
            .iter()
            .filter(|r| {
                r.alias.eq_ignore_ascii_case(alias)
                    || table_name
                        .map(|t| r.alias.eq_ignore_ascii_case(t))
                        .unwrap_or(false)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Expression field extraction
// ---------------------------------------------------------------------------

/// Walks a `ValueExpr` tree, finding `Path(VarRef(alias), [Key(field)])` patterns
/// and recording field requests in the `CompileContext`.
pub(crate) struct ExprFieldExtractor<'a> {
    ctx: &'a mut CompileContext,
}

impl<'a> ExprFieldExtractor<'a> {
    pub fn new(ctx: &'a mut CompileContext) -> Self {
        ExprFieldExtractor { ctx }
    }

    /// Extract field requests from an expression.
    pub fn extract(&mut self, expr: &ValueExpr) {
        self.walk(expr);
    }

    fn walk(&mut self, expr: &ValueExpr) {
        match expr {
            ValueExpr::Path(base, steps) => {
                if let Some((alias, field)) = try_decompose_field_access(base, steps) {
                    self.ctx.request_field(&alias, &field);
                } else {
                    // Complex path — walk sub-expressions for any nested field accesses
                    self.walk(base);
                    for step in steps {
                        if let PathComponent::KeyExpr(key_expr) = step {
                            self.walk(key_expr);
                        }
                    }
                }
            }
            ValueExpr::BinaryExpr(_op, lhs, rhs) => {
                self.walk(lhs);
                self.walk(rhs);
            }
            ValueExpr::UnExpr(_op, operand) => {
                self.walk(operand);
            }
            // A bare variable needs the whole value bound to it.
            ValueExpr::VarRef(alias, _) => {
                self.ctx
                    .request_whole_value(&bindings_name_to_string(alias));
            }
            // Leaf expressions — no field accesses to extract
            ValueExpr::Lit(_) | ValueExpr::DBRef(_) => {}
            // Other expression types (Case, Call, etc.) are not walked yet; they may use
            // any variable, so conservatively disable pushdown.
            _ => self.ctx.request_all_whole_values(),
        }
    }
}

/// Try to decompose `Path(VarRef(alias), [Key(field)])` into (alias, field).
fn try_decompose_field_access(
    base: &ValueExpr,
    steps: &[PathComponent],
) -> Option<(String, String)> {
    if let ValueExpr::VarRef(alias, _) = base {
        if steps.len() == 1 {
            if let PathComponent::Key(field_name) = &steps[0] {
                let alias_str = bindings_name_to_string(alias);
                let field_str = bindings_name_to_string(field_name);
                return Some((alias_str, field_str));
            }
        }
    }
    None
}

fn bindings_name_to_string(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseSensitive(s) => s.as_ref().to_string(),
        BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
        other => format!("{other:?}"),
    }
}
