use crate::error::ParseError;
use crate::parse::lexer;
use std::ops::Range;

use partiql_ast::ast;

use lalrpop_util::ErrorRecovery;
use once_cell::sync::Lazy;
use regex::Regex;

use partiql_ast::ast::{AstNode, NodeId, SymbolPrimitive};

use partiql_source_map::location::{ByteOffset, BytePosition, Location};
use partiql_source_map::metadata::LocationMap;

type ParseErrorRecovery<'input> =
    ErrorRecovery<ByteOffset, lexer::Token<'input>, ParseError<'input, BytePosition>>;
type ParseErrors<'input> = Vec<ParseErrorRecovery<'input>>;

const INIT_LOCATIONS: usize = 100;

/// A provider of 'fresh' [`NodeId`]s.
// NOTE `pub` instead of `pub(crate)` only because LALRPop's generated code uses this in `pub trait __ToTriple`
//       which leads to compile time errors.
//       However, since this module is included privately, this type doesn't leak outside the crate anyway.
pub trait IdGenerator {
    /// Provides a 'fresh' [`NodeId`].
    fn id(&mut self) -> NodeId;
}

/// Auto-incrementing [`IdGenerator`]
pub(crate) struct NodeIdGenerator {
    next_id: ast::NodeId,
}

impl Default for NodeIdGenerator {
    fn default() -> Self {
        NodeIdGenerator { next_id: NodeId(1) }
    }
}

impl IdGenerator for NodeIdGenerator {
    #[inline]
    fn id(&mut self) -> NodeId {
        let mut next = NodeId(&self.next_id.0 + 1);
        std::mem::swap(&mut self.next_id, &mut next);
        next
    }
}

/// State of the parsing during parse.
pub(crate) struct ParserState<'input, Id: IdGenerator> {
    /// Generator for 'fresh' [`NodeId`]s
    pub id_gen: Id,
    /// Maps AST [`NodeId`]s to the location in the source from which each was derived.
    pub locations: LocationMap<NodeId>,
    /// Any errors accumulated during parse.
    pub errors: ParseErrors<'input>,

    /// Pattern to match names of aggregate functions.
    aggregates_pat: &'static Regex,
}

impl<'input> Default for ParserState<'input, NodeIdGenerator> {
    fn default() -> Self {
        ParserState::with_id_gen(NodeIdGenerator::default())
    }
}

// TODO: currently needs to be manually kept in-sync with preprocessor's `built_in_aggs`
// TODO: make extensible
const KNOWN_AGGREGATES: &str = "(?i:^count$)|(?i:^avg$)|(?i:^min$)|(?i:^max$)|(?i:^sum$)";
static KNOWN_AGGREGATE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(KNOWN_AGGREGATES).unwrap());

impl<'input, I> ParserState<'input, I>
where
    I: IdGenerator,
{
    pub fn with_id_gen(id_gen: I) -> Self {
        ParserState {
            id_gen,
            locations: LocationMap::with_capacity(INIT_LOCATIONS),
            errors: ParseErrors::default(),
            aggregates_pat: &KNOWN_AGGREGATE_PATTERN,
        }
    }
}

impl<'input, Id: IdGenerator> ParserState<'input, Id> {
    /// Create a new [`AstNode`] from the inner data which it is to hold and a source location.
    pub fn create_node<T, IntoLoc>(&mut self, node: T, location: IntoLoc) -> AstNode<T>
    where
        IntoLoc: Into<Location<BytePosition>>,
    {
        let location = location.into();
        let id = self.id_gen.id();

        self.locations.insert(id, location);

        AstNode { id, node }
    }

    /// Create a new [`AstNode`] from the inner data which it is to hold and a [`ByteOffset`] range.
    #[inline]
    pub fn node<T>(&mut self, ast: T, Range { start, end }: Range<ByteOffset>) -> AstNode<T> {
        self.create_node(ast, start.into()..end.into())
    }

    /// Check if a given `name` corresponds to a known aggregate function.
    #[inline]
    pub fn is_agg_fn(&self, name: &SymbolPrimitive) -> bool {
        self.aggregates_pat.is_match(&name.value)
    }
}
