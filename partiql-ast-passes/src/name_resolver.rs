use crate::error::{AstTransformError, AstTransformationError};
use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};
use partiql_ast::ast;
use partiql_ast::ast::{GroupByExpr, GroupKey};
use partiql_ast::visit::{Traverse, Visit, Visitor};
use partiql_catalog::Catalog;
use partiql_common::node::NodeId;
use std::sync::atomic::{AtomicU32, Ordering};

type FnvIndexSet<T> = IndexSet<T, FnvBuildHasher>;

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NameRef {
    pub sym: ast::SymbolPrimitive,
    pub lookup: Vec<NameLookup>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NameLookup {
    Global,
    Local,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Symbol {
    Known(ast::SymbolPrimitive),
    Unknown(u32),
}

type NameRefs = FnvIndexSet<NameRef>;
type Names = FnvIndexSet<Symbol>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct KeySchema {
    pub consume: NameRefs,
    pub produce: Names,
}

#[derive(Default, Debug, Clone)]
pub struct KeyRegistry {
    pub in_scope: FnvIndexMap<NodeId, Vec<NodeId>>,
    pub schema: FnvIndexMap<NodeId, KeySchema>,
    pub aliases: FnvIndexMap<NodeId, Symbol>,
}

#[derive(Debug)]
struct IdGenerator {
    next_id: AtomicU32,
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self {
            next_id: AtomicU32::new(1),
        }
    }
}

impl IdGenerator {
    fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

type NameOptions = FnvIndexSet<Option<Symbol>>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct KeyRefs {
    pub consume: NameRefs,
    pub produce_required: Names,
    pub produce_optional: NameOptions,
}

// The enclosing clause; used, in part, to track whether a name is a 'from path' reference
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum EnclosingClause {
    FromLet,
    Query,
}

/// Resolves which clauses in a query produce & consume variable references by walking the
/// AST and collecting variable references. Also partially infers alias if no `AS` alias
/// was provided in the query.
#[derive(Debug)]
#[allow(dead_code)]
pub struct NameResolver<'c> {
    // environment stack tracking
    id_path_to_root: Vec<NodeId>,
    id_child_stack: Vec<Vec<NodeId>>,
    keyref_stack: Vec<KeyRefs>,
    lateral_stack: Vec<Vec<NodeId>>,
    id_gen: IdGenerator,

    // data flow tracking
    enclosing_clause: FnvIndexMap<EnclosingClause, Vec<NodeId>>,
    in_scope: FnvIndexMap<NodeId, Vec<NodeId>>,
    schema: FnvIndexMap<NodeId, KeySchema>,
    aliases: FnvIndexMap<NodeId, Symbol>,

    // errors that occur during name resolution
    errors: Vec<AstTransformError>,
    catalog: &'c dyn Catalog,
}

impl<'c> NameResolver<'c> {
    pub fn new(catalog: &'c dyn Catalog) -> Self {
        NameResolver {
            // environment stack tracking
            id_path_to_root: Default::default(),
            id_child_stack: Default::default(),
            keyref_stack: Default::default(),
            lateral_stack: Default::default(),
            id_gen: Default::default(),

            // data flow tracking
            enclosing_clause: Default::default(),
            in_scope: Default::default(),
            schema: Default::default(),
            aliases: Default::default(),

            // errors that occur during name resolution
            errors: Default::default(),
            catalog,
        }
    }

    pub fn resolve(
        &mut self,
        query: &ast::AstNode<ast::TopLevelQuery>,
    ) -> Result<KeyRegistry, AstTransformationError> {
        query.visit(self);
        if !self.errors.is_empty() {
            return Err(AstTransformationError {
                errors: std::mem::take(&mut self.errors),
            });
        }

        let in_scope = std::mem::take(&mut self.in_scope);
        let schema = std::mem::take(&mut self.schema);
        let aliases = std::mem::take(&mut self.aliases);
        Ok(KeyRegistry {
            in_scope,
            schema,
            aliases,
        })
    }

    #[inline]
    fn current_node(&self) -> &NodeId {
        self.id_path_to_root.last().unwrap()
    }

    #[inline]
    fn is_from_path(&self) -> bool {
        let is_qnode = |typ, id| {
            self.enclosing_clause
                .get(&typ)
                .is_some_and(|nodes| nodes.contains(id))
        };
        for id in self.id_path_to_root.iter().rev() {
            if is_qnode(EnclosingClause::Query, id) {
                return false;
            } else if is_qnode(EnclosingClause::FromLet, id) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn enter_lateral(&mut self) {
        self.lateral_stack.push(vec![]);
    }

    #[inline]
    fn exit_lateral(&mut self) -> Result<Vec<NodeId>, AstTransformError> {
        self.lateral_stack.pop().ok_or_else(|| {
            AstTransformError::IllegalState("Expected non-empty lateral stack".to_string())
        })
    }

    #[inline]
    fn enter_child_stack(&mut self) {
        self.id_child_stack.push(vec![]);
    }

    #[inline]
    fn exit_child_stack(&mut self) -> Result<Vec<NodeId>, AstTransformError> {
        self.id_child_stack.pop().ok_or_else(|| {
            AstTransformError::IllegalState("Expected non-empty child stack".to_string())
        })
    }

    #[inline]
    fn enter_keyref(&mut self) {
        self.keyref_stack.push(KeyRefs::default());
    }

    #[inline]
    fn exit_keyref(&mut self) -> Result<KeyRefs, AstTransformError> {
        self.keyref_stack.pop().ok_or_else(|| {
            AstTransformError::IllegalState("Expected non-empty keyrefs".to_string())
        })
    }

    #[inline]
    fn push_consume_name(&mut self, name: NameRef) {
        self.keyref_stack.last_mut().unwrap().consume.insert(name);
    }
}

impl<'ast, 'c> Visitor<'ast> for NameResolver<'c> {
    fn enter_ast_node(&mut self, id: NodeId) -> Traverse {
        self.id_path_to_root.push(id);
        if let Some(children) = self.id_child_stack.last_mut() {
            children.push(id);
        }
        Traverse::Continue
    }
    fn exit_ast_node(&mut self, id: NodeId) -> Traverse {
        assert_eq!(self.id_path_to_root.pop(), Some(id));
        Traverse::Continue
    }

    fn enter_query(&mut self, _query: &'ast ast::Query) -> Traverse {
        let id = *self.current_node();
        self.enclosing_clause
            .entry(EnclosingClause::Query)
            .or_default()
            .push(id);
        self.enter_keyref();
        Traverse::Continue
    }

    fn exit_query(&mut self, _query: &'ast ast::Query) -> Traverse {
        let id = *self.current_node();
        let keyrefs = match self.exit_keyref() {
            Ok(kr) => kr,
            Err(e) => {
                self.errors.push(e);
                return Traverse::Stop;
            }
        };

        // Collect the variables produced & consumed by this (sub)query.
        let KeyRefs {
            consume,
            produce_required,
            produce_optional,
        } = keyrefs;
        let mut produce: Names = produce_required;
        produce.extend(
            produce_optional
                .iter()
                .filter_map(std::borrow::ToOwned::to_owned),
        );

        let schema = KeySchema { consume, produce };

        self.schema.insert(id, schema);
        Traverse::Continue
    }

    fn enter_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Traverse {
        self.enter_lateral();
        self.enter_child_stack();
        Traverse::Continue
    }

    fn exit_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Traverse {
        if let Err(e) = self.exit_lateral() {
            self.errors.push(e);
            return Traverse::Stop;
        };
        if let Err(e) = self.exit_child_stack() {
            self.errors.push(e);
            return Traverse::Stop;
        };
        Traverse::Continue
    }

    fn enter_join(&mut self, _join: &'ast ast::Join) -> Traverse {
        self.enter_child_stack();
        Traverse::Continue
    }

    fn exit_join(&mut self, _join: &'ast ast::Join) -> Traverse {
        if let Err(e) = self.exit_child_stack() {
            self.errors.push(e);
            return Traverse::Stop;
        };
        Traverse::Continue
    }

    fn enter_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Traverse {
        self.enter_child_stack();

        let id = *self.current_node();
        self.enclosing_clause
            .entry(EnclosingClause::FromLet)
            .or_default()
            .push(id);
        self.enter_keyref();

        // Scopes above this `FROM` in the AST are in-scope to use variables defined by this from
        for in_scope in self.id_path_to_root.iter().rev().skip(1) {
            self.in_scope.entry(*in_scope).or_default().push(id);
        }

        // This `FROM` item is in-scope of variables defined by any preceding items in this `FROM` (e.g., lateral joins)
        for in_scope in self.lateral_stack.last().unwrap() {
            self.in_scope.entry(id).or_default().push(*in_scope);
        }

        self.lateral_stack.last_mut().unwrap().push(id);
        Traverse::Continue
    }

    fn exit_from_let(&mut self, from_let: &'ast ast::FromLet) -> Traverse {
        if let Err(e) = self.exit_child_stack() {
            self.errors.push(e);
            return Traverse::Stop;
        };
        let id = *self.current_node();
        let KeyRefs { consume, .. } = match self.exit_keyref() {
            Ok(kr) => kr,
            Err(e) => {
                self.errors.push(e);
                return Traverse::Stop;
            }
        };

        // get the "as" alias
        // 1. if explicitly given
        // 2. else try to infer if a simple variable reference or path
        // 3. else it is currently 'Unknown'
        let as_alias = if let Some(sym) = &from_let.as_alias {
            Symbol::Known(sym.clone())
        } else if let Some(sym) = infer_alias(&from_let.expr) {
            Symbol::Known(sym)
        } else {
            Symbol::Unknown(self.id_gen.next_id())
        };
        let at_alias = from_let
            .at_alias
            .as_ref()
            .map(|sym| Symbol::Known(sym.to_owned()));
        let produce: Names = std::iter::once(as_alias).chain(at_alias).collect();
        for alias in &produce {
            self.aliases.insert(id, alias.clone());
        }

        self.schema.insert(id, KeySchema { consume, produce });
        Traverse::Continue
    }

    fn enter_var_ref(&mut self, var_ref: &'ast ast::VarRef) -> Traverse {
        let is_from_path = self.is_from_path();

        // in a From path, a prefix `@` means to look locally before globally Cf. specification section 10
        let name = if is_from_path {
            match &var_ref.qualifier {
                ast::ScopeQualifier::Unqualified => NameRef {
                    sym: var_ref.name.clone(),
                    lookup: vec![NameLookup::Global, NameLookup::Local],
                },
                ast::ScopeQualifier::Qualified => NameRef {
                    sym: var_ref.name.clone(),
                    lookup: vec![NameLookup::Local, NameLookup::Global],
                },
            }
        } else {
            NameRef {
                sym: var_ref.name.clone(),
                lookup: vec![NameLookup::Local, NameLookup::Global],
            }
        };

        self.push_consume_name(name);
        Traverse::Continue
    }

    fn exit_project_expr(&mut self, project_expr: &'ast ast::ProjectExpr) -> Traverse {
        let id = self.current_node();
        // get the "as" alias
        // 1. if explicitly given
        // 2. else try to infer if a simple variable reference or path
        // 3. else it is currently 'Unknown'
        let as_alias = if let Some(sym) = &project_expr.as_alias {
            Symbol::Known(sym.clone())
        } else if let Some(sym) = infer_alias(&project_expr.expr) {
            Symbol::Known(sym)
        } else {
            Symbol::Unknown(self.id_gen.next_id())
        };
        self.aliases.insert(*id, as_alias.clone());
        self.keyref_stack
            .last_mut()
            .unwrap()
            .produce_required
            .insert(as_alias);
        Traverse::Continue
    }

    fn enter_group_key(&mut self, _group_key: &'ast GroupKey) -> Traverse {
        self.enter_keyref();
        let id = *self.current_node();

        if self
            .enclosing_clause
            .get(&EnclosingClause::FromLet)
            .is_none()
        {
            self.errors.push(AstTransformError::IllegalState(
                "group_key expects a FromLet enclosing clause".to_string(),
            ));
        }

        self.enclosing_clause
            .get(&EnclosingClause::FromLet)
            .expect("EnclosingClause::FromLet")
            .iter()
            .for_each(|enclosing_clause| {
                self.in_scope.entry(id).or_default().push(*enclosing_clause);
            });

        self.enclosing_clause
            .entry(EnclosingClause::Query)
            .or_default()
            .push(id);
        Traverse::Continue
    }

    fn exit_group_key(&mut self, group_key: &'ast GroupKey) -> Traverse {
        let KeyRefs {
            consume,
            produce_required,
            ..
        } = match self.exit_keyref() {
            Ok(kr) => kr,
            Err(e) => {
                self.errors.push(e);
                return Traverse::Stop;
            }
        };
        let mut produce = produce_required;

        let id = *self.current_node();
        // get the "as" alias for each `GROUP BY` expr
        // 1. if explicitly given
        // 2. else try to infer if a simple variable reference or path
        // 3. else it is currently 'Unknown'
        let as_alias = if let Some(sym) = &group_key.as_alias {
            Symbol::Known(sym.clone())
        } else if let Some(sym) = infer_alias(&group_key.expr) {
            Symbol::Known(sym)
        } else {
            Symbol::Unknown(self.id_gen.next_id())
        };
        self.aliases.insert(id, as_alias.clone());
        produce.insert(as_alias.clone());
        self.keyref_stack
            .last_mut()
            .unwrap()
            .produce_required
            .insert(as_alias);
        self.schema.insert(id, KeySchema { consume, produce });
        Traverse::Continue
    }

    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast GroupByExpr) -> Traverse {
        self.enter_keyref();
        let id = *self.current_node();
        // Scopes above this `GROUP BY` in the AST are in-scope to use variables defined by this GROUP BY
        for in_scope in self.id_path_to_root.iter().rev().skip(1) {
            self.in_scope.entry(*in_scope).or_default().push(id);
        }
        Traverse::Continue
    }

    fn exit_group_by_expr(&mut self, group_by_expr: &'ast GroupByExpr) -> Traverse {
        let id = *self.current_node();
        let KeyRefs {
            consume,
            produce_required,
            ..
        } = match self.exit_keyref() {
            Ok(kr) => kr,
            Err(e) => {
                self.errors.push(e);
                return Traverse::Stop;
            }
        };

        // TODO: delete in_scope for FROM sources in subsequent clauses

        let mut produce: Names = produce_required;
        // add the `GROUP AS` alias
        if let Some(sym) = &group_by_expr.group_as_alias {
            let as_alias = Symbol::Known(sym.clone());
            self.aliases.insert(id, as_alias.clone());
            produce.insert(as_alias);
        }
        self.schema.insert(id, KeySchema { consume, produce });
        Traverse::Continue
    }
}

/// Attempt to infer an alias for a simple variable reference expression.
/// For example infer such that  `SELECT a, b.c.d.e ...` <=> `SELECT a as a, b.c.d.e as e`  
fn infer_alias(expr: &ast::Expr) -> Option<ast::SymbolPrimitive> {
    match expr {
        ast::Expr::VarRef(ast::AstNode { node, .. }) => Some(node.name.clone()),
        ast::Expr::Path(ast::AstNode { node, .. }) => match node.steps.last() {
            Some(ast::PathStep::PathProject(expr)) => infer_alias(&expr.index),
            Some(ast::PathStep::PathIndex(expr)) => infer_alias(&expr.index),
            _ => None,
        },
        _ => None,
    }
}
