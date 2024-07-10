use crate::error::ParseError;
use crate::parse::lexer;
use std::ops::Range;

use lalrpop_util::ErrorRecovery;
use once_cell::sync::Lazy;
use regex::Regex;

use partiql_ast::ast::{AstNode, SymbolPrimitive};
use partiql_ast::builder::{AutoNodeIdGenerator, IdGenerator, NodeBuilder};

use partiql_source_map::location::{ByteOffset, BytePosition, Location};
use partiql_source_map::metadata::LocationMap;

type ParseErrorRecovery<'input> =
    ErrorRecovery<ByteOffset, lexer::Token<'input>, ParseError<'input, BytePosition>>;
type ParseErrors<'input> = Vec<ParseErrorRecovery<'input>>;

const INIT_LOCATIONS: usize = 100;

/// State of the parsing during parse.
pub(crate) struct ParserState<'input, Id: IdGenerator> {
    /// Generator for 'fresh' [`NodeId`]s
    pub node_builder: NodeBuilder<Id>,
    /// Maps AST [`NodeId`]s to the location in the source from which each was derived.
    pub locations: LocationMap,
    /// Any errors accumulated during parse.
    pub errors: ParseErrors<'input>,

    /// Pattern to match names of aggregate functions.
    aggregates_pat: &'static Regex,
}

impl<'input> Default for ParserState<'input, AutoNodeIdGenerator> {
    fn default() -> Self {
        ParserState::with_id_gen(AutoNodeIdGenerator::default())
    }
}

// TODO: currently needs to be manually kept in-sync with preprocessor's `built_in_aggs`
// TODO: make extensible
const KNOWN_AGGREGATES: &str =
    "(?i:^count$)|(?i:^avg$)|(?i:^min$)|(?i:^max$)|(?i:^sum$)|(?i:^any$)|(?i:^some$)|(?i:^every$)";
static KNOWN_AGGREGATE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(KNOWN_AGGREGATES).unwrap());

impl<'input, I> ParserState<'input, I>
where
    I: IdGenerator,
{
    pub fn with_id_gen(id_gen: I) -> Self {
        ParserState {
            node_builder: NodeBuilder::new(id_gen),
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
        let node = self.node_builder.node(node);
        self.locations.insert(node.id, location.into());
        node
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
