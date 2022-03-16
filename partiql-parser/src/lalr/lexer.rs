use logos::{Logos, Span};
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
    #[error("Lexing error: invalid input `{:?}`", .0)]
    InvalidInput(Spanned<String, usize>),
    /// Embedded Ion value is not properly terminated.
    #[error("Lexing error: unterminated ion literal")]
    UnterminatedIonLiteral(Spanned<(), usize>),
    /// Any other lexing error.
    #[error("Lexing error: unknown error")]
    Unknown,
}

/// A lexer from PartiQL text strings to [`LexicalToken`]s
pub(crate) struct Lexer<'a> {
    /// Wrap a logos-generated lexer
    lexer: logos::Lexer<'a, Token>,
}

type SpannedString = Spanned<String, usize>;
pub(crate) type LexicalToken = SpannedResult<Token, usize, LexicalError>;

#[derive(PartialEq)]
enum StringStatus {
    None,
    Single,
    Double,
    Triple,
}

impl StringStatus {
    pub fn in_string(&self) -> bool {
        !self.is_none()
    }
    pub fn is_none(&self) -> bool {
        self == &StringStatus::None
    }
    #[allow(unused)]
    pub fn is_single(&self) -> bool {
        self == &StringStatus::Single
    }
    pub fn is_double(&self) -> bool {
        self == &StringStatus::Double
    }
    #[allow(unused)]
    pub fn is_triple(&self) -> bool {
        self == &StringStatus::Double
    }
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer over `input` text.
    pub fn new(input: &'a str) -> Self {
        Lexer {
            lexer: Token::lexer(input),
        }
    }

    /// Creates an error token at the specified location
    #[inline]
    fn err_at(
        &self,
        start: usize,
        end: usize,
        err_ctor: fn(Spanned<(), usize>) -> LexicalError,
    ) -> LexicalToken {
        Err(err_ctor((start, (), end)))
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

    /// Parses ion literals embedded in backticks (`).
    /// Parses just enough on to make sure not to include a backtick that is inside a string or comment.
    fn ion_string(&mut self) -> LexicalToken {
        let Span { start, .. } = self.lexer.span();
        let remainder: &str = self.lexer.remainder();

        let mut rest = remainder.chars();
        let mut string_status = StringStatus::None;
        'ion_val: loop {
            let curr = rest.next();
            match curr {
                None => {
                    let curr_pos = remainder.len() - rest.as_str().len();
                    return self.err_at(start, curr_pos, LexicalError::UnterminatedIonLiteral);
                }
                Some(c) => {
                    match c {
                        '/' if string_status.is_none() => {
                            match rest.next() {
                                Some('/') => {
                                    // We're inside a single line comment now; consume rest of line
                                    'ion_comment: loop {
                                        match rest.next() {
                                            None => continue 'ion_val,        // error; end of input
                                            Some('\n') => break 'ion_comment, // end of comment
                                            _ => continue 'ion_comment,       // more comment to go
                                        }
                                    }
                                }
                                Some('*') => {
                                    // We're inside a multi lime comment now; consume until '*/'
                                    'ion_multiline_comment: loop {
                                        match rest.next() {
                                            None => continue 'ion_val, // error; end of input
                                            Some('*') => {
                                                match rest.next() {
                                                    None => continue 'ion_val,                 // error; end of input
                                                    Some('/') => break 'ion_multiline_comment, // end of comment
                                                    _ => continue 'ion_multiline_comment, // more comment to go
                                                }
                                            }
                                            _ => continue 'ion_multiline_comment, // more comment to go
                                        }
                                    }
                                }
                                _ => {
                                    // TODO error?
                                }
                            }
                        }
                        '\\' => {
                            if string_status.in_string() {
                                // We're inside a string and just saw a '\'
                                // interpret the next character as an escape and just consume it
                                rest.next();
                            } else {
                                // TODO error?
                            }
                        }
                        '"' if string_status.is_none() => {
                            string_status = StringStatus::Double;
                        }
                        '"' if string_status.is_double() => {
                            string_status = StringStatus::None;
                        }
                        '\'' => {
                            match string_status {
                                StringStatus::Double => {
                                    // nothing to do; `'` is part of a double-quoted string
                                }
                                StringStatus::Single => {
                                    // already in a single-quoted string; terminate it here
                                    string_status = StringStatus::None;
                                }
                                StringStatus::Triple => {
                                    // already in a triple-quoted string
                                    if rest.as_str().starts_with("''") {
                                        // last `'` is followed by 2 more; consume them and terminate string here
                                        rest.next();
                                        rest.next();
                                        string_status = StringStatus::None;
                                    }
                                }
                                StringStatus::None => {
                                    // not yet in a string. determine whether to enter single or triple
                                    if rest.as_str().starts_with("''") {
                                        // last `'` is followed by 2 more; consume them and start triple-quoted string here
                                        rest.next();
                                        rest.next();
                                        string_status = StringStatus::Triple;
                                    } else {
                                        // start single-quote string here
                                        string_status = StringStatus::Single;
                                    }
                                }
                            }
                        }
                        '`' if string_status.is_none() => {
                            let curr_pos = remainder.len() - rest.as_str().len();
                            let contents = &remainder[..curr_pos - 1];
                            self.lexer.bump(curr_pos);
                            return Ok((start, Token::Ion(contents.to_owned()), curr_pos));
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    /// Advances the iterator and returns the next [`LexicalToken`] or [`None`] when input is exhausted.
    fn next(&mut self) -> Option<LexicalToken> {
        match self.lexer.next() {
            None => None,
            Some(token) => match token {
                Token::Error => Some(self.err_here(LexicalError::InvalidInput)),
                // TODO: use logos::Lexer.morph to actually lex ion?
                Token::__BackQuote => Some(self.ion_string()),
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

    // Internal use only
    #[token("`")]
    __BackQuote,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ion() {
        let ion_value = r#" `{'a' // comment ' "
                       :1, /* 
                               comment 
                              */
                      'b':1}` "#;
        let mut lexer = Lexer::new(ion_value);
        if let (_start, Token::Ion(s), _end) = lexer.next().unwrap().unwrap() {
            assert_eq!(ion_value.trim().trim_matches('`'), s);
        } else {
            panic!("Lex failed")
        }
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
    fn err_invalid_input() {
        let query = "SELECT # FROM data GROUP BY a";
        let toks: Result<Vec<_>, LexicalError> = Lexer::new(query).collect();
        assert!(toks.is_err());
        match toks.unwrap_err() {
            LexicalError::InvalidInput((s, t, e)) => {
                assert_eq!(t, "#");
                assert_eq!(s, 7);
                assert_eq!(e, 8);
            }
            _ => panic!("Error should be LexicalError::InvalidInput"),
        }
    }

    #[test]
    fn err_unterminated_ion() {
        let query = r#" ` "fooo` "#;
        let toks: Result<Vec<_>, LexicalError> = Lexer::new(query).collect();
        assert!(toks.is_err());
        match toks.unwrap_err() {
            LexicalError::UnterminatedIonLiteral((s, _t, e)) => {
                assert_eq!(s, 1);
                assert_eq!(e, 8);
            }
            _ => panic!("Error should be LexicalError::UnterminatedIonLiteral"),
        }
    }
}
