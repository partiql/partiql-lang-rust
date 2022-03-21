use logos::{Logos, Span};
use std::cmp::max;
use thiserror::Error;

/// A 3-tuple of (start, `Tok`, end) denoting a token and it start and end offsets.
pub(crate) type Spanned<Tok, Loc> = (Loc, Tok, Loc);
/// A [`Result`] of a [`Spanned`] token.
pub(crate) type SpannedResult<Tok, Loc, Error> = Result<Spanned<Tok, Loc>, Error>;

/// Errors that can be encountered when lexing PartiQL.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LexicalError {
    /// Generic invalid input; likely an unrecognizable token.
    #[error("Lexing error: invalid input `{}`", _0.1)]
    InvalidInput(Spanned<String, usize>),
    /// Embedded Ion value is not properly terminated.
    #[error("Lexing error: unterminated ion literal")]
    UnterminatedIonLiteral(Spanned<(), usize>),
    /// Comment is not properly terminated.
    #[error("Lexing error: unterminated comment")]
    UnterminatedComment(Spanned<(), usize>),
    /// Any other lexing error.
    #[error("Lexing error: unknown error")]
    Unknown(Spanned<(), usize>),
}

type CommentToken = SpannedResult<String, usize, LexicalError>;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
enum Comment {
    #[error]
    #[regex(r"[^/*]+", logos::skip)]
    Any,
    #[token("*/")]
    End,
    #[token("/*")]
    Start,
}

/// A lexer for block comments (enclosed between '/*' & '*/')
pub(crate) struct CommentLexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, Comment>,
    comment_nesting: bool,
}

impl<'a> CommentLexer<'a> {
    /// Creates a new block comment lexer over `input` text.
    /// Nested comment parsing is *off* by default; see [`with_nesting`] to enable nesting.
    pub fn new(input: &'a str) -> Self {
        CommentLexer {
            lexer: Comment::lexer(input),
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
                        let comment = (start, (), end);
                        return Some(Err(LexicalError::Unknown(comment)));
                    }
                    nesting -= 1;
                    if nesting == 0 {
                        break 'comment;
                    }
                }
                None => {
                    return if nesting != 0 {
                        let Span { end, .. } = self.lexer.span();
                        let comment = (start, (), end);
                        Some(Err(LexicalError::UnterminatedComment(comment)))
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
pub enum EmbeddedIon {
    #[error]
    #[regex(r#"([^/*'"`])+"#, logos::skip)]
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
pub(crate) struct EmbeddedIonLexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, EmbeddedIon>,
}

impl<'a> EmbeddedIonLexer<'a> {
    /// Creates a new embedded ion lexer over `input` text.
    pub fn new(input: &'a str) -> Self {
        EmbeddedIonLexer {
            lexer: EmbeddedIon::lexer(input),
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
                            let mut comment_lexer = CommentLexer::new(remaining);
                            match comment_lexer.next() {
                                Some(Ok((s, _c, e))) => self.lexer.bump(e - s - embed_span.len()),
                                Some(Err(LexicalError::UnterminatedComment((_s, v, e)))) => {
                                    let loc = (embed_span.start, v, e);
                                    return Some(Err(LexicalError::UnterminatedComment(loc)));
                                }
                                err @ Some(Err(_)) => return err,
                                None => todo!(),
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
                            let comment = (start, (), end);
                            return Some(Err(LexicalError::UnterminatedIonLiteral(comment)));
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
pub(crate) struct Lexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, Token>,
}

type SpannedString = Spanned<String, usize>;
pub(crate) type LexicalToken = SpannedResult<Token, usize, LexicalError>;

impl<'a> Lexer<'a> {
    /// Creates a new PartiQL lexer over `input` text.
    pub fn new(input: &'a str) -> Self {
        Lexer {
            lexer: Token::lexer(input),
        }
    }

    /// Creates an error token at the current lexer location
    #[inline]
    fn err_here(&self, err_ctor: fn(SpannedString) -> LexicalError) -> LexicalToken {
        let region = self.lexer.slice().to_owned();
        let Span { start, end } = self.lexer.span();
        Err(err_ctor((start, region, end)))
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
                    let mut ion_lexer = EmbeddedIonLexer::new(remaining);
                    ion_lexer.next().map(|res| match res {
                        Ok((s, ion, e)) => {
                            self.lexer.bump(e - s - embed_span.len());
                            Ok((embed_span.start, Token::Ion(ion), e))
                        }
                        Err(LexicalError::UnterminatedIonLiteral((_s, v, e))) => Err(
                            LexicalError::UnterminatedIonLiteral((embed_span.start, v, e)),
                        ),
                        Err(e) => Err(e),
                    })
                }

                Token::CommentBlockStart => {
                    let embed_span = self.lexer.span();
                    let remaining = &self.lexer.source()[embed_span.start..];
                    let mut comment_lexer = CommentLexer::new(remaining).with_nesting();
                    comment_lexer.next().map(|res| match res {
                        Ok((s, comment, e)) => {
                            self.lexer.bump(e - s - embed_span.len());
                            Ok((embed_span.start, Token::CommentBlock(comment), e))
                        }
                        Err(LexicalError::UnterminatedComment((_s, v, e))) => {
                            Err(LexicalError::UnterminatedComment((embed_span.start, v, e)))
                        }
                        Err(e) => Err(e),
                    })
                }

                _ => Some(self.wrap(token)),
            },
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
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
// TODO make pub(crate) ?
pub enum Token {
    // Logos requires one token variant to handle errors,
    // it can be named anything you wish.
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
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
        let mut lexer = Lexer::new(ion_value);

        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, (1, Token::Ion(ion_str), 17) if ion_str == ion_value.trim()));

        assert_eq!(
            EmbeddedIonLexer::new(ion_value.trim()).into_iter().count(),
            1
        );
    }

    #[test]
    fn ion() {
        let ion_value = r#" `{'a' // comment ' "
                       :1, /* 
                               comment 
                              */
                      'b':1}` "#;
        let mut lexer = Lexer::new(ion_value);

        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, (1, Token::Ion(ion_str), 154) if ion_str == ion_value.trim()));

        assert_eq!(
            EmbeddedIonLexer::new(ion_value.trim()).into_iter().count(),
            1
        );
    }

    #[test]
    fn nested_comments() {
        let comments = r##"/*  
                                    /*  / * * * /
                                    /*  ' " ''' ` 
                                    */  text
                                    */  1 2 3 4 5 6,7,8,9 10.112^5
                                    */"##;

        let nested_lex = CommentLexer::new(comments).with_nesting();
        assert_eq!(nested_lex.into_iter().count(), 1);

        let nonnested_lex = CommentLexer::new(comments);
        let toks: Result<Vec<_>, LexicalError> = nonnested_lex.collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(error, LexicalError::Unknown((142, (), 189))));
        assert_eq!(error.to_string(), "Lexing error: unknown error");
    }

    #[test]
    fn select() -> Result<(), LexicalError> {
        let query = "SELECT g FROM data GROUP BY a";
        let lexer = Lexer::new(query);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::Identifier("g".to_owned()),
                Token::From,
                Token::Identifier("data".into()),
                Token::Group,
                Token::By,
                Token::Identifier("a".into())
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn select_comment_line() -> Result<(), LexicalError> {
        let query = "SELECT --comment\ng";
        let lexer = Lexer::new(query);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentLine("--comment".to_owned()),
                Token::Identifier("g".to_owned()),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn select_comment_block() -> Result<(), LexicalError> {
        let query = "SELECT /*comment*/ g";
        let lexer = Lexer::new(query);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentBlock("/*comment*/".to_owned()),
                Token::Identifier("g".to_owned()),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn err_invalid_input() {
        let query = "SELECT # FROM data GROUP BY a";
        let toks: Result<Vec<_>, LexicalError> = Lexer::new(query).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert_eq!(error.to_string(), r##"Lexing error: invalid input `#`"##);
        assert!(matches!(error, LexicalError::InvalidInput((7, s, 8)) if s == "#"));
    }

    #[test]
    fn err_unterminated_ion() {
        let query = r#" ` "fooo` "#;
        let toks: Result<Vec<_>, LexicalError> = Lexer::new(query).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            LexicalError::UnterminatedIonLiteral((1, (), 9))
        ));
        assert_eq!(error.to_string(), "Lexing error: unterminated ion literal");
    }

    #[test]
    fn err_unterminated_comment() {
        let query = r#" /*12345678"#;
        let toks: Result<Vec<_>, LexicalError> = Lexer::new(query).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            LexicalError::UnterminatedComment((1, (), 10))
        ));
        assert_eq!(error.to_string(), "Lexing error: unterminated comment");
    }

    #[test]
    fn err_unterminated_ion_comment() {
        let query = r#" `/*12345678`"#;
        let toks: Result<Vec<_>, LexicalError> = EmbeddedIonLexer::new(query).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            LexicalError::UnterminatedComment((2, (), 11))
        ));
        assert_eq!(error.to_string(), "Lexing error: unterminated comment");
    }
}
