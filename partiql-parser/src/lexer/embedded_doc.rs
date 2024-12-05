use crate::error::LexError;
use crate::lexer::SpannedResult;
use logos::{Logos, Span};
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use partiql_common::syntax::location::ByteOffset;

/// An embedded Doc string (e.g. `[{a: 1}, {b: 2}]`) with [`ByteOffset`] span
/// relative to lexed source.
///
///  Note:
/// - The lexer parses the embedded Doc value enclosed in backticks.
/// - The returned string *does not* include the backticks
/// - The returned `ByteOffset` span *does* include the backticks
type EmbeddedDocStringResult<'input> = SpannedResult<&'input str, ByteOffset, LexError<'input>>;

/// Tokens used to parse Doc literals embedded in backticks (\`)
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r#"[^/*'"`\r\n\u0085\u2028\u2029]+"#)] // skip things that aren't newlines or backticks
enum EmbeddedDocToken {
    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"(([\r])?[\n])|\u0085|\u2028|\u2029")]
    Newline,

    // An embed open/close tag is a (greedily-captured) odd-number of backticks
    #[regex(r"`(``)*")]
    Embed,
}

/// A Lexer for Doc literals embedded in backticks (\`) that returns the parsed [`EmbeddedDocString`]
///
/// Parses just enough Doc to make sure not to include a backtick that is inside a string or comment.
pub struct EmbeddedDocLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, EmbeddedDocToken>,
    tracker: &'tracker mut LineOffsetTracker,
}

impl<'input, 'tracker> EmbeddedDocLexer<'input, 'tracker> {
    /// Creates a new embedded Doc lexer over `input` text.
    #[inline]
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        EmbeddedDocLexer {
            lexer: EmbeddedDocToken::lexer(input),
            tracker,
        }
    }

    /// Parses a single embedded Doc value, quoted between backticks (`), and returns it
    fn next_internal(&mut self) -> Option<EmbeddedDocStringResult<'input>> {
        let next_token = self.lexer.next();
        match next_token {
            Some(Ok(EmbeddedDocToken::Embed)) => {
                let Span {
                    start: b_start,
                    end: b_end,
                } = self.lexer.span();
                let start_quote_len = b_end - b_start;
                loop {
                    let next_tok = self.lexer.next();
                    match next_tok {
                        Some(Ok(EmbeddedDocToken::Newline)) => {
                            // track the newline, and keep accumulating
                            self.tracker.record(self.lexer.span().end.into());
                        }
                        Some(Ok(EmbeddedDocToken::Embed)) => {
                            let Span {
                                start: e_start,
                                end: e_end,
                            } = self.lexer.span();
                            let end_quote_len = e_end - e_start;
                            if end_quote_len >= start_quote_len {
                                let backup = end_quote_len - start_quote_len;
                                let (str_start, str_end) =
                                    (b_start + start_quote_len, e_end - end_quote_len);
                                let doc_value = &self.lexer.source()[str_start..str_end];

                                return Some(Ok((
                                    b_start.into(),
                                    doc_value,
                                    (e_end - backup).into(),
                                )));
                            }
                        }
                        Some(_) => {
                            // just consume all other tokens
                        }
                        None => {
                            let Span { end, .. } = self.lexer.span();
                            return Some(Err((
                                b_start.into(),
                                LexError::UnterminatedDocLiteral,
                                end.into(),
                            )));
                        }
                    }
                }
            }
            _ => None,
        }
    }
}

impl<'input> Iterator for EmbeddedDocLexer<'input, '_> {
    type Item = EmbeddedDocStringResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_internal()
    }
}
