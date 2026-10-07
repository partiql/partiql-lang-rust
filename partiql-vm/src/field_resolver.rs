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

use partiql_logical::{PathComponent, Pattern, ValueExpr};
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

/// Walks a `ValueExpr` tree and records what each scan must provide in the
/// `CompileContext`: a field for every `alias.field[...]` path, and the whole value for
/// every other use of a binding (a bare `alias`, `alias[0]`, ...).
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

    fn walk_all<'e>(&mut self, exprs: impl IntoIterator<Item = &'e ValueExpr>) {
        exprs.into_iter().for_each(|e| self.walk(e));
    }

    fn walk(&mut self, expr: &ValueExpr) {
        match expr {
            ValueExpr::Path(base, steps) => {
                match (&**base, steps.first()) {
                    // `alias.field...`: only `field` is needed from the scan; any further
                    // steps navigate within it.
                    (ValueExpr::VarRef(alias, _), Some(PathComponent::Key(field))) => {
                        self.ctx.request_field(
                            &bindings_name_to_string(alias),
                            &bindings_name_to_string(field),
                        );
                    }
                    _ => self.walk(base),
                }
                for step in steps {
                    match step {
                        PathComponent::KeyExpr(e) | PathComponent::IndexExpr(e) => self.walk(e),
                        _ => {}
                    }
                }
            }
            // A bare variable needs the whole value bound to it.
            ValueExpr::VarRef(alias, _) => {
                self.ctx
                    .request_whole_value(&bindings_name_to_string(alias));
            }
            ValueExpr::Lit(_) | ValueExpr::DBRef(_) => {}
            ValueExpr::UnExpr(_, e) => self.walk(e),
            ValueExpr::BinaryExpr(_, lhs, rhs) => self.walk_all([&**lhs, &**rhs]),
            ValueExpr::DynamicLookup(lookups) => self.walk_all(lookups.iter()),
            ValueExpr::TupleExpr(t) => self.walk_all(t.attrs.iter().chain(&t.values)),
            ValueExpr::ListExpr(l) => self.walk_all(&l.elements),
            ValueExpr::BagExpr(b) => self.walk_all(&b.elements),
            ValueExpr::BetweenExpr(b) => self.walk_all([&*b.value, &*b.from, &*b.to]),
            ValueExpr::PatternMatchExpr(m) => {
                self.walk(&m.value);
                match &m.pattern {
                    Pattern::LikeNonStringNonLiteral(p) => self.walk_all([&*p.pattern, &*p.escape]),
                    Pattern::Like(_) => {}
                    _ => self.ctx.request_all_whole_values(),
                }
            }
            ValueExpr::SimpleCase(c) => {
                self.walk(&c.expr);
                self.walk_cases(&c.cases, c.default.as_deref());
            }
            ValueExpr::SearchedCase(c) => self.walk_cases(&c.cases, c.default.as_deref()),
            ValueExpr::IsTypeExpr(t) => self.walk(&t.expr),
            ValueExpr::NullIfExpr(n) => self.walk_all([&*n.lhs, &*n.rhs]),
            ValueExpr::CoalesceExpr(c) => self.walk_all(&c.elements),
            ValueExpr::Call(c) => self.walk_all(&c.arguments),
            // A subquery or graph match may reference outer bindings in ways not visible
            // here, and future expression kinds are unknown: load whole values.
            _ => self.ctx.request_all_whole_values(),
        }
    }

    fn walk_cases(
        &mut self,
        cases: &[(Box<ValueExpr>, Box<ValueExpr>)],
        default: Option<&ValueExpr>,
    ) {
        for (when, then) in cases {
            self.walk_all([&**when, &**then]);
        }
        if let Some(default) = default {
            self.walk(default);
        }
    }
}

fn bindings_name_to_string(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseSensitive(s) => s.as_ref().to_string(),
        BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_logical::{CallExpr, CallName, Lit, SearchedCase, VarRefType};

    fn var(name: &str) -> ValueExpr {
        ValueExpr::VarRef(
            BindingsName::CaseInsensitive(name.to_string().into()),
            VarRefType::Local,
        )
    }

    fn path(base: &str, keys: &[&str]) -> ValueExpr {
        ValueExpr::Path(
            Box::new(var(base)),
            keys.iter()
                .map(|k| PathComponent::Key(BindingsName::CaseInsensitive(k.to_string().into())))
                .collect(),
        )
    }

    fn extract(expr: &ValueExpr) -> CompileContext {
        let mut ctx = CompileContext::new();
        ExprFieldExtractor::new(&mut ctx).extract(expr);
        ctx
    }

    fn fields(ctx: &CompileContext, alias: &str) -> Vec<String> {
        ctx.requests_for_alias(alias, None)
            .iter()
            .map(|r| r.field_name.clone())
            .collect()
    }

    #[test]
    fn nested_path_requests_its_first_field() {
        let ctx = extract(&path("a", &["x", "y"]));
        assert_eq!(fields(&ctx, "a"), ["x"]);
        assert!(!ctx.needs_whole_value("a", None));
    }

    #[test]
    fn call_and_case_arguments_are_walked() {
        let expr = ValueExpr::SearchedCase(SearchedCase {
            cases: vec![(
                Box::new(path("a", &["flag"])),
                Box::new(ValueExpr::Call(CallExpr {
                    name: CallName::Lower,
                    arguments: vec![path("a", &["name"])],
                })),
            )],
            default: Some(Box::new(ValueExpr::Lit(Box::new(Lit::Null)))),
        });
        let ctx = extract(&expr);
        assert_eq!(fields(&ctx, "a"), ["flag", "name"]);
        assert!(!ctx.needs_whole_value("a", None));
    }

    #[test]
    fn bare_variable_requests_whole_value_of_that_binding_only() {
        let expr = ValueExpr::BinaryExpr(
            partiql_logical::BinaryOp::Eq,
            Box::new(var("a")),
            Box::new(path("b", &["x"])),
        );
        let ctx = extract(&expr);
        assert!(ctx.needs_whole_value("a", None));
        assert!(!ctx.needs_whole_value("b", None));
    }
}
