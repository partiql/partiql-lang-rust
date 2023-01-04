use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};
use num::Integer;
use ordered_float::OrderedFloat;
use partiql_ast::ast;
use partiql_ast::ast::{
    Assignment, Bag, BinOp, BinOpKind, Call, CallAgg, CaseSensitivity, CreateIndex, CreateTable,
    Ddl, DdlOp, Delete, Dml, DmlOp, DropIndex, DropTable, Expr, FromClause, FromLet, FromLetKind,
    GroupByExpr, Insert, InsertValue, Item, Join, JoinKind, JoinSpec, List, Lit, NodeId,
    OnConflict, OrderByExpr, Path, PathStep, ProjectExpr, Projection, ProjectionKind, Query,
    QuerySet, Remove, ScopeQualifier, Select, Set, SetExpr, SetQuantifier, Sexp, Struct,
    SymbolPrimitive, UniOp, UniOpKind, VarRef,
};
use partiql_ast::visit::{Visit, Visitor};
use partiql_logical as logical;
use partiql_logical::{
    BagExpr, BindingsOp, IsTypeExpr, ListExpr, LogicalPlan, OpId, PathComponent, TupleExpr,
    ValueExpr,
};

use partiql_value::{BindingsName, Value};

use std::collections::{HashMap, HashSet};

use std::hash::Hash;

use std::sync::atomic::{AtomicU32, Ordering};

type FnvIndexSet<T> = IndexSet<T, FnvBuildHasher>;

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
    // TODO remove
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
    path_stack: Vec<Vec<PathComponent>>,

    from_lets: HashSet<ast::NodeId>,

    siblings: Vec<Vec<NodeId>>,

    aliases: FnvIndexMap<NodeId, SymbolPrimitive>,

    // generator of 'fresh' ids
    id: IdGenerator,

    // output
    plan: LogicalPlan<BindingsOp>,

    key_registry: KeyRegistry,
}

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

impl AstToLogical {
    pub fn new(registry: KeyRegistry) -> Self {
        AstToLogical {
            id_stack: Default::default(),

            q_stack: Default::default(),
            ctx_stack: Default::default(),
            bexpr_stack: Default::default(),
            vexpr_stack: Default::default(),
            path_stack: Default::default(),

            from_lets: Default::default(),

            siblings: Default::default(),

            aliases: Default::default(),

            // generator of 'fresh' ids
            id: Default::default(),

            // output
            plan: Default::default(),

            key_registry: registry,
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
        // TODO assure non-collision with provided identifiers
        SymbolPrimitive {
            value: self.id.id(),
            case: CaseSensitivity::CaseInsensitive,
        }
    }

    // TODO remove
    #[inline]
    fn infer_id(&self, expr: &ValueExpr, as_alias: &Option<SymbolPrimitive>) -> SymbolPrimitive {
        as_alias
            .to_owned()
            .or_else(|| infer_id(expr))
            .unwrap_or_else(|| self.gen_id())
    }

    fn resolve_varref(&self, varref: &ast::VarRef) -> logical::ValueExpr {
        fn symprim_to_binding(sym: &SymbolPrimitive) -> BindingsName {
            match sym.case {
                CaseSensitivity::CaseSensitive => BindingsName::CaseSensitive(sym.value.clone()),
                CaseSensitivity::CaseInsensitive => {
                    BindingsName::CaseInsensitive(sym.value.clone())
                }
            }
        }
        fn sym_to_binding(sym: &Symbol) -> BindingsName {
            match sym {
                Symbol::Known(sym) => symprim_to_binding(sym),
                Symbol::Unknown(_) => {
                    // TODO
                    BindingsName::CaseInsensitive("_1".to_string())
                }
            }
        }

        for id in self.id_stack.iter().rev() {
            if let Some(key_schema) = self.key_registry.schema.get(id) {
                let key_schema: &KeySchema = key_schema;
                let name_ref: &NameRef = key_schema
                    .consume
                    .iter()
                    .find(|name_ref| name_ref.sym == varref.name)
                    .expect("NameRef");

                let var_binding = symprim_to_binding(&name_ref.sym);
                let var_ref_expr = ValueExpr::VarRef(var_binding.clone());

                let mut lookups = vec![];
                for lookup in &name_ref.lookup {
                    match lookup {
                        NameLookup::Global => {
                            if !lookups.contains(&var_ref_expr) {
                                lookups.push(var_ref_expr.clone())
                            }
                        }
                        NameLookup::Local => {
                            if let Some(scope_ids) = self.key_registry.in_scope.get(id) {
                                let scopes: Vec<&KeySchema> = scope_ids
                                    .iter()
                                    .filter_map(|scope_id| self.key_registry.schema.get(scope_id))
                                    .collect();

                                let mut exact = scopes.iter().filter(|&scope| {
                                    scope.produce.contains(&Symbol::Known(name_ref.sym.clone()))
                                });
                                if let Some(_matching) = exact.next() {
                                    lookups.push(var_ref_expr);
                                    break;
                                }

                                for scope in scopes {
                                    for produce in &scope.produce {
                                        if let Symbol::Known(sym) = produce {
                                            if sym == &varref.name {
                                                let expr =
                                                    ValueExpr::VarRef(sym_to_binding(produce));
                                                if !lookups.contains(&expr) {
                                                    lookups.push(expr);
                                                }
                                                continue;
                                            }
                                        }
                                        // else
                                        let path = logical::ValueExpr::Path(
                                            Box::new(ValueExpr::VarRef(sym_to_binding(produce))),
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

        // TODO error?
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

impl<'ast> Visitor<'ast> for AstToLogical {
    fn enter_ast_node(&mut self, id: NodeId) {
        self.id_stack.push(id);
    }
    fn exit_ast_node(&mut self, id: NodeId) {
        assert_eq!(self.id_stack.pop(), Some(id))
    }

    fn enter_item(&mut self, _item: &'ast Item) {
        panic!("Only query is supported")
    }

    fn enter_ddl(&mut self, _ddl: &'ast Ddl) {
        panic!("Only query is supported")
    }

    fn enter_ddl_op(&mut self, _ddl_op: &'ast DdlOp) {
        panic!("Only query is supported")
    }

    fn enter_create_table(&mut self, _create_table: &'ast CreateTable) {
        panic!("Only query is supported")
    }

    fn enter_drop_table(&mut self, _drop_table: &'ast DropTable) {
        panic!("Only query is supported")
    }

    fn enter_create_index(&mut self, _create_index: &'ast CreateIndex) {
        panic!("Only query is supported")
    }

    fn enter_drop_index(&mut self, _drop_index: &'ast DropIndex) {
        panic!("Only query is supported")
    }

    fn enter_dml(&mut self, _dml: &'ast Dml) {
        panic!("Only query is supported")
    }

    fn enter_dml_op(&mut self, _dml_op: &'ast DmlOp) {
        panic!("Only query is supported")
    }

    fn enter_insert(&mut self, _insert: &'ast Insert) {
        panic!("Only query is supported")
    }

    fn enter_insert_value(&mut self, _insert_value: &'ast InsertValue) {
        panic!("Only query is supported")
    }

    fn enter_set(&mut self, _set: &'ast Set) {
        panic!("Only query is supported")
    }

    fn enter_assignment(&mut self, _assignment: &'ast Assignment) {
        panic!("Only query is supported")
    }

    fn enter_remove(&mut self, _remove: &'ast Remove) {
        panic!("Only query is supported")
    }

    fn enter_delete(&mut self, _delete: &'ast Delete) {
        panic!("Only query is supported")
    }

    fn enter_on_conflict(&mut self, _on_conflict: &'ast OnConflict) {
        panic!("Only query is supported")
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
        let as_key: &Symbol = self
            .key_registry
            .aliases
            .get(self.current_node())
            .expect("alias");
        // TODO intern strings
        let as_key = match as_key {
            Symbol::Known(sym) => sym.value.clone(),
            Symbol::Unknown(id) => format!("_{}", id),
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

    fn enter_call(&mut self, _call: &'ast Call) {
        todo!("call")
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

    fn enter_from_let(&mut self, _from_let: &'ast FromLet) {
        self.from_lets.insert(*self.current_node());
        *self.current_ctx_mut() = QueryContext::FromLet;
        self.enter_env();

        let id = *self.current_node();
        self.siblings.last_mut().unwrap().push(id);

        for sym in [
            &_from_let.as_alias,
            &_from_let.at_alias,
            &_from_let.by_alias,
        ]
        .into_iter()
        .flatten()
        {
            self.aliases.insert(id, sym.clone());
        }
    }

    fn exit_from_let(&mut self, _from_let: &'ast FromLet) {
        *self.current_ctx_mut() = QueryContext::Query;
        let mut env = self.exit_env();
        assert_eq!(env.len(), 1);

        let expr = env.pop().unwrap();

        let FromLet {
            kind,
            as_alias,
            at_alias,
            ..
        } = _from_let;
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

    fn exit_join(&mut self, _join: &'ast Join) {
        let mut benv = self.exit_benv();
        assert_eq!(benv.len(), 2);

        let mut env = self.exit_env();
        assert!((0..1).contains(&env.len()));

        let Join { kind, .. } = _join;

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

    fn enter_join_spec(&mut self, _join_spec: &'ast JoinSpec) {
        match _join_spec {
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

    fn exit_join_spec(&mut self, _join_spec: &'ast JoinSpec) {}

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
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NameRef {
    pub sym: SymbolPrimitive,
    pub lookup: Vec<NameLookup>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NameLookup {
    Global,
    Local,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Symbol {
    Known(SymbolPrimitive),
    Unknown(u32),
}

type NameRefs = FnvIndexSet<NameRef>;
type Names = FnvIndexSet<Symbol>;
type NameOptions = FnvIndexSet<Option<Symbol>>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct KeySchema {
    pub consume: NameRefs,
    pub produce: Names,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct KeyRefs {
    pub consume: NameRefs,
    pub produce_required: Names,
    pub produce_optional: NameOptions,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum QNode {
    FromLet,
    Query,
}

#[derive(Default, Debug, Clone)]
pub struct KeyRegistry {
    in_scope: FnvIndexMap<NodeId, Vec<NodeId>>,
    schema: FnvIndexMap<NodeId, KeySchema>,
    aliases: FnvIndexMap<NodeId, Symbol>,
}

#[derive(Default, Debug)]
pub struct NameResolver {
    // environment stack tracking
    id_path_to_root: Vec<NodeId>,
    id_child_stack: Vec<Vec<NodeId>>,
    keyref_stack: Vec<KeyRefs>,
    lateral_stack: Vec<Vec<NodeId>>,
    id_gen: IdGenerator,

    // data flow tracking
    qnodes: FnvIndexMap<QNode, Vec<ast::NodeId>>,
    in_scope: FnvIndexMap<NodeId, Vec<NodeId>>,
    schema: FnvIndexMap<NodeId, KeySchema>,
    aliases: FnvIndexMap<NodeId, Symbol>,
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
    fn current_node(&self) -> &NodeId {
        self.id_path_to_root.last().unwrap()
    }

    #[inline]
    fn is_from_path(&self) -> bool {
        let is_qnode = |typ, id| {
            self.qnodes
                .get(&typ)
                .map(|nodes| nodes.contains(id))
                .unwrap_or(false)
        };
        for id in self.id_path_to_root.iter().rev() {
            if is_qnode(QNode::Query, id) {
                return false;
            } else if is_qnode(QNode::FromLet, id) {
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
    fn exit_lateral(&mut self) -> Vec<NodeId> {
        self.lateral_stack.pop().expect("lateral level")
    }

    #[inline]
    fn enter_child_stack(&mut self) {
        self.id_child_stack.push(vec![]);
    }

    #[inline]
    fn exit_child_stack(&mut self) -> Vec<NodeId> {
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
    fn enter_ast_node(&mut self, id: NodeId) {
        self.id_path_to_root.push(id);
        if let Some(children) = self.id_child_stack.last_mut() {
            children.push(id);
        }
    }
    fn exit_ast_node(&mut self, id: NodeId) {
        assert_eq!(self.id_path_to_root.pop(), Some(id))
    }

    fn enter_query(&mut self, _query: &'ast Query) {
        let id = *self.current_node();
        self.qnodes
            .entry(QNode::Query)
            .or_insert_with(Vec::new)
            .push(id);
        self.enter_keyref();
    }

    fn exit_query(&mut self, _query: &'ast Query) {
        let id = *self.current_node();
        let keyrefs = self.exit_keyref();

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

    fn enter_from_clause(&mut self, _from_clause: &'ast FromClause) {
        self.enter_lateral();
        self.enter_child_stack();
    }

    fn exit_from_clause(&mut self, _from_clause: &'ast FromClause) {
        self.exit_lateral();
        self.exit_child_stack();
    }

    fn enter_join(&mut self, _join: &'ast Join) {
        self.enter_child_stack();
    }

    fn exit_join(&mut self, _join: &'ast Join) {
        self.exit_child_stack();
    }

    fn enter_from_let(&mut self, _from_let: &'ast FromLet) {
        self.enter_child_stack();

        let id = *self.current_node();
        self.qnodes
            .entry(QNode::FromLet)
            .or_insert_with(Vec::new)
            .push(id);
        self.enter_keyref();

        for in_scope in self.id_path_to_root.iter().rev().skip(1) {
            self.in_scope
                .entry(*in_scope)
                .or_insert_with(Vec::new)
                .push(id);
        }

        for in_scope in self.lateral_stack.last().unwrap() {
            self.in_scope
                .entry(id)
                .or_insert_with(Vec::new)
                .push(*in_scope);
        }

        self.lateral_stack.last_mut().unwrap().push(id);
    }

    fn exit_from_let(&mut self, _from_let: &'ast FromLet) {
        self.exit_child_stack();
        let id = *self.current_node();
        let KeyRefs { consume, .. } = self.exit_keyref();

        let as_alias = if let Some(sym) = &_from_let.as_alias {
            Symbol::Known(sym.clone())
        } else if let Some(sym) = infer_alias(&_from_let.expr) {
            Symbol::Known(sym)
        } else {
            Symbol::Unknown(self.id_gen.next_id())
        };
        let at_alias = _from_let
            .at_alias
            .as_ref()
            .map(|sym| Symbol::Known(sym.to_owned()));
        let produce: Names = std::iter::once(as_alias).chain(at_alias).collect();
        for alias in &produce {
            self.aliases.insert(id, alias.clone());
        }

        self.schema.insert(id, KeySchema { consume, produce });
    }

    fn enter_var_ref(&mut self, _var_ref: &'ast VarRef) {
        let is_from_path = self.is_from_path();

        let name = if is_from_path {
            match &_var_ref.qualifier {
                ScopeQualifier::Unqualified => NameRef {
                    sym: _var_ref.name.clone(),
                    lookup: vec![NameLookup::Global, NameLookup::Local],
                },
                ScopeQualifier::Qualified => NameRef {
                    sym: _var_ref.name.clone(),
                    lookup: vec![NameLookup::Local, NameLookup::Global],
                },
            }
        } else {
            NameRef {
                sym: _var_ref.name.clone(),
                lookup: vec![NameLookup::Local, NameLookup::Global],
            }
        };

        self.push_consume_name(name);
    }

    fn exit_project_expr(&mut self, _project_expr: &'ast ProjectExpr) {
        let id = self.current_node();
        let as_alias = if let Some(sym) = &_project_expr.as_alias {
            Symbol::Known(sym.clone())
        } else if let Some(sym) = infer_alias(&_project_expr.expr) {
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

fn infer_alias(expr: &ast::Expr) -> Option<SymbolPrimitive> {
    match expr {
        Expr::VarRef(ast::AstNode { node, .. }) => Some(node.name.clone()),
        Expr::Path(ast::AstNode { node, .. }) => match node.steps.last() {
            Some(ast::PathStep::PathExpr(expr)) => infer_alias(&expr.index),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{AstToLogical, NameResolver};
    use assert_matches::assert_matches;
    use partiql_ast::ast::Expr;
    use partiql_eval::env::basic::MapBindings;

    use partiql_eval::plan;

    use partiql_logical as logical;
    use partiql_logical::{BindingsOp, LogicalPlan};
    use partiql_parser::{Parsed, Parser};
    use partiql_value::{partiql_bag, partiql_tuple, Value};
    use partiql_value::{Bag, Tuple};

    #[track_caller]
    fn parse(text: &str) -> Parsed {
        Parser::default().parse(text).unwrap()
    }

    #[track_caller]
    fn lower(parsed: &Parsed) -> logical::LogicalPlan<logical::BindingsOp> {
        if let Expr::Query(q) = parsed.ast.as_ref() {
            let mut resolver = NameResolver::default();
            let resolver = resolver.resolve(q);

            let planner = AstToLogical::new(resolver);
            planner.lower_query(q)
        } else {
            panic!("wrong expr type");
        }
    }

    #[track_caller]
    fn evaluate(logical: LogicalPlan<BindingsOp>, bindings: MapBindings<Value>) -> Value {
        let planner = plan::EvaluatorPlanner;

        let mut plan = planner.compile(&logical);
        println!("{:?}", plan.dump_graph());

        if let Ok(out) = plan.execute_mut(bindings) {
            out.result
        } else {
            Value::Missing
        }
    }

    #[track_caller]
    fn evalute_query(query: &str) -> Value {
        let parsed = parse(query);
        let lowered = lower(&parsed);
        evaluate(lowered, Default::default())
    }

    fn data_customer() -> MapBindings<Value> {
        fn customer_tuple(id: i64, first_name: &str, balance: i64) -> Value {
            partiql_tuple![("id", id), ("firstName", first_name), ("balance", balance),].into()
        }

        let customer_val = partiql_bag![
            customer_tuple(5, "jason", 100),
            customer_tuple(4, "sisko", 0),
            customer_tuple(3, "jason", -30),
            customer_tuple(2, "miriam", 20),
            customer_tuple(1, "miriam", 10),
        ];

        let mut bindings = MapBindings::default();
        bindings.insert("customer", customer_val.into());
        bindings
    }

    #[test]
    pub fn test() {
        // Plan for `SELECT DISTINCT firstName, (firstName || firstName) AS doubleName FROM customer WHERE balance > 0`
        let query = "\
        SELECT DISTINCT firstName, (firstName || firstName) AS doubleName \
        FROM customer \
        WHERE balance > 0";
        let parsed = parse(query);
        let lowered = lower(&parsed);
        let out = evaluate(lowered, data_customer());

        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("firstName", "jason"), ("doubleName", "jasonjason")],
                partiql_tuple![("firstName", "miriam"), ("doubleName", "miriammiriam")],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    pub fn test_5() {
        let out = evalute_query("5");
        println!("{:?}", &out);
        assert_matches!(out, Value::Integer(5));
    }

    #[test]
    pub fn test_from_5() {
        let out = evalute_query("SELECT * FROM 5");
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_tuple![("_1", 5)]];
            assert_eq!(*bag, expected);
        });
    }
}
