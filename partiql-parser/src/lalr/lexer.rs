use partiql_core::location::{ByteOffset, BytePosition, LineAndCharPosition, LineOffset};

use logos::{Logos, Span};
use smallvec::{smallvec, SmallVec};
use std::cmp::max;

use std::ops::Range;
use thiserror::Error;

/// Keeps track of source offsets of newlines for the purposes of later calculating
/// line and column information
///
///
/// ## Example
///
/// ```rust
/// use partiql_parser::location::{ByteOffset, LineAndCharPosition};
/// use partiql_parser::LineOffsetTracker;
///
/// let source = "12345\n789012345\n789012345\n789012345";
/// let mut tracker = LineOffsetTracker::default();
/// tracker.record(6.into());
/// tracker.record(16.into());
/// tracker.record(26.into());
///
/// // We added 3 newlines, so there should be 4 lines of source
/// assert_eq!(tracker.num_lines(), 4);
/// assert_eq!(tracker.at(source, ByteOffset(0).into()), LineAndCharPosition::new(0,0));
/// assert_eq!(tracker.at(source, ByteOffset(6).into()), LineAndCharPosition::new(1,0));
/// assert_eq!(tracker.at(source, ByteOffset(30).into()), LineAndCharPosition::new(3,4));
/// ```
pub struct LineOffsetTracker {
    line_starts: SmallVec<[ByteOffset; 16]>,
}

impl Default for LineOffsetTracker {
    fn default() -> Self {
        LineOffsetTracker {
            line_starts: smallvec![ByteOffset(0)], // line 1 starts at offset `0`
        }
    }
}

impl LineOffsetTracker {
    /// Record a newline at `span` in the source
    #[inline(always)]
    pub fn record(&mut self, line_start: ByteOffset) {
        self.line_starts.push(line_start);
    }

    /// Append the line starts from another [`LineOffsetTracker`] to this one, adding `offset` to each.
    #[inline(always)]
    pub fn append(&mut self, other: &LineOffsetTracker, offset: ByteOffset) {
        // skip the first offset in `other`; it is the `0` added by `LineOffsetTracker::default()`
        for start in &other.line_starts[1..] {
            self.record(offset + *start);
        }
    }

    /// Calculate the number of lines of source seen so far.
    #[inline(always)]
    pub fn num_lines(&self) -> usize {
        self.line_starts.len()
    }

    /// Calculates the byte offset span ([`Range`]) of a line.
    ///
    /// `num` is the line number (0-indexed) for which to  calculate the span
    /// `max` is the largest value allowable in the returned [`Range's end`](core::ops::Range)
    #[inline(always)]
    fn byte_span_from_line_num(&self, num: LineOffset, max: ByteOffset) -> Range<ByteOffset> {
        let start = self.line_starts[num.to_usize()];
        let end = self
            .line_starts
            .get((num + 1).to_usize())
            .unwrap_or(&max)
            .min(&max);
        start..*end
    }

    /// Calculates the line number (0-indexed) in which a byte offset is contained.
    ///
    /// `offset` is the byte offset
    #[inline(always)]
    fn line_num_from_byte_offset(&self, offset: ByteOffset) -> LineOffset {
        match self.line_starts.binary_search(&offset) {
            Err(i) => i - 1,
            Ok(i) => i,
        }
        .into()
    }

    /// Calculates a [`LineAndCharPosition`] for a byte offset from the given `&str`
    ///
    /// `source` is source `&str` into which the byte offset applies
    /// `offset` is the byte offset for which to find the [`LineAndCharPosition`]
    ///
    /// # Panics
    ///
    /// This function will panic if:
    ///  - `offset` is larger than the byte length of `source`, or
    ///  - `offset` falls inside a unicode codepoint
    #[inline]
    pub fn at(&self, source: &str, BytePosition(offset): BytePosition) -> LineAndCharPosition {
        if let ByteOffset(0) = offset {
            LineAndCharPosition::new(0, 0)
        } else {
            let line_num = self.line_num_from_byte_offset(offset);
            let line_span = self.byte_span_from_line_num(line_num, source.len().into());
            let span = line_span.start.to_usize()..=offset.to_usize();
            let column_num = source[span].chars().count();

            LineAndCharPosition::new(line_num.to_usize(), column_num - 1)
        }
    }
}

/// A 3-tuple of (start, `Tok`, end) denoting a token and it start and end offsets.
pub type Spanned<Tok, Loc> = (Loc, Tok, Loc);
/// A [`Result`] of a [`Spanned`] token.
pub(crate) type SpannedResult<Tok, Loc, Broke> = Result<Spanned<Tok, Loc>, Spanned<Broke, Loc>>;

/// Errors that can be encountered when lexing PartiQL.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LexError {
    /// Generic invalid input; likely an unrecognizable token.
    #[error("Lexing error: invalid input `{}`", .0)]
    InvalidInput(String),
    /// Embedded Ion value is not properly terminated.
    #[error("Lexing error: unterminated ion literal")]
    UnterminatedIonLiteral,
    /// Comment is not properly terminated.
    #[error("Lexing error: unterminated comment")]
    UnterminatedComment,
    /// Any other lexing error.
    #[error("Lexing error: unknown error")]
    Unknown,
}

/// A block comment string (e.g. `"/* comment here */"`) with [`ByteOffset`] span relative to lexed source.
///
/// Note:
/// - The returned string includes the comment start (`/*`) and end (`*/`) tokens.
/// - The returned ByteOffset span includes the comment start (`/*`) and end (`*/`) tokens.
type CommentStringResult<'input> = SpannedResult<&'input str, ByteOffset, LexError>;

/// Tokens used to parse block comment
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
enum CommentToken {
    #[error]
    // Skip stuff that won't interfere with comment detection
    #[regex(r"[^/*\r\n\u0085\u2028\u2029]+", logos::skip)]
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
struct CommentLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, CommentToken>,
    comment_nesting: bool,
    tracker: &'tracker mut LineOffsetTracker,
}

impl<'input, 'tracker> CommentLexer<'input, 'tracker> {
    /// Creates a new block comment lexer over `input` text.
    /// Nested comment parsing is *off* by default; see [`with_nesting`] to enable nesting.
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        CommentLexer {
            lexer: CommentToken::lexer(input),
            comment_nesting: false,
            tracker,
        }
    }

    /// Toggles *on* the parsing of nested comments
    fn with_nesting(mut self) -> Self {
        self.comment_nesting = true;
        self
    }

    /// Parses a single (possibly nested) block comment and returns it
    fn next(&mut self) -> Option<CommentStringResult<'input>> {
        let Span { start, .. } = self.lexer.span();
        let mut nesting = 0;
        let nesting_inc = if self.comment_nesting { 1 } else { 0 };
        'comment: loop {
            match self.lexer.next() {
                Some(CommentToken::Any) => continue,
                Some(CommentToken::Newline) => {
                    self.tracker.record(self.lexer.span().end.into());
                }
                Some(CommentToken::Start) => nesting = max(1, nesting + nesting_inc),
                Some(CommentToken::End) => {
                    if nesting == 0 {
                        let Span { end, .. } = self.lexer.span();
                        return Some(Err((start.into(), LexError::Unknown, end.into())));
                    }
                    nesting -= 1;
                    if nesting == 0 {
                        break 'comment;
                    }
                }
                None => {
                    return if nesting != 0 {
                        let Span { end, .. } = self.lexer.span();
                        Some(Err((
                            start.into(),
                            LexError::UnterminatedComment,
                            end.into(),
                        )))
                    } else {
                        None
                    }
                }
            }
        }
        let Span { end, .. } = self.lexer.span();
        let comment = &self.lexer.source()[start..end];

        Some(Ok((start.into(), comment, end.into())))
    }
}

impl<'input, 'tracker> Iterator for CommentLexer<'input, 'tracker> {
    type Item = CommentStringResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// An embedded Ion string (e.g. `[{a: 1}, {b: 2}]`) with [`ByteOffset`] span
/// relative to lexed source.
///
///  Note:
/// - The lexer parses the embedded ion value enclosed in backticks.
/// - The returned string *does not* include the backticks
/// - The returned ByteOffset span *does* include the backticks
type EmbeddedIonStringResult<'input> = SpannedResult<&'input str, ByteOffset, LexError>;

/// Tokens used to parse Ion literals embedded in backticks (\`)
#[derive(Logos, Debug, Clone, PartialEq)]
enum EmbeddedIonToken {
    #[error]
    // Skip stuff that doesn't interfere with comment or string detection
    #[regex(r#"[^/*'"`\r\n\u0085\u2028\u2029]+"#, logos::skip)]
    Any,

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
struct EmbeddedIonLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, EmbeddedIonToken>,
    tracker: &'tracker mut LineOffsetTracker,
}

impl<'input, 'tracker> EmbeddedIonLexer<'input, 'tracker> {
    /// Creates a new embedded ion lexer over `input` text.
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        EmbeddedIonLexer {
            lexer: EmbeddedIonToken::lexer(input),
            tracker,
        }
    }

    /// Parses a single embedded ion value, quoted between backticks (`), and returns it
    fn next(&mut self) -> Option<EmbeddedIonStringResult<'input>> {
        let next_token = self.lexer.next();
        match next_token {
            Some(EmbeddedIonToken::Embed) => {
                let Span { start, .. } = self.lexer.span();
                'ion_value: loop {
                    let next_tok = self.lexer.next();
                    match next_tok {
                        Some(EmbeddedIonToken::Newline) => {
                            self.tracker.record(self.lexer.span().end.into());
                        }
                        Some(EmbeddedIonToken::Embed) => {
                            break 'ion_value;
                        }
                        Some(EmbeddedIonToken::CommentBlock) => {
                            let embed = self.lexer.span();
                            let remaining = &self.lexer.source()[embed.start..];
                            let mut comment_tracker = LineOffsetTracker::default();
                            let mut comment_lexer =
                                CommentLexer::new(remaining, &mut comment_tracker);
                            match comment_lexer.next() {
                                Some(Ok((s, _c, e))) => {
                                    self.tracker.append(&comment_tracker, embed.start.into());
                                    self.lexer.bump((e - s).to_usize() - embed.len())
                                }
                                Some(Err((_s, err, e))) => {
                                    return Some(Err((embed.start.into(), err, e)));
                                }
                                None => unreachable!(),
                            }
                        }
                        Some(EmbeddedIonToken::LongString) => {
                            'triple_quote: loop {
                                let next_tok = self.lexer.next();
                                match next_tok {
                                    Some(EmbeddedIonToken::LongString) => break 'triple_quote,
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
        self.next()
    }
}

/// A lexer from PartiQL text strings to [`LexicalToken`]s
pub(crate) struct PartiqlLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, Token<'input>>,
    tracker: &'tracker mut LineOffsetTracker,
}

pub(crate) type LexResult<'input> = SpannedResult<Token<'input>, ByteOffset, LexError>;

impl<'input, 'tracker> PartiqlLexer<'input, 'tracker> {
    /// Creates a new PartiQL lexer over `input` text.
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        PartiqlLexer {
            lexer: Token::lexer(input),
            tracker,
        }
    }

    /// Creates an error token at the current lexer location
    #[inline]
    fn err_here(&self, err_ctor: fn(String) -> LexError) -> LexResult<'input> {
        let region = self.lexer.slice().to_owned();
        let Span { start, end } = self.lexer.span();
        Err((start.into(), err_ctor(region), end.into()))
    }

    /// Wraps a [`Token`] into a [`LexicalToken`] at the current position of the lexer.
    #[inline(always)]
    fn wrap(&mut self, token: Token<'input>) -> LexResult<'input> {
        let Span { start, end } = self.lexer.span();
        Ok((start.into(), token, end.into()))
    }

    /// Advances the iterator and returns the next [`LexicalToken`] or [`None`] when input is exhausted.
    fn next(&mut self) -> Option<LexResult<'input>> {
        'next_tok: loop {
            return match self.lexer.next() {
                None => None,
                Some(token) => match token {
                    Token::Error => Some(self.err_here(LexError::InvalidInput)),

                    Token::Newline => {
                        self.tracker.record(self.lexer.span().end.into());
                        // Newlines shouldn't generate an externally visible token
                        continue 'next_tok;
                    }

                    Token::EmbeddedIonQuote => self.parse_embedded_ion(),

                    Token::CommentBlockStart => self.parse_block_comment(),

                    _ => Some(self.wrap(token)),
                },
            };
        }
    }

    /// Uses [`CommentLexer`] to parse a block comment
    fn parse_block_comment(&mut self) -> Option<LexResult<'input>> {
        let embed = self.lexer.span();
        let remaining = &self.lexer.source()[embed.start..];
        let mut comment_tracker = LineOffsetTracker::default();
        let mut comment_lexer = CommentLexer::new(remaining, &mut comment_tracker).with_nesting();
        comment_lexer.next().map(|res| match res {
            Ok((s, comment, e)) => {
                self.tracker.append(&comment_tracker, embed.start.into());
                self.lexer.bump((e - s).to_usize() - embed.len());
                Ok((embed.start.into(), Token::CommentBlock(comment), e))
            }
            Err((_s, err, e)) => Err((embed.start.into(), err, e)),
        })
    }

    /// Uses [`EmbeddedIonLexer`] to parse an embedded ion value
    fn parse_embedded_ion(&mut self) -> Option<LexResult<'input>> {
        let embed = self.lexer.span();
        let remaining = &self.lexer.source()[embed.start..];
        let mut ion_tracker = LineOffsetTracker::default();
        let mut ion_lexer = EmbeddedIonLexer::new(remaining, &mut ion_tracker);
        ion_lexer.next().map(|res| match res {
            Ok((s, ion, e)) => {
                self.tracker.append(&ion_tracker, embed.start.into());
                self.lexer.bump((e - s).to_usize() - embed.len());
                Ok((embed.end.into(), Token::Ion(ion), e - 1))
            }
            Err((_s, err, e)) => Err((embed.start.into(), err, e)),
        })
    }
}

impl<'input, 'tracker> Iterator for PartiqlLexer<'input, 'tracker> {
    type Item = LexResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// Tokens that the lexer can generate.
///
/// # Note
/// Tokens with names beginning with `__` are used internally and not meant to be used outside lexing.
#[derive(Logos, Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
// TODO make pub(crate) ?
pub enum Token<'input> {
    #[error]
    // Skip whitespace
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,

    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"([\r]?[\n])|\u{0085}|\u{2028}|\u{2029}")]
    Newline,

    #[regex(r"--[^\n]*", |lex| lex.slice())]
    CommentLine(&'input str),
    #[token("/*")]
    CommentBlockStart,
    CommentBlock(&'input str),

    // Brackets
    #[token("[")]
    OpenSquare,
    #[token("]")]
    CloseSquare,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("<<")]
    OpenDblAngle,
    #[token(">>")]
    CloseDblAngle,

    // Symbols
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    BangEqual,
    #[token("<>")]
    LessGreater,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("=")]
    Equal,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("%")]
    Percent,
    #[token("/")]
    Slash,
    #[token("^")]
    Caret,
    #[token(".")]
    Period,

    // unquoted identifiers
    #[regex("[a-zA-Z_$][a-zA-Z0-9_$]*", |lex| lex.slice())]
    // quoted identifiers (quoted with double quotes)
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#,
            |lex| lex.slice().trim_matches('"'))]
    Identifier(&'input str),

    // unquoted @identifiers
    #[regex("@[a-zA-Z_$][a-zA-Z0-9_$]*", |lex| &lex.slice()[1..])]
    // quoted @identifiers (quoted with double quotes)
    #[regex(r#"@"([^"\\]|\\t|\\u|\\n|\\")*""#,
            |lex| lex.slice()[1..].trim_matches('"'))]
    AtIdentifier(&'input str),

    #[regex("[0-9]+", |lex| lex.slice())]
    Int(&'input str),

    #[regex("[0-9]+\\.[0-9]*([eE][-+]?[0-9]+)", |lex| lex.slice())]
    #[regex("\\.[0-9]+([eE][-+]?[0-9]+)", |lex| lex.slice())]
    #[regex("[0-9]+[eE][-+]?[0-9]+", |lex| lex.slice())]
    ExpReal(&'input str),

    #[regex("[0-9]+\\.[0-9]*", |lex| lex.slice())]
    #[regex("\\.[0-9]+", |lex| lex.slice())]
    Real(&'input str),

    // strings are single-quoted in SQL/PartiQL
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\'|(?:''))*'"#,
            |lex| lex.slice().trim_matches('\''))]
    String(&'input str),

    #[token("`")]
    EmbeddedIonQuote,
    Ion(&'input str),

    // Keywords
    #[regex("(?i:All)")]
    All,
    #[regex("(?i:Asc)")]
    Asc,
    #[regex("(?i:And)")]
    And,
    #[regex("(?i:As)")]
    As,
    #[regex("(?i:At)")]
    At,
    #[regex("(?i:Between)")]
    Between,
    #[regex("(?i:By)")]
    By,
    #[regex("(?i:Cross)")]
    Cross,
    #[regex("(?i:Desc)")]
    Desc,
    #[regex("(?i:Escape)")]
    Escape,
    #[regex("(?i:Except)")]
    Except,
    #[regex("(?i:False)")]
    False,
    #[regex("(?i:First)")]
    First,
    #[regex("(?i:Full)")]
    Full,
    #[regex("(?i:From)")]
    From,
    #[regex("(?i:Group)")]
    Group,
    #[regex("(?i:Having)")]
    Having,
    #[regex("(?i:In)")]
    In,
    #[regex("(?i:Inner)")]
    Inner,
    #[regex("(?i:Is)")]
    Is,
    #[regex("(?i:Intersect)")]
    Intersect,
    #[regex("(?i:Join)")]
    Join,
    #[regex("(?i:Last)")]
    Last,
    #[regex("(?i:Lateral)")]
    Lateral,
    #[regex("(?i:Left)")]
    Left,
    #[regex("(?i:Like)")]
    Like,
    #[regex("(?i:Limit)")]
    Limit,
    #[regex("(?i:Missing)")]
    Missing,
    #[regex("(?i:Natural)")]
    Natural,
    #[regex("(?i:Not)")]
    Not,
    #[regex("(?i:Null)")]
    Null,
    #[regex("(?i:Nulls)")]
    Nulls,
    #[regex("(?i:Offset)")]
    Offset,
    #[regex("(?i:On)")]
    On,
    #[regex("(?i:Or)")]
    Or,
    #[regex("(?i:Order)")]
    Order,
    #[regex("(?i:Outer)")]
    Outer,
    #[regex("(?i:Pivot)")]
    Pivot,
    #[regex("(?i:Preserve)")]
    Preserve,
    #[regex("(?i:Right)")]
    Right,
    #[regex("(?i:Select)")]
    Select,
    #[regex("(?i:True)")]
    True,
    #[regex("(?i:Union)")]
    Union,
    #[regex("(?i:Unpivot)")]
    Unpivot,
    #[regex("(?i:Using)")]
    Using,
    #[regex("(?i:Value)")]
    Value,
    #[regex("(?i:Where)")]
    Where,
    #[regex("(?i:With)")]
    With,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::LineAndColumn;

    #[test]
    fn ion_simple() {
        let ion_value = r#" `{'input':1,  'b':1}` "#;

        let mut offset_tracker = LineOffsetTracker::default();
        let ion_lexer = EmbeddedIonLexer::new(ion_value.trim(), &mut offset_tracker);
        assert_eq!(ion_lexer.into_iter().count(), 1);
        assert_eq!(offset_tracker.num_lines(), 1);

        let mut offset_tracker = LineOffsetTracker::default();
        let mut lexer = PartiqlLexer::new(ion_value, &mut offset_tracker);

        let tok = lexer.next().unwrap().unwrap();
        assert!(
            matches!(tok, (ByteOffset(2), Token::Ion(ion_str), ByteOffset(20)) if ion_str == ion_value.trim().trim_matches('`'))
        );
    }

    #[test]
    fn ion() {
        let ion_value = r#" `{'input' // comment ' "
                       :1, /* 
                               comment 
                              */
                      'b':1}` "#;

        let mut offset_tracker = LineOffsetTracker::default();
        let ion_lexer = EmbeddedIonLexer::new(ion_value.trim(), &mut offset_tracker);
        assert_eq!(ion_lexer.into_iter().count(), 1);
        assert_eq!(offset_tracker.num_lines(), 5);

        let mut offset_tracker = LineOffsetTracker::default();
        let mut lexer = PartiqlLexer::new(ion_value, &mut offset_tracker);

        let tok = lexer.next().unwrap().unwrap();
        assert!(
            matches!(tok, (ByteOffset(2), Token::Ion(ion_str), ByteOffset(157)) if ion_str == ion_value.trim().trim_matches('`'))
        );
        assert_eq!(offset_tracker.num_lines(), 5);
    }

    #[test]
    fn nested_comments() {
        let comments = r##"/*  
                                    /*  / * * * /
                                    /*  ' " ''' ` 
                                    */  text
                                    */  1 2 3 4 5 6,7,8,9 10.112^5
                                    */"##;

        let mut offset_tracker = LineOffsetTracker::default();
        let nested_lex = CommentLexer::new(comments, &mut offset_tracker).with_nesting();
        assert_eq!(nested_lex.into_iter().count(), 1);
        assert_eq!(offset_tracker.num_lines(), 6);

        let mut offset_tracker = LineOffsetTracker::default();
        let nonnested_lex = CommentLexer::new(comments, &mut offset_tracker);
        let toks: Result<Vec<_>, Spanned<LexError, ByteOffset>> = nonnested_lex.collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (ByteOffset(142), LexError::Unknown, ByteOffset(189))
        ));
        assert_eq!(error.1.to_string(), "Lexing error: unknown error");
    }

    #[test]
    fn select() -> Result<(), Spanned<LexError, ByteOffset>> {
        let query = "SELECT g\nFROM data\nGROUP BY a";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::Identifier("g"),
                Token::From,
                Token::Identifier("data"),
                Token::Group,
                Token::By,
                Token::Identifier("a")
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        assert_eq!(offset_tracker.num_lines(), 3);
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 0.into())),
            LineAndColumn::new(1, 1).unwrap()
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 1.into())),
            LineAndColumn::new(1, 2).unwrap()
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 9.into())),
            LineAndColumn::new(2, 1).unwrap()
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 19.into())),
            LineAndColumn::new(3, 1).unwrap()
        );

        let offset_r_a = query.rfind('a').unwrap();
        let offset_r_n = query.rfind('\n').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(query.len() - 1))),
            LineAndColumn::new(3, offset_r_a - offset_r_n).unwrap()
        );

        Ok(())
    }

    #[test]
    fn select_unicode() -> Result<(), Spanned<LexError, ByteOffset>> {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::Identifier("🐈"),
                Token::From,
                Token::Identifier("❤ℝ"),
                Token::Group,
                Token::By,
                Token::Identifier("🧸")
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        assert_eq!(offset_tracker.num_lines(), 5);
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 0.into())),
            LineAndColumn::new(1, 1).unwrap()
        );

        let offset_s = query.find('S').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, offset_s.into())),
            LineAndColumn::new(2, 1).unwrap()
        );

        let offset_f = query.find('F').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, offset_f.into())),
            LineAndColumn::new(3, 1).unwrap()
        );

        let offset_g = query.find('G').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, offset_g.into())),
            LineAndColumn::new(5, 1).unwrap()
        );

        Ok(())
    }

    #[test]
    #[should_panic]
    fn panic_offset_overflow() {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        lexer.count();

        offset_tracker.at(query, query.len().into());
    }

    #[test]
    #[should_panic]
    fn panic_offset_into_codepoint() {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        lexer.count();

        offset_tracker.at(query, ByteOffset(1).into());
    }

    #[test]
    fn select_comment_line() -> Result<(), Spanned<LexError, ByteOffset>> {
        let query = "SELECT --comment\n@g from @\"foo\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentLine("--comment"),
                Token::AtIdentifier("g"),
                Token::From,
                Token::AtIdentifier("foo"),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(offset_tracker.num_lines(), 2);
        Ok(())
    }

    #[test]
    fn select_comment_block() -> Result<(), Spanned<LexError, ByteOffset>> {
        let query = "SELECT /*comment*/ g";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentBlock("/*comment*/"),
                Token::Identifier("g"),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(offset_tracker.num_lines(), 1);
        Ok(())
    }

    #[test]
    fn err_invalid_input() {
        let query = "SELECT # FROM data GROUP BY a";
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, Spanned<LexError, ByteOffset>> =
            PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert_eq!(error.1.to_string(), r##"Lexing error: invalid input `#`"##);
        assert!(
            matches!(error, (ByteOffset(7), LexError::InvalidInput(s), ByteOffset(8)) if s == "#")
        );
        assert_eq!(offset_tracker.num_lines(), 1);
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 7.into())),
            LineAndColumn::new(1, 8).unwrap()
        );
    }

    #[test]
    fn err_unterminated_ion() {
        let query = r#" ` "fooo` "#;
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, Spanned<LexError, ByteOffset>> =
            PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (
                ByteOffset(1),
                LexError::UnterminatedIonLiteral,
                ByteOffset(9)
            )
        ));
        assert_eq!(
            error.1.to_string(),
            "Lexing error: unterminated ion literal"
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(1))),
            LineAndColumn::new(1, 2).unwrap()
        );
    }

    #[test]
    fn err_unterminated_comment() {
        let query = r#" /*12345678"#;
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, Spanned<LexError, ByteOffset>> =
            PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (ByteOffset(1), LexError::UnterminatedComment, ByteOffset(10))
        ));
        assert_eq!(error.1.to_string(), "Lexing error: unterminated comment");
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(1))),
            LineAndColumn::new(1, 2).unwrap()
        );
    }

    #[test]
    fn err_unterminated_ion_comment() {
        let query = r#" `/*12345678`"#;
        let mut offset_tracker = LineOffsetTracker::default();
        let ion_lexer = EmbeddedIonLexer::new(query, &mut offset_tracker);
        let toks: Result<Vec<_>, Spanned<LexError, ByteOffset>> = ion_lexer.collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (ByteOffset(2), LexError::UnterminatedComment, ByteOffset(11))
        ));
        assert_eq!(error.1.to_string(), "Lexing error: unterminated comment");
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(2))),
            LineAndColumn::new(1, 3).unwrap()
        );
    }
}
