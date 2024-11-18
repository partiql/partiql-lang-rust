use partiql_ast::ast;

use crate::parse::parser_state::ParserState;
use crate::ParseError;
use bitflags::bitflags;
use partiql_ast::ast::{Expr, Lit};
use partiql_common::node::NodeIdGenerator;
use partiql_common::syntax::location::{ByteOffset, BytePosition};

bitflags! {
    /// Set of AST node attributes to use as synthesized attributes.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct Attrs: u8 {
        const LIT = 0b0000_0001;

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
    fn new(data: T, attrs: Attrs) -> Self {
        Synth { data, attrs }
    }

    #[inline]
    pub fn empty(data: T) -> Self {
        Self::new(data, Attrs::empty())
    }

    #[inline]
    pub fn lit(data: T) -> Self {
        Self::new(data, Attrs::LIT)
    }

    pub fn map_data<U>(self, f: impl FnOnce(T) -> U) -> Synth<U> {
        let Self { data, attrs } = self;
        let data = f(data);
        Synth::new(data, attrs)
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
// Removes extra `Query` nesting if it exists, otherwise return the input.
// e.g. `(SELECT a FROM b ORDER BY c LIMIT d OFFSET e)` should be a Query with no additional nesting.
// Put another way: if `q` is a Query(QuerySet::Expr(Query(inner_q), ...), return Query(inner_q).
// Otherwise, return `q`.
pub(crate) fn strip_query(q: ast::AstNode<ast::Query>) -> ast::AstNode<ast::Query> {
    let outer_id = q.id;
    if let ast::AstNode {
        node: ast::QuerySet::Expr(e),
        id: inner_id,
    } = q.node.set
    {
        if let ast::Expr::Query(
            inner_q @ ast::AstNode {
                node: ast::Query { .. },
                ..
            },
        ) = *e
        {
            inner_q
        } else {
            let set = ast::AstNode {
                id: inner_id,
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
// If `qs` is a `QuerySet::Expr(Expr::Query(inner_q))`, return Query(inner_q). Otherwise, return `qs` wrapped
// in a `Query` with `None` as the `OrderBy` and `LimitOffset`
pub(crate) fn strip_query_set<Id>(
    qs: ast::AstNode<ast::QuerySet>,
    state: &mut ParserState<'_, Id>,
    lo: ByteOffset,
    hi: ByteOffset,
) -> ast::AstNode<ast::Query>
where
    Id: NodeIdGenerator,
{
    if let ast::AstNode {
        node: ast::QuerySet::Expr(q),
        id: inner_id,
    } = qs
    {
        if let ast::Expr::Query(
            inner_q @ ast::AstNode {
                node: ast::Query { .. },
                ..
            },
        ) = *q
        {
            // preserve query including limit/offset & order by if present
            inner_q
        } else {
            let query = ast::Query {
                set: ast::AstNode {
                    id: inner_id,
                    node: ast::QuerySet::Expr(q),
                },
                order_by: None,
                limit_offset: None,
            };
            state.node(query, lo..hi)
        }
    } else {
        let query = ast::Query {
            set: qs,
            order_by: None,
            limit_offset: None,
        };
        state.node(query, lo..hi)
    }
}

#[inline]
// If this is just a parenthesized expr, lift it out of the query AST, otherwise return input
//      e.g. `(1+2)` should be an `Expr`, not wrapped deep in a `Query`
pub(crate) fn strip_expr(q: ast::AstNode<ast::Query>) -> Box<ast::Expr> {
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

#[inline]
#[track_caller]
fn illegal_literal<'a, T>() -> Result<T, crate::error::ParseError<'a, BytePosition>> {
    Err(ParseError::IllegalState("Expected literal".to_string()))
}

pub(crate) type LitFlattenResult<'a, T> = Result<T, ParseError<'a>>;
#[inline]
pub(crate) fn struct_to_lit<'a>(strct: ast::Struct) -> LitFlattenResult<'a, ast::StructLit> {
    strct
        .fields
        .into_iter()
        .map(exprpair_to_lit)
        .collect::<LitFlattenResult<'_, Vec<_>>>()
        .map(|fields| ast::StructLit { fields })
}

#[inline]
pub(crate) fn bag_to_lit<'a>(bag: ast::Bag) -> LitFlattenResult<'a, ast::BagLit> {
    bag.values
        .into_iter()
        .map(|v| expr_to_lit(*v).map(|n| n.node))
        .collect::<LitFlattenResult<'_, Vec<_>>>()
        .map(|values| ast::BagLit { values })
}

#[inline]
pub(crate) fn list_to_lit<'a>(list: ast::List) -> LitFlattenResult<'a, ast::ListLit> {
    list.values
        .into_iter()
        .map(|v| expr_to_lit(*v).map(|n| n.node))
        .collect::<LitFlattenResult<'_, Vec<_>>>()
        .map(|values| ast::ListLit { values })
}

#[inline]
pub(crate) fn exprpair_to_lit<'a>(pair: ast::ExprPair) -> LitFlattenResult<'a, ast::LitField> {
    let ast::ExprPair { first, second } = pair;
    let (first, second) = (expr_to_litstr(*first)?, expr_to_lit(*second)?);
    Ok(ast::LitField { first, second })
}

#[inline]
pub(crate) fn expr_to_litstr<'a>(expr: ast::Expr) -> LitFlattenResult<'a, String> {
    match expr_to_lit(expr)?.node {
        Lit::CharStringLit(s) | Lit::NationalCharStringLit(s) => Ok(s),
        _ => illegal_literal(),
    }
}

#[inline]
pub(crate) fn expr_to_lit<'a>(expr: ast::Expr) -> LitFlattenResult<'a, ast::AstNode<ast::Lit>> {
    match expr {
        Expr::Lit(lit) => Ok(lit),
        _ => illegal_literal(),
    }
}
