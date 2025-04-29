use crate::ast;
use partiql_common::node::NodeId;

/// Indicates if tree traversal of the entire tree should continue or not.
#[derive(PartialEq, Debug)]
pub enum Traverse {
    /// Signals tree traversal of entire tree should continue.
    Continue,
    /// Signals tree traversal of entire tree should stop.
    Stop,
}

pub trait Visit {
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Traverse
    where
        V: Visitor<'ast>;
}

impl<T> Visit for ast::AstNode<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Traverse
    where
        V: Visitor<'ast>,
    {
        if v.enter_ast_node(self.id) == Traverse::Stop {
            return Traverse::Stop;
        }
        if self.node.visit(v) == Traverse::Stop {
            return Traverse::Stop;
        }
        v.exit_ast_node(self.id)
    }
}

impl<T> Visit for &T
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Traverse
    where
        V: Visitor<'ast>,
    {
        (*self).visit(v)
    }
}

impl<T> Visit for Box<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Traverse
    where
        V: Visitor<'ast>,
    {
        (**self).visit(v)
    }
}

impl<T> Visit for Option<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Traverse
    where
        V: Visitor<'ast>,
    {
        if let Some(inner) = self {
            if inner.visit(v) == Traverse::Stop {
                return Traverse::Stop;
            }
        }
        Traverse::Continue
    }
}

impl<T> Visit for Vec<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Traverse
    where
        V: Visitor<'ast>,
    {
        for i in self {
            if i.visit(v) == Traverse::Stop {
                return Traverse::Stop;
            }
        }
        Traverse::Continue
    }
}

pub trait Visitor<'ast> {
    fn enter_ast_node(&mut self, _id: NodeId) -> Traverse {
        Traverse::Continue
    }
    fn exit_ast_node(&mut self, _id: NodeId) -> Traverse {
        Traverse::Continue
    }
    fn enter_item(&mut self, _item: &'ast ast::Item) -> Traverse {
        Traverse::Continue
    }
    fn exit_item(&mut self, _item: &'ast ast::Item) -> Traverse {
        Traverse::Continue
    }
    fn enter_ddl(&mut self, _ddl: &'ast ast::Ddl) -> Traverse {
        Traverse::Continue
    }
    fn exit_ddl(&mut self, _ddl: &'ast ast::Ddl) -> Traverse {
        Traverse::Continue
    }
    fn enter_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) -> Traverse {
        Traverse::Continue
    }
    fn exit_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) -> Traverse {
        Traverse::Continue
    }
    fn enter_create_table(&mut self, _create_table: &'ast ast::CreateTable) -> Traverse {
        Traverse::Continue
    }
    fn exit_create_table(&mut self, _create_table: &'ast ast::CreateTable) -> Traverse {
        Traverse::Continue
    }
    fn enter_drop_table(&mut self, _drop_table: &'ast ast::DropTable) -> Traverse {
        Traverse::Continue
    }
    fn exit_drop_table(&mut self, _drop_table: &'ast ast::DropTable) -> Traverse {
        Traverse::Continue
    }
    fn enter_create_index(&mut self, _create_index: &'ast ast::CreateIndex) -> Traverse {
        Traverse::Continue
    }
    fn exit_create_index(&mut self, _create_index: &'ast ast::CreateIndex) -> Traverse {
        Traverse::Continue
    }
    fn enter_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) -> Traverse {
        Traverse::Continue
    }
    fn exit_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) -> Traverse {
        Traverse::Continue
    }
    fn enter_dml(&mut self, _dml: &'ast ast::Dml) -> Traverse {
        Traverse::Continue
    }
    fn exit_dml(&mut self, _dml: &'ast ast::Dml) -> Traverse {
        Traverse::Continue
    }
    fn enter_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) -> Traverse {
        Traverse::Continue
    }
    fn exit_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) -> Traverse {
        Traverse::Continue
    }
    fn enter_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) -> Traverse {
        Traverse::Continue
    }
    fn exit_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) -> Traverse {
        Traverse::Continue
    }
    fn enter_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) -> Traverse {
        Traverse::Continue
    }
    fn exit_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) -> Traverse {
        Traverse::Continue
    }
    fn enter_insert(&mut self, _insert: &'ast ast::Insert) -> Traverse {
        Traverse::Continue
    }
    fn exit_insert(&mut self, _insert: &'ast ast::Insert) -> Traverse {
        Traverse::Continue
    }
    fn enter_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) -> Traverse {
        Traverse::Continue
    }
    fn exit_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) -> Traverse {
        Traverse::Continue
    }
    fn enter_set(&mut self, _set: &'ast ast::Set) -> Traverse {
        Traverse::Continue
    }
    fn exit_set(&mut self, _set: &'ast ast::Set) -> Traverse {
        Traverse::Continue
    }
    fn enter_assignment(&mut self, _assignment: &'ast ast::Assignment) -> Traverse {
        Traverse::Continue
    }
    fn exit_assignment(&mut self, _assignment: &'ast ast::Assignment) -> Traverse {
        Traverse::Continue
    }
    fn enter_remove(&mut self, _remove: &'ast ast::Remove) -> Traverse {
        Traverse::Continue
    }
    fn exit_remove(&mut self, _remove: &'ast ast::Remove) -> Traverse {
        Traverse::Continue
    }
    fn enter_delete(&mut self, _delete: &'ast ast::Delete) -> Traverse {
        Traverse::Continue
    }
    fn exit_delete(&mut self, _delete: &'ast ast::Delete) -> Traverse {
        Traverse::Continue
    }
    fn enter_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) -> Traverse {
        Traverse::Continue
    }
    fn exit_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) -> Traverse {
        Traverse::Continue
    }
    fn enter_top_level_query(&mut self, _query: &'ast ast::TopLevelQuery) -> Traverse {
        Traverse::Continue
    }
    fn exit_top_level_query(&mut self, _query: &'ast ast::TopLevelQuery) -> Traverse {
        Traverse::Continue
    }
    fn enter_query(&mut self, _query: &'ast ast::Query) -> Traverse {
        Traverse::Continue
    }
    fn exit_query(&mut self, _query: &'ast ast::Query) -> Traverse {
        Traverse::Continue
    }
    fn enter_with_clause(&mut self, _query: &'ast ast::WithClause) -> Traverse {
        Traverse::Continue
    }
    fn exit_with_clause(&mut self, _query: &'ast ast::WithClause) -> Traverse {
        Traverse::Continue
    }
    fn enter_with_element(&mut self, _query: &'ast ast::WithElement) -> Traverse {
        Traverse::Continue
    }
    fn exit_with_element(&mut self, _query: &'ast ast::WithElement) -> Traverse {
        Traverse::Continue
    }
    fn enter_query_set(&mut self, _query_set: &'ast ast::QuerySet) -> Traverse {
        Traverse::Continue
    }
    fn exit_query_set(&mut self, _query_set: &'ast ast::QuerySet) -> Traverse {
        Traverse::Continue
    }
    fn enter_bag_op_expr(&mut self, _set_expr: &'ast ast::BagOpExpr) -> Traverse {
        Traverse::Continue
    }
    fn exit_bag_op_expr(&mut self, _set_expr: &'ast ast::BagOpExpr) -> Traverse {
        Traverse::Continue
    }
    fn enter_select(&mut self, _select: &'ast ast::Select) -> Traverse {
        Traverse::Continue
    }
    fn exit_select(&mut self, _select: &'ast ast::Select) -> Traverse {
        Traverse::Continue
    }
    fn enter_query_table(&mut self, _table: &'ast ast::QueryTable) -> Traverse {
        Traverse::Continue
    }
    fn exit_query_table(&mut self, _table: &'ast ast::QueryTable) -> Traverse {
        Traverse::Continue
    }
    fn enter_projection(&mut self, _projection: &'ast ast::Projection) -> Traverse {
        Traverse::Continue
    }
    fn exit_projection(&mut self, _projection: &'ast ast::Projection) -> Traverse {
        Traverse::Continue
    }
    fn enter_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) -> Traverse {
        Traverse::Continue
    }
    fn exit_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) -> Traverse {
        Traverse::Continue
    }
    fn enter_project_item(&mut self, _project_item: &'ast ast::ProjectItem) -> Traverse {
        Traverse::Continue
    }
    fn exit_project_item(&mut self, _project_item: &'ast ast::ProjectItem) -> Traverse {
        Traverse::Continue
    }
    fn enter_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) -> Traverse {
        Traverse::Continue
    }
    fn exit_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) -> Traverse {
        Traverse::Continue
    }
    fn enter_project_all(&mut self, _project_all: &'ast ast::ProjectAll) -> Traverse {
        Traverse::Continue
    }
    fn exit_project_all(&mut self, _project_all: &'ast ast::ProjectAll) -> Traverse {
        Traverse::Continue
    }
    fn enter_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) -> Traverse {
        Traverse::Continue
    }
    fn exit_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) -> Traverse {
        Traverse::Continue
    }
    fn enter_exclusion(&mut self, _exclusion: &'ast ast::Exclusion) -> Traverse {
        Traverse::Continue
    }
    fn exit_exclusion(&mut self, _exclusion: &'ast ast::Exclusion) -> Traverse {
        Traverse::Continue
    }
    fn enter_exclude_path(&mut self, _path: &'ast ast::ExcludePath) -> Traverse {
        Traverse::Continue
    }
    fn exit_exclude_path(&mut self, _path: &'ast ast::ExcludePath) -> Traverse {
        Traverse::Continue
    }
    fn enter_exclude_path_step(&mut self, _step: &'ast ast::ExcludePathStep) -> Traverse {
        Traverse::Continue
    }
    fn exit_exclude_path_step(&mut self, _step: &'ast ast::ExcludePathStep) -> Traverse {
        Traverse::Continue
    }
    fn enter_expr(&mut self, _expr: &'ast ast::Expr) -> Traverse {
        Traverse::Continue
    }
    fn exit_expr(&mut self, _expr: &'ast ast::Expr) -> Traverse {
        Traverse::Continue
    }
    fn enter_lit(&mut self, _lit: &'ast ast::Lit) -> Traverse {
        Traverse::Continue
    }
    fn exit_lit(&mut self, _lit: &'ast ast::Lit) -> Traverse {
        Traverse::Continue
    }
    fn enter_var_ref(&mut self, _var_ref: &'ast ast::VarRef) -> Traverse {
        Traverse::Continue
    }
    fn exit_var_ref(&mut self, _var_ref: &'ast ast::VarRef) -> Traverse {
        Traverse::Continue
    }
    fn enter_bin_op(&mut self, _bin_op: &'ast ast::BinOp) -> Traverse {
        Traverse::Continue
    }
    fn exit_bin_op(&mut self, _bin_op: &'ast ast::BinOp) -> Traverse {
        Traverse::Continue
    }
    fn enter_uni_op(&mut self, _uni_op: &'ast ast::UniOp) -> Traverse {
        Traverse::Continue
    }
    fn exit_uni_op(&mut self, _uni_op: &'ast ast::UniOp) -> Traverse {
        Traverse::Continue
    }
    fn enter_like(&mut self, _like: &'ast ast::Like) -> Traverse {
        Traverse::Continue
    }
    fn exit_like(&mut self, _like: &'ast ast::Like) -> Traverse {
        Traverse::Continue
    }
    fn enter_between(&mut self, _between: &'ast ast::Between) -> Traverse {
        Traverse::Continue
    }
    fn exit_between(&mut self, _between: &'ast ast::Between) -> Traverse {
        Traverse::Continue
    }
    fn enter_in(&mut self, _in: &'ast ast::In) -> Traverse {
        Traverse::Continue
    }
    fn exit_in(&mut self, _in: &'ast ast::In) -> Traverse {
        Traverse::Continue
    }
    fn enter_case(&mut self, _case: &'ast ast::Case) -> Traverse {
        Traverse::Continue
    }
    fn exit_case(&mut self, _case: &'ast ast::Case) -> Traverse {
        Traverse::Continue
    }
    fn enter_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) -> Traverse {
        Traverse::Continue
    }
    fn exit_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) -> Traverse {
        Traverse::Continue
    }
    fn enter_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) -> Traverse {
        Traverse::Continue
    }
    fn exit_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) -> Traverse {
        Traverse::Continue
    }
    fn enter_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) -> Traverse {
        Traverse::Continue
    }
    fn exit_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) -> Traverse {
        Traverse::Continue
    }
    fn enter_struct(&mut self, _struct: &'ast ast::Struct) -> Traverse {
        Traverse::Continue
    }
    fn exit_struct(&mut self, _struct: &'ast ast::Struct) -> Traverse {
        Traverse::Continue
    }
    fn enter_bag(&mut self, _bag: &'ast ast::Bag) -> Traverse {
        Traverse::Continue
    }
    fn exit_bag(&mut self, _bag: &'ast ast::Bag) -> Traverse {
        Traverse::Continue
    }
    fn enter_list(&mut self, _list: &'ast ast::List) -> Traverse {
        Traverse::Continue
    }
    fn exit_list(&mut self, _list: &'ast ast::List) -> Traverse {
        Traverse::Continue
    }
    fn enter_call(&mut self, _call: &'ast ast::Call) -> Traverse {
        Traverse::Continue
    }
    fn exit_call(&mut self, _call: &'ast ast::Call) -> Traverse {
        Traverse::Continue
    }
    fn enter_call_arg(&mut self, _call_arg: &'ast ast::CallArg) -> Traverse {
        Traverse::Continue
    }
    fn exit_call_arg(&mut self, _call_arg: &'ast ast::CallArg) -> Traverse {
        Traverse::Continue
    }
    fn enter_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) -> Traverse {
        Traverse::Continue
    }
    fn exit_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) -> Traverse {
        Traverse::Continue
    }
    fn enter_call_arg_named_type(
        &mut self,
        _call_arg_named_type: &'ast ast::CallArgNamedType,
    ) -> Traverse {
        Traverse::Continue
    }
    fn exit_call_arg_named_type(
        &mut self,
        _call_arg_named_type: &'ast ast::CallArgNamedType,
    ) -> Traverse {
        Traverse::Continue
    }
    fn enter_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) -> Traverse {
        Traverse::Continue
    }
    fn exit_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) -> Traverse {
        Traverse::Continue
    }
    fn enter_path(&mut self, _path: &'ast ast::Path) -> Traverse {
        Traverse::Continue
    }
    fn exit_path(&mut self, _path: &'ast ast::Path) -> Traverse {
        Traverse::Continue
    }
    fn enter_path_step(&mut self, _path_step: &'ast ast::PathStep) -> Traverse {
        Traverse::Continue
    }
    fn exit_path_step(&mut self, _path_step: &'ast ast::PathStep) -> Traverse {
        Traverse::Continue
    }
    fn enter_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) -> Traverse {
        Traverse::Continue
    }
    fn exit_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) -> Traverse {
        Traverse::Continue
    }
    fn enter_let(&mut self, _let: &'ast ast::Let) -> Traverse {
        Traverse::Continue
    }
    fn exit_let(&mut self, _let: &'ast ast::Let) -> Traverse {
        Traverse::Continue
    }
    fn enter_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) -> Traverse {
        Traverse::Continue
    }
    fn exit_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) -> Traverse {
        Traverse::Continue
    }
    fn enter_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Traverse {
        Traverse::Continue
    }
    fn exit_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Traverse {
        Traverse::Continue
    }
    fn enter_from_source(&mut self, _from_clause: &'ast ast::FromSource) -> Traverse {
        Traverse::Continue
    }
    fn exit_from_source(&mut self, _from_clause: &'ast ast::FromSource) -> Traverse {
        Traverse::Continue
    }
    fn enter_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Traverse {
        Traverse::Continue
    }
    fn exit_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Traverse {
        Traverse::Continue
    }
    fn enter_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Traverse {
        Traverse::Continue
    }
    fn exit_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Traverse {
        Traverse::Continue
    }
    fn enter_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Traverse {
        Traverse::Continue
    }
    fn exit_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Traverse {
        Traverse::Continue
    }
    fn enter_join(&mut self, _join: &'ast ast::Join) -> Traverse {
        Traverse::Continue
    }
    fn exit_join(&mut self, _join: &'ast ast::Join) -> Traverse {
        Traverse::Continue
    }
    fn enter_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) -> Traverse {
        Traverse::Continue
    }
    fn exit_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_table(&mut self, _gtable: &'ast ast::GraphTable) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_table(&mut self, _gtable: &'ast ast::GraphTable) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_match(&mut self, _gmatch: &'ast ast::GraphMatch) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_match(&mut self, _gmatch: &'ast ast::GraphMatch) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_pattern(&mut self, _graph_pattern: &'ast ast::GraphPattern) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_pattern(&mut self, _graph_pattern: &'ast ast::GraphPattern) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_path_pattern(
        &mut self,
        _graph_pattern: &'ast ast::GraphPathPattern,
    ) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_path_pattern(&mut self, _graph_pattern: &'ast ast::GraphPathPattern) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_path_sub_pattern(
        &mut self,
        _graph_pattern: &'ast ast::GraphPathSubPattern,
    ) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_path_sub_pattern(
        &mut self,
        _graph_pattern: &'ast ast::GraphPathSubPattern,
    ) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_match_path_pattern(
        &mut self,
        _graph_pattern: &'ast ast::GraphMatchPathPattern,
    ) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_match_path_pattern(
        &mut self,
        _graph_pattern: &'ast ast::GraphMatchPathPattern,
    ) -> Traverse {
        Traverse::Continue
    }

    fn enter_graph_match_path_pattern_quantified(
        &mut self,
        _graph_pattern: &'ast ast::GraphMatchPathPatternQuantified,
    ) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_match_path_pattern_quantified(
        &mut self,
        _graph_pattern: &'ast ast::GraphMatchPathPatternQuantified,
    ) -> Traverse {
        Traverse::Continue
    }
    fn enter_graph_match_node(&mut self, _graph_pattern: &'ast ast::GraphMatchNode) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_match_node(&mut self, _graph_pattern: &'ast ast::GraphMatchNode) -> Traverse {
        Traverse::Continue
    }
    fn enter_graph_match_edge(&mut self, _graph_pattern: &'ast ast::GraphMatchEdge) -> Traverse {
        Traverse::Continue
    }
    fn exit_graph_match_edge(&mut self, _graph_pattern: &'ast ast::GraphMatchEdge) -> Traverse {
        Traverse::Continue
    }

    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) -> Traverse {
        Traverse::Continue
    }
    fn exit_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) -> Traverse {
        Traverse::Continue
    }
    fn enter_group_key(&mut self, _group_key: &'ast ast::GroupKey) -> Traverse {
        Traverse::Continue
    }
    fn exit_group_key(&mut self, _group_key: &'ast ast::GroupKey) -> Traverse {
        Traverse::Continue
    }
    fn enter_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) -> Traverse {
        Traverse::Continue
    }
    fn exit_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) -> Traverse {
        Traverse::Continue
    }
    fn enter_limit_offset_clause(
        &mut self,
        _limit_offset: &'ast ast::LimitOffsetClause,
    ) -> Traverse {
        Traverse::Continue
    }
    fn exit_limit_offset_clause(
        &mut self,
        _limit_offset: &'ast ast::LimitOffsetClause,
    ) -> Traverse {
        Traverse::Continue
    }
    fn enter_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) -> Traverse {
        Traverse::Continue
    }
    fn exit_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) -> Traverse {
        Traverse::Continue
    }
    fn enter_custom_type(&mut self, _custom_type: &'ast ast::CustomType) -> Traverse {
        Traverse::Continue
    }
    fn exit_custom_type(&mut self, _custom_type: &'ast ast::CustomType) -> Traverse {
        Traverse::Continue
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::visit::{Traverse, Visitor};
    use ast::{AstNode, BinOp, BinOpKind, Expr, Lit};
    use partiql_common::node::NodeId;
    use std::ops::AddAssign;

    #[test]
    fn visit_accum() {
        #[derive(Default)]
        struct Accum {
            val: Option<i64>,
        }

        impl<'ast> Visitor<'ast> for Accum {
            fn enter_lit(&mut self, literal: &'ast Lit) -> Traverse {
                match literal {
                    Lit::Int8Lit(l) => self.val.get_or_insert(0i64).add_assign(i64::from(*l)),
                    Lit::Int16Lit(l) => self.val.get_or_insert(0i64).add_assign(i64::from(*l)),
                    Lit::Int32Lit(l) => self.val.get_or_insert(0i64).add_assign(i64::from(*l)),
                    Lit::Int64Lit(l) => self.val.get_or_insert(0i64).add_assign(l),
                    _ => {}
                }
                Traverse::Continue
            }
        }

        fn create_bin_op(op: BinOpKind, lhs: Expr, rhs: Expr) -> Expr {
            Expr::BinOp(AstNode {
                id: NodeId(1),
                node: BinOp {
                    kind: op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            })
        }

        fn create_bin_op_lit(op: BinOpKind, lhs: Lit, rhs: Lit) -> Expr {
            let lhs = Expr::Lit(AstNode {
                id: NodeId(1),
                node: lhs,
            });
            let rhs = Expr::Lit(AstNode {
                id: NodeId(1),
                node: rhs,
            });
            create_bin_op(op, lhs, rhs)
        }

        let lhs = create_bin_op_lit(BinOpKind::Add, Lit::Int32Lit(5), Lit::Int16Lit(4));
        let rhs = create_bin_op_lit(BinOpKind::Mul, Lit::Int8Lit(-20), Lit::Int64Lit(3000));
        let ast = create_bin_op(BinOpKind::Div, lhs, rhs);

        let mut acc = Accum::default();

        use super::Visit;
        assert_eq!(Traverse::Continue, ast.visit(&mut acc));

        let val = acc.val;
        assert!(matches!(val, Some(2989)));
    }
}
