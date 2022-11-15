use partiql_ast_macros::visitor_impl;

pub trait Visit {
    fn visit<'v, V>(&'v self, _: &mut V)
    where
        V: Visitor<'v>;
}

impl<T> Visit for AstNode<T>
where
    T: Visit,
{
    fn visit<'v, V>(&'v self, v: &mut V)
    where
        V: Visitor<'v>,
    {
        self.node.visit(v)
    }
}

impl<T> Visit for &T
where
    T: Visit,
{
    fn visit<'v, V>(&'v self, v: &mut V)
    where
        V: Visitor<'v>,
    {
        (*self).visit(v)
    }
}

impl<T> Visit for Box<T>
where
    T: Visit,
{
    fn visit<'v, V>(&'v self, v: &mut V)
    where
        V: Visitor<'v>,
    {
        (**self).visit(v)
    }
}

impl<T> Visit for Option<T>
where
    T: Visit,
{
    fn visit<'v, V>(&'v self, v: &mut V)
    where
        V: Visitor<'v>,
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
    fn visit<'v, V>(&'v self, v: &mut V)
    where
        V: Visitor<'v>,
    {
        for i in self {
            i.visit(v)
        }
    }
}

use crate::ast::*;
pub trait Visitor<'v> {
    visitor_impl!(Item);
    visitor_impl!(Ddl);
    visitor_impl!(DdlOp);
    visitor_impl!(CreateTable);
    visitor_impl!(DropTable);
    visitor_impl!(CreateIndex);
    visitor_impl!(DropIndex);
    visitor_impl!(Dml);
    visitor_impl!(DmlOp);
    visitor_impl!(Insert);
    visitor_impl!(InsertValue);
    visitor_impl!(Set);
    visitor_impl!(Assignment);
    visitor_impl!(Remove);
    visitor_impl!(Delete);
    visitor_impl!(OnConflict);
    visitor_impl!(Query);
    visitor_impl!(QuerySet);
    visitor_impl!(SetExpr);
    visitor_impl!(Expr);
    visitor_impl!(Lit);
    visitor_impl!(VarRef);
    visitor_impl!(BinOp);
    visitor_impl!(UniOp);
    visitor_impl!(Like);
    visitor_impl!(Between);
    visitor_impl!(In);
    visitor_impl!(Case);
    visitor_impl!(SimpleCase);
    visitor_impl!(SearchedCase);
    visitor_impl!(ExprPair);
    visitor_impl!(Struct);
    visitor_impl!(Bag);
    visitor_impl!(List);
    visitor_impl!(Sexp);
    visitor_impl!(Call);
    visitor_impl!(CallArg);
    visitor_impl!(CallArgNamed);
    visitor_impl!(CallArgNamedType);
    visitor_impl!(CallAgg);
    visitor_impl!(Select);
    visitor_impl!(Path);
    visitor_impl!(PathStep);
    visitor_impl!(PathExpr);
    visitor_impl!(Projection);
    visitor_impl!(ProjectionKind);
    visitor_impl!(ProjectItem);
    visitor_impl!(ProjectPivot);
    visitor_impl!(ProjectAll);
    visitor_impl!(ProjectExpr);
    visitor_impl!(Let);
    visitor_impl!(LetBinding);
    visitor_impl!(FromClause);
    visitor_impl!(FromLet);
    visitor_impl!(Join);
    visitor_impl!(JoinSpec);
    visitor_impl!(GroupByExpr);
    visitor_impl!(GroupKey);
    visitor_impl!(OrderByExpr);
    visitor_impl!(SortSpec);
    visitor_impl!(ReturningExpr);
    visitor_impl!(ReturningElem);
    visitor_impl!(CustomType);
}

#[cfg(test)]
mod tests {
    use crate::ast::{AstNode, BinOp, BinOpKind, Expr, Lit, NodeId};
    use crate::visit::Visitor;
    use std::ops::AddAssign;

    #[test]
    fn test_ast_init() {
        #[derive(Default)]
        struct Accum {
            val: Option<i64>,
        }

        impl<'v> Visitor<'v> for Accum {
            fn enter_lit(&mut self, literal: &'v crate::ast::Lit) {
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
