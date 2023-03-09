use crate::ast;
use crate::ast::NodeId;

pub trait Visit {
    fn visit<'ast, V>(&'ast self, _: &mut V)
    where
        V: Visitor<'ast>;
}

impl<T> Visit for ast::AstNode<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V)
    where
        V: Visitor<'ast>,
    {
        v.enter_ast_node(self.id);
        self.node.visit(v);
        v.exit_ast_node(self.id);
    }
}

impl<T> Visit for &T
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V)
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
    fn visit<'ast, V>(&'ast self, v: &mut V)
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
    fn visit<'ast, V>(&'ast self, v: &mut V)
    where
        V: Visitor<'ast>,
    {
        if let Some(inner) = self {
            inner.visit(v)
        }
    }
}

impl<T> Visit for Vec<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V)
    where
        V: Visitor<'ast>,
    {
        for i in self {
            i.visit(v)
        }
    }
}

pub trait Visitor<'ast> {
    fn enter_ast_node(&mut self, _id: NodeId) {}
    fn exit_ast_node(&mut self, _id: NodeId) {}

    fn enter_item(&mut self, _item: &'ast ast::Item) {}
    fn exit_item(&mut self, _item: &'ast ast::Item) {}

    fn enter_ddl(&mut self, _ddl: &'ast ast::Ddl) {}
    fn exit_ddl(&mut self, _ddl: &'ast ast::Ddl) {}
    fn enter_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) {}
    fn exit_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) {}
    fn enter_create_table(&mut self, _create_table: &'ast ast::CreateTable) {}
    fn exit_create_table(&mut self, _create_table: &'ast ast::CreateTable) {}
    fn enter_drop_table(&mut self, _drop_table: &'ast ast::DropTable) {}
    fn exit_drop_table(&mut self, _drop_table: &'ast ast::DropTable) {}
    fn enter_create_index(&mut self, _create_index: &'ast ast::CreateIndex) {}
    fn exit_create_index(&mut self, _create_index: &'ast ast::CreateIndex) {}
    fn enter_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) {}
    fn exit_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) {}

    fn enter_dml(&mut self, _dml: &'ast ast::Dml) {}
    fn exit_dml(&mut self, _dml: &'ast ast::Dml) {}
    fn enter_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) {}
    fn exit_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) {}
    fn enter_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) {}
    fn exit_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) {}
    fn enter_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) {}
    fn exit_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) {}
    fn enter_insert(&mut self, _insert: &'ast ast::Insert) {}
    fn exit_insert(&mut self, _insert: &'ast ast::Insert) {}
    fn enter_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) {}
    fn exit_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) {}
    fn enter_set(&mut self, _set: &'ast ast::Set) {}
    fn exit_set(&mut self, _set: &'ast ast::Set) {}
    fn enter_assignment(&mut self, _assignment: &'ast ast::Assignment) {}
    fn exit_assignment(&mut self, _assignment: &'ast ast::Assignment) {}
    fn enter_remove(&mut self, _remove: &'ast ast::Remove) {}
    fn exit_remove(&mut self, _remove: &'ast ast::Remove) {}
    fn enter_delete(&mut self, _delete: &'ast ast::Delete) {}
    fn exit_delete(&mut self, _delete: &'ast ast::Delete) {}
    fn enter_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) {}
    fn exit_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) {}

    fn enter_query(&mut self, _query: &'ast ast::Query) {}
    fn exit_query(&mut self, _query: &'ast ast::Query) {}
    fn enter_with_clause(&mut self, _query: &'ast ast::WithClause) {}
    fn exit_with_clause(&mut self, _query: &'ast ast::WithClause) {}
    fn enter_with_element(&mut self, _query: &'ast ast::WithElement) {}
    fn exit_with_element(&mut self, _query: &'ast ast::WithElement) {}
    fn enter_query_set(&mut self, _query_set: &'ast ast::QuerySet) {}
    fn exit_query_set(&mut self, _query_set: &'ast ast::QuerySet) {}
    fn enter_set_expr(&mut self, _set_expr: &'ast ast::SetExpr) {}
    fn exit_set_expr(&mut self, _set_expr: &'ast ast::SetExpr) {}
    fn enter_select(&mut self, _select: &'ast ast::Select) {}
    fn exit_select(&mut self, _select: &'ast ast::Select) {}
    fn enter_projection(&mut self, _projection: &'ast ast::Projection) {}
    fn exit_projection(&mut self, _projection: &'ast ast::Projection) {}
    fn enter_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) {}
    fn exit_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) {}
    fn enter_project_item(&mut self, _project_item: &'ast ast::ProjectItem) {}
    fn exit_project_item(&mut self, _project_item: &'ast ast::ProjectItem) {}
    fn enter_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) {}
    fn exit_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) {}
    fn enter_project_all(&mut self, _project_all: &'ast ast::ProjectAll) {}
    fn exit_project_all(&mut self, _project_all: &'ast ast::ProjectAll) {}
    fn enter_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) {}
    fn exit_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) {}
    fn enter_expr(&mut self, _expr: &'ast ast::Expr) {}
    fn exit_expr(&mut self, _expr: &'ast ast::Expr) {}
    fn enter_lit(&mut self, _lit: &'ast ast::Lit) {}
    fn exit_lit(&mut self, _lit: &'ast ast::Lit) {}
    fn enter_var_ref(&mut self, _var_ref: &'ast ast::VarRef) {}
    fn exit_var_ref(&mut self, _var_ref: &'ast ast::VarRef) {}
    fn enter_bin_op(&mut self, _bin_op: &'ast ast::BinOp) {}
    fn exit_bin_op(&mut self, _bin_op: &'ast ast::BinOp) {}
    fn enter_uni_op(&mut self, _uni_op: &'ast ast::UniOp) {}
    fn exit_uni_op(&mut self, _uni_op: &'ast ast::UniOp) {}
    fn enter_like(&mut self, _like: &'ast ast::Like) {}
    fn exit_like(&mut self, _like: &'ast ast::Like) {}
    fn enter_between(&mut self, _between: &'ast ast::Between) {}
    fn exit_between(&mut self, _between: &'ast ast::Between) {}
    fn enter_in(&mut self, _in: &'ast ast::In) {}
    fn exit_in(&mut self, _in: &'ast ast::In) {}
    fn enter_case(&mut self, _case: &'ast ast::Case) {}
    fn exit_case(&mut self, _case: &'ast ast::Case) {}
    fn enter_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) {}
    fn exit_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) {}
    fn enter_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) {}
    fn exit_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) {}
    fn enter_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) {}
    fn exit_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) {}
    fn enter_struct(&mut self, _struct: &'ast ast::Struct) {}
    fn exit_struct(&mut self, _struct: &'ast ast::Struct) {}
    fn enter_bag(&mut self, _bag: &'ast ast::Bag) {}
    fn exit_bag(&mut self, _bag: &'ast ast::Bag) {}
    fn enter_list(&mut self, _list: &'ast ast::List) {}
    fn exit_list(&mut self, _list: &'ast ast::List) {}
    fn enter_sexp(&mut self, _sexp: &'ast ast::Sexp) {}
    fn exit_sexp(&mut self, _sexp: &'ast ast::Sexp) {}
    fn enter_call(&mut self, _call: &'ast ast::Call) {}
    fn exit_call(&mut self, _call: &'ast ast::Call) {}
    fn enter_call_arg(&mut self, _call_arg: &'ast ast::CallArg) {}
    fn exit_call_arg(&mut self, _call_arg: &'ast ast::CallArg) {}
    fn enter_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) {}
    fn exit_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) {}
    fn enter_call_arg_named_type(&mut self, _call_arg_named_type: &'ast ast::CallArgNamedType) {}
    fn exit_call_arg_named_type(&mut self, _call_arg_named_type: &'ast ast::CallArgNamedType) {}
    fn enter_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) {}
    fn exit_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) {}
    fn enter_path(&mut self, _path: &'ast ast::Path) {}
    fn exit_path(&mut self, _path: &'ast ast::Path) {}
    fn enter_path_step(&mut self, _path_step: &'ast ast::PathStep) {}
    fn exit_path_step(&mut self, _path_step: &'ast ast::PathStep) {}
    fn enter_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) {}
    fn exit_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) {}
    fn enter_let(&mut self, _let: &'ast ast::Let) {}
    fn exit_let(&mut self, _let: &'ast ast::Let) {}
    fn enter_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) {}
    fn exit_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) {}
    fn enter_from_clause(&mut self, _from_clause: &'ast ast::FromClause) {}
    fn exit_from_clause(&mut self, _from_clause: &'ast ast::FromClause) {}
    fn enter_from_source(&mut self, _from_clause: &'ast ast::FromSource) {}
    fn exit_from_source(&mut self, _from_clause: &'ast ast::FromSource) {}
    fn enter_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) {}
    fn exit_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) {}
    fn enter_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) {}
    fn exit_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) {}
    fn enter_from_let(&mut self, _from_let: &'ast ast::FromLet) {}
    fn exit_from_let(&mut self, _from_let: &'ast ast::FromLet) {}
    fn enter_join(&mut self, _join: &'ast ast::Join) {}
    fn exit_join(&mut self, _join: &'ast ast::Join) {}
    fn enter_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) {}
    fn exit_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) {}
    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) {}
    fn exit_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) {}
    fn enter_group_key(&mut self, _group_key: &'ast ast::GroupKey) {}
    fn exit_group_key(&mut self, _group_key: &'ast ast::GroupKey) {}
    fn enter_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) {}
    fn exit_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) {}
    fn enter_limit_offset_clause(&mut self, _limit_offset: &'ast ast::LimitOffsetClause) {}
    fn exit_limit_offset_clause(&mut self, _limit_offset: &'ast ast::LimitOffsetClause) {}
    fn enter_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) {}
    fn exit_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) {}
    fn enter_custom_type(&mut self, _custom_type: &'ast ast::CustomType) {}
    fn exit_custom_type(&mut self, _custom_type: &'ast ast::CustomType) {}
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::visit::Visitor;
    use ast::{AstNode, BinOp, BinOpKind, Expr, Lit, NodeId};
    use std::ops::AddAssign;

    #[test]
    fn visit_accum() {
        #[derive(Default)]
        struct Accum {
            val: Option<i64>,
        }

        impl<'ast> Visitor<'ast> for Accum {
            fn enter_lit(&mut self, literal: &'ast Lit) {
                match literal {
                    Lit::Int8Lit(l) => self.val.get_or_insert(0i64).add_assign(*l as i64),
                    Lit::Int16Lit(l) => self.val.get_or_insert(0i64).add_assign(*l as i64),
                    Lit::Int32Lit(l) => self.val.get_or_insert(0i64).add_assign(*l as i64),
                    Lit::Int64Lit(l) => self.val.get_or_insert(0i64).add_assign(l),
                    _ => {}
                }
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

        let val = acc.val;
        assert!(matches!(val, Some(2989)));
    }
}
