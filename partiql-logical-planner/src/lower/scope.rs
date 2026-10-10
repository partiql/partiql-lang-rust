//! Lexical scopes and variable-reference resolution.
//!
//! A [`Scope`] holds the names one query level binds, in declaration order, and borrows the
//! scope of the enclosing query. Lowering builds it clause by clause (FROM, then GROUP BY), so
//! at any point it holds exactly the names visible to the clause being lowered: a FROM item's
//! source sees the items before it but not itself, and GROUP BY keys are visible only to the
//! clauses after GROUP BY.

use partiql_ast::ast::{CaseSensitivity, ScopeQualifier, SymbolPrimitive, VarRef};
use partiql_ast_passes::error::AstTransformError;
use partiql_logical::{PathComponent, ValueExpr, VarRefType};
use partiql_value::BindingsName;

use super::{symprim_to_binding, Lowerer, Result, VarRefResolution};

/// The clause an expression belongs to; selects the name-resolution rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Clause {
    /// The source expression of a FROM item.
    From,
    /// A join's `ON` predicate.
    On,
    Where,
    GroupBy,
    Having,
    Select,
    OrderBy,
    Limit,
    /// A predicate inside a graph `MATCH` pattern.
    Graph,
    /// The expression of an expression query (e.g. a top-level `1 + 2`).
    Value,
}

/// What introduced a [`Binding`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BindingKind {
    /// A FROM item's `AS` name.
    FromAs,
    /// A FROM item's `AT` name.
    FromAt,
    /// A GROUP BY key.
    GroupKey,
    /// A GROUP BY's `GROUP AS` name.
    GroupAs,
    /// A node or edge variable of a graph pattern.
    Graph,
}

#[derive(Debug, Clone)]
pub(super) struct Binding {
    pub name: SymbolPrimitive,
    /// The name was generated (`_1`, ...) because the query gave none.
    pub generated: bool,
    pub kind: BindingKind,
}

/// The names bound by one query level (or graph pattern), plus the enclosing scope.
#[derive(Debug, Default)]
pub(super) struct Scope<'p> {
    parent: Option<&'p Scope<'p>>,
    bindings: Vec<Binding>,
}

impl<'p> Scope<'p> {
    pub fn new(parent: Option<&'p Scope<'p>>) -> Self {
        Scope {
            parent,
            bindings: vec![],
        }
    }

    /// A scope with the same parent and only the first `len` bindings of `self`.
    pub fn prefix(&self, len: usize) -> Scope<'p> {
        Scope {
            parent: self.parent,
            bindings: self.bindings[..len].to_vec(),
        }
    }

    pub fn bind(&mut self, name: SymbolPrimitive, generated: bool, kind: BindingKind) {
        self.bindings.push(Binding {
            name,
            generated,
            kind,
        });
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// Bindings of `self` from index `from` on.
    pub fn bindings_from(&self, from: usize) -> &[Binding] {
        &self.bindings[from..]
    }

    /// This scope, then each enclosing scope.
    fn levels(&self) -> impl Iterator<Item = &Scope<'p>> {
        std::iter::successors(Some(self), |s| s.parent)
    }

    /// Whether any visible binding is named `name`.
    fn binds(&self, name: &SymbolPrimitive) -> bool {
        self.levels()
            .flat_map(|level| &level.bindings)
            .any(|b| names_match(name, &b.name.value))
    }

    /// `name` as an attribute of the single FROM binding of the innermost level that has any;
    /// an error if that level has several. GROUP BY keys and `AT` names are not candidates.
    fn implicit_attr(&self, name: &SymbolPrimitive) -> Result<Option<ValueExpr>> {
        for level in self.levels() {
            let candidates: Vec<&Binding> = level
                .bindings
                .iter()
                .filter(|b| b.kind == BindingKind::FromAs)
                .collect();
            match candidates.as_slice() {
                [] => continue,
                [from] => {
                    return Ok(Some(ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(
                            BindingsName::CaseInsensitive(from.name.value.clone().into()),
                            VarRefType::Local,
                        )),
                        vec![PathComponent::Key(symprim_to_binding(name)?)],
                    )))
                }
                _ => {
                    return Err(AstTransformError::AmbiguousReference {
                        name: name.value.clone(),
                        candidates: candidates.iter().map(|b| b.name.value.clone()).collect(),
                    })
                }
            }
        }
        Ok(None)
    }
}

/// A SELECT-list item's alias and expression; visible to ORDER BY.
#[derive(Debug, Clone)]
pub(super) struct SelectAlias {
    pub alias: String,
    pub expr: ValueExpr,
}

/// Where an expression is being lowered.
#[derive(Debug, Clone, Copy)]
pub(super) struct Cx<'s> {
    pub scope: &'s Scope<'s>,
    pub clause: Clause,
    /// Whether this position expects a single value, so a scalar-position SQL `SELECT`
    /// subquery here is coerced to its single value (spec section 9.1).
    pub coerce: bool,
    /// The SELECT list, when lowering the ORDER BY of a SELECT query.
    pub select_aliases: Option<&'s [SelectAlias]>,
}

impl<'s> Cx<'s> {
    pub fn new(scope: &'s Scope<'s>, clause: Clause) -> Self {
        Cx {
            scope,
            clause,
            coerce: false,
            select_aliases: None,
        }
    }

    pub fn coerce(self, coerce: bool) -> Self {
        Cx { coerce, ..self }
    }
}

/// Static resolution matches binding names case-insensitively.
fn names_match(name: &SymbolPrimitive, binding: &str) -> bool {
    unicase::eq(name.value.as_str(), binding)
}

fn push_unique(lookups: &mut Vec<ValueExpr>, expr: ValueExpr) {
    if !lookups.contains(&expr) {
        lookups.push(expr);
    }
}

fn local(name: &SymbolPrimitive) -> Result<ValueExpr> {
    Ok(ValueExpr::VarRef(
        symprim_to_binding(name)?,
        VarRefType::Local,
    ))
}

fn global(name: &SymbolPrimitive) -> Result<ValueExpr> {
    Ok(ValueExpr::VarRef(
        symprim_to_binding(name)?,
        VarRefType::Global,
    ))
}

impl Lowerer<'_> {
    pub(super) fn resolve_var(&self, var: &VarRef, cx: &Cx<'_>) -> Result<ValueExpr> {
        match self.resolution {
            VarRefResolution::Dynamic => self.resolve_dynamic(var, cx),
            VarRefResolution::Static => self.resolve_static(var, cx),
        }
    }

    /// A catalog table named `name`.
    fn resolve_global(&self, name: &SymbolPrimitive) -> Result<Option<ValueExpr>> {
        Ok(match self.catalog.resolve_type(&name.value) {
            Some(_) => Some(ValueExpr::DBRef(partiql_logical::DBRef {
                catalog: self.catalog.name().to_string(),
                path: vec![symprim_to_binding(name)?],
            })),
            None => None,
        })
    }

    /// [`VarRefResolution::Static`]: resolve to a single local `VarRef`, `Path` or `DBRef`.
    ///
    /// Order (spec section 10; partiql-lang-kotlin's `TypeEnv`): an unqualified name in a FROM
    /// source is a table first, then a visible binding; anywhere else (or with `@`) a visible
    /// binding comes first. Failing both, outside FROM it is an attribute of the single FROM
    /// binding in scope, else a global. In ORDER BY, a SELECT alias that no binding shadows
    /// stands for its projected expression.
    fn resolve_static(&self, var: &VarRef, cx: &Cx<'_>) -> Result<ValueExpr> {
        let name = &var.name;
        if cx.clause == Clause::OrderBy && !cx.scope.binds(name) {
            let alias = cx
                .select_aliases
                .into_iter()
                .flatten()
                .find(|a| names_match(name, &a.alias));
            if let Some(alias) = alias {
                return Ok(alias.expr.clone());
            }
        }

        let globals_first = match var.qualifier {
            ScopeQualifier::Unqualified => cx.clause == Clause::From,
            ScopeQualifier::Qualified => false,
            _ => {
                return Err(AstTransformError::NotYetImplemented(
                    "scope qualifier".into(),
                ))
            }
        };
        let resolved = if globals_first {
            match self.resolve_global(name)? {
                Some(expr) => Some(expr),
                None if cx.scope.binds(name) => Some(local(name)?),
                None => None,
            }
        } else if cx.scope.binds(name) {
            Some(local(name)?)
        } else {
            self.resolve_global(name)?
        };
        if let Some(expr) = resolved {
            return Ok(expr);
        }
        // `FROM t1, t2` joins two tables; it never means `t1.t2`.
        if cx.clause != Clause::From {
            if let Some(expr) = cx.scope.implicit_attr(name)? {
                return Ok(expr);
            }
        }
        global(name)
    }

    /// [`VarRefResolution::Dynamic`]: a [`ValueExpr::DynamicLookup`] of every candidate, in
    /// order; the evaluator uses the first that is not `MISSING`. Candidates come from the
    /// current query level only: the evaluator binds enclosing queries' variables as globals.
    fn resolve_dynamic(&self, var: &VarRef, cx: &Cx<'_>) -> Result<ValueExpr> {
        let name = &var.name;
        let binding = symprim_to_binding(name)?;
        let mut lookups: Vec<ValueExpr> = vec![];

        if let (Clause::OrderBy, Some(aliases)) = (cx.clause, cx.select_aliases) {
            // A SELECT alias of a projected variable reads that variable.
            let target = aliases
                .iter()
                .find(|a| match name.case {
                    CaseSensitivity::CaseSensitive => name.value == a.alias,
                    _ => unicase::eq(name.value.as_str(), a.alias.as_str()),
                })
                .and_then(|a| match &a.expr {
                    ValueExpr::VarRef(target, _) => Some(target.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| binding.clone());
            lookups.push(ValueExpr::DynamicLookup(Box::new(vec![ValueExpr::VarRef(
                target,
                VarRefType::Local,
            )])));
        }

        let locals_first = match var.qualifier {
            ScopeQualifier::Unqualified => cx.clause != Clause::From,
            ScopeQualifier::Qualified => true,
            _ => {
                return Err(AstTransformError::NotYetImplemented(
                    "scope qualifier".into(),
                ))
            }
        };
        let add_global = |lookups: &mut Vec<ValueExpr>| -> Result<()> {
            push_unique(lookups, global(name)?);
            Ok(())
        };
        if !locals_first {
            add_global(&mut lookups)?;
        }

        let bindings = cx.scope.bindings();
        if bindings.iter().any(|b| !b.generated && b.name == *name) {
            lookups.push(local(name)?);
        } else {
            for b in bindings {
                let var = |n: &SymbolPrimitive| -> Result<ValueExpr> {
                    Ok(ValueExpr::VarRef(
                        if b.generated {
                            BindingsName::CaseInsensitive(n.value.clone().into())
                        } else {
                            symprim_to_binding(n)?
                        },
                        VarRefType::Local,
                    ))
                };
                let same = if b.generated {
                    b.name.value == name.value
                } else {
                    name.case == CaseSensitivity::CaseInsensitive
                        && unicase::eq(b.name.value.as_str(), name.value.as_str())
                };
                let expr = if same {
                    var(&b.name)?
                } else if !b.generated && self.catalog.resolve_type(&name.value).is_some() {
                    global(name)?
                } else {
                    ValueExpr::Path(
                        Box::new(var(&b.name)?),
                        vec![PathComponent::Key(binding.clone())],
                    )
                };
                push_unique(&mut lookups, expr);
            }
        }

        if locals_first {
            add_global(&mut lookups)?;
        }
        Ok(ValueExpr::DynamicLookup(Box::new(lookups)))
    }
}
