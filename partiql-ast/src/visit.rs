use crate::ast;
use crate::ast::NodeId;

#[derive(PartialEq)]
pub enum Recurse {
    Continue,
    Stop,
}

pub trait Visit {
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Recurse
    where
        V: Visitor<'ast>;
}

impl<T> Visit for ast::AstNode<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Recurse
    where
        V: Visitor<'ast>,
    {
        if v.enter_ast_node(self.id) == Recurse::Stop {
            return Recurse::Stop;
        }
        if self.node.visit(v) == Recurse::Stop {
            return Recurse::Stop;
        }
        v.exit_ast_node(self.id)
    }
}

impl<T> Visit for &T
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Recurse
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
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Recurse
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
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Recurse
    where
        V: Visitor<'ast>,
    {
        if let Some(inner) = self {
            if inner.visit(v) == Recurse::Stop {
                return Recurse::Stop;
            }
        }
        Recurse::Continue
    }
}

impl<T> Visit for Vec<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Recurse
    where
        V: Visitor<'ast>,
    {
        for i in self {
            if i.visit(v) == Recurse::Stop {
                return Recurse::Stop;
            }
        }
        Recurse::Continue
    }
}

pub trait Visitor<'ast> {
    fn enter_ast_node(&mut self, _id: NodeId) -> Recurse {
        Recurse::Continue
    }
    fn exit_ast_node(&mut self, _id: NodeId) -> Recurse {
        Recurse::Continue
    }
    fn enter_item(&mut self, _item: &'ast ast::Item) -> Recurse {
        Recurse::Continue
    }
    fn exit_item(&mut self, _item: &'ast ast::Item) -> Recurse {
        Recurse::Continue
    }
    fn enter_ddl(&mut self, _ddl: &'ast ast::Ddl) -> Recurse {
        Recurse::Continue
    }
    fn exit_ddl(&mut self, _ddl: &'ast ast::Ddl) -> Recurse {
        Recurse::Continue
    }
    fn enter_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) -> Recurse {
        Recurse::Continue
    }
    fn exit_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) -> Recurse {
        Recurse::Continue
    }
    fn enter_create_table(&mut self, _create_table: &'ast ast::CreateTable) -> Recurse {
        Recurse::Continue
    }
    fn exit_create_table(&mut self, _create_table: &'ast ast::CreateTable) -> Recurse {
        Recurse::Continue
    }
    fn enter_drop_table(&mut self, _drop_table: &'ast ast::DropTable) -> Recurse {
        Recurse::Continue
    }
    fn exit_drop_table(&mut self, _drop_table: &'ast ast::DropTable) -> Recurse {
        Recurse::Continue
    }
    fn enter_create_index(&mut self, _create_index: &'ast ast::CreateIndex) -> Recurse {
        Recurse::Continue
    }
    fn exit_create_index(&mut self, _create_index: &'ast ast::CreateIndex) -> Recurse {
        Recurse::Continue
    }
    fn enter_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) -> Recurse {
        Recurse::Continue
    }
    fn exit_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) -> Recurse {
        Recurse::Continue
    }
    fn enter_dml(&mut self, _dml: &'ast ast::Dml) -> Recurse {
        Recurse::Continue
    }
    fn exit_dml(&mut self, _dml: &'ast ast::Dml) -> Recurse {
        Recurse::Continue
    }
    fn enter_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) -> Recurse {
        Recurse::Continue
    }
    fn exit_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) -> Recurse {
        Recurse::Continue
    }
    fn enter_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) -> Recurse {
        Recurse::Continue
    }
    fn exit_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) -> Recurse {
        Recurse::Continue
    }
    fn enter_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) -> Recurse {
        Recurse::Continue
    }
    fn exit_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) -> Recurse {
        Recurse::Continue
    }
    fn enter_insert(&mut self, _insert: &'ast ast::Insert) -> Recurse {
        Recurse::Continue
    }
    fn exit_insert(&mut self, _insert: &'ast ast::Insert) -> Recurse {
        Recurse::Continue
    }
    fn enter_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) -> Recurse {
        Recurse::Continue
    }
    fn exit_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) -> Recurse {
        Recurse::Continue
    }
    fn enter_set(&mut self, _set: &'ast ast::Set) -> Recurse {
        Recurse::Continue
    }
    fn exit_set(&mut self, _set: &'ast ast::Set) -> Recurse {
        Recurse::Continue
    }
    fn enter_assignment(&mut self, _assignment: &'ast ast::Assignment) -> Recurse {
        Recurse::Continue
    }
    fn exit_assignment(&mut self, _assignment: &'ast ast::Assignment) -> Recurse {
        Recurse::Continue
    }
    fn enter_remove(&mut self, _remove: &'ast ast::Remove) -> Recurse {
        Recurse::Continue
    }
    fn exit_remove(&mut self, _remove: &'ast ast::Remove) -> Recurse {
        Recurse::Continue
    }
    fn enter_delete(&mut self, _delete: &'ast ast::Delete) -> Recurse {
        Recurse::Continue
    }
    fn exit_delete(&mut self, _delete: &'ast ast::Delete) -> Recurse {
        Recurse::Continue
    }
    fn enter_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) -> Recurse {
        Recurse::Continue
    }
    fn exit_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) -> Recurse {
        Recurse::Continue
    }
    fn enter_query(&mut self, _query: &'ast ast::Query) -> Recurse {
        Recurse::Continue
    }
    fn exit_query(&mut self, _query: &'ast ast::Query) -> Recurse {
        Recurse::Continue
    }
    fn enter_with_clause(&mut self, _query: &'ast ast::WithClause) -> Recurse {
        Recurse::Continue
    }
    fn exit_with_clause(&mut self, _query: &'ast ast::WithClause) -> Recurse {
        Recurse::Continue
    }
    fn enter_with_element(&mut self, _query: &'ast ast::WithElement) -> Recurse {
        Recurse::Continue
    }
    fn exit_with_element(&mut self, _query: &'ast ast::WithElement) -> Recurse {
        Recurse::Continue
    }
    fn enter_query_set(&mut self, _query_set: &'ast ast::QuerySet) -> Recurse {
        Recurse::Continue
    }
    fn exit_query_set(&mut self, _query_set: &'ast ast::QuerySet) -> Recurse {
        Recurse::Continue
    }
    fn enter_set_expr(&mut self, _set_expr: &'ast ast::SetExpr) -> Recurse {
        Recurse::Continue
    }
    fn exit_set_expr(&mut self, _set_expr: &'ast ast::SetExpr) -> Recurse {
        Recurse::Continue
    }
    fn enter_select(&mut self, _select: &'ast ast::Select) -> Recurse {
        Recurse::Continue
    }
    fn exit_select(&mut self, _select: &'ast ast::Select) -> Recurse {
        Recurse::Continue
    }
    fn enter_query_table(&mut self, _table: &'ast ast::QueryTable) -> Recurse {
        Recurse::Continue
    }
    fn exit_query_table(&mut self, _table: &'ast ast::QueryTable) -> Recurse {
        Recurse::Continue
    }
    fn enter_projection(&mut self, _projection: &'ast ast::Projection) -> Recurse {
        Recurse::Continue
    }
    fn exit_projection(&mut self, _projection: &'ast ast::Projection) -> Recurse {
        Recurse::Continue
    }
    fn enter_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) -> Recurse {
        Recurse::Continue
    }
    fn exit_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) -> Recurse {
        Recurse::Continue
    }
    fn enter_project_item(&mut self, _project_item: &'ast ast::ProjectItem) -> Recurse {
        Recurse::Continue
    }
    fn exit_project_item(&mut self, _project_item: &'ast ast::ProjectItem) -> Recurse {
        Recurse::Continue
    }
    fn enter_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) -> Recurse {
        Recurse::Continue
    }
    fn exit_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) -> Recurse {
        Recurse::Continue
    }
    fn enter_project_all(&mut self, _project_all: &'ast ast::ProjectAll) -> Recurse {
        Recurse::Continue
    }
    fn exit_project_all(&mut self, _project_all: &'ast ast::ProjectAll) -> Recurse {
        Recurse::Continue
    }
    fn enter_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) -> Recurse {
        Recurse::Continue
    }
    fn exit_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) -> Recurse {
        Recurse::Continue
    }
    fn enter_expr(&mut self, _expr: &'ast ast::Expr) -> Recurse {
        Recurse::Continue
    }
    fn exit_expr(&mut self, _expr: &'ast ast::Expr) -> Recurse {
        Recurse::Continue
    }
    fn enter_lit(&mut self, _lit: &'ast ast::Lit) -> Recurse {
        Recurse::Continue
    }
    fn exit_lit(&mut self, _lit: &'ast ast::Lit) -> Recurse {
        Recurse::Continue
    }
    fn enter_var_ref(&mut self, _var_ref: &'ast ast::VarRef) -> Recurse {
        Recurse::Continue
    }
    fn exit_var_ref(&mut self, _var_ref: &'ast ast::VarRef) -> Recurse {
        Recurse::Continue
    }
    fn enter_bin_op(&mut self, _bin_op: &'ast ast::BinOp) -> Recurse {
        Recurse::Continue
    }
    fn exit_bin_op(&mut self, _bin_op: &'ast ast::BinOp) -> Recurse {
        Recurse::Continue
    }
    fn enter_uni_op(&mut self, _uni_op: &'ast ast::UniOp) -> Recurse {
        Recurse::Continue
    }
    fn exit_uni_op(&mut self, _uni_op: &'ast ast::UniOp) -> Recurse {
        Recurse::Continue
    }
    fn enter_like(&mut self, _like: &'ast ast::Like) -> Recurse {
        Recurse::Continue
    }
    fn exit_like(&mut self, _like: &'ast ast::Like) -> Recurse {
        Recurse::Continue
    }
    fn enter_between(&mut self, _between: &'ast ast::Between) -> Recurse {
        Recurse::Continue
    }
    fn exit_between(&mut self, _between: &'ast ast::Between) -> Recurse {
        Recurse::Continue
    }
    fn enter_in(&mut self, _in: &'ast ast::In) -> Recurse {
        Recurse::Continue
    }
    fn exit_in(&mut self, _in: &'ast ast::In) -> Recurse {
        Recurse::Continue
    }
    fn enter_case(&mut self, _case: &'ast ast::Case) -> Recurse {
        Recurse::Continue
    }
    fn exit_case(&mut self, _case: &'ast ast::Case) -> Recurse {
        Recurse::Continue
    }
    fn enter_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) -> Recurse {
        Recurse::Continue
    }
    fn exit_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) -> Recurse {
        Recurse::Continue
    }
    fn enter_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) -> Recurse {
        Recurse::Continue
    }
    fn exit_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) -> Recurse {
        Recurse::Continue
    }
    fn enter_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) -> Recurse {
        Recurse::Continue
    }
    fn exit_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) -> Recurse {
        Recurse::Continue
    }
    fn enter_struct(&mut self, _struct: &'ast ast::Struct) -> Recurse {
        Recurse::Continue
    }
    fn exit_struct(&mut self, _struct: &'ast ast::Struct) -> Recurse {
        Recurse::Continue
    }
    fn enter_bag(&mut self, _bag: &'ast ast::Bag) -> Recurse {
        Recurse::Continue
    }
    fn exit_bag(&mut self, _bag: &'ast ast::Bag) -> Recurse {
        Recurse::Continue
    }
    fn enter_list(&mut self, _list: &'ast ast::List) -> Recurse {
        Recurse::Continue
    }
    fn exit_list(&mut self, _list: &'ast ast::List) -> Recurse {
        Recurse::Continue
    }
    fn enter_sexp(&mut self, _sexp: &'ast ast::Sexp) -> Recurse {
        Recurse::Continue
    }
    fn exit_sexp(&mut self, _sexp: &'ast ast::Sexp) -> Recurse {
        Recurse::Continue
    }
    fn enter_call(&mut self, _call: &'ast ast::Call) -> Recurse {
        Recurse::Continue
    }
    fn exit_call(&mut self, _call: &'ast ast::Call) -> Recurse {
        Recurse::Continue
    }
    fn enter_call_arg(&mut self, _call_arg: &'ast ast::CallArg) -> Recurse {
        Recurse::Continue
    }
    fn exit_call_arg(&mut self, _call_arg: &'ast ast::CallArg) -> Recurse {
        Recurse::Continue
    }
    fn enter_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) -> Recurse {
        Recurse::Continue
    }
    fn exit_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) -> Recurse {
        Recurse::Continue
    }
    fn enter_call_arg_named_type(
        &mut self,
        _call_arg_named_type: &'ast ast::CallArgNamedType,
    ) -> Recurse {
        Recurse::Continue
    }
    fn exit_call_arg_named_type(
        &mut self,
        _call_arg_named_type: &'ast ast::CallArgNamedType,
    ) -> Recurse {
        Recurse::Continue
    }
    fn enter_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) -> Recurse {
        Recurse::Continue
    }
    fn exit_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) -> Recurse {
        Recurse::Continue
    }
    fn enter_path(&mut self, _path: &'ast ast::Path) -> Recurse {
        Recurse::Continue
    }
    fn exit_path(&mut self, _path: &'ast ast::Path) -> Recurse {
        Recurse::Continue
    }
    fn enter_path_step(&mut self, _path_step: &'ast ast::PathStep) -> Recurse {
        Recurse::Continue
    }
    fn exit_path_step(&mut self, _path_step: &'ast ast::PathStep) -> Recurse {
        Recurse::Continue
    }
    fn enter_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) -> Recurse {
        Recurse::Continue
    }
    fn exit_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) -> Recurse {
        Recurse::Continue
    }
    fn enter_let(&mut self, _let: &'ast ast::Let) -> Recurse {
        Recurse::Continue
    }
    fn exit_let(&mut self, _let: &'ast ast::Let) -> Recurse {
        Recurse::Continue
    }
    fn enter_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) -> Recurse {
        Recurse::Continue
    }
    fn exit_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) -> Recurse {
        Recurse::Continue
    }
    fn enter_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Recurse {
        Recurse::Continue
    }
    fn exit_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Recurse {
        Recurse::Continue
    }
    fn enter_from_source(&mut self, _from_clause: &'ast ast::FromSource) -> Recurse {
        Recurse::Continue
    }
    fn exit_from_source(&mut self, _from_clause: &'ast ast::FromSource) -> Recurse {
        Recurse::Continue
    }
    fn enter_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Recurse {
        Recurse::Continue
    }
    fn exit_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Recurse {
        Recurse::Continue
    }
    fn enter_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Recurse {
        Recurse::Continue
    }
    fn exit_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Recurse {
        Recurse::Continue
    }
    fn enter_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Recurse {
        Recurse::Continue
    }
    fn exit_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Recurse {
        Recurse::Continue
    }
    fn enter_join(&mut self, _join: &'ast ast::Join) -> Recurse {
        Recurse::Continue
    }
    fn exit_join(&mut self, _join: &'ast ast::Join) -> Recurse {
        Recurse::Continue
    }
    fn enter_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) -> Recurse {
        Recurse::Continue
    }
    fn exit_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) -> Recurse {
        Recurse::Continue
    }
    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) -> Recurse {
        Recurse::Continue
    }
    fn exit_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) -> Recurse {
        Recurse::Continue
    }
    fn enter_group_key(&mut self, _group_key: &'ast ast::GroupKey) -> Recurse {
        Recurse::Continue
    }
    fn exit_group_key(&mut self, _group_key: &'ast ast::GroupKey) -> Recurse {
        Recurse::Continue
    }
    fn enter_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) -> Recurse {
        Recurse::Continue
    }
    fn exit_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) -> Recurse {
        Recurse::Continue
    }
    fn enter_limit_offset_clause(
        &mut self,
        _limit_offset: &'ast ast::LimitOffsetClause,
    ) -> Recurse {
        Recurse::Continue
    }
    fn exit_limit_offset_clause(&mut self, _limit_offset: &'ast ast::LimitOffsetClause) -> Recurse {
        Recurse::Continue
    }
    fn enter_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) -> Recurse {
        Recurse::Continue
    }
    fn exit_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) -> Recurse {
        Recurse::Continue
    }
    fn enter_custom_type(&mut self, _custom_type: &'ast ast::CustomType) -> Recurse {
        Recurse::Continue
    }
    fn exit_custom_type(&mut self, _custom_type: &'ast ast::CustomType) -> Recurse {
        Recurse::Continue
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::visit::{Recurse, Visitor};
    use ast::{AstNode, BinOp, BinOpKind, Expr, Lit, NodeId};
    use std::ops::AddAssign;

    #[test]
    fn visit_accum() {
        #[derive(Default)]
        struct Accum {
            val: Option<i64>,
        }

        impl<'ast> Visitor<'ast> for Accum {
            fn enter_lit(&mut self, literal: &'ast Lit) -> Recurse {
                match literal {
                    Lit::Int8Lit(l) => self.val.get_or_insert(0i64).add_assign(*l as i64),
                    Lit::Int16Lit(l) => self.val.get_or_insert(0i64).add_assign(*l as i64),
                    Lit::Int32Lit(l) => self.val.get_or_insert(0i64).add_assign(*l as i64),
                    Lit::Int64Lit(l) => self.val.get_or_insert(0i64).add_assign(l),
                    _ => {}
                }
                Recurse::Continue
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
        ast.visit(&mut acc);
        // todo error handling

        let val = acc.val;
        assert!(matches!(val, Some(2989)));
    }
}
