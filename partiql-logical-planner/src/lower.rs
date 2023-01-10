use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use num::Integer;
use ordered_float::OrderedFloat;
use partiql_ast::ast;
use partiql_ast::ast::{
    Assignment, Bag, Between, BinOp, BinOpKind, Call, CallAgg, CallArg, CallArgNamed,
    CaseSensitivity, CreateIndex, CreateTable, Ddl, DdlOp, Delete, Dml, DmlOp, DropIndex,
    DropTable, FromClause, FromLet, FromLetKind, GroupByExpr, Insert, InsertValue, Item, Join,
    JoinKind, JoinSpec, Like, List, Lit, NodeId, OnConflict, OrderByExpr, Path, PathStep,
    ProjectExpr, Projection, ProjectionKind, Query, QuerySet, Remove, SearchedCase, Select, Set,
    SetExpr, SetQuantifier, Sexp, SimpleCase, Struct, SymbolPrimitive, UniOp, UniOpKind, VarRef,
};
use partiql_ast::visit::{Visit, Visitor};
use partiql_logical as logical;
use partiql_logical::{
    BagExpr, BetweenExpr, BindingsOp, IsTypeExpr, LikeMatch, ListExpr, LogicalPlan, OpId,
    PathComponent, Pattern, PatternMatchExpr, TupleExpr, ValueExpr,
};

use partiql_value::{BindingsName, Value};

use std::collections::{HashMap, HashSet};

use crate::call_defs::{function_call_def, CallArgument, FnSymTab};
use crate::name_resolver;
use itertools::Itertools;
use std::sync::atomic::{AtomicU32, Ordering};

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

#[derive(Copy, Clone, Debug)]
enum QueryContext {
    FromLet,
    Path,
    Query,
}

#[derive(Clone, Debug, Default)]
struct QueryClauses {
    from_clause: Option<logical::OpId>,
    let_clause: Option<logical::OpId>,
    where_clause: Option<logical::OpId>,
    group_by_clause: Option<logical::OpId>,
    having_clause: Option<logical::OpId>,
    order_by_clause: Option<logical::OpId>,
    limit_clause: Option<logical::OpId>,
    offset_clause: Option<logical::OpId>,
    select_clause: Option<logical::OpId>,
    distinct: Option<logical::OpId>,
}

impl QueryClauses {
    pub fn evaluation_order(&self) -> Vec<OpId> {
        [
            self.from_clause,
            self.let_clause,
            self.where_clause,
            self.group_by_clause,
            self.having_clause,
            self.order_by_clause,
            self.limit_clause,
            self.offset_clause,
            self.select_clause,
            self.distinct,
        ]
        .iter()
        .cloned()
        .flatten()
        .collect()
    }
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
    fn id(&self) -> String {
        format!("_{}", self.next_id())
    }

    fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct AstToLogical {
    // current stack of node ids
    id_stack: Vec<NodeId>,

    q_stack: Vec<QueryClauses>,
    ctx_stack: Vec<QueryContext>,
    bexpr_stack: Vec<Vec<logical::OpId>>,
    vexpr_stack: Vec<Vec<ValueExpr>>,
    arg_stack: Vec<Vec<CallArgument>>,
    path_stack: Vec<Vec<PathComponent>>,

    from_lets: HashSet<ast::NodeId>,

    siblings: Vec<Vec<NodeId>>,

    aliases: FnvIndexMap<NodeId, SymbolPrimitive>,

    // generator of 'fresh' ids
    id: IdGenerator,

    // output
    plan: LogicalPlan<BindingsOp>,

    key_registry: name_resolver::KeyRegistry,
    fnsym_tab: FnSymTab,
}

/// Attempt to infer an alias for a simple variable reference expression.
/// For example infer such that  `SELECT a, b.c.d.e ...` <=> `SELECT a as a, b.c.d.e as e`  
fn infer_id(expr: &ValueExpr) -> Option<SymbolPrimitive> {
    let sensitive = |value| {
        Some(SymbolPrimitive {
            value,
            case: CaseSensitivity::CaseSensitive,
        })
    };
    let insensitive = |value| {
        Some(SymbolPrimitive {
            value,
            case: CaseSensitivity::CaseInsensitive,
        })
    };

    match expr {
        ValueExpr::VarRef(BindingsName::CaseInsensitive(s)) => insensitive(s.clone()),
        ValueExpr::VarRef(BindingsName::CaseSensitive(s)) => sensitive(s.clone()),
        ValueExpr::Path(_root, steps) => match steps.last() {
            Some(PathComponent::Key(BindingsName::CaseInsensitive(s))) => insensitive(s.clone()),
            Some(PathComponent::Key(BindingsName::CaseSensitive(s))) => sensitive(s.clone()),
            Some(PathComponent::KeyExpr(ke)) => match &**ke {
                ValueExpr::VarRef(BindingsName::CaseInsensitive(s)) => insensitive(s.clone()),
                ValueExpr::VarRef(BindingsName::CaseSensitive(s)) => sensitive(s.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

// TODO Error/Result
fn require_lit(expr: &ValueExpr) -> &Value {
    match expr {
        ValueExpr::Lit(lit) => lit.as_ref(),
        _ => todo!("Error on not-literal"),
    }
}

fn require_str(lit: &Value) -> &str {
    match lit {
        Value::String(s) => s.as_ref(),
        _ => todo!("Error on not-string"),
    }
}

impl AstToLogical {
    pub fn new(registry: name_resolver::KeyRegistry) -> Self {
        AstToLogical {
            id_stack: Default::default(),

            q_stack: Default::default(),
            ctx_stack: Default::default(),
            bexpr_stack: Default::default(),
            vexpr_stack: Default::default(),
            arg_stack: Default::default(),
            path_stack: Default::default(),

            from_lets: Default::default(),

            siblings: Default::default(),

            aliases: Default::default(),

            // generator of 'fresh' ids
            id: Default::default(),

            // output
            plan: Default::default(),

            key_registry: registry,
            fnsym_tab: function_call_def(),
        }
    }

    pub fn lower_query(
        mut self,
        query: &ast::AstNode<ast::Query>,
    ) -> logical::LogicalPlan<logical::BindingsOp> {
        query.visit(&mut self);
        self.plan
    }

    #[inline]
    fn current_node(&self) -> &NodeId {
        self.id_stack.last().unwrap()
    }

    #[inline]
    fn gen_id(&self) -> SymbolPrimitive {
        // TODO assure non-collision with provided identifiers. e.g., we shouldn't generate `_1` if the query contains `AS _1`
        SymbolPrimitive {
            value: self.id.id(),
            case: CaseSensitivity::CaseInsensitive,
        }
    }

    #[inline]
    fn infer_id(&self, expr: &ValueExpr, as_alias: &Option<SymbolPrimitive>) -> SymbolPrimitive {
        as_alias
            .to_owned()
            .or_else(|| infer_id(expr))
            .unwrap_or_else(|| self.gen_id())
    }

    fn resolve_varref(&self, varref: &ast::VarRef) -> logical::ValueExpr {
        // Convert a `SymbolPrimitive` into a `BindingsName`
        fn symprim_to_binding(sym: &SymbolPrimitive) -> BindingsName {
            match sym.case {
                CaseSensitivity::CaseSensitive => BindingsName::CaseSensitive(sym.value.clone()),
                CaseSensitivity::CaseInsensitive => {
                    BindingsName::CaseInsensitive(sym.value.clone())
                }
            }
        }
        // Convert a `name_resolver::Symbol` into a `BindingsName`
        fn sym_to_binding(sym: &name_resolver::Symbol) -> Option<BindingsName> {
            match sym {
                name_resolver::Symbol::Known(sym) => Some(symprim_to_binding(sym)),
                name_resolver::Symbol::Unknown(_) => None,
            }
        }

        for id in self.id_stack.iter().rev() {
            if let Some(key_schema) = self.key_registry.schema.get(id) {
                let key_schema: &name_resolver::KeySchema = key_schema;
                let name_ref: &name_resolver::NameRef = key_schema
                    .consume
                    .iter()
                    .find(|name_ref| name_ref.sym == varref.name)
                    .expect("NameRef");

                let var_binding = symprim_to_binding(&name_ref.sym);
                let var_ref_expr = ValueExpr::VarRef(var_binding.clone());

                let mut lookups = vec![];
                for lookup in &name_ref.lookup {
                    match lookup {
                        name_resolver::NameLookup::Global => {
                            if !lookups.contains(&var_ref_expr) {
                                lookups.push(var_ref_expr.clone())
                            }
                        }
                        name_resolver::NameLookup::Local => {
                            if let Some(scope_ids) = self.key_registry.in_scope.get(id) {
                                let scopes: Vec<&name_resolver::KeySchema> = scope_ids
                                    .iter()
                                    .filter_map(|scope_id| self.key_registry.schema.get(scope_id))
                                    .collect();

                                let mut exact = scopes.iter().filter(|&scope| {
                                    scope.produce.contains(&name_resolver::Symbol::Known(
                                        name_ref.sym.clone(),
                                    ))
                                });
                                if let Some(_matching) = exact.next() {
                                    lookups.push(var_ref_expr);
                                    break;
                                }

                                for scope in scopes {
                                    for produce in &scope.produce {
                                        if let name_resolver::Symbol::Known(sym) = produce {
                                            if sym == &varref.name {
                                                let expr = ValueExpr::VarRef(
                                                    sym_to_binding(produce).unwrap_or_else(|| {
                                                        symprim_to_binding(&self.gen_id())
                                                    }),
                                                );
                                                if !lookups.contains(&expr) {
                                                    lookups.push(expr);
                                                }
                                                continue;
                                            }
                                        }
                                        // else
                                        let path = logical::ValueExpr::Path(
                                            Box::new(ValueExpr::VarRef(
                                                sym_to_binding(produce).unwrap_or_else(|| {
                                                    symprim_to_binding(&self.gen_id())
                                                }),
                                            )),
                                            vec![PathComponent::Key(var_binding.clone())],
                                        );

                                        if !lookups.contains(&path) {
                                            lookups.push(path)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return ValueExpr::DynamicLookup(Box::new(lookups));
            }
        }

        // TODO in the presence of schema, error if the variable reference doesn't correspond to a data table

        // assume global
        ValueExpr::VarRef(symprim_to_binding(&varref.name))
    }

    #[inline]
    fn enter_q(&mut self) {
        self.q_stack.push(Default::default());
        self.ctx_stack.push(QueryContext::Query);
    }

    #[inline]
    fn exit_q(&mut self) -> QueryClauses {
        self.ctx_stack.pop();
        self.q_stack.pop().expect("q level")
    }

    #[inline]
    fn current_ctx(&self) -> Option<&QueryContext> {
        self.ctx_stack.last()
    }

    #[inline]
    fn current_ctx_mut(&mut self) -> &mut QueryContext {
        self.ctx_stack.last_mut().unwrap()
    }

    #[inline]
    fn current_clauses_mut(&mut self) -> &mut QueryClauses {
        self.q_stack.last_mut().unwrap()
    }

    #[inline]
    fn enter_benv(&mut self) {
        self.bexpr_stack.push(vec![]);
    }

    #[inline]
    fn exit_benv(&mut self) -> Vec<logical::OpId> {
        self.bexpr_stack.pop().expect("bexpr level")
    }

    #[inline]
    fn push_bexpr(&mut self, bexpr: logical::OpId) {
        self.bexpr_stack.last_mut().unwrap().push(bexpr);
    }

    #[inline]
    fn enter_env(&mut self) {
        self.vexpr_stack.push(vec![]);
    }

    #[inline]
    fn exit_env(&mut self) -> Vec<ValueExpr> {
        self.vexpr_stack.pop().expect("environment level")
    }

    #[inline]
    fn push_vexpr(&mut self, vexpr: ValueExpr) {
        self.vexpr_stack.last_mut().unwrap().push(vexpr);
    }

    #[inline]
    fn push_value(&mut self, val: Value) {
        self.push_vexpr(ValueExpr::Lit(Box::new(val)));
    }

    #[inline]
    fn enter_call(&mut self) {
        self.arg_stack.push(vec![]);
    }

    #[inline]
    fn exit_call(&mut self) -> Vec<CallArgument> {
        self.arg_stack.pop().expect("environment level")
    }

    #[inline]
    fn push_call_arg(&mut self, arg: CallArgument) {
        self.arg_stack.last_mut().unwrap().push(arg);
    }

    #[inline]
    fn enter_path(&mut self) {
        self.path_stack.push(vec![]);
        self.ctx_stack.push(QueryContext::Path);
    }

    #[inline]
    fn exit_path(&mut self) -> Vec<PathComponent> {
        self.ctx_stack.pop();
        self.path_stack.pop().expect("path level")
    }

    #[inline]
    fn push_path_step(&mut self, step: PathComponent) {
        self.path_stack.last_mut().unwrap().push(step);
    }
}

// SQL (and therefore PartiQL) text (and therefore AST) is not lexically-scoped as is the
// case with most programming languages with which we are familiar.
//
// In order to properly process expressions (especially the name references), this visitor essentially
// processes the AST in a kind of post-order traversal, where most node processing is performed after
// that node's children. The `exit_<x>` calls correspond to the post-order processing.
// Often it is necessary to do a little bit of work (preparing data structures into which children
// will collect) in the `enter_<x>` methods. The children of the node are visited in between the
// `enter_<x>` and `exit_<x>` calls.
//
// For 'leaf' nodes of the tree (e.g. variable references, etc.), the node doesn't have any children,
// so there is nothing done between the `enter_<x>` and `exit_<x>` calls.
// By convention, processing for them is done in the `enter_<x>` calls here.
//
impl<'ast> Visitor<'ast> for AstToLogical {
    fn enter_ast_node(&mut self, id: NodeId) {
        self.id_stack.push(id);
    }
    fn exit_ast_node(&mut self, id: NodeId) {
        assert_eq!(self.id_stack.pop(), Some(id))
    }

    fn enter_item(&mut self, _item: &'ast Item) {
        panic!("Only query is currently supported")
    }

    fn enter_ddl(&mut self, _ddl: &'ast Ddl) {
        panic!("Only query is currently supported")
    }

    fn enter_ddl_op(&mut self, _ddl_op: &'ast DdlOp) {
        panic!("Only query is currently supported")
    }

    fn enter_create_table(&mut self, _create_table: &'ast CreateTable) {
        panic!("Only query is currently supported")
    }

    fn enter_drop_table(&mut self, _drop_table: &'ast DropTable) {
        panic!("Only query is currently supported")
    }

    fn enter_create_index(&mut self, _create_index: &'ast CreateIndex) {
        panic!("Only query is currently supported")
    }

    fn enter_drop_index(&mut self, _drop_index: &'ast DropIndex) {
        panic!("Only query is currently supported")
    }

    fn enter_dml(&mut self, _dml: &'ast Dml) {
        panic!("Only query is currently supported")
    }

    fn enter_dml_op(&mut self, _dml_op: &'ast DmlOp) {
        panic!("Only query is currently supported")
    }

    fn enter_insert(&mut self, _insert: &'ast Insert) {
        panic!("Only query is currently supported")
    }

    fn enter_insert_value(&mut self, _insert_value: &'ast InsertValue) {
        panic!("Only query is currently supported")
    }

    fn enter_set(&mut self, _set: &'ast Set) {
        panic!("Only query is currently supported")
    }

    fn enter_assignment(&mut self, _assignment: &'ast Assignment) {
        panic!("Only query is currently supported")
    }

    fn enter_remove(&mut self, _remove: &'ast Remove) {
        panic!("Only query is currently supported")
    }

    fn enter_delete(&mut self, _delete: &'ast Delete) {
        panic!("Only query is currently supported")
    }

    fn enter_on_conflict(&mut self, _on_conflict: &'ast OnConflict) {
        panic!("Only query is currently supported")
    }

    fn enter_query(&mut self, _query: &'ast Query) {
        self.enter_benv();
        self.siblings.push(vec![]);
    }

    fn exit_query(&mut self, _query: &'ast Query) {
        self.siblings.pop();
        let mut benv = self.exit_benv();
        assert_eq!(benv.len(), 1);

        let out = benv.pop().unwrap();

        let sink_id = self.plan.add_operator(BindingsOp::Sink);
        self.plan.add_flow(out, sink_id);
    }

    fn enter_query_set(&mut self, _query_set: &'ast QuerySet) {
        self.enter_env();

        match _query_set {
            QuerySet::SetOp(_) => todo!("QuerySet::SetOp"),
            QuerySet::Select(_) => {}
            QuerySet::Expr(_) => {}
            QuerySet::Values(_) => todo!("QuerySet::Values"),
        }
    }

    fn exit_query_set(&mut self, _query_set: &'ast QuerySet) {
        let env = self.exit_env();

        match _query_set {
            QuerySet::SetOp(_) => todo!("QuerySet::SetOp"),
            QuerySet::Select(_) => {}
            QuerySet::Expr(_) => {
                //
                assert_eq!(env.len(), 1);
                let expr = env.into_iter().next().unwrap();
                let op = BindingsOp::ExprQuery(logical::ExprQuery { expr });
                let id = self.plan.add_operator(op);
                self.push_bexpr(id);
            }
            QuerySet::Values(_) => todo!("QuerySet::Values"),
        }
    }

    fn enter_set_expr(&mut self, _set_expr: &'ast SetExpr) {}

    fn exit_set_expr(&mut self, _set_expr: &'ast SetExpr) {}

    fn enter_select(&mut self, _select: &'ast Select) {
        self.enter_q();
    }

    fn exit_select(&mut self, _select: &'ast Select) {
        let clauses = self.exit_q();

        let mut clauses = clauses.evaluation_order().into_iter();
        let mut src_id = clauses.next().expect("no from clause");
        for dst_id in clauses {
            self.plan.add_flow(src_id, dst_id);
            src_id = dst_id;
        }

        self.push_bexpr(src_id);
    }

    fn enter_projection(&mut self, _projection: &'ast Projection) {
        self.enter_benv();
        self.enter_env();
    }

    fn exit_projection(&mut self, _projection: &'ast Projection) {
        let benv = self.exit_benv();
        assert_eq!(benv.len(), 0);
        let env = self.exit_env();
        assert_eq!(env.len(), 0);

        if let Some(SetQuantifier::Distinct) = _projection.setq {
            let id = self.plan.add_operator(BindingsOp::Distinct);
            self.current_clauses_mut().distinct.replace(id);
        }
    }

    fn enter_projection_kind(&mut self, _projection_kind: &'ast ProjectionKind) {
        self.enter_benv();
        self.enter_env();
    }

    fn exit_projection_kind(&mut self, _projection_kind: &'ast ProjectionKind) {
        let benv = self.exit_benv();
        assert_eq!(benv.len(), 0); // TODO sub-query
        let env = self.exit_env();

        let select: BindingsOp = match _projection_kind {
            ProjectionKind::ProjectStar => logical::BindingsOp::ProjectAll,
            ProjectionKind::ProjectList(_) => {
                let mut exprs = HashMap::with_capacity(env.len() / 2);
                let mut iter = env.into_iter();
                while let Some(value) = iter.next() {
                    let alias = iter.next().unwrap();
                    let alias = match alias {
                        ValueExpr::Lit(lit) => match *lit {
                            Value::String(s) => (*s).clone(),
                            _ => panic!("unexpected literal"),
                        },
                        _ => panic!("unexpected alias type"),
                    };
                    exprs.insert(alias, value);
                }

                logical::BindingsOp::Project(logical::Project { exprs })
            }
            ProjectionKind::ProjectPivot(_) => todo!("ProjectionKind::ProjectPivot"),
            ProjectionKind::ProjectValue(_) => {
                assert_eq!(env.len(), 1);
                let expr = env.into_iter().next().unwrap();
                logical::BindingsOp::ProjectValue(logical::ProjectValue { expr })
            }
        };
        let id = self.plan.add_operator(select);
        self.current_clauses_mut().select_clause.replace(id);
    }

    fn exit_project_expr(&mut self, _project_expr: &'ast ProjectExpr) {
        let _expr = self.vexpr_stack.last().unwrap().last().unwrap();
        let as_key: &name_resolver::Symbol = self
            .key_registry
            .aliases
            .get(self.current_node())
            .expect("alias");
        // TODO intern strings
        let as_key = match as_key {
            name_resolver::Symbol::Known(sym) => sym.value.clone(),
            name_resolver::Symbol::Unknown(id) => format!("_{}", id),
        };
        self.push_value(as_key.into());
    }

    fn enter_bin_op(&mut self, _bin_op: &'ast BinOp) {
        self.enter_env();
    }

    fn exit_bin_op(&mut self, _bin_op: &'ast BinOp) {
        let mut env = self.exit_env();
        assert_eq!(env.len(), 2);

        let rhs = env.pop().unwrap();
        let lhs = env.pop().unwrap();
        if _bin_op.kind == BinOpKind::Is {
            let is_type = match rhs {
                ValueExpr::Lit(lit) => match lit.as_ref() {
                    Value::Null => logical::Type::NullType,
                    Value::Missing => logical::Type::MissingType,
                    _ => todo!("unsupported rhs literal for `IS`"),
                },
                _ => todo!("unsupported rhs for `IS`"),
            };
            self.push_vexpr(ValueExpr::IsTypeExpr(IsTypeExpr {
                not: false,
                expr: Box::new(lhs),
                is_type,
            }));
        } else {
            let op = match _bin_op.kind {
                BinOpKind::Add => logical::BinaryOp::Add,
                BinOpKind::Div => logical::BinaryOp::Div,
                BinOpKind::Exp => logical::BinaryOp::Exp,
                BinOpKind::Mod => logical::BinaryOp::Mod,
                BinOpKind::Mul => logical::BinaryOp::Mul,
                BinOpKind::Sub => logical::BinaryOp::Sub,
                BinOpKind::And => logical::BinaryOp::And,
                BinOpKind::Or => logical::BinaryOp::Or,
                BinOpKind::Concat => logical::BinaryOp::Concat,
                BinOpKind::Eq => logical::BinaryOp::Eq,
                BinOpKind::Gt => logical::BinaryOp::Gt,
                BinOpKind::Gte => logical::BinaryOp::Gteq,
                BinOpKind::Lt => logical::BinaryOp::Lt,
                BinOpKind::Lte => logical::BinaryOp::Lteq,
                BinOpKind::Ne => logical::BinaryOp::Neq,
                BinOpKind::Is => unreachable!(),
            };
            self.push_vexpr(ValueExpr::BinaryExpr(op, Box::new(lhs), Box::new(rhs)));
        }
    }

    fn enter_uni_op(&mut self, _uni_op: &'ast UniOp) {
        self.enter_env();
    }

    fn exit_uni_op(&mut self, _uni_op: &'ast UniOp) {
        let mut env = self.exit_env();
        assert_eq!(env.len(), 1);

        let expr = env.pop().unwrap();
        let op = match _uni_op.kind {
            UniOpKind::Pos => logical::UnaryOp::Pos,
            UniOpKind::Neg => logical::UnaryOp::Neg,
            UniOpKind::Not => logical::UnaryOp::Not,
        };
        self.push_vexpr(ValueExpr::UnExpr(op, Box::new(expr)));
    }

    fn enter_between(&mut self, _between: &'ast Between) {
        self.enter_env();
    }

    fn exit_between(&mut self, _between: &'ast Between) {
        let mut env = self.exit_env();
        assert_eq!(env.len(), 3);
        let to = Box::new(env.pop().unwrap());
        let from = Box::new(env.pop().unwrap());
        let value = Box::new(env.pop().unwrap());
        self.push_vexpr(ValueExpr::BetweenExpr(BetweenExpr { value, from, to }));
    }

    fn enter_like(&mut self, _like: &'ast Like) {
        self.enter_env();
    }

    fn exit_like(&mut self, _like: &'ast Like) {
        let mut env = self.exit_env();
        assert!((2..=3).contains(&env.len()));
        let escape = if env.len() == 3 {
            require_str(require_lit(&env.pop().unwrap())).to_string()
        } else {
            "".to_string()
        };
        let pattern = require_str(require_lit(&env.pop().unwrap())).to_string();
        let value = Box::new(env.pop().unwrap());
        let pattern = Pattern::LIKE(LikeMatch { pattern, escape });
        self.push_vexpr(ValueExpr::PatternMatchExpr(PatternMatchExpr {
            value,
            pattern,
        }));
    }

    fn enter_call(&mut self, _call: &'ast Call) {
        self.enter_call();
    }

    fn exit_call(&mut self, _call: &'ast Call) {
        // TODO better argument validation/error messaging
        let env = self.exit_call();
        let name = _call.func_name.value.to_lowercase();

        if let Some(call_def) = self.fnsym_tab.lookup(name.as_str()) {
            self.push_vexpr(call_def.lookup(&env));
        } else {
            todo!("Unsupported function name")
        }
    }

    fn enter_call_arg(&mut self, _call_arg: &'ast CallArg) {
        self.enter_env();
    }

    fn exit_call_arg(&mut self, _call_arg: &'ast CallArg) {
        let mut env = self.exit_env();
        match _call_arg {
            CallArg::Star() => todo!(),
            CallArg::Positional(_) => {
                assert_eq!(env.len(), 1);
                self.push_call_arg(CallArgument::Positional(env.pop().unwrap()));
            }
            CallArg::Named(CallArgNamed { name, .. }) => {
                assert_eq!(env.len(), 1);
                let name = name.value.to_lowercase();
                self.push_call_arg(CallArgument::Named(name, env.pop().unwrap()));
            }
            CallArg::PositionalType(_) => todo!("CallArg::PositionalType"),
            CallArg::NamedType(_) => todo!("CallArg::NamedType"),
        }
    }

    // Values & Value Constructors

    fn enter_lit(&mut self, _lit: &'ast Lit) {
        let val = match _lit {
            Lit::Null => Value::Null,
            Lit::Missing => Value::Missing,
            Lit::Int8Lit(n) => Value::Integer(*n as i64),
            Lit::Int16Lit(n) => Value::Integer(*n as i64),
            Lit::Int32Lit(n) => Value::Integer(*n as i64),
            Lit::Int64Lit(n) => Value::Integer(*n),
            Lit::DecimalLit(d) => Value::Decimal(*d),
            Lit::NumericLit(n) => Value::Decimal(*n),
            Lit::RealLit(f) => Value::Real(OrderedFloat::from(*f as f64)),
            Lit::FloatLit(f) => Value::Real(OrderedFloat::from(*f as f64)),
            Lit::DoubleLit(f) => Value::Real(OrderedFloat::from(*f)),
            Lit::BoolLit(b) => Value::Boolean(*b),
            Lit::IonStringLit(s) => Value::from_ion(s),
            Lit::CharStringLit(s) => Value::String(Box::new(s.clone())),
            Lit::NationalCharStringLit(s) => Value::String(Box::new(s.clone())),
            Lit::BitStringLit(_) => todo!("BitStringLit"),
            Lit::HexStringLit(_) => todo!("HexStringLit"),
            Lit::CollectionLit(_) => todo!("CollectionLit"),
            Lit::TypedLit(_, _) => todo!("TypedLit"),
        };
        self.push_value(val);
    }

    fn enter_struct(&mut self, _struct: &'ast Struct) {
        self.enter_env()
    }

    fn exit_struct(&mut self, _struct: &'ast Struct) {
        let env = self.exit_env();
        assert!(env.len().is_even());

        let len = env.len() / 2;
        let mut attrs = Vec::with_capacity(len);
        let mut values = Vec::with_capacity(len);

        let mut iter = env.into_iter();
        while let Some(attr) = iter.next() {
            let value = iter.next().unwrap();
            attrs.push(attr);
            values.push(value);
        }

        self.push_vexpr(ValueExpr::TupleExpr(TupleExpr { attrs, values }));
    }

    fn enter_bag(&mut self, _bag: &'ast Bag) {
        self.enter_env()
    }

    fn exit_bag(&mut self, _bag: &'ast Bag) {
        let elements = self.exit_env();
        self.push_vexpr(ValueExpr::BagExpr(BagExpr { elements }));
    }

    fn enter_list(&mut self, _list: &'ast List) {
        self.enter_env()
    }

    fn exit_list(&mut self, _list: &'ast List) {
        let elements = self.exit_env();
        self.push_vexpr(ValueExpr::ListExpr(ListExpr { elements }));
    }

    fn enter_sexp(&mut self, _sexp: &'ast Sexp) {
        self.enter_env()
    }

    fn exit_sexp(&mut self, _sexp: &'ast Sexp) {
        todo!("exit_sexp")
    }

    fn enter_call_agg(&mut self, _call_agg: &'ast CallAgg) {
        todo!("call_agg")
    }

    fn exit_call_agg(&mut self, _call_agg: &'ast CallAgg) {}

    fn enter_var_ref(&mut self, _var_ref: &'ast VarRef) {
        let is_from_path = matches!(self.current_ctx(), Some(QueryContext::FromLet));
        let is_path = matches!(self.current_ctx(), Some(QueryContext::Path));
        let should_resolve = !is_from_path && !is_path;

        if should_resolve {
            let options = self.resolve_varref(_var_ref);
            self.push_vexpr(options);
        } else {
            // TODO scope qualifier
            let VarRef {
                name: SymbolPrimitive { value, case },
                qualifier: _,
            } = _var_ref;
            let name = match case {
                CaseSensitivity::CaseSensitive => BindingsName::CaseSensitive(value.clone()),
                CaseSensitivity::CaseInsensitive => BindingsName::CaseInsensitive(value.clone()),
            };
            self.push_vexpr(ValueExpr::VarRef(name));
        }
    }

    fn exit_var_ref(&mut self, _var_ref: &'ast VarRef) {}

    fn enter_path(&mut self, _path: &'ast Path) {
        self.enter_env();
        self.enter_path();
    }

    fn exit_path(&mut self, _path: &'ast Path) {
        let mut env = self.exit_env();
        assert_eq!(env.len(), 1);

        let steps = self.exit_path();
        let root = env.pop().unwrap();

        self.push_vexpr(ValueExpr::Path(Box::new(root), steps));
    }

    fn enter_path_step(&mut self, _path_step: &'ast PathStep) {
        if let PathStep::PathExpr(_) = _path_step {
            self.enter_env();
        }
    }

    fn exit_path_step(&mut self, _path_step: &'ast PathStep) {
        let step = match _path_step {
            PathStep::PathExpr(_s) => {
                let mut env = self.exit_env();
                assert_eq!(env.len(), 1);

                let path = env.pop().unwrap();
                match path {
                    ValueExpr::Lit(val) => match *val {
                        Value::Integer(idx) => logical::PathComponent::Index(idx),
                        Value::String(k) => {
                            logical::PathComponent::Key(BindingsName::CaseInsensitive(*k))
                        }
                        expr => logical::PathComponent::IndexExpr(Box::new(ValueExpr::Lit(
                            Box::new(expr),
                        ))),
                    },
                    ValueExpr::VarRef(name) => logical::PathComponent::Key(name),
                    expr => {
                        // TODO if type is statically STRING, then use KeyExpr
                        logical::PathComponent::IndexExpr(Box::new(expr))
                    }
                }
            }
            PathStep::PathWildCard => todo!("PathWildCard"),
            PathStep::PathUnpivot => todo!("PathUnpivot"),
        };

        self.push_path_step(step);
    }

    fn enter_from_clause(&mut self, _from_clause: &'ast FromClause) {
        self.enter_benv();
        self.enter_env();
    }

    fn exit_from_clause(&mut self, _from_clause: &'ast FromClause) {
        let mut benv = self.exit_benv();
        assert_eq!(benv.len(), 1);
        let env = self.exit_env();
        assert_eq!(env.len(), 0);

        self.current_clauses_mut()
            .from_clause
            .replace(benv.pop().unwrap());
    }

    fn enter_from_let(&mut self, from_let: &'ast FromLet) {
        self.from_lets.insert(*self.current_node());
        *self.current_ctx_mut() = QueryContext::FromLet;
        self.enter_env();

        let id = *self.current_node();
        self.siblings.last_mut().unwrap().push(id);

        for sym in [&from_let.as_alias, &from_let.at_alias, &from_let.by_alias]
            .into_iter()
            .flatten()
        {
            self.aliases.insert(id, sym.clone());
        }
    }

    fn exit_from_let(&mut self, from_let: &'ast FromLet) {
        *self.current_ctx_mut() = QueryContext::Query;
        let mut env = self.exit_env();
        assert_eq!(env.len(), 1);

        let expr = env.pop().unwrap();

        let FromLet {
            kind,
            as_alias,
            at_alias,
            ..
        } = from_let;
        let as_key = self.infer_id(&expr, as_alias).value;
        let at_key = at_alias
            .as_ref()
            .map(|SymbolPrimitive { value, case: _ }| value.clone());

        let bexpr = match kind {
            FromLetKind::Scan => logical::BindingsOp::Scan(logical::Scan {
                expr,
                as_key,
                at_key,
            }),
            FromLetKind::Unpivot => logical::BindingsOp::Unpivot(logical::Unpivot {
                expr,
                as_key,
                at_key,
            }),
        };
        let id = self.plan.add_operator(bexpr);
        self.push_bexpr(id);
    }

    fn enter_join(&mut self, _join: &'ast Join) {
        self.enter_benv();
        self.enter_env();
    }

    fn exit_join(&mut self, join: &'ast Join) {
        let mut benv = self.exit_benv();
        assert_eq!(benv.len(), 2);

        let mut env = self.exit_env();
        assert!((0..1).contains(&env.len()));

        let Join { kind, .. } = join;

        let kind = match kind {
            JoinKind::Inner => logical::JoinKind::Inner,
            JoinKind::Left => logical::JoinKind::Left,
            JoinKind::Right => logical::JoinKind::Right,
            JoinKind::Full => logical::JoinKind::Full,
            JoinKind::Cross => logical::JoinKind::Cross,
        };

        let on = env.pop();

        let rid = benv.pop().unwrap();
        let lid = benv.pop().unwrap();
        let left = Box::new(self.plan.operator(lid).unwrap().clone());
        let right = Box::new(self.plan.operator(rid).unwrap().clone());
        let join = logical::BindingsOp::Join(logical::Join {
            kind,
            on,
            left,
            right,
        });
        let join = self.plan.add_operator(join);
        self.push_bexpr(join);
    }

    fn enter_join_spec(&mut self, join_spec: &'ast JoinSpec) {
        match join_spec {
            JoinSpec::On(_) => {
                // visitor recurse into expr will put the condition in the current env
            }
            JoinSpec::Using(_) => {
                todo!("JoinSpec::Using")
            }
            JoinSpec::Natural => {
                todo!("JoinSpec::Natural")
            }
        };
    }

    fn enter_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) {
        self.enter_env();
    }

    fn exit_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) {
        let mut env = self.exit_env();
        assert_eq!(env.len(), 1);

        let filter = logical::BindingsOp::Filter(logical::Filter {
            expr: env.pop().unwrap(),
        });
        let id = self.plan.add_operator(filter);

        self.current_clauses_mut().where_clause.replace(id);
    }

    fn enter_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) {
        self.enter_env();
    }

    fn exit_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) {
        let _env = self.exit_env();
        todo!("having clause");
    }

    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast GroupByExpr) {
        self.enter_env();
    }

    fn exit_group_by_expr(&mut self, _group_by_expr: &'ast GroupByExpr) {
        let _env = self.exit_env();
        todo!("group by clause");
    }

    fn enter_order_by_expr(&mut self, _order_by_expr: &'ast OrderByExpr) {
        self.enter_env();
    }

    fn exit_order_by_expr(&mut self, _order_by_expr: &'ast OrderByExpr) {
        let _env = self.exit_env();
        todo!("order by clause");
    }

    fn enter_simple_case(&mut self, _simple_case: &'ast SimpleCase) {
        self.enter_env();
    }

    fn exit_simple_case(&mut self, _simple_case: &'ast SimpleCase) {
        let mut env = self.exit_env();
        assert!(env.len() >= 2);

        let default = if env.len().is_even() {
            Some(Box::new(env.pop().unwrap()))
        } else {
            None
        };

        let mut params = env.into_iter();
        let expr = Box::new(params.next().unwrap());

        let cases = params
            .chunks(2)
            .into_iter()
            .map(|mut c| {
                let when = c.next().unwrap();
                let then = c.next().unwrap();

                (Box::new(when), Box::new(then))
            })
            .collect_vec();

        self.push_vexpr(ValueExpr::SimpleCase(logical::SimpleCase {
            expr,
            cases,
            default,
        }))
    }

    fn enter_searched_case(&mut self, _searched_case: &'ast SearchedCase) {
        self.enter_env();
    }

    fn exit_searched_case(&mut self, _searched_case: &'ast SearchedCase) {
        let mut env = self.exit_env();
        assert!(!env.is_empty());

        let default = if env.len().is_odd() {
            Some(Box::new(env.pop().unwrap()))
        } else {
            None
        };

        let cases = env
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(|mut c| {
                let when = c.next().unwrap();
                let then = c.next().unwrap();

                (Box::new(when), Box::new(then))
            })
            .collect_vec();
        self.push_vexpr(ValueExpr::SearchedCase(logical::SearchedCase {
            cases,
            default,
        }))
    }
}
