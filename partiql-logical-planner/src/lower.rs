use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use num::Integer;
use ordered_float::OrderedFloat;
use partiql_ast::ast;
use partiql_ast::ast::{
    Assignment, Bag, BagOpExpr, BagOperator, Between, BinOp, BinOpKind, Call, CallAgg, CallArg,
    CallArgNamed, CaseSensitivity, CreateIndex, CreateTable, Ddl, DdlOp, Delete, Dml, DmlOp,
    DropIndex, DropTable, Expr, FromClause, FromLet, FromLetKind, GroupByExpr, GroupKey,
    GroupingStrategy, Insert, InsertValue, Item, Join, JoinKind, JoinSpec, Like, List, Lit, NodeId,
    NullOrderingSpec, OnConflict, OrderByExpr, OrderingSpec, Path, PathStep, ProjectExpr,
    Projection, ProjectionKind, Query, QuerySet, Remove, SearchedCase, Select, Set, SetQuantifier,
    Sexp, SimpleCase, SortSpec, Struct, SymbolPrimitive, UniOp, UniOpKind, VarRef,
};
use partiql_ast::visit::{Traverse, Visit, Visitor};
use partiql_logical as logical;
use partiql_logical::{
    AggregateExpression, BagExpr, BagOp, BetweenExpr, BindingsOp, IsTypeExpr, LikeMatch,
    LikeNonStringNonLiteralMatch, ListExpr, LogicalPlan, OpId, PathComponent, Pattern,
    PatternMatchExpr, SortSpecOrder, TupleExpr, ValueExpr, VarRefType,
};
use std::borrow::Cow;

use partiql_value::{BindingsName, Value};

use std::collections::{HashMap, HashSet};

use crate::builtins::{FnSymTab, FN_SYM_TAB};
use itertools::Itertools;
use partiql_ast_passes::name_resolver;
use partiql_catalog::call_defs::{CallArgument, CallDef};

use partiql_ast_passes::error::{AstTransformError, AstTransformationError};

use partiql_ast_passes::name_resolver::NameRef;
use partiql_catalog::Catalog;
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical::AggFunc::{AggAny, AggAvg, AggCount, AggEvery, AggMax, AggMin, AggSum};
use partiql_logical::ValueExpr::DynamicLookup;
use std::sync::atomic::{AtomicU32, Ordering};

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

#[macro_export]
macro_rules! eq_or_fault {
    ($self:ident, $lhs:expr, $rhs:expr, $msg:expr) => {
        if $lhs != $rhs {
            $self
                .errors
                .push(AstTransformError::IllegalState($msg.to_string()));
            return partiql_ast::visit::Traverse::Stop;
        }
    };
}

#[macro_export]
macro_rules! true_or_fault {
    ($self:ident, $expr:expr, $msg:expr) => {
        if !$expr {
            $self
                .errors
                .push(AstTransformError::IllegalState($msg.to_string()));
            return partiql_ast::visit::Traverse::Stop;
        }
    };
}

#[macro_export]
macro_rules! not_yet_implemented_fault {
    ($self:ident, $msg:expr) => {
        not_yet_implemented_err!($self, $msg);
        return partiql_ast::visit::Traverse::Stop;
    };
}

#[macro_export]
macro_rules! not_yet_implemented_err {
    ($self:ident, $msg:expr) => {
        $self
            .errors
            .push(AstTransformError::NotYetImplemented($msg.to_string()));
    };
}

#[derive(Copy, Clone, Debug)]
enum QueryContext {
    FromLet,
    Path,
    Order,
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
    limit_offset_clause: Option<logical::OpId>,
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
            self.limit_offset_clause,
            self.select_clause,
            self.distinct,
        ]
        .iter()
        .copied()
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
pub struct AstToLogical<'a> {
    // current stack of node ids
    id_stack: Vec<NodeId>,

    q_stack: Vec<QueryClauses>,
    ctx_stack: Vec<QueryContext>,
    bexpr_stack: Vec<Vec<logical::OpId>>,
    vexpr_stack: Vec<Vec<ValueExpr>>,
    arg_stack: Vec<Vec<CallArgument>>,
    path_stack: Vec<Vec<PathComponent>>,
    sort_stack: Vec<Vec<logical::SortSpec>>,
    aggregate_exprs: Vec<AggregateExpression>,

    from_lets: HashSet<ast::NodeId>,

    projection_renames: Vec<FnvIndexMap<String, BindingsName<'a>>>,

    aliases: FnvIndexMap<NodeId, SymbolPrimitive>,

    // generator of 'fresh' ids
    id: IdGenerator,
    agg_id: IdGenerator,

    // output
    plan: LogicalPlan<BindingsOp>,

    // catalog & data flow data
    key_registry: name_resolver::KeyRegistry,
    fnsym_tab: &'static FnSymTab,
    catalog: &'a dyn Catalog,

    // list of errors encountered during AST lowering
    errors: Vec<AstTransformError>,
}

/// Attempt to infer an alias for a simple variable reference expression.
/// For example infer such that  `SELECT a, b.c.d.e ...` <=> `SELECT a as a, b.c.d.e as e`  
fn infer_id(expr: &ValueExpr) -> Option<SymbolPrimitive> {
    let sensitive = |value: &str| {
        Some(SymbolPrimitive {
            value: value.to_string(),
            case: CaseSensitivity::CaseSensitive,
        })
    };
    let insensitive = |value: &str| {
        Some(SymbolPrimitive {
            value: value.to_string(),
            case: CaseSensitivity::CaseInsensitive,
        })
    };

    match expr {
        ValueExpr::VarRef(BindingsName::CaseInsensitive(s), _) => insensitive(s.as_ref()),
        ValueExpr::VarRef(BindingsName::CaseSensitive(s), _) => sensitive(s.as_ref()),
        ValueExpr::Path(_root, steps) => match steps.last() {
            Some(PathComponent::Key(BindingsName::CaseInsensitive(s))) => insensitive(s.as_ref()),
            Some(PathComponent::Key(BindingsName::CaseSensitive(s))) => sensitive(s.as_ref()),
            Some(PathComponent::KeyExpr(ke)) => match &**ke {
                ValueExpr::VarRef(BindingsName::CaseInsensitive(s), _) => insensitive(s.as_ref()),
                ValueExpr::VarRef(BindingsName::CaseSensitive(s), _) => sensitive(s.as_ref()),
                _ => None,
            },
            _ => None,
        },
        ValueExpr::DynamicLookup(d) => infer_id(d.first().unwrap()),
        _ => None,
    }
}

impl<'a> AstToLogical<'a> {
    pub fn new(catalog: &'a dyn Catalog, registry: name_resolver::KeyRegistry) -> Self {
        let fnsym_tab: &FnSymTab = &FN_SYM_TAB;
        AstToLogical {
            id_stack: Default::default(),

            q_stack: Default::default(),
            ctx_stack: Default::default(),
            bexpr_stack: Default::default(),
            vexpr_stack: Default::default(),
            arg_stack: Default::default(),
            path_stack: Default::default(),
            sort_stack: Default::default(),
            aggregate_exprs: Default::default(),

            from_lets: Default::default(),

            projection_renames: Default::default(),

            aliases: Default::default(),

            // generator of 'fresh' ids
            id: Default::default(),
            agg_id: Default::default(),

            // output
            plan: Default::default(),

            key_registry: registry,
            fnsym_tab,
            catalog,

            errors: vec![],
        }
    }

    pub fn lower_query(
        mut self,
        query: &ast::AstNode<ast::TopLevelQuery>,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        query.visit(&mut self);
        if !self.errors.is_empty() {
            return Err(AstTransformationError {
                errors: self.errors,
            });
        }
        Ok(self.plan)
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
        fn binding_to_static<'a>(binding: &'a BindingsName<'a>) -> BindingsName<'static> {
            match binding {
                BindingsName::CaseSensitive(n) => {
                    BindingsName::CaseSensitive(Cow::Owned(n.as_ref().to_string()))
                }
                BindingsName::CaseInsensitive(n) => {
                    BindingsName::CaseInsensitive(Cow::Owned(n.as_ref().to_string()))
                }
            }
        }

        // Convert a `SymbolPrimitive` into a `BindingsName`
        fn symprim_to_binding(sym: &SymbolPrimitive) -> BindingsName<'static> {
            match sym.case {
                CaseSensitivity::CaseSensitive => {
                    BindingsName::CaseSensitive(Cow::Owned(sym.value.clone()))
                }
                CaseSensitivity::CaseInsensitive => {
                    BindingsName::CaseInsensitive(Cow::Owned(sym.value.clone()))
                }
            }
        }
        // Convert a `name_resolver::Symbol` into a `BindingsName`
        fn sym_to_binding(sym: &name_resolver::Symbol) -> Option<BindingsName<'static>> {
            match sym {
                name_resolver::Symbol::Known(sym) => Some(symprim_to_binding(sym)),
                name_resolver::Symbol::Unknown(_) => None,
            }
        }

        for id in self.id_stack.iter().rev() {
            if let Some(key_schema) = self.key_registry.schema.get(id) {
                let key_schema: &name_resolver::KeySchema = key_schema;

                let name_ref: &NameRef = key_schema
                    .consume
                    .iter()
                    .find(|name_ref| name_ref.sym == varref.name)
                    .expect("NameRef");

                let var_binding = symprim_to_binding(&name_ref.sym);
                let mut lookups = vec![];

                if matches!(self.current_ctx(), Some(QueryContext::Order)) {
                    if let Some(renames) = self.projection_renames.last() {
                        let binding = renames
                            .iter()
                            .find(|(k, _)| {
                                let SymbolPrimitive { value, case } = &name_ref.sym;
                                match case {
                                    CaseSensitivity::CaseSensitive => value == *k,
                                    CaseSensitivity::CaseInsensitive => unicase::eq(value, *k),
                                }
                            })
                            .map_or_else(
                                || symprim_to_binding(&name_ref.sym),
                                |(_k, v)| binding_to_static(v),
                            );

                        lookups.push(DynamicLookup(Box::new(vec![ValueExpr::VarRef(
                            binding,
                            VarRefType::Local,
                        )])));
                    }
                }

                for lookup in &name_ref.lookup {
                    match lookup {
                        name_resolver::NameLookup::Global => {
                            let var_ref_expr =
                                ValueExpr::VarRef(var_binding.clone(), VarRefType::Global);
                            if !lookups.contains(&var_ref_expr) {
                                lookups.push(var_ref_expr.clone());
                            }
                        }
                        name_resolver::NameLookup::Local => {
                            if let Some(scope_ids) = self.key_registry.in_scope.get(id) {
                                let scopes: Vec<&name_resolver::KeySchema> = scope_ids
                                    .iter()
                                    .filter_map(|scope_id| self.key_registry.schema.get(scope_id))
                                    .collect();

                                let mut exact = scopes.iter().filter(|key_schema| {
                                    key_schema.produce.contains(&name_resolver::Symbol::Known(
                                        name_ref.sym.clone(),
                                    ))
                                });

                                if let Some(_matching) = exact.next() {
                                    let var_ref_expr =
                                        ValueExpr::VarRef(var_binding.clone(), VarRefType::Local);
                                    lookups.push(var_ref_expr);
                                    continue;
                                }

                                for schema in scopes {
                                    for produce in &schema.produce {
                                        if let name_resolver::Symbol::Known(sym) = produce {
                                            if (sym == &varref.name)
                                                || (sym.value.to_lowercase()
                                                    == varref.name.value.to_lowercase()
                                                    && varref.name.case
                                                        == ast::CaseSensitivity::CaseInsensitive)
                                            {
                                                let expr = ValueExpr::VarRef(
                                                    sym_to_binding(produce).unwrap_or_else(|| {
                                                        symprim_to_binding(&self.gen_id())
                                                    }),
                                                    VarRefType::Local,
                                                );
                                                if !lookups.contains(&expr) {
                                                    lookups.push(expr);
                                                }

                                                continue;
                                            } else if let Some(_type_entry) = self
                                                .catalog
                                                .resolve_type(name_ref.sym.value.as_ref())
                                            {
                                                let expr = ValueExpr::VarRef(
                                                    var_binding.clone(),
                                                    VarRefType::Global,
                                                );
                                                if !lookups.contains(&expr) {
                                                    lookups.push(expr);
                                                }
                                                continue;
                                            } else {
                                                let path = logical::ValueExpr::Path(
                                                    Box::new(ValueExpr::VarRef(
                                                        sym_to_binding(produce).unwrap_or_else(
                                                            || symprim_to_binding(&self.gen_id()),
                                                        ),
                                                        VarRefType::Local,
                                                    )),
                                                    vec![PathComponent::Key(var_binding.clone())],
                                                );

                                                if !lookups.contains(&path) {
                                                    lookups.push(path);
                                                }
                                            }
                                        } else if let name_resolver::Symbol::Unknown(num) = produce
                                        {
                                            let formatted_num = format!("_{num}");
                                            if formatted_num == varref.name.value {
                                                let expr = ValueExpr::VarRef(
                                                    BindingsName::CaseInsensitive(Cow::Owned(
                                                        formatted_num,
                                                    )),
                                                    VarRefType::Local,
                                                );
                                                if !lookups.contains(&expr) {
                                                    lookups.push(expr);
                                                    continue;
                                                }
                                            } else {
                                                let path = logical::ValueExpr::Path(
                                                    Box::new(ValueExpr::VarRef(
                                                        sym_to_binding(produce).unwrap_or({
                                                            BindingsName::CaseInsensitive(
                                                                Cow::Owned(formatted_num),
                                                            )
                                                        }),
                                                        VarRefType::Local,
                                                    )),
                                                    vec![PathComponent::Key(var_binding.clone())],
                                                );

                                                if !lookups.contains(&path) {
                                                    lookups.push(path);
                                                }
                                            }
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
        ValueExpr::VarRef(symprim_to_binding(&varref.name), VarRefType::Global)
    }

    #[inline]
    fn enter_q(&mut self) {
        self.q_stack.push(Default::default());
        self.ctx_stack.push(QueryContext::Query);
        self.projection_renames.push(Default::default());
    }

    #[inline]
    fn exit_q(&mut self) -> QueryClauses {
        self.projection_renames.pop().expect("q level");
        self.ctx_stack.pop().expect("q level");
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
        self.ctx_stack.push(QueryContext::Query);
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

    #[inline]
    fn enter_sort(&mut self) {
        self.sort_stack.push(vec![]);
        self.ctx_stack.push(QueryContext::Order);
    }

    #[inline]
    fn exit_sort(&mut self) -> Vec<logical::SortSpec> {
        self.ctx_stack.pop();
        self.sort_stack.pop().expect("sort specs")
    }

    #[inline]
    fn push_sort_spec(&mut self, spec: logical::SortSpec) {
        self.sort_stack.last_mut().unwrap().push(spec);
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
impl<'a, 'ast> Visitor<'ast> for AstToLogical<'a> {
    fn enter_ast_node(&mut self, id: NodeId) -> Traverse {
        self.id_stack.push(id);
        Traverse::Continue
    }
    fn exit_ast_node(&mut self, id: NodeId) -> Traverse {
        let cur_node = self.id_stack.pop();
        eq_or_fault!(self, cur_node, Some(id), "id_stack node id != id");
        Traverse::Continue
    }

    fn enter_item(&mut self, _item: &'ast Item) -> Traverse {
        not_yet_implemented_fault!(self, "Item");
    }

    fn enter_ddl(&mut self, _ddl: &'ast Ddl) -> Traverse {
        not_yet_implemented_fault!(self, "Ddl".to_string());
    }

    fn enter_ddl_op(&mut self, _ddl_op: &'ast DdlOp) -> Traverse {
        not_yet_implemented_fault!(self, "DdlOp".to_string());
    }

    fn enter_create_table(&mut self, _create_table: &'ast CreateTable) -> Traverse {
        not_yet_implemented_fault!(self, "CreateTable".to_string());
    }

    fn enter_drop_table(&mut self, _drop_table: &'ast DropTable) -> Traverse {
        not_yet_implemented_fault!(self, "DropTable".to_string());
    }

    fn enter_create_index(&mut self, _create_index: &'ast CreateIndex) -> Traverse {
        not_yet_implemented_fault!(self, "CreateIndex".to_string());
    }

    fn enter_drop_index(&mut self, _drop_index: &'ast DropIndex) -> Traverse {
        not_yet_implemented_fault!(self, "DropIndex".to_string());
    }

    fn enter_dml(&mut self, _dml: &'ast Dml) -> Traverse {
        not_yet_implemented_fault!(self, "Dml".to_string());
    }

    fn enter_dml_op(&mut self, _dml_op: &'ast DmlOp) -> Traverse {
        not_yet_implemented_fault!(self, "DmlOp".to_string());
    }

    fn enter_insert(&mut self, _insert: &'ast Insert) -> Traverse {
        not_yet_implemented_fault!(self, "Insert".to_string());
    }

    fn enter_insert_value(&mut self, _insert_value: &'ast InsertValue) -> Traverse {
        not_yet_implemented_fault!(self, "InsertValue".to_string());
    }

    fn enter_set(&mut self, _set: &'ast Set) -> Traverse {
        not_yet_implemented_fault!(self, "Set".to_string());
    }

    fn enter_assignment(&mut self, _assignment: &'ast Assignment) -> Traverse {
        not_yet_implemented_fault!(self, "Assignment".to_string());
    }

    fn enter_remove(&mut self, _remove: &'ast Remove) -> Traverse {
        not_yet_implemented_fault!(self, "Remove".to_string());
    }

    fn enter_delete(&mut self, _delete: &'ast Delete) -> Traverse {
        not_yet_implemented_fault!(self, "Delete".to_string());
    }

    fn enter_on_conflict(&mut self, _on_conflict: &'ast OnConflict) -> Traverse {
        not_yet_implemented_fault!(self, "OnConflict".to_string());
    }

    fn enter_top_level_query(&mut self, _query: &'ast ast::TopLevelQuery) -> Traverse {
        self.enter_benv();
        Traverse::Continue
    }
    fn exit_top_level_query(&mut self, _query: &'ast ast::TopLevelQuery) -> Traverse {
        let mut benv = self.exit_benv();
        eq_or_fault!(self, benv.len(), 1, "Expect benv.len() == 1");
        let out = benv.pop().unwrap();
        let sink_id = self.plan.add_operator(BindingsOp::Sink);
        self.plan.add_flow(out, sink_id);
        Traverse::Continue
    }

    fn enter_query(&mut self, query: &'ast Query) -> Traverse {
        self.enter_benv();
        if let QuerySet::Select(_) = query.set.node {
            self.enter_q();
        }
        Traverse::Continue
    }

    fn exit_query(&mut self, query: &'ast Query) -> Traverse {
        let benv = self.exit_benv();
        match query.set.node {
            QuerySet::Select(_) => {
                let clauses = self.exit_q();
                let mut clauses = clauses.evaluation_order().into_iter();
                if let Some(mut src_id) = clauses.next() {
                    for dst_id in clauses {
                        self.plan.add_flow(src_id, dst_id);
                        src_id = dst_id;
                    }
                    self.push_bexpr(src_id);
                }
            }
            _ => {
                true_or_fault!(
                    self,
                    (1..=3).contains(&benv.len()),
                    "benv.len() is not between 1 and 3"
                );
                let mut out = *benv.first().unwrap();
                benv.into_iter().skip(1).for_each(|op| {
                    self.plan.add_flow(out, op);
                    out = op;
                });
                self.push_bexpr(out);
            }
        }
        Traverse::Continue
    }

    fn enter_query_set(&mut self, _query_set: &'ast QuerySet) -> Traverse {
        self.enter_env();
        self.enter_benv();

        match _query_set {
            QuerySet::BagOp(_) => {}
            QuerySet::Select(_) => {}
            QuerySet::Expr(_) => {}
            QuerySet::Values(_) => {
                not_yet_implemented_fault!(self, "QuerySet::Values".to_string());
            }
            QuerySet::Table(_) => {
                not_yet_implemented_fault!(self, "QuerySet::Table".to_string());
            }
        }
        Traverse::Continue
    }

    fn exit_query_set(&mut self, query_set: &'ast QuerySet) -> Traverse {
        let env = self.exit_env();
        let mut benv = self.exit_benv();

        match query_set {
            QuerySet::BagOp(bag_op) => {
                eq_or_fault!(self, benv.len(), 2, "benv.len() != 2");
                let rid = benv.pop().unwrap();
                let lid = benv.pop().unwrap();

                let bag_operator = match bag_op.node.bag_op {
                    BagOperator::Union => logical::BagOperator::Union,
                    BagOperator::Except => logical::BagOperator::Except,
                    BagOperator::Intersect => logical::BagOperator::Intersect,
                    BagOperator::OuterUnion => logical::BagOperator::OuterUnion,
                    BagOperator::OuterExcept => logical::BagOperator::OuterExcept,
                    BagOperator::OuterIntersect => logical::BagOperator::OuterIntersect,
                };
                let setq = match bag_op.node.setq {
                    SetQuantifier::All => logical::SetQuantifier::All,
                    SetQuantifier::Distinct => logical::SetQuantifier::Distinct,
                };

                let id = self.plan.add_operator(BindingsOp::BagOp(BagOp {
                    bag_op: bag_operator,
                    setq,
                }));
                self.plan.add_flow_with_branch_num(lid, id, 0);
                self.plan.add_flow_with_branch_num(rid, id, 1);
                self.push_bexpr(id);
            }
            QuerySet::Select(_) => {}
            QuerySet::Expr(_) => {
                eq_or_fault!(self, env.len(), 1, "env.len() != 1");
                let expr = env.into_iter().next().unwrap();
                let op = BindingsOp::ExprQuery(logical::ExprQuery { expr });
                let id = self.plan.add_operator(op);
                self.push_bexpr(id);
            }
            QuerySet::Values(_) => {
                not_yet_implemented_fault!(self, "QuerySet::Values".to_string());
            }
            QuerySet::Table(_) => {
                not_yet_implemented_fault!(self, "QuerySet::Table".to_string());
            }
        }
        Traverse::Continue
    }

    fn enter_bag_op_expr(&mut self, _set_expr: &'ast BagOpExpr) -> Traverse {
        Traverse::Continue
    }

    fn exit_bag_op_expr(&mut self, _set_expr: &'ast BagOpExpr) -> Traverse {
        Traverse::Continue
    }

    fn enter_select(&mut self, select: &'ast Select) -> Traverse {
        if select.having.is_some() && select.group_by.is_none() {
            self.errors.push(AstTransformError::HavingWithoutGroupBy);
            Traverse::Stop
        } else {
            Traverse::Continue
        }
    }

    fn exit_select(&mut self, _select: &'ast Select) -> Traverse {
        // PartiQL permits SQL aggregations without a GROUP BY (e.g. SELECT SUM(t.a) FROM ...)
        // What follows adds a GROUP BY clause with the rewrite `... GROUP BY true AS $__gk`
        if !self.aggregate_exprs.is_empty() && self.current_clauses_mut().group_by_clause.is_none()
        {
            let exprs = HashMap::from([(
                "$__gk".to_string(),
                ValueExpr::Lit(Box::new(Value::from(true))),
            )]);
            let group_by: BindingsOp = BindingsOp::GroupBy(logical::GroupBy {
                strategy: logical::GroupingStrategy::GroupFull,
                exprs,
                aggregate_exprs: self.aggregate_exprs.clone(),
                group_as_alias: None,
            });
            let id = self.plan.add_operator(group_by);
            self.current_clauses_mut().group_by_clause.replace(id);
        }
        Traverse::Continue
    }

    fn enter_projection(&mut self, _projection: &'ast Projection) -> Traverse {
        self.enter_benv();
        self.enter_env();
        Traverse::Continue
    }

    fn exit_projection(&mut self, _projection: &'ast Projection) -> Traverse {
        let benv = self.exit_benv();
        eq_or_fault!(self, benv.len(), 0, "benv.len() != 0");

        let env = self.exit_env();
        eq_or_fault!(self, env.len(), 0, "env.len() != 0");

        if let Some(SetQuantifier::Distinct) = _projection.setq {
            let id = self.plan.add_operator(BindingsOp::Distinct);
            self.current_clauses_mut().distinct.replace(id);
        }
        Traverse::Continue
    }

    fn enter_projection_kind(&mut self, _projection_kind: &'ast ProjectionKind) -> Traverse {
        self.enter_benv();
        self.enter_env();
        Traverse::Continue
    }

    fn exit_projection_kind(&mut self, _projection_kind: &'ast ProjectionKind) -> Traverse {
        let benv = self.exit_benv();
        if !benv.is_empty() {
            not_yet_implemented_fault!(self, "Subquery within project".to_string());
        }
        let env = self.exit_env();

        let select: BindingsOp = match _projection_kind {
            ProjectionKind::ProjectStar => logical::BindingsOp::ProjectAll,
            ProjectionKind::ProjectList(_) => {
                true_or_fault!(self, env.len().is_even(), "env.len() is not even");
                let mut exprs = Vec::with_capacity(env.len() / 2);
                let mut iter = env.into_iter();
                while let Some(value) = iter.next() {
                    let alias = iter.next().unwrap();
                    let alias = match alias {
                        ValueExpr::Lit(lit) => match *lit {
                            Value::String(s) => (*s).clone(),
                            _ => {
                                // Report error but allow visitor to continue
                                self.errors.push(AstTransformError::IllegalState(
                                    "Unexpected literal type".to_string(),
                                ));
                                String::new()
                            }
                        },
                        _ => {
                            // Report error but allow visitor to continue
                            self.errors.push(AstTransformError::IllegalState(
                                "Invalid alias type".to_string(),
                            ));
                            String::new()
                        }
                    };

                    if !alias.is_empty() {
                        if let ValueExpr::VarRef(name, _vrtype) = &value {
                            self.projection_renames
                                .last_mut()
                                .expect("renames")
                                .insert(alias.clone(), name.clone());
                        }
                    }
                    exprs.push((alias, value));
                }

                logical::BindingsOp::Project(logical::Project { exprs })
            }
            ProjectionKind::ProjectPivot(_) => {
                eq_or_fault!(self, env.len(), 2, "env.len() != 2");

                let mut iter = env.into_iter();
                let key = iter.next().unwrap();
                let value = iter.next().unwrap();
                logical::BindingsOp::Pivot(logical::Pivot { key, value })
            }
            ProjectionKind::ProjectValue(_) => {
                eq_or_fault!(self, env.len(), 1, "env.len() != 1");

                let expr = env.into_iter().next().unwrap();
                logical::BindingsOp::ProjectValue(logical::ProjectValue { expr })
            }
        };
        let id = self.plan.add_operator(select);
        self.current_clauses_mut().select_clause.replace(id);
        Traverse::Continue
    }

    fn exit_project_expr(&mut self, _project_expr: &'ast ProjectExpr) -> Traverse {
        let as_key: &name_resolver::Symbol = self
            .key_registry
            .aliases
            .get(self.current_node())
            .expect("alias");
        // TODO intern strings
        let as_key = match as_key {
            name_resolver::Symbol::Known(sym) => sym.value.clone(),
            name_resolver::Symbol::Unknown(id) => format!("_{id}"),
        };
        self.push_value(as_key.into());
        Traverse::Continue
    }

    fn enter_bin_op(&mut self, _bin_op: &'ast BinOp) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_bin_op(&mut self, _bin_op: &'ast BinOp) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 2, "env.len() != 2");

        let rhs = env.pop().unwrap();
        let lhs = env.pop().unwrap();
        if _bin_op.kind == BinOpKind::Is {
            let is_type = match rhs {
                ValueExpr::Lit(lit) => match lit.as_ref() {
                    Value::Null => logical::Type::NullType,
                    Value::Missing => logical::Type::MissingType,
                    _ => {
                        not_yet_implemented_fault!(
                            self,
                            "Unsupported rhs literal for `IS`".to_string()
                        );
                    }
                },
                _ => {
                    not_yet_implemented_fault!(self, "Unsupported rhs for `IS`".to_string());
                }
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
        Traverse::Continue
    }

    fn enter_uni_op(&mut self, _uni_op: &'ast UniOp) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_uni_op(&mut self, _uni_op: &'ast UniOp) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 1, "env.len() != 1");

        let expr = env.pop().unwrap();
        let op = match _uni_op.kind {
            UniOpKind::Pos => logical::UnaryOp::Pos,
            UniOpKind::Neg => logical::UnaryOp::Neg,
            UniOpKind::Not => logical::UnaryOp::Not,
        };
        self.push_vexpr(ValueExpr::UnExpr(op, Box::new(expr)));
        Traverse::Continue
    }

    fn enter_between(&mut self, _between: &'ast Between) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_between(&mut self, _between: &'ast Between) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 3, "env.len() != 3");

        let to = Box::new(env.pop().unwrap());
        let from = Box::new(env.pop().unwrap());
        let value = Box::new(env.pop().unwrap());
        self.push_vexpr(ValueExpr::BetweenExpr(BetweenExpr { value, from, to }));
        Traverse::Continue
    }

    fn enter_in(&mut self, _in: &'ast ast::In) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }
    fn exit_in(&mut self, _in: &'ast ast::In) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 2, "env.len() != 2");

        let rhs = env.pop().unwrap();
        let lhs = env.pop().unwrap();
        self.push_vexpr(logical::ValueExpr::BinaryExpr(
            logical::BinaryOp::In,
            Box::new(lhs),
            Box::new(rhs),
        ));
        Traverse::Continue
    }

    fn enter_like(&mut self, _like: &'ast Like) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_like(&mut self, _like: &'ast Like) -> Traverse {
        let mut env = self.exit_env();
        true_or_fault!(
            self,
            (2..=3).contains(&env.len()),
            "env.len() is not between 2 and 3"
        );
        let escape_ve = if env.len() == 3 {
            env.pop().unwrap()
        } else {
            ValueExpr::Lit(Box::new(Value::String(Box::default())))
        };
        let pattern_ve = env.pop().unwrap();
        let value = Box::new(env.pop().unwrap());

        let pattern = match (&pattern_ve, &escape_ve) {
            (ValueExpr::Lit(pattern_lit), ValueExpr::Lit(escape_lit)) => {
                match (pattern_lit.as_ref(), escape_lit.as_ref()) {
                    (Value::String(pattern), Value::String(escape)) => Pattern::Like(LikeMatch {
                        pattern: pattern.to_string(),
                        escape: escape.to_string(),
                    }),
                    _ => Pattern::LikeNonStringNonLiteral(LikeNonStringNonLiteralMatch {
                        pattern: Box::new(pattern_ve),
                        escape: Box::new(escape_ve),
                    }),
                }
            }
            _ => Pattern::LikeNonStringNonLiteral(LikeNonStringNonLiteralMatch {
                pattern: Box::new(pattern_ve),
                escape: Box::new(escape_ve),
            }),
        };

        let pattern = ValueExpr::PatternMatchExpr(PatternMatchExpr { value, pattern });
        self.push_vexpr(pattern);
        Traverse::Continue
    }

    fn enter_call(&mut self, _call: &'ast Call) -> Traverse {
        self.enter_call();
        Traverse::Continue
    }

    fn exit_call(&mut self, call: &'ast Call) -> Traverse {
        // TODO better argument validation/error messaging
        let args = self.exit_call();
        let name = call.func_name.value.to_lowercase();

        let call_def_to_vexpr =
            |call_def: &CallDef| call_def.lookup(&args, &name).map_err(Into::into);

        let call_expr = self
            .fnsym_tab
            .lookup(&name)
            .map(call_def_to_vexpr)
            .or_else(|| {
                self.catalog
                    .get_function(&name)
                    .map(|e| call_def_to_vexpr(e.call_def()))
            })
            .unwrap_or_else(|| Err(AstTransformError::UnsupportedFunction(name.clone())));

        let expr = match call_expr {
            Ok(expr) => expr,
            Err(err) => {
                self.errors.push(err);
                ValueExpr::Lit(Box::new(Value::Missing)) // dummy expression to allow lowering to continue
            }
        };
        self.push_vexpr(expr);
        Traverse::Continue
    }

    fn enter_call_arg(&mut self, _call_arg: &'ast CallArg) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_call_arg(&mut self, _call_arg: &'ast CallArg) -> Traverse {
        let mut env = self.exit_env();
        match _call_arg {
            CallArg::Star() => {
                self.push_call_arg(CallArgument::Star);
            }
            CallArg::Positional(_) => {
                eq_or_fault!(self, env.len(), 1, "env.len() != 1");

                self.push_call_arg(CallArgument::Positional(env.pop().unwrap()));
            }
            CallArg::Named(CallArgNamed { name, .. }) => {
                eq_or_fault!(self, env.len(), 1, "env.len() != 1");

                let name = name.value.to_lowercase();
                self.push_call_arg(CallArgument::Named(name, env.pop().unwrap()));
            }
            CallArg::PositionalType(_) => {
                not_yet_implemented_fault!(self, "PositionalType call argument".to_string());
            }
            CallArg::NamedType(_) => {
                not_yet_implemented_fault!(self, "NamedType call argument".to_string());
            }
        }
        Traverse::Continue
    }

    // Values & Value Constructors

    fn enter_lit(&mut self, lit: &'ast Lit) -> Traverse {
        let val = match lit_to_value(lit) {
            Ok(v) => v,
            Err(e) => {
                // Report error but allow visitor to continue
                self.errors.push(e);
                Value::Missing
            }
        };
        self.push_value(val);
        Traverse::Continue
    }

    fn enter_struct(&mut self, _struct: &'ast Struct) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_struct(&mut self, _struct: &'ast Struct) -> Traverse {
        let env = self.exit_env();
        true_or_fault!(self, env.len().is_even(), "env.len() is not even");

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
        Traverse::Continue
    }

    fn enter_bag(&mut self, _bag: &'ast Bag) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_bag(&mut self, _bag: &'ast Bag) -> Traverse {
        let elements = self.exit_env();
        self.push_vexpr(ValueExpr::BagExpr(BagExpr { elements }));
        Traverse::Continue
    }

    fn enter_list(&mut self, _list: &'ast List) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_list(&mut self, _list: &'ast List) -> Traverse {
        let elements = self.exit_env();
        self.push_vexpr(ValueExpr::ListExpr(ListExpr { elements }));
        Traverse::Continue
    }

    fn enter_sexp(&mut self, _sexp: &'ast Sexp) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_sexp(&mut self, _sexp: &'ast Sexp) -> Traverse {
        not_yet_implemented_fault!(self, "Sexp".to_string());
    }

    fn enter_call_agg(&mut self, _call_agg: &'ast CallAgg) -> Traverse {
        self.enter_call();
        Traverse::Continue
    }

    fn exit_call_agg(&mut self, call_agg: &'ast CallAgg) -> Traverse {
        // Relates to the SQL aggregation functions (e.g. AVG, COUNT, SUM) -- not the `COLL_`
        // functions
        let mut env = self.exit_call();
        let name = call_agg.func_name.value.to_lowercase();

        // Rewrites the SQL aggregation function call to be a variable reference that the `GROUP BY`
        // clause will add to the binding tuples.
        // E.g. SELECT a, SUM(b) FROM t GROUP BY a
        //      SELECT a AS a, $__agg_1 AS b FROM t GROUP BY a
        let new_name = "$__agg".to_owned() + &self.agg_id.id();
        let new_binding_name = BindingsName::CaseSensitive(Cow::Owned(new_name.clone()));
        let new_expr = ValueExpr::VarRef(new_binding_name, VarRefType::Local);
        self.push_vexpr(new_expr);

        true_or_fault!(self, !env.is_empty(), "env is empty");
        // Default set quantifier if the set quantifier keyword is omitted will be `ALL`
        let (setq, arg) = match env.pop().unwrap() {
            CallArgument::Positional(ve) => (logical::SetQuantifier::All, ve),
            CallArgument::Named(name, ve) => match name.as_ref() {
                "all" => (logical::SetQuantifier::All, ve),
                "distinct" => (logical::SetQuantifier::Distinct, ve),
                _ => {
                    self.errors.push(AstTransformError::IllegalState(
                        "Invalid set quantifier".to_string(),
                    ));
                    return Traverse::Stop;
                }
            },
            CallArgument::Star => (
                logical::SetQuantifier::All,
                ValueExpr::Lit(Box::new(Value::Integer(1))),
            ),
        };

        let agg_expr = match name.as_str() {
            "avg" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggAvg,
                setq,
            },
            "count" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggCount,
                setq,
            },
            "max" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggMax,
                setq,
            },
            "min" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggMin,
                setq,
            },
            "sum" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggSum,
                setq,
            },
            "any" | "some" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggAny,
                setq,
            },
            "every" => AggregateExpression {
                name: new_name,
                expr: arg,
                func: AggEvery,
                setq,
            },
            _ => {
                // Include as an error but allow lowering to proceed for multiple error reporting
                self.errors
                    .push(AstTransformError::UnsupportedFunction(name));
                // continue lowering with `AggAvg` aggregation function
                AggregateExpression {
                    name: new_name,
                    expr: arg,
                    func: AggAvg,
                    setq,
                }
            }
        };
        self.aggregate_exprs.push(agg_expr);
        Traverse::Continue
    }

    fn enter_var_ref(&mut self, var_ref: &'ast VarRef) -> Traverse {
        let is_path = matches!(self.current_ctx(), Some(QueryContext::Path));
        if !is_path {
            let options = self.resolve_varref(var_ref);
            self.push_vexpr(options);
        } else {
            let VarRef {
                name: SymbolPrimitive { value, case },
                qualifier: _,
            } = var_ref;
            let name = match case {
                CaseSensitivity::CaseSensitive => {
                    BindingsName::CaseSensitive(Cow::Owned(value.clone()))
                }
                CaseSensitivity::CaseInsensitive => {
                    BindingsName::CaseInsensitive(Cow::Owned(value.clone()))
                }
            };
            self.push_vexpr(ValueExpr::VarRef(name, VarRefType::Local));
        }
        Traverse::Continue
    }

    fn exit_var_ref(&mut self, _var_ref: &'ast VarRef) -> Traverse {
        Traverse::Continue
    }

    fn enter_path(&mut self, _path: &'ast Path) -> Traverse {
        self.enter_env();
        self.enter_path();
        Traverse::Continue
    }

    fn exit_path(&mut self, _path: &'ast Path) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 1, "env.len() != 1");

        let steps = self.exit_path();
        let root = env.pop().unwrap();

        self.push_vexpr(ValueExpr::Path(Box::new(root), steps));
        Traverse::Continue
    }

    fn enter_path_step(&mut self, _path_step: &'ast PathStep) -> Traverse {
        if let PathStep::PathExpr(expr) = _path_step {
            self.enter_env();
            match *(expr.index) {
                Expr::VarRef(_) => {
                    // covers case of var refs along path: a.b.c <-- the "b" and "c" in "a.b.c" are local lookups only
                    let qc = self.ctx_stack.last_mut().unwrap();
                    *qc = QueryContext::Path;
                }
                _ => {
                    // covers case of a.b[c + 1] <-- the "c" in "c + 1" could be a dynamic lookup
                    let qc = self.ctx_stack.last_mut().unwrap();
                    *qc = QueryContext::Query;
                }
            }
        }
        Traverse::Continue
    }

    fn exit_path_step(&mut self, _path_step: &'ast PathStep) -> Traverse {
        let step = match _path_step {
            PathStep::PathExpr(_s) => {
                let mut env = self.exit_env();
                eq_or_fault!(self, env.len(), 1, "env.len() != 1");

                let path = env.pop().unwrap();
                match path {
                    ValueExpr::Lit(val) => match *val {
                        Value::Integer(idx) => logical::PathComponent::Index(idx),
                        Value::String(k) => logical::PathComponent::Key(
                            BindingsName::CaseInsensitive(Cow::Owned(*k)),
                        ),
                        expr => logical::PathComponent::IndexExpr(Box::new(ValueExpr::Lit(
                            Box::new(expr),
                        ))),
                    },
                    ValueExpr::VarRef(name, _) => logical::PathComponent::Key(name),
                    expr => {
                        // TODO if type is statically STRING, then use KeyExpr
                        logical::PathComponent::IndexExpr(Box::new(expr))
                    }
                }
            }
            PathStep::PathWildCard => {
                not_yet_implemented_fault!(self, "PathStep::PathWildCard".to_string());
            }
            PathStep::PathUnpivot => {
                not_yet_implemented_fault!(self, "PathStep::PathUnpivot".to_string());
            }
        };

        self.push_path_step(step);
        Traverse::Continue
    }

    fn enter_from_clause(&mut self, _from_clause: &'ast FromClause) -> Traverse {
        self.enter_benv();
        self.enter_env();
        Traverse::Continue
    }

    fn exit_from_clause(&mut self, _from_clause: &'ast FromClause) -> Traverse {
        let mut benv = self.exit_benv();
        eq_or_fault!(self, benv.len(), 1, "benv.len() != 1");

        let env = self.exit_env();
        eq_or_fault!(self, env.len(), 0, "env.len() != 0");

        self.current_clauses_mut()
            .from_clause
            .replace(benv.pop().unwrap());
        Traverse::Continue
    }

    fn enter_from_let(&mut self, from_let: &'ast FromLet) -> Traverse {
        self.from_lets.insert(*self.current_node());
        *self.current_ctx_mut() = QueryContext::FromLet;
        self.enter_env();

        let id = *self.current_node();

        for sym in [&from_let.as_alias, &from_let.at_alias, &from_let.by_alias]
            .into_iter()
            .flatten()
        {
            self.aliases.insert(id, sym.clone());
        }
        Traverse::Continue
    }

    fn exit_from_let(&mut self, from_let: &'ast FromLet) -> Traverse {
        *self.current_ctx_mut() = QueryContext::Query;
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 1, "env.len() != 1");

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
        Traverse::Continue
    }

    fn enter_join(&mut self, _join: &'ast Join) -> Traverse {
        self.enter_benv();
        self.enter_env();
        Traverse::Continue
    }

    fn exit_join(&mut self, join: &'ast Join) -> Traverse {
        let mut benv = self.exit_benv();
        eq_or_fault!(self, benv.len(), 2, "benv.len() != 2");

        let mut env = self.exit_env();
        true_or_fault!(
            self,
            (0..=1).contains(&env.len()),
            "env.len() is not between 0 and 1"
        );

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
            left,
            right,
            on,
        });
        let join = self.plan.add_operator(join);
        self.plan.add_flow_with_branch_num(lid, join, 0);
        self.plan.add_flow_with_branch_num(rid, join, 1);
        self.push_bexpr(join);
        Traverse::Continue
    }

    fn enter_join_spec(&mut self, join_spec: &'ast JoinSpec) -> Traverse {
        match join_spec {
            JoinSpec::On(_) => {
                // visitor recurse into expr will put the condition in the current env
            }
            JoinSpec::Using(_) => {
                not_yet_implemented_fault!(self, "JoinSpec::Using".to_string());
            }
            JoinSpec::Natural => {
                not_yet_implemented_fault!(self, "JoinSpec::Natural".to_string());
            }
        };
        Traverse::Continue
    }

    fn enter_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 1, "env.len() != 1");

        let filter = logical::BindingsOp::Filter(logical::Filter {
            expr: env.pop().unwrap(),
        });
        let id = self.plan.add_operator(filter);

        self.current_clauses_mut().where_clause.replace(id);
        Traverse::Continue
    }

    fn enter_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 1, "env.len() is 1");

        let having = BindingsOp::Having(logical::Having {
            expr: env.pop().unwrap(),
        });
        let id = self.plan.add_operator(having);

        self.current_clauses_mut().having_clause.replace(id);
        Traverse::Continue
    }

    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast GroupByExpr) -> Traverse {
        self.enter_benv();
        self.enter_env();
        Traverse::Continue
    }

    fn exit_group_by_expr(&mut self, _group_by_expr: &'ast GroupByExpr) -> Traverse {
        let aggregate_exprs = self.aggregate_exprs.clone();
        let benv = self.exit_benv();
        if !benv.is_empty() {
            {
                not_yet_implemented_fault!(self, "Subquery in group by".to_string());
            }
        }
        let env = self.exit_env();
        true_or_fault!(self, env.len().is_even(), "env.len() is not even");

        let group_as_alias = _group_by_expr
            .group_as_alias
            .as_ref()
            .map(|SymbolPrimitive { value, case: _ }| value.clone());

        let strategy = match _group_by_expr.strategy {
            GroupingStrategy::GroupFull => logical::GroupingStrategy::GroupFull,
            GroupingStrategy::GroupPartial => logical::GroupingStrategy::GroupPartial,
        };

        // What follows is an approach to implement section 11.2.1 of the PartiQL spec
        // (https://partiql.org/assets/PartiQL-Specification.pdf#subsubsection.11.2.1)
        // "Grouping Attributes and Direct Use of Grouping Expressions"
        // Consider the query:
        //   SELECT t.a + 1 AS a FROM t GROUP BY t.a + 1 AS some_alias
        // Since the group by key expression (t.a + 1) is the same as the select list expression, we
        // can replace the query to be `SELECT some_alias AS a FROM t GROUP BY t.a + 1 AS some_alias`
        // This isn't quite correct as it doesn't deal with SELECT VALUE expressions and expressions
        // that are in the `HAVING` and `ORDER BY` clauses.
        let select_clause_op_id = self.current_clauses_mut().select_clause;
        if select_clause_op_id.is_none() {
            self.errors.push(AstTransformError::IllegalState(
                "select_clause_op_id is None".to_string(),
            ));
            return Traverse::Stop;
        }
        let select_clause = self
            .plan
            .operator_as_mut(select_clause_op_id.expect("select_clause_op_id not None"))
            .unwrap();
        let mut binding = Vec::new();
        let select_clause_exprs = match select_clause {
            BindingsOp::Project(ref mut project) => &mut project.exprs,
            BindingsOp::ProjectAll => &mut binding,
            BindingsOp::ProjectValue(_) => &mut binding, // TODO: replacement of SELECT VALUE expressions
            _ => {
                self.errors.push(AstTransformError::IllegalState(
                    "Unexpected project type".to_string(),
                ));
                return Traverse::Stop;
            }
        };
        let mut exprs = HashMap::with_capacity(env.len() / 2);
        let mut iter = env.into_iter();

        while let Some(value) = iter.next() {
            let alias = iter.next().unwrap();
            let alias = match alias {
                ValueExpr::Lit(lit) => match *lit {
                    Value::String(s) => (*s).clone(),
                    _ => {
                        // Report error but allow visitor to continue
                        self.errors.push(AstTransformError::IllegalState(
                            "Unexpected literal type".to_string(),
                        ));
                        String::new()
                    }
                },
                _ => {
                    self.errors.push(AstTransformError::IllegalState(
                        "Unexpected alias type".to_string(),
                    ));
                    return Traverse::Stop;
                }
            };
            for (alias, expr) in select_clause_exprs.iter_mut() {
                if *expr == value {
                    let new_binding_name = BindingsName::CaseSensitive(Cow::Owned(alias.clone()));
                    let new_expr = ValueExpr::VarRef(new_binding_name, VarRefType::Local);
                    *expr = new_expr;
                }
            }
            exprs.insert(alias, value);
        }

        let group_by: BindingsOp = BindingsOp::GroupBy(logical::GroupBy {
            strategy,
            exprs,
            aggregate_exprs,
            group_as_alias,
        });

        let id = self.plan.add_operator(group_by);
        self.current_clauses_mut().group_by_clause.replace(id);
        Traverse::Continue
    }

    fn exit_group_key(&mut self, _group_key: &'ast GroupKey) -> Traverse {
        let as_key: &name_resolver::Symbol = self
            .key_registry
            .aliases
            .get(self.current_node())
            .expect("alias");
        // TODO intern strings
        let as_key = match as_key {
            name_resolver::Symbol::Known(sym) => sym.value.clone(),
            name_resolver::Symbol::Unknown(id) => format!("_{id}"),
        };
        self.push_value(as_key.into());
        Traverse::Continue
    }

    fn enter_order_by_expr(&mut self, _order_by_expr: &'ast OrderByExpr) -> Traverse {
        self.enter_sort();
        Traverse::Continue
    }

    fn exit_order_by_expr(&mut self, _order_by_expr: &'ast OrderByExpr) -> Traverse {
        let specs = self.exit_sort();
        let order_by = logical::BindingsOp::OrderBy(logical::OrderBy { specs });
        let id = self.plan.add_operator(order_by);
        if matches!(self.current_ctx(), Some(QueryContext::Query)) {
            self.current_clauses_mut().order_by_clause.replace(id);
        } else {
            self.push_bexpr(id);
        }
        Traverse::Continue
    }

    fn enter_sort_spec(&mut self, _sort_spec: &'ast SortSpec) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_sort_spec(&mut self, sort_spec: &'ast SortSpec) -> Traverse {
        let mut env = self.exit_env();
        eq_or_fault!(self, env.len(), 1, "env.len() is 1");

        let expr = env.pop().unwrap();
        let order = match sort_spec
            .ordering_spec
            .as_ref()
            .unwrap_or(&OrderingSpec::Asc)
        {
            OrderingSpec::Asc => logical::SortSpecOrder::Asc,
            OrderingSpec::Desc => logical::SortSpecOrder::Desc,
        };

        let null_order = match sort_spec.null_ordering_spec {
            None => match order {
                SortSpecOrder::Asc => logical::SortSpecNullOrder::Last,
                SortSpecOrder::Desc => logical::SortSpecNullOrder::First,
            },
            Some(NullOrderingSpec::First) => logical::SortSpecNullOrder::First,
            Some(NullOrderingSpec::Last) => logical::SortSpecNullOrder::Last,
        };

        self.push_sort_spec(logical::SortSpec {
            expr,
            order,
            null_order,
        });
        Traverse::Continue
    }

    fn enter_limit_offset_clause(
        &mut self,
        _limit_offset: &'ast ast::LimitOffsetClause,
    ) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_limit_offset_clause(&mut self, limit_offset: &'ast ast::LimitOffsetClause) -> Traverse {
        let mut env = self.exit_env();
        true_or_fault!(
            self,
            (1..=2).contains(&env.len()),
            "env.len() is  not between 1 and 2"
        );

        let offset = if limit_offset.offset.is_some() {
            env.pop()
        } else {
            None
        };
        let limit = if limit_offset.limit.is_some() {
            env.pop()
        } else {
            None
        };

        let limit_offset = logical::BindingsOp::LimitOffset(logical::LimitOffset { limit, offset });
        let id = self.plan.add_operator(limit_offset);
        if matches!(self.current_ctx(), Some(QueryContext::Query)) {
            self.current_clauses_mut().limit_offset_clause.replace(id);
        } else {
            self.push_bexpr(id);
        }
        Traverse::Continue
    }

    fn enter_simple_case(&mut self, _simple_case: &'ast SimpleCase) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_simple_case(&mut self, _simple_case: &'ast SimpleCase) -> Traverse {
        let mut env = self.exit_env();
        true_or_fault!(self, env.len() >= 2, "env.len < 2");

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
        }));
        Traverse::Continue
    }

    fn enter_searched_case(&mut self, _searched_case: &'ast SearchedCase) -> Traverse {
        self.enter_env();
        Traverse::Continue
    }

    fn exit_searched_case(&mut self, _searched_case: &'ast SearchedCase) -> Traverse {
        let mut env = self.exit_env();
        true_or_fault!(self, !env.is_empty(), "env is empty");

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
        }));
        Traverse::Continue
    }
}

fn lit_to_value(lit: &Lit) -> Result<Value, AstTransformError> {
    fn expect_lit(v: &Expr) -> Result<Value, AstTransformError> {
        match v {
            Expr::Lit(l) => lit_to_value(&l.node),
            _ => Err(AstTransformError::IllegalState(
                "non literal in literal aggregate".to_string(),
            )),
        }
    }

    fn tuple_pair(pair: &ast::ExprPair) -> Option<Result<(String, Value), AstTransformError>> {
        let key = match expect_lit(pair.first.as_ref()) {
            Ok(Value::String(s)) => s.as_ref().clone(),
            Ok(_) => {
                return Some(Err(AstTransformError::IllegalState(
                    "non string literal in literal struct key".to_string(),
                )))
            }
            Err(e) => return Some(Err(e)),
        };

        match expect_lit(pair.second.as_ref()) {
            Ok(Value::Missing) => None,
            Ok(val) => Some(Ok((key, val))),
            Err(e) => Some(Err(e)),
        }
    }

    let val = match lit {
        Lit::Null => Value::Null,
        Lit::Missing => Value::Missing,
        Lit::Int8Lit(n) => Value::Integer(i64::from(*n)),
        Lit::Int16Lit(n) => Value::Integer(i64::from(*n)),
        Lit::Int32Lit(n) => Value::Integer(i64::from(*n)),
        Lit::Int64Lit(n) => Value::Integer(*n),
        Lit::DecimalLit(d) => Value::Decimal(Box::new(*d)),
        Lit::NumericLit(n) => Value::Decimal(Box::new(*n)),
        Lit::RealLit(f) => Value::Real(OrderedFloat::from(f64::from(*f))),
        Lit::FloatLit(f) => Value::Real(OrderedFloat::from(f64::from(*f))),
        Lit::DoubleLit(f) => Value::Real(OrderedFloat::from(*f)),
        Lit::BoolLit(b) => Value::Boolean(*b),
        Lit::IonStringLit(s) => parse_embedded_ion_str(s)?,
        Lit::CharStringLit(s) => Value::String(Box::new(s.clone())),
        Lit::NationalCharStringLit(s) => Value::String(Box::new(s.clone())),
        Lit::BitStringLit(_) => {
            return Err(AstTransformError::NotYetImplemented(
                "Lit::BitStringLit".to_string(),
            ))
        }
        Lit::HexStringLit(_) => {
            return Err(AstTransformError::NotYetImplemented(
                "Lit::HexStringLit".to_string(),
            ))
        }
        Lit::BagLit(b) => {
            let bag: Result<partiql_value::Bag, _> = b
                .node
                .values
                .iter()
                .map(|l| expect_lit(l.as_ref()))
                .collect();
            Value::from(bag?)
        }
        Lit::ListLit(l) => {
            let l: Result<partiql_value::List, _> = l
                .node
                .values
                .iter()
                .map(|l| expect_lit(l.as_ref()))
                .collect();
            Value::from(l?)
        }
        Lit::StructLit(s) => {
            let tuple: Result<partiql_value::Tuple, _> =
                s.node.fields.iter().filter_map(tuple_pair).collect();
            Value::from(tuple?)
        }
        Lit::TypedLit(_, _) => {
            return Err(AstTransformError::NotYetImplemented(
                "Lit::TypedLit".to_string(),
            ))
        }
    };
    Ok(val)
}

fn parse_embedded_ion_str(contents: &str) -> Result<Value, AstTransformError> {
    fn lit_err(literal: &str, err: impl std::error::Error) -> AstTransformError {
        AstTransformError::Literal {
            literal: literal.into(),
            error: err.to_string(),
        }
    }

    let reader = ion_rs::ReaderBuilder::new()
        .build(contents)
        .map_err(|e| lit_err(contents, e))?;
    let mut iter = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
        .build(reader)
        .map_err(|e| lit_err(contents, e))?;

    iter.next()
        .ok_or_else(|| AstTransformError::Literal {
            literal: contents.into(),
            error: "Contains no value".into(),
        })?
        .map_err(|e| lit_err(contents, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogicalPlanner;
    use partiql_catalog::{PartiqlCatalog, TypeEnvEntry};
    use partiql_logical::BindingsOp::Project;
    use partiql_logical::ValueExpr;
    use partiql_types::dynamic;

    #[test]
    fn test_plan_non_existent_fns() {
        let catalog = PartiqlCatalog::default();
        let statement = "foo(1, 2) + bar(3)";
        let parsed = partiql_parser::Parser::default()
            .parse(statement)
            .expect("Expect successful parse");
        let planner = LogicalPlanner::new(&catalog);
        let logical = planner.lower(&parsed);
        assert!(logical.is_err());
        let lowering_errs = logical.expect_err("Expect errs").errors;
        assert_eq!(lowering_errs.len(), 2);
        assert_eq!(
            lowering_errs.first(),
            Some(&AstTransformError::UnsupportedFunction("foo".to_string()))
        );
        assert_eq!(
            lowering_errs.get(1),
            Some(&AstTransformError::UnsupportedFunction("bar".to_string()))
        );
    }

    #[test]
    fn test_plan_bad_num_arguments() {
        let catalog = PartiqlCatalog::default();
        let statement = "abs(1, 2) + mod(3)";
        let parsed = partiql_parser::Parser::default()
            .parse(statement)
            .expect("Expect successful parse");
        let planner = LogicalPlanner::new(&catalog);
        let logical = planner.lower(&parsed);
        assert!(logical.is_err());
        let lowering_errs = logical.expect_err("Expect errs").errors;
        assert_eq!(lowering_errs.len(), 2);
        assert_eq!(
            lowering_errs.first(),
            Some(&AstTransformError::InvalidNumberOfArguments(
                "abs".to_string()
            ))
        );
        assert_eq!(
            lowering_errs.get(1),
            Some(&AstTransformError::InvalidNumberOfArguments(
                "mod".to_string()
            ))
        );
    }

    #[test]
    fn test_plan_type_entry_in_catalog() {
        // Expected Logical Plan
        let mut expected_logical = LogicalPlan::new();
        let my_id = ValueExpr::Path(
            Box::new(ValueExpr::DynamicLookup(Box::new(vec![
                ValueExpr::VarRef(
                    BindingsName::CaseInsensitive("c".to_string().into()),
                    VarRefType::Local,
                ),
                ValueExpr::VarRef(
                    BindingsName::CaseInsensitive("c".to_string().into()),
                    VarRefType::Global,
                ),
            ]))),
            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                "id".to_string().into(),
            ))],
        );

        let my_name = ValueExpr::Path(
            Box::new(ValueExpr::DynamicLookup(Box::new(vec![ValueExpr::VarRef(
                BindingsName::CaseInsensitive("customers".to_string().into()),
                VarRefType::Global,
            )]))),
            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                "name".to_string().into(),
            ))],
        );

        let project = expected_logical.add_operator(Project(logical::Project {
            exprs: Vec::from([
                ("my_id".to_string(), my_id),
                ("my_name".to_string(), my_name),
            ]),
        }));

        let scan = expected_logical.add_operator(BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::DynamicLookup(Box::new(vec![ValueExpr::VarRef(
                BindingsName::CaseInsensitive("customers".to_string().into()),
                VarRefType::Global,
            )])),
            as_key: "c".to_string(),
            at_key: None,
        }));
        let sink = expected_logical.add_operator(BindingsOp::Sink);
        expected_logical.add_flow_with_branch_num(scan, project, 0);
        expected_logical.add_flow_with_branch_num(project, sink, 0);

        let mut catalog = PartiqlCatalog::default();
        let _oid = catalog.add_type_entry(TypeEnvEntry::new("customers", &[], dynamic!()));
        let statement = "SELECT c.id AS my_id, customers.name AS my_name FROM customers AS c";
        let parsed = partiql_parser::Parser::default()
            .parse(statement)
            .expect("Expect successful parse");
        let planner = LogicalPlanner::new(&catalog);
        let logical = planner.lower(&parsed).expect("Expect successful lowering");
        assert_eq!(expected_logical, logical);

        println!("logical: {:?}", &logical);
    }
}
