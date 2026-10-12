//! The `partiql-parser` lexer, compiled into this crate for REPL highlighting.
//!
//! The parser keeps its lexer private, so instead of widening its public API
//! we build its token sources here via `#[path]`. Highlighting therefore uses
//! exactly the tokens the grammar sees, and a new `Token` variant breaks the
//! exhaustive match in `highlight::category` until it is classified.
//!
//! The included files refer to `crate::lexer::*` and `crate::error::LexError`,
//! so this module and `error` must sit at the crate root. The aliases below
//! mirror `partiql-parser/src/lexer/mod.rs`, except that [`LexResult`] keeps
//! the raw [`ByteOffset`]-spanned error instead of converting to `ParseError`.
//
// TODO: move REPL highlighting to a tree-sitter PartiQL grammar and drop
// these `#[path]` includes of the parser's sources.

use partiql_common::syntax::location::ByteOffset;

use crate::error::LexError;

#[path = "../../partiql-parser/src/lexer/comment.rs"]
mod comment;
#[path = "../../partiql-parser/src/lexer/embedded_doc.rs"]
mod embedded_doc;
#[path = "../../partiql-parser/src/lexer/partiql.rs"]
mod partiql;

pub(crate) use comment::*;
pub(crate) use embedded_doc::*;
pub(crate) use partiql::*;

pub(crate) type Spanned<Tok, Loc> = (Loc, Tok, Loc);
pub(crate) type SpannedResult<Tok, Loc, Broke> = Result<Spanned<Tok, Loc>, Spanned<Broke, Loc>>;
pub(crate) type InternalLexResult<'input> =
    SpannedResult<Token<'input>, ByteOffset, LexError<'input>>;
pub(crate) type LexResult<'input> = InternalLexResult<'input>;
