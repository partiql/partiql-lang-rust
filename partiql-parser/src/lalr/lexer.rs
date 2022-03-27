use crate::location::{BytePos, LineAndColumn};

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
/// use partiql_parser::location::LineAndColumn;
/// use partiql_parser::LineOffsetTracker;
///
/// let source = "12345\n789012345\n789012345\n789012345";
/// let mut tracker = LineOffsetTracker::default();
/// tracker.record(5..6);
/// tracker.record(15..16);
/// tracker.record(25..26);
///
/// // We added 3 newlines, so there should be 4 lines of source
/// assert_eq!(tracker.num_lines(), 4);
/// assert_eq!(tracker.at(source, 0), LineAndColumn::new(1,1).unwrap());
/// assert_eq!(tracker.at(source, 6), LineAndColumn::new(2,1).unwrap());
/// assert_eq!(tracker.at(source, 30), LineAndColumn::new(4,5).unwrap());
/// ```
pub struct LineOffsetTracker {
    line_starts: SmallVec<[BytePos; 16]>,
}

impl Default for LineOffsetTracker {
    fn default() -> Self {
        LineOffsetTracker {
            line_starts: smallvec![BytePos(0)], // line 1 starts at offset `0`
        }
    }
}

impl LineOffsetTracker {
    /// Record a newline at `span` in the source
    #[inline(always)]
    pub fn record(&mut self, span: Range<usize>) {
        self.line_starts.push(span.end.into());
    }

    /// Calculate the number of lines of source seen so far.
    #[inline(always)]
    pub fn num_lines(&self) -> usize {
        self.line_starts.len()
    }

    /// Calculates the byte offset span ([`Range`]) of a line.
    ///
    /// `num` is the line number (1-indexed) for which to  calculate the span
    /// `max` is the largest value allowable in the returned [`Range's end`](core::ops::Range)
    ///
    /// # Panics
    ///
    /// This function will panic if `num` is not within the number of lines
    /// of source seen (i.e. 1 <= `num` <= self.num_lines()`).
    #[inline(always)]
    fn line_span_from_line_num(&self, num: usize, max: usize) -> Range<usize> {
        assert!(1 <= num);
        assert!(num <= self.num_lines());
        let start = self.line_starts[num - 1].to_usize();
        let end = self
            .line_starts
            .get(num)
            .map(BytePos::to_usize)
            .unwrap_or(max)
            .min(max);
        start..end
    }

    /// Calculates the line number (1-indexed) in which a byte offset is contained.
    ///
    /// `offset` is the byte offset
    #[inline(always)]
    fn line_num_from_byte_offset(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset.into()) {
            Err(i) => i,
            Ok(i) => i + 1,
        }
    }

    /// Calculates a [`LineAndColumn`] for a byte offset from the given `&str`
    ///
    /// `source` is source `&str` into which the byte offset applies
    /// `offset` is the byte offset for which to find the [`LineAndColumn`]
    ///
    /// # Panics
    ///
    /// This function will panic if:
    ///  - `offset` is larger than the byte length of `source`, or
    ///  - `offset` falls inside a unicode codepoint
    #[inline]
    pub fn at(&self, source: &str, offset: usize) -> LineAndColumn {
        if offset == 0 {
            // SAFETY: `1` is always non-zero
            unsafe { LineAndColumn::new_unchecked(1, 1) }
        } else {
            let line_num = self.line_num_from_byte_offset(offset);
            let line_span = self.line_span_from_line_num(line_num, source.len());
            let column_num = source[line_span.start..=offset].chars().count();

            // SAFETY: line_num is always nonzero, see `line_num_from_byte_offset`
            // SAFETY: column_num is always nonzero
            //  `source[line_span.start..=offset]` is at least 1 char, else the slice would panic
            unsafe { LineAndColumn::new_unchecked(line_num, column_num) }
        }
    }
}

/// A 3-tuple of (start, `Tok`, end) denoting a token and it start and end offsets.
pub(crate) type Spanned<Tok, Loc> = (Loc, Tok, Loc);
/// A [`Result`] of a [`Spanned`] token.
pub(crate) type SpannedResult<Tok, Loc, Broke> = Result<Spanned<Tok, Loc>, Spanned<Broke, Loc>>;

/// Errors that can be encountered when lexing PartiQL.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LexicalError {
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

type CommentToken = SpannedResult<String, usize, LexicalError>;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(extras = &'s mut LineOffsetTracker)]
enum Comment {
    #[error]
    // Skip stuff that won't interfere with comment detection
    #[regex(r"[^/*\r\n\u0085\u2028\u2029]+", logos::skip)]
    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"(([\r])?[\n])|\u0085|\u2028|\u2029", 
        |lex| {lex.extras.record(lex.span()); logos::Skip})]
    Any,
    #[token("*/")]
    End,
    #[token("/*")]
    Start,
}

/// A lexer for block comments (enclosed between '/*' & '*/')
struct CommentLexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, Comment>,
    comment_nesting: bool,
}

impl<'a> CommentLexer<'a> {
    /// Creates a new block comment lexer over `input` text.
    /// Nested comment parsing is *off* by default; see [`with_nesting`] to enable nesting.
    pub fn new(input: &'a str, counter: &'a mut LineOffsetTracker) -> Self {
        CommentLexer {
            lexer: Comment::lexer_with_extras(input, counter),
            comment_nesting: false,
        }
    }

    /// Toggles *on* the parsing of nested comments
    fn with_nesting(mut self) -> Self {
        self.comment_nesting = true;
        self
    }

    /// Parses a single (possibly nested) block comment and returns it
    fn next(&mut self) -> Option<CommentToken> {
        let Span { start, .. } = self.lexer.span();
        let mut nesting = 0;
        let nesting_inc = if self.comment_nesting { 1 } else { 0 };
        'comment: loop {
            match self.lexer.next() {
                Some(Comment::Any) => continue,
                Some(Comment::Start) => nesting = max(1, nesting + nesting_inc),
                Some(Comment::End) => {
                    if nesting == 0 {
                        let Span { end, .. } = self.lexer.span();
                        return Some(Err((start, LexicalError::Unknown, end)));
                    }
                    nesting -= 1;
                    if nesting == 0 {
                        break 'comment;
                    }
                }
                None => {
                    return if nesting != 0 {
                        let Span { end, .. } = self.lexer.span();
                        Some(Err((start, LexicalError::UnterminatedComment, end)))
                    } else {
                        None
                    }
                }
            }
        }
        let Span { end, .. } = self.lexer.span();
        let comment = self.lexer.source()[start..end].to_owned();

        Some(Ok((start, comment, end)))
    }
}

impl<'a> Iterator for CommentLexer<'a> {
    type Item = CommentToken;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

type IonToken = SpannedResult<String, usize, LexicalError>;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(extras = &'s mut LineOffsetTracker)]
enum EmbeddedIon {
    #[error]
    // Skip stuff that doesn't interfere with comment or string detection
    #[regex(r#"[^/*'"`\r\n\u0085\u2028\u2029]+"#, logos::skip)]
    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"(([\r])?[\n])|\u0085|\u2028|\u2029",
        |lex| {lex.extras.record(lex.span()); logos::Skip})]
    Any,

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

/// A Lexer for ion literals embedded in backticks (`).
/// Parses just enough on to make sure not to include a backtick that is inside a string or comment.
struct EmbeddedIonLexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, EmbeddedIon>,
}

impl<'a> EmbeddedIonLexer<'a> {
    /// Creates a new embedded ion lexer over `input` text.
    pub fn new(input: &'a str, counter: &'a mut LineOffsetTracker) -> Self {
        EmbeddedIonLexer {
            lexer: EmbeddedIon::lexer_with_extras(input, counter),
        }
    }

    /// Parses a single embedded ion value, quoted between backticks (`), and returns it
    fn next(&mut self) -> Option<IonToken> {
        let next_token = self.lexer.next();
        match next_token {
            Some(EmbeddedIon::Embed) => {
                let Span { start, .. } = self.lexer.span();
                'ion_value: loop {
                    let next_tok = self.lexer.next();
                    match next_tok {
                        Some(EmbeddedIon::Embed) => {
                            break 'ion_value;
                        }
                        Some(EmbeddedIon::CommentBlock) => {
                            let embed_span = self.lexer.span();
                            let remaining = &self.lexer.source()[embed_span.start..];
                            let mut comment_lexer = CommentLexer::new(remaining, self.lexer.extras);
                            match comment_lexer.next() {
                                Some(Ok((s, _c, e))) => self.lexer.bump(e - s - embed_span.len()),
                                Some(Err((_s, err, e))) => {
                                    return Some(Err((embed_span.start, err, e)));
                                }
                                None => unreachable!(),
                            }
                        }
                        Some(EmbeddedIon::LongString) => {
                            'triple_quote: loop {
                                let next_tok = self.lexer.next();
                                match next_tok {
                                    Some(EmbeddedIon::LongString) => break 'triple_quote,
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
                            return Some(Err((start, LexicalError::UnterminatedIonLiteral, end)));
                        }
                    }
                }
                let Span { end, .. } = self.lexer.span();
                let ion_value = self.lexer.source()[start..end].to_owned();

                Some(Ok((start, ion_value, end)))
            }
            _ => None,
        }
    }
}

impl<'a> Iterator for EmbeddedIonLexer<'a> {
    type Item = IonToken;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// A lexer from PartiQL text strings to [`LexicalToken`]s
pub(crate) struct PartiqlLexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, Token>,
}

pub(crate) type LexicalToken = SpannedResult<Token, usize, LexicalError>;

impl<'a> PartiqlLexer<'a> {
    /// Creates a new PartiQL lexer over `input` text.
    pub fn new(input: &'a str, counter: &'a mut LineOffsetTracker) -> Self {
        PartiqlLexer {
            lexer: Token::lexer_with_extras(input, counter),
        }
    }

    /// Creates an error token at the current lexer location
    #[inline]
    fn err_here(&self, err_ctor: fn(String) -> LexicalError) -> LexicalToken {
        let region = self.lexer.slice().to_owned();
        let Span { start, end } = self.lexer.span();
        Err((start, err_ctor(region), end))
    }

    /// Wraps a [`Token`] into a [`LexicalToken`] at the current position of the lexer.
    #[inline(always)]
    fn wrap(&mut self, token: Token) -> LexicalToken {
        let Span { start, end } = self.lexer.span();
        Ok((start, token, end))
    }

    /// Advances the iterator and returns the next [`LexicalToken`] or [`None`] when input is exhausted.
    fn next(&mut self) -> Option<LexicalToken> {
        match self.lexer.next() {
            None => None,
            Some(token) => match token {
                Token::Error => Some(self.err_here(LexicalError::InvalidInput)),

                Token::EmbeddedIonQuote => {
                    let embed_span = self.lexer.span();
                    let remaining = &self.lexer.source()[embed_span.start..];
                    let mut ion_lexer = EmbeddedIonLexer::new(remaining, self.lexer.extras);
                    ion_lexer.next().map(|res| match res {
                        Ok((s, ion, e)) => {
                            self.lexer.bump(e - s - embed_span.len());
                            Ok((embed_span.start, Token::Ion(ion), e))
                        }
                        Err((_s, err, e)) => Err((embed_span.start, err, e)),
                    })
                }

                Token::CommentBlockStart => {
                    let embed_span = self.lexer.span();
                    let remaining = &self.lexer.source()[embed_span.start..];
                    let mut comment_lexer =
                        CommentLexer::new(remaining, self.lexer.extras).with_nesting();
                    comment_lexer.next().map(|res| match res {
                        Ok((s, comment, e)) => {
                            self.lexer.bump(e - s - embed_span.len());
                            Ok((embed_span.start, Token::CommentBlock(comment), e))
                        }
                        Err((_s, err, e)) => Err((embed_span.start, err, e)),
                    })
                }

                _ => Some(self.wrap(token)),
            },
        }
    }
}

impl<'a> Iterator for PartiqlLexer<'a> {
    type Item = LexicalToken;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// Tokens that the lexer can generate.
///
/// # Note
/// Tokens with names beginning with `__` are used internally and not meant to be used outside lexing.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(extras = &'s mut LineOffsetTracker)]
// TODO make pub(crate) ?
pub enum Token {
    #[error]
    // Skip whitespace
    #[regex(r"[ \t\f]+", logos::skip)]
    // Skip newlines, but record their position.
    // For line break recommendations,
    //   see https://www.unicode.org/standard/reports/tr13/tr13-5.html
    #[regex(r"([\r]?[\n])|\u{0085}|\u{2028}|\u{2029}",
        callback = |lex| {lex.extras.record(lex.span()); logos::Skip})]
    Error,

    #[regex(r"--[^\n]*", |lex| lex.slice().to_owned())]
    CommentLine(String),
    #[token("/*")]
    CommentBlockStart,
    CommentBlock(String),

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
    #[regex("[a-zA-Z_$][a-zA-Z0-9_$]*", |lex| lex.slice().to_owned())]
    // quoted identifiers (quoted with double quotes)
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#,
            |lex| lex.slice().trim_matches('"').to_owned())]
    Identifier(String),

    // unquoted @identifiers
    #[regex("@[a-zA-Z_$][a-zA-Z0-9_$]*", |lex| lex.slice()[1..].to_owned())]
    // quoted @identifiers (quoted with double quotes)
    #[regex(r#"@"([^"\\]|\\t|\\u|\\n|\\")*""#,
            |lex| lex.slice()[1..].trim_matches('"').to_owned())]
    AtIdentifier(String),

    #[regex("[0-9]+", |lex| lex.slice().to_owned())]
    Int(String),

    #[regex("[0-9]+\\.[0-9]*([eE][-+]?[0-9]+)", |lex| lex.slice().to_owned())]
    #[regex("\\.[0-9]+([eE][-+]?[0-9]+)", |lex| lex.slice().to_owned())]
    #[regex("[0-9]+[eE][-+]?[0-9]+", |lex| lex.slice().to_owned())]
    ExpReal(String),

    #[regex("[0-9]+\\.[0-9]*", |lex| lex.slice().to_owned())]
    #[regex("\\.[0-9]+", |lex| lex.slice().to_owned())]
    Real(String),

    // strings are single-quoted in SQL/PartiQL
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\'|(?:''))*'"#,
            |lex| lex.slice().trim_matches('\'').to_owned())]
    String(String),

    #[token("`")]
    EmbeddedIonQuote,
    Ion(String),

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

    #[test]
    fn ion_simple() {
        let ion_value = r#" `{'a':1,  'b':1}` "#;
        let mut offset_tracker = LineOffsetTracker::default();
        let mut lexer = PartiqlLexer::new(ion_value, &mut offset_tracker);

        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, (1, Token::Ion(ion_str), 17) if ion_str == ion_value.trim()));

        let mut offset_tracker = LineOffsetTracker::default();
        assert_eq!(
            EmbeddedIonLexer::new(ion_value.trim(), &mut offset_tracker)
                .into_iter()
                .count(),
            1
        );
        assert_eq!(offset_tracker.num_lines(), 1);
    }

    #[test]
    fn ion() {
        let ion_value = r#" `{'a' // comment ' "
                       :1, /* 
                               comment 
                              */
                      'b':1}` "#;
        let mut offset_tracker = LineOffsetTracker::default();
        let mut lexer = PartiqlLexer::new(ion_value, &mut offset_tracker);

        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, (1, Token::Ion(ion_str), 154) if ion_str == ion_value.trim()));
        assert_eq!(offset_tracker.num_lines(), 5);

        let mut offset_tracker = LineOffsetTracker::default();
        assert_eq!(
            EmbeddedIonLexer::new(ion_value.trim(), &mut offset_tracker)
                .into_iter()
                .count(),
            1
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
        let toks: Result<Vec<_>, Spanned<LexicalError, usize>> = nonnested_lex.collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(error, (142, LexicalError::Unknown, 189)));
        assert_eq!(error.1.to_string(), "Lexing error: unknown error");
    }

    #[test]
    fn select() -> Result<(), Spanned<LexicalError, usize>> {
        let query = "SELECT g\nFROM data\nGROUP BY a";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::Identifier("g".to_owned()),
                Token::From,
                Token::Identifier("data".to_owned()),
                Token::Group,
                Token::By,
                Token::Identifier("a".to_owned())
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        assert_eq!(offset_tracker.num_lines(), 3);
        assert_eq!(
            offset_tracker.at(query, 0),
            LineAndColumn::new(1, 1).unwrap()
        );
        assert_eq!(
            offset_tracker.at(query, 1),
            LineAndColumn::new(1, 2).unwrap()
        );
        assert_eq!(
            offset_tracker.at(query, 9),
            LineAndColumn::new(2, 1).unwrap()
        );
        assert_eq!(
            offset_tracker.at(query, 19),
            LineAndColumn::new(3, 1).unwrap()
        );

        let offset_r_a = query.rfind('a').unwrap();
        let offset_r_n = query.rfind('\n').unwrap();
        assert_eq!(
            offset_tracker.at(query, query.len() - 1),
            LineAndColumn::new(3, offset_r_a - offset_r_n).unwrap()
        );

        Ok(())
    }

    #[test]
    fn select_unicode() -> Result<(), Spanned<LexicalError, usize>> {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::Identifier("🐈".to_owned()),
                Token::From,
                Token::Identifier("❤ℝ".to_owned()),
                Token::Group,
                Token::By,
                Token::Identifier("🧸".to_owned())
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        assert_eq!(offset_tracker.num_lines(), 5);
        assert_eq!(
            offset_tracker.at(query, 0),
            LineAndColumn::new(1, 1).unwrap()
        );

        let offset_s = query.find('S').unwrap();
        assert_eq!(
            offset_tracker.at(query, offset_s),
            LineAndColumn::new(2, 1).unwrap()
        );

        let offset_f = query.find('F').unwrap();
        assert_eq!(
            offset_tracker.at(query, offset_f),
            LineAndColumn::new(3, 1).unwrap()
        );

        let offset_g = query.find('G').unwrap();
        assert_eq!(
            offset_tracker.at(query, offset_g),
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

        offset_tracker.at(query, query.len());
    }

    #[test]
    #[should_panic]
    fn panic_offset_into_codepoint() {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        lexer.count();

        offset_tracker.at(query, 1);
    }

    #[test]
    fn select_comment_line() -> Result<(), Spanned<LexicalError, usize>> {
        let query = "SELECT --comment\n@g from @\"foo\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentLine("--comment".to_owned()),
                Token::AtIdentifier("g".to_owned()),
                Token::From,
                Token::AtIdentifier("foo".to_owned()),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(offset_tracker.num_lines(), 2);
        Ok(())
    }

    #[test]
    fn select_comment_block() -> Result<(), Spanned<LexicalError, usize>> {
        let query = "SELECT /*comment*/ g";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentBlock("/*comment*/".to_owned()),
                Token::Identifier("g".to_owned()),
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
        let toks: Result<Vec<_>, Spanned<LexicalError, usize>> =
            PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert_eq!(error.1.to_string(), r##"Lexing error: invalid input `#`"##);
        assert!(matches!(error, (7, LexicalError::InvalidInput(s), 8) if s == "#"));
        assert_eq!(offset_tracker.num_lines(), 1);
        assert_eq!(
            offset_tracker.at(query, 7),
            LineAndColumn::new(1, 8).unwrap()
        );
    }

    #[test]
    fn err_unterminated_ion() {
        let query = r#" ` "fooo` "#;
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, Spanned<LexicalError, usize>> =
            PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (1, LexicalError::UnterminatedIonLiteral, 9)
        ));
        assert_eq!(
            error.1.to_string(),
            "Lexing error: unterminated ion literal"
        );
        assert_eq!(
            offset_tracker.at(query, 1),
            LineAndColumn::new(1, 2).unwrap()
        );
    }

    #[test]
    fn err_unterminated_comment() {
        let query = r#" /*12345678"#;
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, Spanned<LexicalError, usize>> =
            PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(error, (1, LexicalError::UnterminatedComment, 10)));
        assert_eq!(error.1.to_string(), "Lexing error: unterminated comment");
        assert_eq!(
            offset_tracker.at(query, 1),
            LineAndColumn::new(1, 2).unwrap()
        );
    }

    #[test]
    fn err_unterminated_ion_comment() {
        let query = r#" `/*12345678`"#;
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, Spanned<LexicalError, usize>> =
            EmbeddedIonLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(error, (2, LexicalError::UnterminatedComment, 11)));
        assert_eq!(error.1.to_string(), "Lexing error: unterminated comment");
        assert_eq!(
            offset_tracker.at(query, 2),
            LineAndColumn::new(1, 3).unwrap()
        );
    }
}
