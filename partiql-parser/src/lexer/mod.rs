use partiql_common::syntax::location::{ByteOffset, BytePosition, ToLocated};

use crate::error::{LexError, ParseError};

mod comment;
mod embedded_ion;
mod partiql;

pub use comment::*;
pub use embedded_ion::*;
pub use partiql::*;

/// A 3-tuple of (start, `Tok`, end) denoting a token and it start and end offsets.
pub type Spanned<Tok, Loc> = (Loc, Tok, Loc);
/// A [`Result`] of a [`Spanned`] token.
pub(crate) type SpannedResult<Tok, Loc, Broke> = Result<Spanned<Tok, Loc>, Spanned<Broke, Loc>>;

pub(crate) type InternalLexResult<'input> =
    SpannedResult<Token<'input>, ByteOffset, LexError<'input>>;
pub(crate) type LexResult<'input> =
    Result<Spanned<Token<'input>, ByteOffset>, ParseError<'input, BytePosition>>;

impl<'input> From<Spanned<LexError<'input>, ByteOffset>> for ParseError<'input, BytePosition> {
    fn from(res: Spanned<LexError<'input>, ByteOffset>) -> Self {
        let (start, cause, end) = res;
        ParseError::LexicalError(
            cause.to_located(BytePosition::from(start)..BytePosition::from(end)),
        )
    }
}

/// A lexer that wraps another lexer and skips comments.
pub(crate) struct CommentSkippingLexer<'input, L>
where
    L: Iterator<Item = LexResult<'input>>,
{
    lexer: L,
}

impl<'input, L> CommentSkippingLexer<'input, L>
where
    L: Iterator<Item = LexResult<'input>>,
{
    /// Creates a new `CommentSkippingLexer` wrapping `lexer`
    #[inline]
    pub fn new(lexer: L) -> Self {
        Self { lexer }
    }
}

impl<'input, L> Iterator for CommentSkippingLexer<'input, L>
where
    L: Iterator<Item = LexResult<'input>>,
{
    type Item = LexResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        'next_tok: loop {
            let next = self.lexer.next();
            if matches!(
                next,
                Some(Ok((_, Token::CommentBlock(_) | Token::CommentLine(_), _)))
            ) {
                continue 'next_tok;
            }
            return next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_common::syntax::line_offset_tracker::{LineOffsetError, LineOffsetTracker};
    use partiql_common::syntax::location::{
        CharOffset, LineAndCharPosition, LineAndColumn, LineOffset, Located, Location,
    };

    use itertools::Itertools;

    #[test]
    fn display() -> Result<(), ParseError<'static, BytePosition>> {
        let symbols =
            "( [ { } ] ) << >> ; , < > <= >= != <> = == - + * ? % / ^ . || : --foo /*block*/";
        let primitives = r#"unquoted_ident "quoted_ident" @unquoted_atident @"quoted_atident""#;
        let keywords =
            "WiTH Where Value uSiNg Unpivot UNION True Select right Preserve pivoT Outer Order Or \
             On Offset Nulls Null Not Natural Missing Limit Like Left Lateral Last Join \
             Intersect Is Inner In Having Group From For Full First False Except Escape Desc \
             Cross Table Time Timestamp Date By Between At As And Asc All Values Case When Then Else End";
        let symbols = symbols.split(' ').chain(primitives.split(' '));
        let keywords = keywords.split(' ');

        let text = symbols.interleave(keywords).join("\n");
        let s = text.as_str();

        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(s, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>().unwrap();

        #[rustfmt::skip]
        let expected = vec![
            "(", "WITH", "[", "WHERE", "{", "VALUE", "}", "USING", "]", "UNPIVOT", ")", "UNION",
            "<<", "TRUE", ">>", "SELECT", ";", "RIGHT", ",", "PRESERVE", "<", "PIVOT", ">", "OUTER",
            "<=", "ORDER", ">=", "OR", "!=", "ON", "<>", "OFFSET", "=", "NULLS", "==", "NULL", "-",
            "NOT", "+", "NATURAL", "*", "MISSING", "?", "LIMIT", "%", "LIKE", "/", "LEFT", "^",
            "LATERAL", ".", "LAST", "||", "JOIN", ":", "INTERSECT", "--", "IS", "/**/", "INNER",
            "<unquoted_ident:UNQUOTED_IDENT>", "IN", "<quoted_ident:QUOTED_IDENT>", "HAVING",
            "<unquoted_atident:UNQUOTED_ATIDENT>", "GROUP", "<quoted_atident:QUOTED_ATIDENT>",
            "FROM", "FOR", "FULL", "FIRST", "FALSE", "EXCEPT", "ESCAPE", "DESC", "CROSS", "TABLE",
            "TIME", "TIMESTAMP", "DATE", "BY", "BETWEEN", "AT", "AS", "AND", "ASC", "ALL", "VALUES",
            "CASE", "WHEN", "THEN", "ELSE", "END"
        ];
        let displayed = toks
            .into_iter()
            .map(|(_s, t, _e)| t.to_string())
            .collect::<Vec<_>>();
        assert_eq!(expected, displayed);

        Ok(())
    }

    #[test]
    fn ion_simple() {
        let ion_value = r"    `{'input':1,  'b':1}`--comment ";

        let mut offset_tracker = LineOffsetTracker::default();
        let ion_lexer = EmbeddedIonLexer::new(ion_value.trim(), &mut offset_tracker);
        assert_eq!(ion_lexer.into_iter().count(), 1);
        assert_eq!(offset_tracker.num_lines(), 1);

        let mut offset_tracker = LineOffsetTracker::default();
        let mut lexer = PartiqlLexer::new(ion_value, &mut offset_tracker);

        let tok = lexer.next().unwrap().unwrap();
        assert!(
            matches!(tok, (ByteOffset(5), Token::Ion(ion_str), ByteOffset(24)) if ion_str == "{'input':1,  'b':1}")
        );
        let tok = lexer.next().unwrap().unwrap();
        assert!(
            matches!(tok, (ByteOffset(25), Token::CommentLine(cmt_str), ByteOffset(35)) if cmt_str == "--comment ")
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
            matches!(tok, (ByteOffset(2), Token::Ion(ion_str), ByteOffset(158)) if ion_str == ion_value.trim().trim_matches('`'))
        );
        assert_eq!(offset_tracker.num_lines(), 5);
    }

    #[test]
    fn nested_comments() {
        let comments = r#"/*  
                                    /*  / * * * /
                                    /*  ' " ''' ` 
                                    */  text
                                    */  1 2 3 4 5 6,7,8,9 10.112^5
                                    */"#;

        // track nested comments
        let mut offset_tracker = LineOffsetTracker::default();
        let nested_lex = CommentLexer::new(comments, &mut offset_tracker).with_nesting();
        let count = nested_lex.into_iter().count();
        assert_eq!(count, 1);
        assert_eq!(offset_tracker.num_lines(), 6);

        // don't track nested comments
        let mut offset_tracker = LineOffsetTracker::default();
        let nonnested_lex = CommentLexer::new(comments, &mut offset_tracker);
        let toks: Result<Vec<_>, Spanned<LexError<'_>, ByteOffset>> = nonnested_lex.collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (
                ByteOffset(187),
                LexError::UnterminatedComment,
                ByteOffset(189)
            )
        ));
        assert_eq!(error.1.to_string(), "Lexing error: unterminated comment");
    }

    #[test]
    fn select() -> Result<(), ParseError<'static, BytePosition>> {
        let query = r#"SELECT g
            FROM "data"
            GROUP BY a"#;
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        let mut pre_offset_tracker = LineOffsetTracker::default();
        let pre_lexer = PartiqlLexer::new(query, &mut pre_offset_tracker);
        let pre_toks: Vec<_> = pre_lexer.collect::<Result<_, _>>()?;

        let expected_toks = vec![
            Token::Select,
            Token::UnquotedIdent("g"),
            Token::From,
            Token::QuotedIdent("data"),
            Token::Group,
            Token::By,
            Token::UnquotedIdent("a"),
        ];
        assert_eq!(
            expected_toks,
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(
            expected_toks,
            pre_toks
                .into_iter()
                .map(|(_s, t, _e)| t)
                .collect::<Vec<_>>()
        );

        assert_eq!(offset_tracker.num_lines(), 3);
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 0.into()).unwrap()),
            LineAndColumn::new(1, 1).unwrap()
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 1.into()).unwrap()),
            LineAndColumn::new(1, 2).unwrap()
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 9.into()).unwrap()),
            LineAndColumn::new(2, 1).unwrap()
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 19.into()).unwrap()),
            LineAndColumn::new(2, 11).unwrap()
        );

        let offset_r_a = query.rfind('a').unwrap();
        let offset_r_n = query.rfind('\n').unwrap();
        assert_eq!(
            LineAndColumn::from(
                offset_tracker
                    .at(query, BytePosition::from(query.len() - 1))
                    .unwrap()
            ),
            LineAndColumn::new(3, offset_r_a - offset_r_n).unwrap()
        );

        Ok(())
    }

    #[test]
    fn select_unicode() -> Result<(), ParseError<'static, BytePosition>> {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::QuotedIdent("🐈"),
                Token::From,
                Token::QuotedIdent("❤ℝ"),
                Token::Group,
                Token::By,
                Token::QuotedIdent("🧸")
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        assert_eq!(offset_tracker.num_lines(), 5);
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 0.into()).unwrap()),
            LineAndColumn::new(1, 1).unwrap()
        );

        let offset_s = query.find('S').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, offset_s.into()).unwrap()),
            LineAndColumn::new(2, 1).unwrap()
        );

        let offset_f = query.find('F').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, offset_f.into()).unwrap()),
            LineAndColumn::new(3, 1).unwrap()
        );

        let offset_g = query.find('G').unwrap();
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, offset_g.into()).unwrap()),
            LineAndColumn::new(5, 1).unwrap()
        );

        Ok(())
    }

    #[test]
    fn offset_overflow() {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY x";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        lexer.count();

        let last = offset_tracker.at(query, ByteOffset(query.len() as u32).into());
        assert!(matches!(
            last,
            Ok(LineAndCharPosition {
                line: LineOffset(4),
                char: CharOffset(10)
            })
        ));

        let overflow = offset_tracker.at(query, ByteOffset(1 + query.len() as u32).into());
        assert!(matches!(overflow, Err(LineOffsetError::EndOfInput)));
    }

    #[test]
    fn offset_into_codepoint() {
        let query = "\u{2028}SELECT \"🐈\"\r\nFROM \"❤\u{211D}\"\u{2029}\u{0085}GROUP BY \"🧸\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        lexer.count();

        assert_eq!(
            offset_tracker.at(query, ByteOffset(1).into()),
            Err(LineOffsetError::InsideUnicodeCodepoint)
        );
    }

    #[test]
    fn select_comment_line() -> Result<(), ParseError<'static, BytePosition>> {
        let query = "SELECT --comment\n@g from @\"foo\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentLine("--comment"),
                Token::UnquotedAtIdentifier("g"),
                Token::From,
                Token::QuotedAtIdentifier("foo"),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(offset_tracker.num_lines(), 2);
        Ok(())
    }

    #[test]
    fn select_comment_block() -> Result<(), ParseError<'static, BytePosition>> {
        let query = "SELECT /*comment*/ g";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::CommentBlock("/*comment*/"),
                Token::UnquotedIdent("g"),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(offset_tracker.num_lines(), 1);
        Ok(())
    }

    /// In the future, the following identifiers may be converted into reserved keywords. In that case,
    /// the following test will need to be modified.
    #[test]
    fn select_non_reserved_keywords() -> Result<(), ParseError<'static, BytePosition>> {
        let query =
            "SELECT acyclic, BoTh, DOMAIN, SiMpLe, Trail, leading, TRailing, USER\nfrom @\"foo\"";
        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PartiqlLexer::new(query, &mut offset_tracker);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::Select,
                Token::UnquotedIdent("acyclic"),
                Token::Comma,
                Token::UnquotedIdent("BoTh"),
                Token::Comma,
                Token::UnquotedIdent("DOMAIN"),
                Token::Comma,
                Token::UnquotedIdent("SiMpLe"),
                Token::Comma,
                Token::UnquotedIdent("Trail"),
                Token::Comma,
                Token::UnquotedIdent("leading"),
                Token::Comma,
                Token::UnquotedIdent("TRailing"),
                Token::Comma,
                Token::UnquotedIdent("USER"),
                Token::From,
                Token::QuotedAtIdentifier("foo"),
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );
        assert_eq!(offset_tracker.num_lines(), 2);
        Ok(())
    }

    #[test]
    fn err_invalid_input() {
        let query = "SELECT # FROM data GROUP BY a";
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, _> = PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert_eq!(
            error.to_string(),
            r"Lexing error: invalid input `#` at `(b7..b8)`"
        );
        assert!(matches!(error,
            ParseError::LexicalError(Located {
                inner: LexError::InvalidInput(s),
                location: Location{start: BytePosition(ByteOffset(7)), end: BytePosition(ByteOffset(8))}
            }) if s == "#"));
        assert_eq!(offset_tracker.num_lines(), 1);
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, 7.into()).unwrap()),
            LineAndColumn::new(1, 8).unwrap()
        );
    }

    #[test]
    fn err_unterminated_ion() {
        let query = r#" ` "fooo` "#;
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, _> = PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();

        assert!(matches!(
            error,
            ParseError::LexicalError(Located {
                inner: LexError::UnterminatedIonLiteral,
                location: Location {
                    start: BytePosition(ByteOffset(1)),
                    end: BytePosition(ByteOffset(10))
                }
            })
        ));
        assert_eq!(
            error.to_string(),
            "Lexing error: unterminated ion literal at `(b1..b10)`"
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(1)).unwrap()),
            LineAndColumn::new(1, 2).unwrap()
        );
    }

    #[test]
    fn err_unterminated_comment() {
        let query = r" /*12345678";
        let mut offset_tracker = LineOffsetTracker::default();
        let toks: Result<Vec<_>, _> = PartiqlLexer::new(query, &mut offset_tracker).collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            ParseError::LexicalError(Located {
                inner: LexError::UnterminatedComment,
                location: Location {
                    start: BytePosition(ByteOffset(1)),
                    end: BytePosition(ByteOffset(11))
                }
            })
        ));
        assert_eq!(
            error.to_string(),
            "Lexing error: unterminated comment at `(b1..b11)`"
        );
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(1)).unwrap()),
            LineAndColumn::new(1, 2).unwrap()
        );
    }

    #[test]
    fn err_unterminated_ion_comment() {
        let query = r" `/*12345678`";
        let mut offset_tracker = LineOffsetTracker::default();
        let ion_lexer = EmbeddedIonLexer::new(query, &mut offset_tracker);
        let toks: Result<Vec<_>, Spanned<LexError<'_>, ByteOffset>> = ion_lexer.collect();
        assert!(toks.is_err());
        let error = toks.unwrap_err();
        assert!(matches!(
            error,
            (ByteOffset(2), LexError::UnterminatedComment, ByteOffset(13))
        ));
        assert_eq!(error.1.to_string(), "Lexing error: unterminated comment");
        assert_eq!(
            LineAndColumn::from(offset_tracker.at(query, BytePosition::from(2)).unwrap()),
            LineAndColumn::new(1, 3).unwrap()
        );
    }
}
