use crate::error::LexError;
use crate::lexer::{CommentLexer, EmbeddedDocLexer, InternalLexResult, LexResult};
use logos::{Logos, Span};
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use partiql_common::syntax::location::ByteOffset;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Formatter;

/// A lexer from `PartiQL` text strings to [`Token`]s
pub(crate) struct PartiqlLexer<'input, 'tracker> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'input, Token<'input>>,
    tracker: &'tracker mut LineOffsetTracker,
}

impl<'input, 'tracker> PartiqlLexer<'input, 'tracker> {
    /// Creates a new `PartiQL` lexer over `input` text.
    #[inline]
    pub fn new(input: &'input str, tracker: &'tracker mut LineOffsetTracker) -> Self {
        PartiqlLexer {
            lexer: Token::lexer(input),
            tracker,
        }
    }

    /// Creates an error token at the current lexer location
    #[inline]
    fn err_here(
        &self,
        err_ctor: fn(Cow<'input, str>) -> LexError<'input>,
    ) -> InternalLexResult<'input> {
        let region = self.lexer.slice();
        let Span { start, end } = self.lexer.span();
        Err((start.into(), err_ctor(region.into()), end.into()))
    }

    #[inline(always)]
    pub fn slice(&self) -> &'input str {
        self.lexer.slice()
    }

    /// Wraps a [`Token`] into a [`Token`] at the current position of the lexer.
    #[inline(always)]
    fn wrap(&mut self, token: Token<'input>) -> InternalLexResult<'input> {
        let Span { start, end } = self.lexer.span();
        Ok((start.into(), token, end.into()))
    }

    /// Advances the iterator and returns the next [`Token`] or [`None`] when input is exhausted.
    #[inline]
    pub(crate) fn next_internal(&mut self) -> Option<InternalLexResult<'input>> {
        'next_tok: loop {
            return match self.lexer.next() {
                None => None,
                Some(Ok(token)) => match token {
                    Token::Newline => {
                        self.tracker.record(self.lexer.span().end.into());
                        // Newlines shouldn't generate an externally visible token
                        continue 'next_tok;
                    }

                    Token::EmbeddedDocQuote => self.parse_embedded_doc(),
                    Token::EmptyEmbeddedDocQuote => self.parse_empty_embedded_doc(),

                    Token::CommentBlockStart => self.parse_block_comment(),

                    _ => Some(self.wrap(token)),
                },
                Some(Err(_)) => Some(self.err_here(LexError::InvalidInput)),
            };
        }
    }

    /// Uses [`CommentLexer`] to parse a block comment
    fn parse_block_comment(&mut self) -> Option<InternalLexResult<'input>> {
        let embed = self.lexer.span();
        let remaining = &self.lexer.source()[embed.start..];
        let mut comment_tracker = LineOffsetTracker::default();
        let mut comment_lexer = CommentLexer::new(remaining, &mut comment_tracker).with_nesting();
        comment_lexer.next().map(|res| match res {
            Ok((s, comment, e)) => {
                let val_len = e - s;
                let val_start = embed.start.into(); // embed end is 1 past the starting '/*'
                let val_end = val_start + val_len;
                self.tracker.append(&comment_tracker, embed.start.into());
                self.lexer.bump(val_len.to_usize() - embed.len());
                Ok((val_start, Token::CommentBlock(comment), val_end))
            }
            Err((s, err, e)) => {
                let offset: ByteOffset = embed.start.into();
                Err((s + offset, err, e + offset))
            }
        })
    }

    /// Uses [`EmbeddedDocLexer`] to parse an embedded doc value
    fn parse_embedded_doc(&mut self) -> Option<InternalLexResult<'input>> {
        let embed = self.lexer.span();
        let remaining = &self.lexer.source()[embed.start..];
        let mut doc_tracker = LineOffsetTracker::default();
        let mut doc_lexer = EmbeddedDocLexer::new(remaining, &mut doc_tracker);
        doc_lexer.next().map(|res| match res {
            Ok((s, doc, e)) => {
                let val_len = e - s;
                let val_start = embed.start.into(); // embed end is 1 past the starting '/*'
                let val_end = val_start + val_len;
                self.tracker.append(&doc_tracker, embed.start.into());
                self.lexer.bump(val_len.to_usize() - embed.len());
                Ok((val_start, Token::EmbeddedDoc(doc), val_end))
            }
            Err((s, err, e)) => {
                let offset: ByteOffset = embed.start.into();
                Err((s + offset, err, e + offset))
            }
        })
    }

    #[inline]
    fn parse_empty_embedded_doc(&mut self) -> Option<InternalLexResult<'input>> {
        let embed = self.lexer.span();
        let mid = embed.start + ((embed.end - embed.start) / 2);
        let doc = &self.lexer.source()[mid..mid];
        Some(self.wrap(Token::EmbeddedDoc(doc)))
    }
}

impl<'input, 'tracker> Iterator for PartiqlLexer<'input, 'tracker> {
    type Item = LexResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_internal()
            .map(|res| res.map_err(std::convert::Into::into))
    }
}

/// Tokens that the lexer can generate.
///
/// # Note
/// Tokens with names beginning with `__` are used internally and not meant to be used outside lexing.
#[derive(Logos, Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
// TODO make pub(crate) ?
// Skip whitespace
#[logos(skip r"[ \t\f]+")]
pub enum Token<'input> {
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
    #[token("?")]
    SqlParameter,
    #[token("%")]
    Percent,
    #[token("/")]
    Slash,
    #[token("^")]
    Caret,
    #[token(".")]
    Period,
    #[token("||")]
    DblPipe,

    // unquoted identifiers
    #[regex("[a-zA-Z_$][a-zA-Z0-9_$]*", |lex| lex.slice())]
    UnquotedIdent(&'input str),

    // quoted identifiers (quoted with double quotes)
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#,
            |lex| lex.slice().trim_matches('"'))]
    QuotedIdent(&'input str),

    // unquoted @identifiers
    #[regex("@[a-zA-Z_$][a-zA-Z0-9_$]*", |lex| &lex.slice()[1..])]
    UnquotedAtIdentifier(&'input str),

    // quoted @identifiers (quoted with double quotes)
    #[regex(r#"@"([^"\\]|\\t|\\u|\\n|\\")*""#,
            |lex| lex.slice()[1..].trim_matches('"'))]
    QuotedAtIdentifier(&'input str),

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
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\'|\\|(?:''))*'"#,
        |lex| lex.slice().trim_matches('\''))]
    String(&'input str),

    // An embed open/close tag is a (greedily-captured) odd-number of backticks
    #[regex(r"`(``)*")]
    EmbeddedDocQuote,
    // An empty embedded doc is a (greedily-captured) even-number of backticks
    #[regex(r"(``)+")]
    EmptyEmbeddedDocQuote,
    EmbeddedDoc(&'input str),

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
    #[regex("(?i:Case)")]
    Case,
    #[regex("(?i:Cross)")]
    Cross,
    #[regex("(?i:Cycle)")]
    Cycle,
    #[regex("(?i:Date)")]
    Date,
    #[regex("(?i:Desc)")]
    Desc,
    #[regex("(?i:Distinct)")]
    Distinct,
    #[regex("(?i:Else)")]
    Else,
    #[regex("(?i:End)")]
    End,
    #[regex("(?i:Escape)")]
    Escape,
    #[regex("(?i:Except)")]
    Except,
    #[regex("(?i:Exclude)")]
    Exclude,
    #[regex("(?i:False)")]
    False,
    #[regex("(?i:First)")]
    First,
    #[regex("(?i:For)")]
    For,
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
    #[regex("(?i:Partial)")]
    Partial,
    #[regex("(?i:Pivot)")]
    Pivot,
    #[regex("(?i:Preserve)")]
    Preserve,
    #[regex("(?i:Right)")]
    Right,
    #[regex("(?i:Recursive)")]
    Recursive,
    #[regex("(?i:Select)")]
    Select,
    #[regex("(?i:Search)")]
    Search,
    #[regex("(?i:Table)")]
    Table,
    #[regex("(?i:Time)")]
    Time,
    #[regex("(?i:Timestamp)")]
    Timestamp,
    #[regex("(?i:Then)")]
    Then,
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
    #[regex("(?i:Values)")]
    Values,
    #[regex("(?i:When)")]
    When,
    #[regex("(?i:Where)")]
    Where,
    #[regex("(?i:With)")]
    With,
    #[regex("(?i:Without)")]
    Without,
    #[regex("(?i:Zone)")]
    Zone,
}

impl<'input> Token<'input> {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::All
                | Token::Asc
                | Token::And
                | Token::As
                | Token::At
                | Token::Between
                | Token::By
                | Token::Case
                | Token::Cross
                | Token::Cycle
                | Token::Date
                | Token::Desc
                | Token::Distinct
                | Token::Escape
                | Token::Except
                | Token::First
                | Token::For
                | Token::Full
                | Token::From
                | Token::Group
                | Token::Having
                | Token::In
                | Token::Inner
                | Token::Is
                | Token::Intersect
                | Token::Join
                | Token::Last
                | Token::Lateral
                | Token::Left
                | Token::Like
                | Token::Limit
                | Token::Missing
                | Token::Natural
                | Token::Not
                | Token::Null
                | Token::Nulls
                | Token::Offset
                | Token::On
                | Token::Or
                | Token::Order
                | Token::Outer
                | Token::Partial
                | Token::Pivot
                | Token::Preserve
                | Token::Right
                | Token::Recursive
                | Token::Search
                | Token::Select
                | Token::Table
                | Token::Time
                | Token::Timestamp
                | Token::Then
                | Token::Union
                | Token::Unpivot
                | Token::Using
                | Token::Value
                | Token::Values
                | Token::Where
                | Token::With
        )
    }
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Newline => write!(f, "\\n"),
            Token::CommentLine(_) => write!(f, "--"),
            Token::CommentBlockStart => write!(f, "/*"),
            Token::CommentBlock(_) => write!(f, "/**/"),
            Token::OpenSquare => write!(f, "["),
            Token::CloseSquare => write!(f, "]"),
            Token::OpenCurly => write!(f, "{{"),
            Token::CloseCurly => write!(f, "}}"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenDblAngle => write!(f, "<<"),
            Token::CloseDblAngle => write!(f, ">>"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::EqualEqual => write!(f, "=="),
            Token::BangEqual => write!(f, "!="),
            Token::LessGreater => write!(f, "<>"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::Equal => write!(f, "="),
            Token::LessThan => write!(f, "<"),
            Token::GreaterThan => write!(f, ">"),
            Token::Minus => write!(f, "-"),
            Token::Plus => write!(f, "+"),
            Token::Star => write!(f, "*"),
            Token::SqlParameter => write!(f, "?"),
            Token::Percent => write!(f, "%"),
            Token::Slash => write!(f, "/"),
            Token::Caret => write!(f, "^"),
            Token::Period => write!(f, "."),
            Token::DblPipe => write!(f, "||"),
            Token::UnquotedIdent(id) => write!(f, "<{id}:UNQUOTED_IDENT>"),
            Token::QuotedIdent(id) => write!(f, "<{id}:QUOTED_IDENT>"),
            Token::UnquotedAtIdentifier(id) => write!(f, "<{id}:UNQUOTED_ATIDENT>"),
            Token::QuotedAtIdentifier(id) => write!(f, "<{id}:QUOTED_ATIDENT>"),
            Token::Int(txt) => write!(f, "<{txt}:INT>"),
            Token::ExpReal(txt) => write!(f, "<{txt}:REAL>"),
            Token::Real(txt) => write!(f, "<{txt}:REAL>"),
            Token::String(txt) => write!(f, "<{txt}:STRING>"),
            Token::EmbeddedDocQuote => write!(f, "<DOC>"),
            Token::EmbeddedDoc(txt) => write!(f, "<```{txt}```:DOC>"),
            Token::EmptyEmbeddedDocQuote => write!(f, "<``:DOC>"),

            Token::All
            | Token::Asc
            | Token::And
            | Token::As
            | Token::At
            | Token::Between
            | Token::By
            | Token::Case
            | Token::Cross
            | Token::Cycle
            | Token::Date
            | Token::Desc
            | Token::Distinct
            | Token::Else
            | Token::End
            | Token::Escape
            | Token::Except
            | Token::Exclude
            | Token::False
            | Token::First
            | Token::For
            | Token::Full
            | Token::From
            | Token::Group
            | Token::Having
            | Token::In
            | Token::Inner
            | Token::Is
            | Token::Intersect
            | Token::Join
            | Token::Last
            | Token::Lateral
            | Token::Left
            | Token::Like
            | Token::Limit
            | Token::Missing
            | Token::Natural
            | Token::Not
            | Token::Null
            | Token::Nulls
            | Token::Offset
            | Token::On
            | Token::Or
            | Token::Order
            | Token::Outer
            | Token::Partial
            | Token::Pivot
            | Token::Preserve
            | Token::Right
            | Token::Recursive
            | Token::Search
            | Token::Select
            | Token::Table
            | Token::Time
            | Token::Timestamp
            | Token::Then
            | Token::True
            | Token::Union
            | Token::Unpivot
            | Token::Using
            | Token::Value
            | Token::Values
            | Token::When
            | Token::Where
            | Token::With
            | Token::Without
            | Token::Zone => {
                write!(f, "{}", format!("{self:?}").to_uppercase())
            }
        }
    }
}
