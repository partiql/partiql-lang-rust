use partiql_common::syntax::location::ByteOffset;
use regex::{Regex, RegexSet, RegexSetBuilder};

use std::collections::VecDeque;

use std::ops::Range;

use crate::error::LexError;

use crate::lexer::{InternalLexResult, LexResult, PartiqlLexer, Spanned, Token};

use crate::token_parser::{BufferedToken, TokenParser};
use once_cell::sync::Lazy;
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;

pub(crate) static BUILT_INS: Lazy<FnExprSet<'static>> = Lazy::new(built_ins);

/// A single "function expression" argument match.
#[derive(Debug, Clone)]
pub(crate) enum FnExprArgMatch<'a> {
    /// Any 1 [`Token`] that is not function punctuation (i.e., '(', ')', ',') and potentially not a keyword.
    ///
    /// Generally this will be followed by a [`AnyZeroOrMore`] match, in order to match 1 or more [`Token`]s
    /// `bool` tuple value denotes if keyword is allowed to be considered as match.
    AnyOne(bool),
    /// 0 or more [`Token`]s that are not function punctuation (i.e., '(', ')', ',') and not a keyword
    ///
    /// Generally this will be preceded by a [`AnyOne`] match, in order to match 1 or more [`Token`]s
    /// `bool` tuple value denotes if keyword is allowed to be considered as match.
    AnyZeroOrMore(bool),
    /// Explicitly match a single [`Token`]
    #[allow(dead_code)]
    Match(Token<'a>),
    /// Explicitly match a [`Token`] that is a keyword that represents a 'named' argument.
    ///
    /// For example, the `for` in `substring(x for 2)`
    NamedArgKw(Token<'a>),
    /// Match a regex against an identifier that represents a 'named' argument.
    ///
    /// For example, the `leading` in `trim(leading ' ' from x)`
    NamedArgId(Regex),
    /// Synthesize a value for a named argument if one is not explicitly provided. Used for default values.
    ///
    /// For example, in `trim(leading from x)`, there is a synthesized `' '` that is the target of the
    /// `leading` such that it is interpreted as `trim(leading ' ' from x)`
    Synthesize(Token<'a>),
}

/// A "function expression" argument match list.
pub(crate) type FnExprArgList<'a> = Vec<FnExprArgMatch<'a>>;

/// A "function expression" match.
#[derive(Debug, Clone)]
pub(crate) struct FnExpr<'a> {
    /// Name(s) of "function expression"
    pub fn_names: Vec<&'a str>,
    /// A collection of possible argument patterns for this function expression.
    pub patterns: Vec<FnExprArgList<'a>>,
}

mod built_ins {
    use super::{FnExpr, FnExprArgMatch, Token};
    use regex::Regex;

    use FnExprArgMatch::{
        AnyOne, AnyZeroOrMore as AnyStar, NamedArgId as Id, NamedArgKw as Kw, Synthesize as Syn,
    };

    const TRIM_SPECIFIER: &str = "(?i:leading)|(?i:trailing)|(?i:both)";

    pub(crate) fn built_in_trim() -> FnExpr<'static> {
        let re = Regex::new(TRIM_SPECIFIER).unwrap();
        FnExpr {
            fn_names: vec!["trim"],
            #[rustfmt::skip]
            patterns: vec![
                // e.g., trim(leading 'tt' from x) => trim("leading": 'tt', "from": x)
                vec![Id(re.clone()), AnyOne(true), AnyStar(false), Kw(Token::From), AnyOne(true), AnyStar(false)],
                // e.g., trim(trailing from x) => trim("trailing": ' ', "from": x)
                vec![Id(re), Syn(Token::String(" ")), Kw(Token::From), AnyOne(true), AnyStar(false)],
                // e.g., trim(' ' from x) => trim(' ', "from": x)
                vec![AnyOne(true), AnyStar(false), Kw(Token::From), AnyOne(true), AnyStar(false)],
                // e.g., trim(from x) => trim("from": x)
                vec![Kw(Token::From), AnyOne(true), AnyStar(false)],
            ],
        }
    }

    const EXTRACT_SPECIFIER: &str =
        "(?i:second)|(?i:minute)|(?i:hour)|(?i:day)|(?i:month)|(?i:year)|(?i:timezone_hour)|(?i:timezone_minute)";

    pub(crate) fn built_in_extract() -> FnExpr<'static> {
        let re = Regex::new(EXTRACT_SPECIFIER).unwrap();
        FnExpr {
            fn_names: vec!["extract"],
            #[rustfmt::skip]
            patterns: vec![
                // e.g., extract(day from x) => extract("day":true, "from": x)
                // Note the `true` passed to Any* as we need to support type-related keywords after `FROM`
                // such as `TIME WITH TIME ZONE`
                vec![Id(re), Syn(Token::True), Kw(Token::From), AnyOne(true), AnyStar(true)]
            ],
        }
    }

    pub(crate) fn built_in_position() -> FnExpr<'static> {
        FnExpr {
            fn_names: vec!["position"],
            #[rustfmt::skip]
            patterns: vec![
                // e.g. position('foo' in 'xyzfooxyz') => position('foo', in: 'xyzfooxyz')
                vec![AnyOne(true), AnyStar(false), Kw(Token::In), AnyOne(true), AnyStar(false)]
            ],
        }
    }

    const PLACING: &str = "(?i:placing)";
    pub(crate) fn built_in_overlay() -> FnExpr<'static> {
        let re = Regex::new(PLACING).unwrap();
        FnExpr {
            fn_names: vec!["overlay"],
            #[rustfmt::skip]
            patterns: vec![
                // `OVERLAY('hello' PLACING 'XX' FROM 2 FOR 3)` => overlay('hello', PLACING: 'XX', from: 2, for: 3)
                vec![AnyOne(true), AnyStar(false), Id(re.clone()), AnyOne(true), AnyStar(false), Kw(Token::From), AnyOne(true), AnyStar(false), Kw(Token::For), AnyOne(true), AnyStar(false)],
                // `OVERLAY('hello' PLACING 'XX' FROM 2)` => overlay('hello', PLACING: 'XX', from: 2)
                vec![AnyOne(true), AnyStar(false), Id(re), AnyOne(true), AnyStar(false), Kw(Token::From), AnyOne(true), AnyStar(false)],
            ],
        }
    }

    pub(crate) fn built_in_aggs() -> FnExpr<'static> {
        FnExpr {
            // TODO: currently needs to be manually kept in-sync with parsers's `KNOWN_AGGREGATES`
            fn_names: vec!["count", "avg", "min", "max", "sum", "any", "some", "every"],
            #[rustfmt::skip]
            patterns: vec![
                // e.g., count(all x) => count("all": x)
                vec![Kw(Token::All), AnyOne(true), AnyStar(false)],
                // e.g., count(distinct x) => count("distinct": x)
                vec![Kw(Token::Distinct), AnyOne(true), AnyStar(false)],
            ],
        }
    }

    pub(crate) fn built_in_substring() -> FnExpr<'static> {
        FnExpr {
            fn_names: vec!["substring"],
            #[rustfmt::skip]
            patterns: vec![
                // e.g. substring(x from 2 for 3) => substring(x, "from":2, "for":3)
                vec![AnyOne(true), AnyStar(false), Kw(Token::From), AnyOne(true), AnyStar(false), Kw(Token::For), AnyOne(true), AnyStar(false)],
                // e.g. substring(x from 2) => substring(x, "from":2)
                vec![AnyOne(true), AnyStar(false), Kw(Token::From), AnyOne(true), AnyStar(false)],
                // e.g. substring(x for 3) => substring(x, "for":3)
                vec![AnyOne(true), AnyStar(false), Kw(Token::For), AnyOne(true), AnyStar(false)],
            ],
        }
    }

    pub(crate) fn built_in_cast() -> FnExpr<'static> {
        FnExpr {
            fn_names: vec!["cast"],
            #[rustfmt::skip]
            patterns: vec![
                // e.g., cast(9 as VARCHAR(5)) => cast(9 "as": VARCHAR(5))
                // Note the `true` passed to Any* as we need to support type-related keywords after `AS`
                vec![AnyOne(true), AnyStar(false), Kw(Token::As), AnyOne(true), AnyStar(true)]
            ],
        }
    }
}

/// A set of "function expression"s.
///
/// Note: Regexes are added to `fn_names` in the same order as arguments are added to `fn_exprs`, so
/// the index returned by `fn_names.match(_)` can be used to index into `fn_exprs` to find the corresponding
/// argument list.
#[derive(Debug, Clone)]
pub(crate) struct FnExprSet<'a> {
    /// A [`RegexSet`] that is the union of multiple "function expression" names
    fn_names: RegexSet,
    /// A union of multiple "function expression" matches
    fn_exprs: Vec<FnExpr<'a>>,
}

impl<'a> FnExprSet<'a> {
    pub fn new(fn_exprs: Vec<FnExpr<'a>>) -> Self {
        let pats = fn_exprs.iter().map(|spc| {
            if spc.fn_names.len() == 1 {
                spc.fn_names[0].to_owned()
            } else {
                spc.fn_names
                    .iter()
                    .map(|n| format!("(?:{n})"))
                    .collect::<Vec<_>>()
                    .join("|")
            }
        });
        let fn_names = RegexSetBuilder::new(pats)
            .case_insensitive(true)
            .build()
            .unwrap();
        FnExprSet { fn_names, fn_exprs }
    }

    /// Find the [`FnExpr`] corresponding to a given function name, if it exists.
    #[inline]
    pub fn find(&self, name: &'a str) -> Option<&FnExpr<'a>> {
        self.fn_names
            .matches(name)
            .into_iter()
            .next()
            .map(|idx| &self.fn_exprs[idx])
    }

    /// `true` if there is a [`FnExpr`] corresponding to a given function name.
    #[inline]
    pub fn contains(&self, name: &'a str) -> bool {
        self.fn_names.is_match(name)
    }
}

pub(crate) fn built_ins() -> FnExprSet<'static> {
    FnExprSet::new(vec![
        built_ins::built_in_trim(),
        built_ins::built_in_aggs(),
        built_ins::built_in_extract(),
        built_ins::built_in_position(),
        built_ins::built_in_overlay(),
        built_ins::built_in_substring(),
        built_ins::built_in_cast(),
    ])
}

type SpannedToken<'input> = Spanned<Token<'input>, ByteOffset>;
type SpannedTokenVec<'input> = Vec<SpannedToken<'input>>;

/// The outcome of attempting to match a [`Token`] against [`FnExprArgMatch`] requirements.
#[derive(Debug, Clone)]
enum ArgMatch<'input> {
    /// Match failed.
    Failed,
    /// Match succeeded and should consume the specified number of [`Tokens`]s.
    Consume(usize),
    /// Match succeeded and should consume the specified number of [`Tokens`]s and replace those
    /// [`Token`]s with the specified replacements.
    Replace((usize, SpannedTokenVec<'input>)),
}

/// A preprocessor over [`PartiqlLexer`]'s stream of [`Token`]s that transforms SQL-style function
/// expressions (e.g., `trim(leading ' ' from '   blah    ')`) into PartiQL-style functions with
/// named arguments (e.g., `trim("leading": ' ', "from" : '   blah   ')`).
///
/// This parses an "Island Grammar" subset of SQL/PartiQL; It recognizes just enough punctuation,
/// identifiers, and keywords to perform the required translations of SQL-style function expressions
/// into rewritten token streams representing function calls with named arguments.
///
/// It internally buffers tokens only when a function expression rewrite occurs.
pub(crate) struct PreprocessingPartiqlLexer<'input, 'tracker>
where
    'input: 'tracker,
{
    fn_exprs: &'input FnExprSet<'input>,
    parser: TokenParser<'input, 'tracker>,
    buff: VecDeque<InternalLexResult<'input>>,
}

type Substitutions<'input> = Vec<Option<SpannedTokenVec<'input>>>;

impl<'input, 'tracker> PreprocessingPartiqlLexer<'input, 'tracker>
where
    'input: 'tracker,
{
    /// Creates a new `PartiQL` lexer over `input` text.
    #[inline]
    pub fn new(
        input: &'input str,
        tracker: &'tracker mut LineOffsetTracker,
        fn_exprs: &'input FnExprSet<'input>,
    ) -> PreprocessingPartiqlLexer<'input, 'tracker> {
        PreprocessingPartiqlLexer {
            fn_exprs,
            parser: TokenParser::new(PartiqlLexer::new(input, tracker)),
            buff: VecDeque::with_capacity(20),
        }
    }

    /// Advances the iterator and returns the next [`Token`] or [`None`] when input is exhausted,
    /// performing preprocessing of SQL-style function expressions.
    #[inline]
    fn next(&mut self) -> Option<InternalLexResult<'input>> {
        if !self.buff.is_empty() {
            self.buff.pop_front()
        } else {
            match self.parser.consume() {
                Some(Ok(())) => match self.parser.flush_1() {
                    None => None,
                    Some(token) => {
                        let (tok, buffered) = self.parse_fn_expr(token, 0);
                        if let Some(buffered) = buffered {
                            self.buff.extend(buffered.into_iter().map(Ok));
                        }
                        Some(Ok(tok))
                    }
                },
                Some(Err(e)) => {
                    self.buff
                        .extend(self.parser.flush().into_iter().map(|(t, _)| Ok(t)));
                    self.buff.push_back(Err(e));
                    self.buff.pop_front()
                }
                None => None,
            }
        }
    }

    /// If the next [`Token`] is the start of a function expression, then parse the expression and
    /// determine its substitution, else just return the token.
    #[inline]
    fn parse_fn_expr(
        &mut self,
        (tok, _): BufferedToken<'input>,
        next_idx: usize,
    ) -> (SpannedToken<'input>, Option<SpannedTokenVec<'input>>) {
        if let (_, Token::UnquotedIdent(id) | Token::QuotedIdent(id), _) = tok {
            if let Some(((_, Token::OpenParen, _), _)) = self.parser.peek_n(next_idx) {
                if let Some(fn_expr) = self.fn_exprs.find(id) {
                    let replacement = match self.rewrite_fn_expr(fn_expr) {
                        Ok(rewrites) => rewrites,
                        Err(_err) => self.parser.flush().into_iter().map(|(t, _)| t).collect(),
                    };
                    return (tok, Some(replacement));
                }
            }
        }

        (tok, None)
    }

    /// Parse and rewrite the [`Token`]s representing the specified function expression.
    fn rewrite_fn_expr(
        &mut self,
        fn_expr: &'input FnExpr<'input>,
    ) -> Result<SpannedTokenVec<'input>, Spanned<LexError<'input>, ByteOffset>> {
        // Consume the opening '('
        self.parser.expect(&Token::OpenParen)?;

        let fn_expr_args = &fn_expr.patterns;
        let mut patterns: Vec<(&[FnExprArgMatch<'_>], Substitutions<'_>)> = fn_expr_args
            .iter()
            .map(|args| (args.as_slice(), vec![]))
            .collect();

        let mut nesting = 1;
        let mut span: Option<Range<ByteOffset>> = None;
        while nesting > 0 && !patterns.is_empty() {
            let is_nested = nesting > 1;
            let next_tok = self.parser.peek_n(0);
            match &next_tok {
                None => break,
                Some(buffered @ ((s, tok, e), _)) => {
                    span = match span {
                        None => Some(*s..*e),
                        Some(range) => Some(range.start..*e),
                    };
                    match tok {
                        Token::OpenParen
                        | Token::OpenSquare
                        | Token::OpenDblAngle
                        | Token::OpenCurly => {
                            patterns.iter_mut().for_each(|(_, subs)| subs.push(None));
                            nesting += 1;
                            self.parser.consume();
                        }
                        Token::CloseParen
                        | Token::CloseSquare
                        | Token::CloseDblAngle
                        | Token::CloseCurly => {
                            patterns.iter_mut().for_each(|(_, subs)| subs.push(None));
                            nesting -= 1;
                            self.parser.consume();
                        }
                        Token::CommentBlock(_) | Token::CommentLine(_) => {
                            patterns.iter_mut().for_each(|(_, subs)| subs.push(None));
                            self.parser.consume();
                        }
                        Token::UnquotedIdent(id) | Token::QuotedIdent(id)
                            if self.fn_exprs.contains(id) =>
                        {
                            let buffered: BufferedToken<'input> = (*buffered).clone();
                            // backup the state of the buffered tokens
                            let backup = self.parser.flush();

                            // try to parse the identifier as the beginning of a nested fn_expr
                            self.parser.consume();
                            let name = self.parser.flush_1();
                            let (first, rest) = self.parse_fn_expr(buffered.clone(), 0);

                            // re-buffer the identifier and backed-up buffered tokens
                            self.parser.unflush_1(name);
                            self.parser.unflush(backup);

                            // see whether parsing as a nested fn_expr succeeded
                            match rest {
                                Some(substitutions) => {
                                    // the identifier parsed as a nested fn_expr
                                    let replacement: Vec<_> =
                                        std::iter::once(first).chain(substitutions).collect();
                                    patterns = self.process_patterns(
                                        &buffered,
                                        is_nested,
                                        patterns,
                                        Some(replacement),
                                    );
                                }
                                None => {
                                    // could not parse the identifier as a fn_expr; put back the identifier
                                    self.parser.unconsume();
                                    patterns =
                                        self.process_patterns(&buffered, is_nested, patterns, None);
                                    self.parser.consume();
                                }
                            }
                        }
                        _ => {
                            let buffered = (*buffered).clone();
                            patterns = self.process_patterns(&buffered, is_nested, patterns, None);
                            self.parser.consume();
                        }
                    }
                }
            }
        }

        // Get the first successful matching pattern's substitutions
        let pattern =
            patterns
                .into_iter()
                .find_map(|(args, subs)| if args.len() == 1 { Some(subs) } else { None });

        // Rewrite the consumed tokens as per the substitution list
        match pattern {
            None => {
                let range = span.unwrap_or_else(|| 0.into()..0.into());
                Err((range.start, LexError::Unknown, range.end))
            }
            Some(subs) => Ok(Self::rewrite_tokens(self.parser.flush(), subs)),
        }
    }

    /// Match the next [`Token`] against a list of patterns and return a new list of patterns.
    ///
    /// Matches that fail are removed from the list.
    /// For matches that succeed:
    ///   - the match slice is advanced to the next [`FnExprArgMatch`] if appropriate
    ///   - the substitution list is built up to include replacement information
    #[inline]
    fn process_patterns(
        &mut self,
        buffered: &BufferedToken<'input>,
        is_nested: bool,
        patterns: Vec<(&'input [FnExprArgMatch<'input>], Substitutions<'input>)>,
        token_replacement: Option<Vec<SpannedToken<'input>>>,
    ) -> Vec<(&'input [FnExprArgMatch<'input>], Substitutions<'input>)> {
        // TODO replace with Vec#retain_mut when stable
        // See https://github.com/rust-lang/rust/issues/90829
        patterns
            .into_iter()
            .filter_map(|(args, mut subs)| {
                match self.match_arg(buffered, is_nested, subs.is_empty(), args) {
                    ArgMatch::Failed => None,
                    ArgMatch::Consume(n) => {
                        subs.push(token_replacement.clone());
                        args.get(n..).map(|a| (a, subs))
                    }
                    ArgMatch::Replace((n, r)) => {
                        subs.push(Some(r));
                        args.get(n..).map(|a| (a, subs))
                    }
                }
            })
            .collect()
    }

    /// Match a single [`BufferedToken`] against [`FnExprArgMatch`] requirements.
    ///
    ///# Arguments
    ///
    /// * `tok` - The current [`Token`] being considered for matching.
    /// * `is_nested` - Whether the preprocessor is considering [`Tokens`] inside a nested expression
    ///   (i.e., inside parens).
    /// * `is_init_arg` - Whether this is the first argument being considered for the function expression's
    ///   parameters.
    /// * `matchers` - A slice of the remaining arguments for a single pattern for the function expression.
    #[allow(clippy::only_used_in_recursion)]
    fn match_arg(
        &self,
        tok: &BufferedToken<'input>,
        is_nested: bool,
        is_init_arg: bool,
        matchers: &[FnExprArgMatch<'input>],
    ) -> ArgMatch<'input> {
        use FnExprArgMatch::{AnyOne, AnyZeroOrMore, Match, NamedArgId, NamedArgKw, Synthesize};

        match (&matchers[0], tok) {
            (AnyZeroOrMore(_), _) if is_nested => ArgMatch::Consume(0),
            (AnyZeroOrMore(keyword_allowed), ((_, t, _), _)) => match &matchers.get(1) {
                Some(_m) => match self.match_arg(tok, is_nested, false, &matchers[1..]) {
                    ArgMatch::Failed => {
                        if (t.is_keyword() && !keyword_allowed) || t == &Token::Comma {
                            ArgMatch::Failed
                        } else {
                            ArgMatch::Consume(0)
                        }
                    }
                    ArgMatch::Consume(n) => ArgMatch::Consume(n + 1),
                    ArgMatch::Replace((n, r)) => ArgMatch::Replace((n + 1, r)),
                },
                None => {
                    if (t.is_keyword() && !keyword_allowed) || t == &Token::Comma {
                        ArgMatch::Failed
                    } else {
                        ArgMatch::Consume(0)
                    }
                }
            },
            (AnyOne(_), _) if is_nested => ArgMatch::Consume(1),
            (AnyOne(_), ((_, Token::Comma, _), _)) => ArgMatch::Failed,
            (AnyOne(keyword_allowed), ((_, t, _), _)) if t.is_keyword() && !keyword_allowed => {
                ArgMatch::Failed
            }
            (AnyOne(_), _) => ArgMatch::Consume(1),
            (Match(target), ((_, tok, _), _)) if target == tok => ArgMatch::Consume(1),
            (NamedArgId(re), (tok_id @ (s, Token::UnquotedIdent(id), e), _)) if re.is_match(id) => {
                let args = [
                    (*s, Token::Comma, *s),
                    tok_id.clone(),
                    (*e, Token::Colon, *e),
                ];
                let args = if is_init_arg { &args[1..] } else { &args }.to_owned();
                ArgMatch::Replace((1, args))
            }
            (NamedArgKw(kw), ((s, t, e), txt)) if kw == t => {
                let args = [
                    (*s, Token::Comma, *s),
                    (*s, Token::QuotedIdent(txt), *e),
                    (*e, Token::Colon, *e),
                ];
                let args = if is_init_arg { &args[1..] } else { &args }.to_owned();
                ArgMatch::Replace((1, args))
            }
            (Synthesize(syn), ((s, _, _), _)) => match &matchers.get(1) {
                Some(_m) => match self.match_arg(tok, false, false, &matchers[1..]) {
                    ArgMatch::Failed => ArgMatch::Failed,
                    ArgMatch::Consume(n) => ArgMatch::Replace((n + 1, vec![(*s, syn.clone(), *s)])),
                    ArgMatch::Replace((n, mut r)) => {
                        r.insert(0, (*s, syn.clone(), *s));
                        ArgMatch::Replace((n + 1, r))
                    }
                },
                None => ArgMatch::Failed,
            },
            (_, _) => ArgMatch::Failed,
        }
    }

    /// Rewrite the buffered [`Token`]s representing the function expression as per the substitution
    /// list that was built up during parsing.
    #[inline]
    fn rewrite_tokens(
        mut toks: Vec<BufferedToken<'input>>,
        substitution: Substitutions<'input>,
    ) -> SpannedTokenVec<'input> {
        // insert initial '('
        let mut rewrite = vec![toks[0].0.clone()];

        // insert subsequent tokens or their substitutions
        for ((t, _), r) in std::iter::zip(toks.drain(1..toks.len() - 1), substitution) {
            match (r, t) {
                (None, t) => rewrite.push(t),
                (Some(subs), _) => rewrite.extend(subs),
            }
        }

        // insert final ')'
        if let Some(t) = toks.pop().map(|(t, _)| t) {
            rewrite.push(t);
        }

        rewrite
    }
}

impl<'input, 'tracker> Iterator for PreprocessingPartiqlLexer<'input, 'tracker>
where
    'input: 'tracker,
{
    type Item = LexResult<'input>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next().map(|res| res.map_err(std::convert::Into::into))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;

    use crate::ParseError;

    #[test]
    fn cast() -> Result<(), ParseError<'static>> {
        let query = "CAST(a AS VARCHAR)";

        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PreprocessingPartiqlLexer::new(query, &mut offset_tracker, &BUILT_INS);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        assert_eq!(
            vec![
                Token::UnquotedIdent("CAST"),
                Token::OpenParen,
                Token::UnquotedIdent("a"),
                Token::Comma,
                Token::QuotedIdent("AS"),
                Token::Colon,
                Token::UnquotedIdent("VARCHAR"),
                Token::CloseParen,
            ],
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        Ok(())
    }

    #[test]
    fn composed() -> Result<(), ParseError<'static>> {
        let query =
            "cast(trim(LEADING 'Foo' from substring('BarFooBar' from 4 for 6)) AS VARCHAR(20))";

        let mut offset_tracker = LineOffsetTracker::default();
        let lexer = PreprocessingPartiqlLexer::new(query, &mut offset_tracker, &BUILT_INS);
        let toks: Vec<_> = lexer.collect::<Result<_, _>>()?;

        let substring_expect = vec![
            Token::UnquotedIdent("substring"),
            Token::OpenParen,
            Token::String("BarFooBar"),
            Token::Comma,
            Token::QuotedIdent("from"),
            Token::Colon,
            Token::Int("4"),
            Token::Comma,
            Token::QuotedIdent("for"),
            Token::Colon,
            Token::Int("6"),
            Token::CloseParen,
        ];

        let trim_expect = [
            vec![
                Token::UnquotedIdent("trim"),
                Token::OpenParen,
                Token::UnquotedIdent("LEADING"),
                Token::Colon,
                Token::String("Foo"),
                Token::Comma,
                Token::QuotedIdent("from"),
                Token::Colon,
            ],
            substring_expect,
            vec![Token::CloseParen],
        ]
        .concat();

        let cast_expect = [
            vec![Token::UnquotedIdent("cast"), Token::OpenParen],
            trim_expect,
            vec![
                Token::Comma,
                Token::QuotedIdent("AS"),
                Token::Colon,
                Token::UnquotedIdent("VARCHAR"),
                Token::OpenParen,
                Token::Int("20"),
                Token::CloseParen,
                Token::CloseParen,
            ],
        ]
        .concat();

        assert_eq!(
            cast_expect,
            toks.into_iter().map(|(_s, t, _e)| t).collect::<Vec<_>>()
        );

        Ok(())
    }

    #[test]
    fn preprocessor() -> Result<(), ParseError<'static>> {
        fn to_tokens<'a>(
            lexer: impl Iterator<Item = LexResult<'a>>,
        ) -> Result<Vec<Token<'a>>, ParseError<'a>> {
            lexer
                .map(|result| result.map(|(_, t, _)| t))
                .collect::<Result<Vec<_>, _>>()
        }
        fn lex(query: &str) -> Result<Vec<Token<'_>>, ParseError<'_>> {
            let mut offset_tracker = LineOffsetTracker::default();
            let lexer = PartiqlLexer::new(query, &mut offset_tracker);
            to_tokens(lexer)
        }
        fn preprocess(query: &str) -> Result<Vec<Token<'_>>, ParseError<'_>> {
            let mut offset_tracker = LineOffsetTracker::default();
            let lexer = PreprocessingPartiqlLexer::new(query, &mut offset_tracker, &BUILT_INS);
            to_tokens(lexer)
        }
        assert_eq!(
            preprocess(r"trim(both from missing)")?,
            lex(r#"trim(both: ' ', "from": missing)"#)?
        );

        // Valid, but missing final paren
        assert_eq!(
            preprocess(r"substring('FooBar' from 2 for 3")?,
            lex(r#"substring('FooBar', "from": 2, "for": 3"#)?
        );

        assert_eq!(
            preprocess(r"trim(LEADING 'Foo' from 'FooBar')")?,
            lex(r#"trim(LEADING : 'Foo', "from" : 'FooBar')"#)?
        );

        assert_eq!(
            preprocess(r"trim(LEADING /*blah*/ 'Foo' from 'FooBar')")?,
            lex(r#"trim(LEADING : /*blah*/ 'Foo', "from" : 'FooBar')"#)?
        );

        assert_eq!(
            preprocess(
                r"trim(LEADING --blah
                                             'Foo' from 'FooBar')"
            )?,
            lex(r#"trim(LEADING : --blah
                                         'Foo', "from" : 'FooBar')"#)?
        );

        // Trim Specification in all 3 spots
        assert_eq!(
            preprocess(r"trim(BOTH TrAiLiNg from TRAILING)")?,
            lex(r#"trim(BOTH : TrAiLiNg, "from" : TRAILING)"#)?
        );

        // Trim specification in 1st and 2nd spot
        assert_eq!(
            preprocess(r"trim(LEADING LEADING from 'FooBar')")?,
            lex(r#"trim(LEADING : LEADING, "from" : 'FooBar')"#)?
        );
        assert_eq!(
            preprocess(r"trim(LEADING TrAiLiNg from 'FooBar')")?,
            lex(r#"trim(LEADING : TrAiLiNg, "from" : 'FooBar')"#)?
        );
        assert_eq!(
            preprocess(r"trim(tRaIlInG TrAiLiNg from 'FooBar')")?,
            lex(r#"trim(tRaIlInG : TrAiLiNg, "from" : 'FooBar')"#)?
        );

        // Trim specification in 1st and 3rd spot
        assert_eq!(
            preprocess(r"trim(LEADING 'Foo' from leaDing)")?,
            lex(r#"trim(LEADING : 'Foo', "from" : leaDing)"#)?
        );

        // Trim Specification (quoted) in 2nd and 3rd spot
        assert_eq!(
            preprocess(r"trim('LEADING' from leaDing)")?,
            lex(r#"trim('LEADING', "from" : leaDing)"#)?
        );

        // Trim Specification in 3rd spot only
        assert_eq!(
            preprocess(r"trim('a' from leaDing)")?,
            lex(r#"trim('a', "from" : leaDing)"#)?
        );

        assert_eq!(
            preprocess(r"trim(leading from '   Bar')")?,
            lex(r#"trim(leading : ' ',  "from" : '   Bar')"#)?
        );
        assert_eq!(
            preprocess(r"trim(TrAiLiNg 'Bar' from 'FooBar')")?,
            lex(r#"trim(TrAiLiNg : 'Bar',  "from" : 'FooBar')"#)?
        );
        assert_eq!(
            preprocess(r"trim(TRAILING from 'Bar   ')")?,
            lex(r#"trim(TRAILING: ' ', "from": 'Bar   ')"#)?
        );
        assert_eq!(
            preprocess(r"trim(BOTH 'Foo' from 'FooBarBar')")?,
            lex(r#"trim(BOTH: 'Foo', "from": 'FooBarBar')"#)?
        );
        assert_eq!(
            preprocess(r"trim(botH from '   Bar   ')")?,
            lex(r#"trim(botH: ' ', "from": '   Bar   ')"#)?
        );
        assert_eq!(
            preprocess(r"trim(from '   Bar   ')")?,
            lex(r#"trim("from": '   Bar   ')"#)?
        );

        assert_eq!(
            preprocess(r"position('o' in 'foo')")?,
            lex(r#"position('o', "in" : 'foo')"#)?
        );

        assert_eq!(
            preprocess(r"substring('FooBar' from 2 for 3)")?,
            lex(r#"substring('FooBar', "from": 2, "for": 3)"#)?
        );
        assert_eq!(
            preprocess(r"substring('FooBar' from 2)")?,
            lex(r#"substring('FooBar', "from": 2)"#)?
        );
        assert_eq!(
            preprocess(r"substring('FooBar' for 3)")?,
            lex(r#"substring('FooBar', "for": 3)"#)?
        );
        assert_eq!(
            preprocess(r"substring('FooBar',1,3)")?,
            lex(r"substring('FooBar', 1,3)")?
        );
        assert_eq!(
            preprocess(r"substring('FooBar',3)")?,
            lex(r"substring('FooBar', 3)")?
        );

        assert_eq!(preprocess(r"CAST(9 AS b)")?, lex(r#"CAST(9, "AS": b)"#)?);
        assert_eq!(
            preprocess(r"CAST(a AS VARCHAR)")?,
            lex(r#"CAST(a, "AS": VARCHAR)"#)?
        );
        assert_eq!(
            preprocess(r"CAST(a AS VARCHAR(20))")?,
            lex(r#"CAST(a, "AS": VARCHAR(20))"#)?
        );
        assert_eq!(
            preprocess(r"CAST(TRUE AS INTEGER)")?,
            lex(r#"CAST(TRUE, "AS": INTEGER)"#)?
        );
        assert_eq!(
            preprocess(r"CAST( (4 in (1,2,3,4))  AS INTEGER)")?,
            lex(r#"CAST( (4 in (1,2,3,4)) , "AS": INTEGER)"#)?
        );
        assert_eq!(
            preprocess(r"cast([1, 2] as INT)")?,
            lex(r#"cast([1, 2] , "as": INT)"#)?
        );
        assert_eq!(
            preprocess(r"cast(<<1, 2>> as INT)")?,
            lex(r#"cast(<<1, 2>> , "as": INT)"#)?
        );
        assert_eq!(
            preprocess(r"cast({a:1} as INT)")?,
            lex(r#"cast({a:1} , "as": INT)"#)?
        );

        assert_eq!(
            preprocess(r"extract(timezone_minute from a)")?,
            lex(r#"extract(timezone_minute:True, "from" : a)"#)?
        );
        assert_eq!(
            preprocess(r"extract(timezone_hour from a)")?,
            lex(r#"extract(timezone_hour:True, "from" : a)"#)?
        );
        assert_eq!(
            preprocess(r"extract(year from a)")?,
            lex(r#"extract(year:True, "from" : a)"#)?
        );
        assert_eq!(
            preprocess(r"extract(month from a)")?,
            lex(r#"extract(month:True, "from" : a)"#)?
        );

        assert_eq!(
            preprocess(r"extract(day from a)")?,
            lex(r#"extract(day:True, "from" : a)"#)?
        );

        assert_eq!(
            preprocess(r"extract(day from day)")?,
            lex(r#"extract(day:True, "from" : day)"#)?
        );
        assert_eq!(
            preprocess(r"extract(hour from a)")?,
            lex(r#"extract(hour:True, "from" : a)"#)?
        );
        assert_eq!(
            preprocess(r"extract(minute from a)")?,
            lex(r#"extract(minute:True, "from" : a)"#)?
        );
        assert_eq!(
            preprocess(r"extract(second from a)")?,
            lex(r#"extract(second:True, "from" : a)"#)?
        );
        assert_eq!(
            preprocess(r"extract(hour from TIME WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(r#"extract(hour:True, "from" : TIME WITH TIME ZONE '01:23:45.678-06:30')"#)?
        );
        assert_eq!(
            preprocess(r"extract(minute from TIME WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(r#"extract(minute:True, "from" : TIME WITH TIME ZONE '01:23:45.678-06:30')"#)?
        );
        assert_eq!(
            preprocess(r"extract(second from TIME WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(r#"extract(second:True, "from" : TIME WITH TIME ZONE '01:23:45.678-06:30')"#)?
        );
        assert_eq!(
            preprocess(r"extract(timezone_hour from TIME WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(
                r#"extract(timezone_hour:True, "from" : TIME WITH TIME ZONE '01:23:45.678-06:30')"#
            )?
        );
        assert_eq!(
            preprocess(r"extract(timezone_minute from TIME WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(
                r#"extract(timezone_minute:True, "from" : TIME WITH TIME ZONE '01:23:45.678-06:30')"#
            )?
        );
        assert_eq!(
            preprocess(r"extract(hour from TIME (2) WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(r#"extract(hour:True, "from" : TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"#)?
        );
        assert_eq!(
            preprocess(r"extract(minute from TIME (2) WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(r#"extract(minute:True, "from" : TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"#)?
        );
        assert_eq!(
            preprocess(r"extract(second from TIME (2) WITH TIME ZONE '01:23:45.678-06:30')")?,
            lex(r#"extract(second:True, "from" : TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"#)?
        );
        assert_eq!(
            preprocess(
                r"extract(timezone_hour from TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"
            )?,
            lex(
                r#"extract(timezone_hour:True, "from" : TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"#
            )?
        );
        assert_eq!(
            preprocess(
                r"extract(timezone_minute from TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"
            )?,
            lex(
                r#"extract(timezone_minute:True, "from" : TIME (2) WITH TIME ZONE '01:23:45.678-06:30')"#
            )?
        );

        assert_eq!(preprocess(r"count(a)")?, lex(r"count(a)")?);
        assert_eq!(
            preprocess(r"count(DISTINCT a)")?,
            lex(r#"count("DISTINCT": a)"#)?
        );
        assert_eq!(preprocess(r"count(all a)")?, lex(r#"count("all": a)"#)?);
        let q_count_1 = r"count(1)";
        assert_eq!(preprocess(q_count_1)?, lex(q_count_1)?);
        let q_count_star = r"count(*)";
        assert_eq!(preprocess(q_count_star)?, lex(q_count_star)?);

        assert_eq!(preprocess(r"sum(a)")?, lex(r"sum(a)")?);
        assert_eq!(
            preprocess(r"sum(DISTINCT a)")?,
            lex(r#"sum("DISTINCT": a)"#)?
        );
        assert_eq!(preprocess(r"sum(all a)")?, lex(r#"sum("all": a)"#)?);
        let q_sum_1 = r"sum(1)";
        assert_eq!(preprocess(q_sum_1)?, lex(q_sum_1)?);
        let q_sum_star = r"sum(*)";
        assert_eq!(preprocess(q_sum_star)?, lex(q_sum_star)?);

        assert_eq!(
            preprocess(r"COUNT(DISTINCT [1,1,1,1,2])")?,
            lex(r#"COUNT("DISTINCT" : [1,1,1,1,2])"#)?
        );

        let empty_q = "";
        assert_eq!(preprocess(empty_q)?, lex(empty_q)?);

        let union_q = "SELECT a FROM b UNION (SELECT x FROM y ORDER BY a LIMIT 10 OFFSET 5)";
        assert_eq!(preprocess(union_q)?, lex(union_q)?);

        Ok(())
    }
}
