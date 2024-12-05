use std::borrow::Cow;
use std::cmp::max;

use crate::error::LexError;
use crate::lexer::SpannedResult;
use logos::{Logos, Span};
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use partiql_common::syntax::location::ByteOffset;

/// A block comment string (e.g. `"/* comment here */"`) with [`ByteOffset`] span relative to lexed source.
///
/// Note:
/// - The returned string includes the comment start (`/*`) and end (`*/`) tokens.
/// - The returned `ByteOffset` span includes the comment start (`/*`) and end (`*/`) tokens.
type CommentStringResult<'input> = SpannedResult<&'input str, ByteOffset, LexError<'input>>;

/// Tokens used to parse block comment
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[^/*\r\n\u0085\u2028\u2029]+")]
enum CommentToken {
    // Skip stuff that won't interfere with comment detection
    #[regex(r"[/*]", logos::skip)]
    Any,
    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"(([\r])?[\n])|\u0085|\u2028|\u2029")]
    Newline,
    #[token("*/")]
    End,
    #[token("/*")]
    Start,
}

/// A lexer for block comments (enclosed between '/*' & '*/') that returns the parsed [`CommentString`]
pub struct CommentLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, CommentToken>,
    comment_nesting: bool,
    tracker: &'tracker mut LineOffsetTracker,
}

impl<'input, 'tracker> CommentLexer<'input, 'tracker> {
    /// Creates a new block comment lexer over `input` text.
    /// Nested comment parsing is *off* by default; see [`with_nesting`] to enable nesting.
    #[inline]
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        CommentLexer {
            lexer: CommentToken::lexer(input),
            comment_nesting: false,
            tracker,
        }
    }

    /// Toggles *on* the parsing of nested comments
    #[inline]
    pub fn with_nesting(mut self) -> Self {
        self.comment_nesting = true;
        self
    }

    /// Creates an error token at the current lexer location
    #[inline]
    fn err_here(
        &self,
        err_ctor: fn(Cow<'input, str>) -> LexError<'input>,
    ) -> CommentStringResult<'input> {
        let Span { start, .. } = self.lexer.span();
        self.err_ends_here(start, err_ctor)
    }

    /// Creates an error token ending at the current lexer location
    #[inline]
    fn err_ends_here(
        &self,
        start: usize,
        err_ctor: fn(Cow<'input, str>) -> LexError<'input>,
    ) -> CommentStringResult<'input> {
        let region = self.lexer.slice();
        let Span { end, .. } = self.lexer.span();
        Err((start.into(), err_ctor(region.into()), end.into()))
    }

    /// Parses a single (possibly nested) block comment and returns it
    fn next_internal(&mut self) -> Option<CommentStringResult<'input>> {
        let Span { start, .. } = self.lexer.span();
        let mut nesting = 0;
        let nesting_inc = i32::from(self.comment_nesting);
        'comment: loop {
            match self.lexer.next() {
                Some(Ok(CommentToken::Any)) => continue,
                Some(Ok(CommentToken::Newline)) => {
                    self.tracker.record(self.lexer.span().end.into());
                }
                Some(Ok(CommentToken::Start)) => nesting = max(1, nesting + nesting_inc),
                Some(Ok(CommentToken::End)) => {
                    if nesting == 0 {
                        // saw a `*/` while not in a comment
                        return Some(self.err_here(|_| LexError::UnterminatedComment));
                    }
                    nesting -= 1;
                    if nesting == 0 {
                        break 'comment;
                    }
                }
                Some(Err(_)) => return Some(self.err_here(LexError::InvalidInput)),
                None => {
                    let result = if nesting != 0 {
                        // ran out of input while inside a comment
                        Some(self.err_ends_here(start, |_| LexError::UnterminatedComment))
                    } else {
                        None
                    };
                    return result;
                }
            }
        }
        let Span { end, .. } = self.lexer.span();
        let comment = &self.lexer.source()[start..end];

        Some(Ok((start.into(), comment, end.into())))
    }
}

impl<'input> Iterator for CommentLexer<'input, '_> {
    type Item = CommentStringResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_internal()
    }
}
