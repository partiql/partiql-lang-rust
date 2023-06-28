use partiql_ast::ast;

use crate::parse::parser_state::{IdGenerator, ParserState};
use bitflags::bitflags;
use partiql_ast::ast::AstNode;

bitflags! {
    /// Set of AST node attributes to use as synthesized attributes.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct Attrs: u8 {
        const LIT = 0b00000001;

        const INTERSECTABLE = Self::LIT.bits();
        const UNIONABLE = 0;
    }
}

impl Attrs {
    /// Combine attributes from two nodes.
    #[inline]
    pub fn synthesize(self, other: Self) -> Attrs {
        ((self & Attrs::INTERSECTABLE) & (other & Attrs::INTERSECTABLE))
            | ((self & Attrs::UNIONABLE) | (other & Attrs::UNIONABLE))
    }
}

/// Wrapper attaching synthesized attributes `Attrs` with an AST node.
pub(crate) struct Synth<T> {
    pub(crate) data: T,
    pub(crate) attrs: Attrs,
}

impl<T> Synth<T> {
    #[inline]
    pub fn new(data: T, attrs: Attrs) -> Self {
        Synth { data, attrs }
    }

    #[inline]
    pub fn empty(data: T) -> Self {
        Self::new(data, Attrs::empty())
    }
}

impl<T> FromIterator<Synth<T>> for Synth<Vec<T>> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = Synth<T>>>(iter: I) -> Synth<Vec<T>> {
        let mut attrs = Attrs::all();
        let iterator = iter.into_iter().map(|Synth { data, attrs: a }| {
            attrs = attrs.synthesize(a);
            data
        });
        let data = iterator.collect::<Vec<_>>();
        Synth { data, attrs }
    }
}

pub(crate) enum CallSite {
    Call(ast::Call),
    CallAgg(ast::CallAgg),
}

#[inline]
// if this is just a parenthesized expr, lift it out of the query AST, otherwise return input
//      e.g. `(1+2)` should be a ExprKind::Expr, not wrapped deep in a ExprKind::Query
pub(crate) fn strip_expr_query(q: Box<ast::Expr>) -> Box<ast::Expr> {
    if let ast::Expr::Query(ast::AstNode {
        node:
            ast::Query {
                set:
                    ast::AstNode {
                        node: ast::QuerySet::Expr(e),
                        ..
                    },
                order_by: None,
                limit_offset: None,
            },
        ..
    }) = *q
    {
        e
    } else {
        q
    }
}

#[inline]
// todo docs
pub(crate) fn strip_queryset<Id>(
    qs: ast::AstNode<ast::QuerySet>,
    state: &mut ParserState<Id>,
) -> ast::AstNode<ast::Query>
where
    Id: IdGenerator,
{
    if let ast::AstNode {
        node: ast::QuerySet::Expr(q),
        id: id,
    } = qs
    {
        if let ast::Expr::Query(
            qnode @ ast::AstNode {
                node: ast::Query { .. },
                ..
            },
        ) = *q
        {
            // preserve query including limit/offset & order by if present
            qnode
        } else {
            // todo handle unwrap
            let range = state.locations.get(&qs.id).unwrap();
            let (lo, hi) = (range.start.0, range.end.0);
            let query = ast::Query {
                set: ast::AstNode {
                    id: id,
                    node: ast::QuerySet::Expr(q),
                },
                order_by: None,
                limit_offset: None,
            };
            state.node(query, lo..hi)
        }
    } else {
        // todo handle unwrap
        let range = state.locations.get(&qs.id).unwrap();
        let (lo, hi) = (range.start.0, range.end.0);
        let query = ast::Query {
            set: qs,
            order_by: None,
            limit_offset: None,
        };
        state.node(query, lo..hi)
    }
}

#[inline]
// todo docs
pub(crate) fn strip_query(q: AstNode<ast::Query>) -> AstNode<ast::Query> {
    let outer_id = q.id;
    if let ast::AstNode {
        node: ast::QuerySet::Expr(e),
        id,
    } = q.node.set
    {
        if let ast::Expr::Query(
            qnode @ ast::AstNode {
                node: ast::Query { .. },
                ..
            },
        ) = *e
        {
            qnode
        } else {
            let set = AstNode {
                id: id,
                node: ast::QuerySet::Expr(e),
            };
            ast::AstNode {
                id: outer_id,
                node: ast::Query {
                    set,
                    order_by: None,
                    limit_offset: None,
                },
            }
        }
    } else {
        q
    }
}

#[inline]
// if this is just a parenthesized expr, lift it out of the query AST, otherwise return input
//      e.g. `(1+2)` should be a ExprKind::Expr, not wrapped deep in a ExprKind::Query
pub(crate) fn strip_expr(q: AstNode<ast::Query>) -> Box<ast::Expr> {
    if let ast::AstNode {
        node:
            ast::Query {
                set:
                    ast::AstNode {
                        node: ast::QuerySet::Expr(e),
                        ..
                    },
                order_by: None,
                limit_offset: None,
            },
        ..
    } = q
    {
        e
    } else {
        Box::new(ast::Expr::Query(q))
    }
}
