use crate::error::LexError;
use crate::lexer::{CommentLexer, SpannedResult};
use logos::{Logos, Span};
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use partiql_common::syntax::location::ByteOffset;

/// An embedded Ion string (e.g. `[{a: 1}, {b: 2}]`) with [`ByteOffset`] span
/// relative to lexed source.
///
///  Note:
/// - The lexer parses the embedded ion value enclosed in backticks.
/// - The returned string *does not* include the backticks
/// - The returned `ByteOffset` span *does* include the backticks
type EmbeddedIonStringResult<'input> = SpannedResult<&'input str, ByteOffset, LexError<'input>>;

/// Tokens used to parse Ion literals embedded in backticks (\`)
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r#"[^/*'"`\r\n\u0085\u2028\u2029]+"#)]
enum EmbeddedIonToken {
    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"(([\r])?[\n])|\u0085|\u2028|\u2029")]
    Newline,

    #[token("`")]
    Embed,

    #[regex(r"//[^\n]*")]
    CommentLine,
    #[token("/*")]
    CommentBlock,

    #[regex(r#""([^"\\]|\\t|\\u|\\")*""#)]
    String,
    #[regex(r#"'([^'\\]|\\t|\\u|\\')*'"#)]
    Symbol,
    #[token("'''")]
    LongString,
}

/// A Lexer for Ion literals embedded in backticks (\`) that returns the parsed [`EmbeddedIonString`]
///
/// Parses just enough Ion to make sure not to include a backtick that is inside a string or comment.
pub struct EmbeddedIonLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, EmbeddedIonToken>,
    tracker: &'tracker mut LineOffsetTracker,
}

impl<'input, 'tracker> EmbeddedIonLexer<'input, 'tracker> {
    /// Creates a new embedded ion lexer over `input` text.
    #[inline]
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        EmbeddedIonLexer {
            lexer: EmbeddedIonToken::lexer(input),
            tracker,
        }
    }

    /// Parses a single embedded ion value, quoted between backticks (`), and returns it
    fn next_internal(&mut self) -> Option<EmbeddedIonStringResult<'input>> {
        let next_token = self.lexer.next();
        match next_token {
            Some(Ok(EmbeddedIonToken::Embed)) => {
                let Span { start, .. } = self.lexer.span();
                'ion_value: loop {
                    let next_tok = self.lexer.next();
                    match next_tok {
                        Some(Ok(EmbeddedIonToken::Newline)) => {
                            self.tracker.record(self.lexer.span().end.into());
                        }
                        Some(Ok(EmbeddedIonToken::Embed)) => {
                            break 'ion_value;
                        }
                        Some(Ok(EmbeddedIonToken::CommentBlock)) => {
                            let embed = self.lexer.span();
                            let remaining = &self.lexer.source()[embed.start..];
                            let mut comment_tracker = LineOffsetTracker::default();
                            let mut comment_lexer =
                                CommentLexer::new(remaining, &mut comment_tracker);
                            match comment_lexer.next() {
                                Some(Ok((s, _c, e))) => {
                                    self.tracker.append(&comment_tracker, embed.start.into());
                                    self.lexer.bump((e - s).to_usize() - embed.len());
                                }
                                Some(Err((s, err, e))) => {
                                    let offset: ByteOffset = embed.start.into();
                                    return Some(Err((s + offset, err, e + offset)));
                                }
                                None => unreachable!(),
                            }
                        }
                        Some(Ok(EmbeddedIonToken::LongString)) => {
                            'triple_quote: loop {
                                let next_tok = self.lexer.next();
                                match next_tok {
                                    Some(Ok(EmbeddedIonToken::LongString)) => break 'triple_quote,
                                    Some(_) => (), // just consume all other tokens
                                    None => continue 'ion_value,
                                }
                            }
                        }
                        Some(_) => {
                            // just consume all other tokens
                        }
                        None => {
                            let Span { end, .. } = self.lexer.span();
                            return Some(Err((
                                start.into(),
                                LexError::UnterminatedIonLiteral,
                                end.into(),
                            )));
                        }
                    }
                }
                let Span { end, .. } = self.lexer.span();
                let (str_start, str_end) = (start + 1, end - 1);
                let ion_value = &self.lexer.source()[str_start..str_end];

                Some(Ok((start.into(), ion_value, end.into())))
            }
            _ => None,
        }
    }
}

impl<'input, 'tracker> Iterator for EmbeddedIonLexer<'input, 'tracker> {
    type Item = EmbeddedIonStringResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_internal()
    }
}
