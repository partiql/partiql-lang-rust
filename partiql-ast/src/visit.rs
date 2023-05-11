use crate::ast;
use crate::ast::NodeId;

pub trait Visit {
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor<'ast>;
}

impl<T> Visit for ast::AstNode<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor<'ast>
    {
        v.enter_ast_node(self.id)?;
        self.node.visit(v)?;
        v.exit_ast_node(self.id)
    }
}

impl<T> Visit for &T
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Result<(), V::Error>
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
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Result<(), V::Error>
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
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor<'ast>,
    {
        if let Some(inner) = self {
            inner.visit(v)?
        }
        Ok(())
    }
}

impl<T> Visit for Vec<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor<'ast>,
    {
        for i in self {
            i.visit(v)?
        }
        Ok(())
    }
}

pub trait Visitor<'ast> {
    type Error;

    fn enter_ast_node(&mut self, _id: NodeId) -> Result<(), Self::Error> { Ok(()) }
    fn exit_ast_node(&mut self, _id: NodeId) -> Result<(), Self::Error> { Ok(()) }
    fn enter_item(&mut self, _item: &'ast ast::Item) -> Result<(), Self::Error> { Ok(()) }
    fn exit_item(&mut self, _item: &'ast ast::Item) -> Result<(), Self::Error> { Ok(()) }
    fn enter_ddl(&mut self, _ddl: &'ast ast::Ddl) -> Result<(), Self::Error> { Ok(()) }
    fn exit_ddl(&mut self, _ddl: &'ast ast::Ddl) -> Result<(), Self::Error> { Ok(()) }
    fn enter_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) -> Result<(), Self::Error> { Ok(()) }
    fn exit_ddl_op(&mut self, _ddl_op: &'ast ast::DdlOp) -> Result<(), Self::Error> { Ok(()) }
    fn enter_create_table(&mut self, _create_table: &'ast ast::CreateTable) -> Result<(), Self::Error> { Ok(()) }
    fn exit_create_table(&mut self, _create_table: &'ast ast::CreateTable) -> Result<(), Self::Error> { Ok(()) }
    fn enter_drop_table(&mut self, _drop_table: &'ast ast::DropTable) -> Result<(), Self::Error> { Ok(()) }
    fn exit_drop_table(&mut self, _drop_table: &'ast ast::DropTable) -> Result<(), Self::Error> { Ok(()) }
    fn enter_create_index(&mut self, _create_index: &'ast ast::CreateIndex) -> Result<(), Self::Error> { Ok(()) }
    fn exit_create_index(&mut self, _create_index: &'ast ast::CreateIndex) -> Result<(), Self::Error> { Ok(()) }
    fn enter_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) -> Result<(), Self::Error> { Ok(()) }
    fn exit_drop_index(&mut self, _drop_index: &'ast ast::DropIndex) -> Result<(), Self::Error> { Ok(()) }
    fn enter_dml(&mut self, _dml: &'ast ast::Dml) -> Result<(), Self::Error> { Ok(()) }
    fn exit_dml(&mut self, _dml: &'ast ast::Dml) -> Result<(), Self::Error> { Ok(()) }
    fn enter_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) -> Result<(), Self::Error> { Ok(()) }
    fn exit_dml_op(&mut self, _dml_op: &'ast ast::DmlOp) -> Result<(), Self::Error> { Ok(()) }
    fn enter_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_returning_expr(&mut self, _returning_expr: &'ast ast::ReturningExpr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) -> Result<(), Self::Error> { Ok(()) }
    fn exit_returning_elem(&mut self, _returning_elem: &'ast ast::ReturningElem) -> Result<(), Self::Error> { Ok(()) }
    fn enter_insert(&mut self, _insert: &'ast ast::Insert) -> Result<(), Self::Error> { Ok(()) }
    fn exit_insert(&mut self, _insert: &'ast ast::Insert) -> Result<(), Self::Error> { Ok(()) }
    fn enter_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) -> Result<(), Self::Error> { Ok(()) }
    fn exit_insert_value(&mut self, _insert_value: &'ast ast::InsertValue) -> Result<(), Self::Error> { Ok(()) }
    fn enter_set(&mut self, _set: &'ast ast::Set) -> Result<(), Self::Error> { Ok(()) }
    fn exit_set(&mut self, _set: &'ast ast::Set) -> Result<(), Self::Error> { Ok(()) }
    fn enter_assignment(&mut self, _assignment: &'ast ast::Assignment) -> Result<(), Self::Error> { Ok(()) }
    fn exit_assignment(&mut self, _assignment: &'ast ast::Assignment) -> Result<(), Self::Error> { Ok(()) }
    fn enter_remove(&mut self, _remove: &'ast ast::Remove) -> Result<(), Self::Error> { Ok(()) }
    fn exit_remove(&mut self, _remove: &'ast ast::Remove) -> Result<(), Self::Error> { Ok(()) }
    fn enter_delete(&mut self, _delete: &'ast ast::Delete) -> Result<(), Self::Error> { Ok(()) }
    fn exit_delete(&mut self, _delete: &'ast ast::Delete) -> Result<(), Self::Error> { Ok(()) }
    fn enter_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) -> Result<(), Self::Error> { Ok(()) }
    fn exit_on_conflict(&mut self, _on_conflict: &'ast ast::OnConflict) -> Result<(), Self::Error> { Ok(()) }
    fn enter_query(&mut self, _query: &'ast ast::Query) -> Result<(), Self::Error> { Ok(()) }
    fn exit_query(&mut self, _query: &'ast ast::Query) -> Result<(), Self::Error> { Ok(()) }
    fn enter_with_clause(&mut self, _query: &'ast ast::WithClause) -> Result<(), Self::Error> { Ok(()) }
    fn exit_with_clause(&mut self, _query: &'ast ast::WithClause) -> Result<(), Self::Error> { Ok(()) }
    fn enter_with_element(&mut self, _query: &'ast ast::WithElement) -> Result<(), Self::Error> { Ok(()) }
    fn exit_with_element(&mut self, _query: &'ast ast::WithElement) -> Result<(), Self::Error> { Ok(()) }
    fn enter_query_set(&mut self, _query_set: &'ast ast::QuerySet) -> Result<(), Self::Error> { Ok(()) }
    fn exit_query_set(&mut self, _query_set: &'ast ast::QuerySet) -> Result<(), Self::Error> { Ok(()) }
    fn enter_set_expr(&mut self, _set_expr: &'ast ast::SetExpr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_set_expr(&mut self, _set_expr: &'ast ast::SetExpr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_select(&mut self, _select: &'ast ast::Select) -> Result<(), Self::Error> { Ok(()) }
    fn exit_select(&mut self, _select: &'ast ast::Select) -> Result<(), Self::Error> { Ok(()) }
    fn enter_query_table(&mut self, _table: &'ast ast::QueryTable) -> Result<(), Self::Error> { Ok(()) }
    fn exit_query_table(&mut self, _table: &'ast ast::QueryTable) -> Result<(), Self::Error> { Ok(()) }
    fn enter_projection(&mut self, _projection: &'ast ast::Projection) -> Result<(), Self::Error> { Ok(()) }
    fn exit_projection(&mut self, _projection: &'ast ast::Projection) -> Result<(), Self::Error> { Ok(()) }
    fn enter_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) -> Result<(), Self::Error> { Ok(()) }
    fn exit_projection_kind(&mut self, _projection_kind: &'ast ast::ProjectionKind) -> Result<(), Self::Error> { Ok(()) }
    fn enter_project_item(&mut self, _project_item: &'ast ast::ProjectItem) -> Result<(), Self::Error> { Ok(()) }
    fn exit_project_item(&mut self, _project_item: &'ast ast::ProjectItem) -> Result<(), Self::Error> { Ok(()) }
    fn enter_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) -> Result<(), Self::Error> { Ok(()) }
    fn exit_project_pivot(&mut self, _project_pivot: &'ast ast::ProjectPivot) -> Result<(), Self::Error> { Ok(()) }
    fn enter_project_all(&mut self, _project_all: &'ast ast::ProjectAll) -> Result<(), Self::Error> { Ok(()) }
    fn exit_project_all(&mut self, _project_all: &'ast ast::ProjectAll) -> Result<(), Self::Error> { Ok(()) }
    fn enter_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_project_expr(&mut self, _project_expr: &'ast ast::ProjectExpr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_expr(&mut self, _expr: &'ast ast::Expr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_expr(&mut self, _expr: &'ast ast::Expr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_lit(&mut self, _lit: &'ast ast::Lit) -> Result<(), Self::Error> { Ok(()) }
    fn exit_lit(&mut self, _lit: &'ast ast::Lit) -> Result<(), Self::Error> { Ok(()) }
    fn enter_var_ref(&mut self, _var_ref: &'ast ast::VarRef) -> Result<(), Self::Error> { Ok(()) }
    fn exit_var_ref(&mut self, _var_ref: &'ast ast::VarRef) -> Result<(), Self::Error> { Ok(()) }
    fn enter_bin_op(&mut self, _bin_op: &'ast ast::BinOp) -> Result<(), Self::Error> { Ok(()) }
    fn exit_bin_op(&mut self, _bin_op: &'ast ast::BinOp) -> Result<(), Self::Error> { Ok(()) }
    fn enter_uni_op(&mut self, _uni_op: &'ast ast::UniOp) -> Result<(), Self::Error> { Ok(()) }
    fn exit_uni_op(&mut self, _uni_op: &'ast ast::UniOp) -> Result<(), Self::Error> { Ok(()) }
    fn enter_like(&mut self, _like: &'ast ast::Like) -> Result<(), Self::Error> { Ok(()) }
    fn exit_like(&mut self, _like: &'ast ast::Like) -> Result<(), Self::Error> { Ok(()) }
    fn enter_between(&mut self, _between: &'ast ast::Between) -> Result<(), Self::Error> { Ok(()) }
    fn exit_between(&mut self, _between: &'ast ast::Between) -> Result<(), Self::Error> { Ok(()) }
    fn enter_in(&mut self, _in: &'ast ast::In) -> Result<(), Self::Error> { Ok(()) }
    fn exit_in(&mut self, _in: &'ast ast::In) -> Result<(), Self::Error> { Ok(()) }
    fn enter_case(&mut self, _case: &'ast ast::Case) -> Result<(), Self::Error> { Ok(()) }
    fn exit_case(&mut self, _case: &'ast ast::Case) -> Result<(), Self::Error> { Ok(()) }
    fn enter_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) -> Result<(), Self::Error> { Ok(()) }
    fn exit_simple_case(&mut self, _simple_case: &'ast ast::SimpleCase) -> Result<(), Self::Error> { Ok(()) }
    fn enter_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) -> Result<(), Self::Error> { Ok(()) }
    fn exit_searched_case(&mut self, _searched_case: &'ast ast::SearchedCase) -> Result<(), Self::Error> { Ok(()) }
    fn enter_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) -> Result<(), Self::Error> { Ok(()) }
    fn exit_expr_pair(&mut self, _expr_pair: &'ast ast::ExprPair) -> Result<(), Self::Error> { Ok(()) }
    fn enter_struct(&mut self, _struct: &'ast ast::Struct) -> Result<(), Self::Error> { Ok(()) }
    fn exit_struct(&mut self, _struct: &'ast ast::Struct) -> Result<(), Self::Error> { Ok(()) }
    fn enter_bag(&mut self, _bag: &'ast ast::Bag) -> Result<(), Self::Error> { Ok(()) }
    fn exit_bag(&mut self, _bag: &'ast ast::Bag) -> Result<(), Self::Error> { Ok(()) }
    fn enter_list(&mut self, _list: &'ast ast::List) -> Result<(), Self::Error> { Ok(()) }
    fn exit_list(&mut self, _list: &'ast ast::List) -> Result<(), Self::Error> { Ok(()) }
    fn enter_sexp(&mut self, _sexp: &'ast ast::Sexp) -> Result<(), Self::Error> { Ok(()) }
    fn exit_sexp(&mut self, _sexp: &'ast ast::Sexp) -> Result<(), Self::Error> { Ok(()) }
    fn enter_call(&mut self, _call: &'ast ast::Call) -> Result<(), Self::Error> { Ok(()) }
    fn exit_call(&mut self, _call: &'ast ast::Call) -> Result<(), Self::Error> { Ok(()) }
    fn enter_call_arg(&mut self, _call_arg: &'ast ast::CallArg) -> Result<(), Self::Error> { Ok(()) }
    fn exit_call_arg(&mut self, _call_arg: &'ast ast::CallArg) -> Result<(), Self::Error> { Ok(()) }
    fn enter_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) -> Result<(), Self::Error> { Ok(()) }
    fn exit_call_arg_named(&mut self, _call_arg_named: &'ast ast::CallArgNamed) -> Result<(), Self::Error> { Ok(()) }
    fn enter_call_arg_named_type(&mut self, _call_arg_named_type: &'ast ast::CallArgNamedType) -> Result<(), Self::Error> { Ok(()) }
    fn exit_call_arg_named_type(&mut self, _call_arg_named_type: &'ast ast::CallArgNamedType) -> Result<(), Self::Error> { Ok(()) }
    fn enter_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) -> Result<(), Self::Error> { Ok(()) }
    fn exit_call_agg(&mut self, _call_agg: &'ast ast::CallAgg) -> Result<(), Self::Error> { Ok(()) }
    fn enter_path(&mut self, _path: &'ast ast::Path) -> Result<(), Self::Error> { Ok(()) }
    fn exit_path(&mut self, _path: &'ast ast::Path) -> Result<(), Self::Error> { Ok(()) }
    fn enter_path_step(&mut self, _path_step: &'ast ast::PathStep) -> Result<(), Self::Error> { Ok(()) }
    fn exit_path_step(&mut self, _path_step: &'ast ast::PathStep) -> Result<(), Self::Error> { Ok(()) }
    fn enter_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_path_expr(&mut self, _path_expr: &'ast ast::PathExpr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_let(&mut self, _let: &'ast ast::Let) -> Result<(), Self::Error> { Ok(()) }
    fn exit_let(&mut self, _let: &'ast ast::Let) -> Result<(), Self::Error> { Ok(()) }
    fn enter_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) -> Result<(), Self::Error> { Ok(()) }
    fn exit_let_binding(&mut self, _let_binding: &'ast ast::LetBinding) -> Result<(), Self::Error> { Ok(()) }
    fn enter_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Result<(), Self::Error> { Ok(()) }
    fn exit_from_clause(&mut self, _from_clause: &'ast ast::FromClause) -> Result<(), Self::Error> { Ok(()) }
    fn enter_from_source(&mut self, _from_clause: &'ast ast::FromSource) -> Result<(), Self::Error> { Ok(()) }
    fn exit_from_source(&mut self, _from_clause: &'ast ast::FromSource) -> Result<(), Self::Error> { Ok(()) }
    fn enter_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Result<(), Self::Error> { Ok(()) }
    fn exit_where_clause(&mut self, _where_clause: &'ast ast::WhereClause) -> Result<(), Self::Error> { Ok(()) }
    fn enter_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Result<(), Self::Error> { Ok(()) }
    fn exit_having_clause(&mut self, _having_clause: &'ast ast::HavingClause) -> Result<(), Self::Error> { Ok(()) }
    fn enter_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Result<(), Self::Error> { Ok(()) }
    fn exit_from_let(&mut self, _from_let: &'ast ast::FromLet) -> Result<(), Self::Error> { Ok(()) }
    fn enter_join(&mut self, _join: &'ast ast::Join) -> Result<(), Self::Error> { Ok(()) }
    fn exit_join(&mut self, _join: &'ast ast::Join) -> Result<(), Self::Error> { Ok(()) }
    fn enter_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) -> Result<(), Self::Error> { Ok(()) }
    fn exit_join_spec(&mut self, _join_spec: &'ast ast::JoinSpec) -> Result<(), Self::Error> { Ok(()) }
    fn enter_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_group_by_expr(&mut self, _group_by_expr: &'ast ast::GroupByExpr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_group_key(&mut self, _group_key: &'ast ast::GroupKey) -> Result<(), Self::Error> { Ok(()) }
    fn exit_group_key(&mut self, _group_key: &'ast ast::GroupKey) -> Result<(), Self::Error> { Ok(()) }
    fn enter_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) -> Result<(), Self::Error> { Ok(()) }
    fn exit_order_by_expr(&mut self, _order_by_expr: &'ast ast::OrderByExpr) -> Result<(), Self::Error> { Ok(()) }
    fn enter_limit_offset_clause(&mut self, _limit_offset: &'ast ast::LimitOffsetClause) -> Result<(), Self::Error> { Ok(()) }
    fn exit_limit_offset_clause(&mut self, _limit_offset: &'ast ast::LimitOffsetClause) -> Result<(), Self::Error> { Ok(()) }
    fn enter_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) -> Result<(), Self::Error> { Ok(()) }
    fn exit_sort_spec(&mut self, _sort_spec: &'ast ast::SortSpec) -> Result<(), Self::Error> { Ok(()) }
    fn enter_custom_type(&mut self, _custom_type: &'ast ast::CustomType) -> Result<(), Self::Error> { Ok(()) }
    fn exit_custom_type(&mut self, _custom_type: &'ast ast::CustomType) -> Result<(), Self::Error> { Ok(()) }
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
            type Error = ();

            fn enter_ast_node(&mut self, _id: NodeId) -> Result<(), Self::Error> {
                todo!()
            }

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
