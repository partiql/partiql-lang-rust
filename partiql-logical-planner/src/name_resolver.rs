use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};
use partiql_ast::ast;
use partiql_ast::visit::{Visit, Visitor};
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
    pub in_scope: FnvIndexMap<ast::NodeId, Vec<ast::NodeId>>,
    pub schema: FnvIndexMap<ast::NodeId, KeySchema>,
    pub aliases: FnvIndexMap<ast::NodeId, Symbol>,
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
#[derive(Default, Debug)]
pub struct NameResolver {
    // environment stack tracking
    id_path_to_root: Vec<ast::NodeId>,
    id_child_stack: Vec<Vec<ast::NodeId>>,
    keyref_stack: Vec<KeyRefs>,
    lateral_stack: Vec<Vec<ast::NodeId>>,
    id_gen: IdGenerator,

    // data flow tracking
    enclosing_clause: FnvIndexMap<EnclosingClause, Vec<ast::NodeId>>,
    in_scope: FnvIndexMap<ast::NodeId, Vec<ast::NodeId>>,
    schema: FnvIndexMap<ast::NodeId, KeySchema>,
    aliases: FnvIndexMap<ast::NodeId, Symbol>,
}

impl NameResolver {
    pub fn resolve(&mut self, query: &ast::AstNode<ast::Query>) -> KeyRegistry {
        query.visit(self);

        let in_scope = std::mem::take(&mut self.in_scope);
        let schema = std::mem::take(&mut self.schema);
        let aliases = std::mem::take(&mut self.aliases);
        KeyRegistry {
            in_scope,
            schema,
            aliases,
        }
    }

    #[inline]
    fn current_node(&self) -> &ast::NodeId {
        self.id_path_to_root.last().unwrap()
    }

    #[inline]
    fn is_from_path(&self) -> bool {
        let is_qnode = |typ, id| {
            self.enclosing_clause
                .get(&typ)
                .map(|nodes| nodes.contains(id))
                .unwrap_or(false)
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
    fn exit_lateral(&mut self) -> Vec<ast::NodeId> {
        self.lateral_stack.pop().expect("lateral level")
    }

    #[inline]
    fn enter_child_stack(&mut self) {
        self.id_child_stack.push(vec![]);
    }

    #[inline]
    fn exit_child_stack(&mut self) -> Vec<ast::NodeId> {
        self.id_child_stack.pop().expect("child level")
    }

    #[inline]
    fn enter_keyref(&mut self) {
        self.keyref_stack.push(KeyRefs::default());
    }

    #[inline]
    fn exit_keyref(&mut self) -> KeyRefs {
        self.keyref_stack.pop().expect("io level")
    }

    #[inline]
    fn push_consume_name(&mut self, name: NameRef) {
        self.keyref_stack.last_mut().unwrap().consume.insert(name);
    }
}

impl<'ast> Visitor<'ast> for NameResolver {
    fn enter_ast_node(&mut self, id: ast::NodeId) {
        self.id_path_to_root.push(id);
        if let Some(children) = self.id_child_stack.last_mut() {
            children.push(id);
        }
    }
    fn exit_ast_node(&mut self, id: ast::NodeId) {
        assert_eq!(self.id_path_to_root.pop(), Some(id))
    }

    fn enter_query(&mut self, _query: &'ast ast::Query) {
        let id = *self.current_node();
        self.enclosing_clause
            .entry(EnclosingClause::Query)
            .or_insert_with(Vec::new)
            .push(id);
        self.enter_keyref();
    }

    fn exit_query(&mut self, _query: &'ast ast::Query) {
        let id = *self.current_node();
        let keyrefs = self.exit_keyref();

        // Collect the variables produced & consumed by this (sub)query.
        let KeyRefs {
            consume,
            produce_required,
            produce_optional,
        } = keyrefs;
        let mut produce: Names = produce_required;
        produce.extend(produce_optional.iter().flat_map(|sym| sym.to_owned()));

        let schema = KeySchema { consume, produce };

        self.schema.insert(id, schema);
    }

    fn enter_from_clause(&mut self, _from_clause: &'ast ast::FromClause) {
        self.enter_lateral();
        self.enter_child_stack();
    }

    fn exit_from_clause(&mut self, _from_clause: &'ast ast::FromClause) {
        self.exit_lateral();
        self.exit_child_stack();
    }

    fn enter_join(&mut self, _join: &'ast ast::Join) {
        self.enter_child_stack();
    }

    fn exit_join(&mut self, _join: &'ast ast::Join) {
        self.exit_child_stack();
    }

    fn enter_from_let(&mut self, _from_let: &'ast ast::FromLet) {
        self.enter_child_stack();

        let id = *self.current_node();
        self.enclosing_clause
            .entry(EnclosingClause::FromLet)
            .or_insert_with(Vec::new)
            .push(id);
        self.enter_keyref();

        // Scopes above this `FROM` in the AST are in-scope to use variables defined by this from
        for in_scope in self.id_path_to_root.iter().rev().skip(1) {
            self.in_scope
                .entry(*in_scope)
                .or_insert_with(Vec::new)
                .push(id);
        }

        // This `FROM` item is in-scope of variables defined by any preceding items in this `FROM` (e.g., lateral joins)
        for in_scope in self.lateral_stack.last().unwrap() {
            self.in_scope
                .entry(id)
                .or_insert_with(Vec::new)
                .push(*in_scope);
        }

        self.lateral_stack.last_mut().unwrap().push(id);
    }

    fn exit_from_let(&mut self, from_let: &'ast ast::FromLet) {
        self.exit_child_stack();
        let id = *self.current_node();
        let KeyRefs { consume, .. } = self.exit_keyref();

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
    }

    fn enter_var_ref(&mut self, var_ref: &'ast ast::VarRef) {
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
    }

    fn exit_project_expr(&mut self, project_expr: &'ast ast::ProjectExpr) {
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
    }
}

/// Attempt to infer an alias for a simple variable reference expression.
/// For example infer such that  `SELECT a, b.c.d.e ...` <=> `SELECT a as a, b.c.d.e as e`  
fn infer_alias(expr: &ast::Expr) -> Option<ast::SymbolPrimitive> {
    match expr {
        ast::Expr::VarRef(ast::AstNode { node, .. }) => Some(node.name.clone()),
        ast::Expr::Path(ast::AstNode { node, .. }) => match node.steps.last() {
            Some(ast::PathStep::PathExpr(expr)) => infer_alias(&expr.index),
            _ => None,
        },
        _ => None,
    }
}
