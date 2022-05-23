use crate::lexer::{PartiqlLexer, Spanned, Token};
use crate::LexError;
use partiql_source_map::location::ByteOffset;
use std::collections::VecDeque;

/// A [`Token`] and its associated `&str` slice; buffered from the lexer for parsing/matching.
pub(crate) type BufferedToken<'input> = (Spanned<Token<'input>, ByteOffset>, &'input str);

/// A minimal pratt-style parser base over [`Token`]s
pub(crate) struct TokenParser<'input, 'tracker> {
    lexer: PartiqlLexer<'input, 'tracker>,
    buffered: VecDeque<BufferedToken<'input>>,
    consumed_c: usize,
}

const TOKEN_BUFFER_INIT_CAPACITY: usize = 20;

impl<'input, 'tracker> TokenParser<'input, 'tracker> {
    pub fn new(lexer: PartiqlLexer<'input, 'tracker>) -> Self {
        Self {
            lexer,
            buffered: VecDeque::with_capacity(TOKEN_BUFFER_INIT_CAPACITY),
            consumed_c: 0,
        }
    }

    /// Consume the least-recently buffered [`Token`], buffering one first if necessary.
    #[inline]
    pub fn consume(&mut self) -> Option<Result<(), Spanned<LexError<'input>, ByteOffset>>> {
        match self.buffer(1) {
            Some(Ok(_)) => {
                self.consumed_c += 1;
                Some(Ok(()))
            }
            other => other,
        }
    }

    #[inline]
    /// Unconsume the most-recently consumed [`Token`]. Never buffers new tokens.
    pub fn unconsume(&mut self) {
        self.consumed_c = self.consumed_c.saturating_sub(1);
    }

    /// Unbuffer and return the least-recently consumed [`Token`]. Never buffers new tokens.
    #[inline]
    pub fn flush_1(&mut self) -> Option<BufferedToken<'input>> {
        self.consumed_c = self.consumed_c.saturating_sub(1);
        self.buffered.pop_front()
    }

    /// Rebuffer a previously [`flush_1`]ed [`Token`].
    #[inline]
    pub fn unflush_1(&mut self, tok: Option<BufferedToken<'input>>) {
        if let Some(t) = tok {
            self.buffered.push_front(t);
            self.consumed_c += 1;
        }
    }

    /// Unbuffer and return the all consumed [`Token`]s. Never buffers new tokens.
    #[inline]
    pub fn flush(&mut self) -> Vec<BufferedToken<'input>> {
        let len = std::mem::replace(&mut self.consumed_c, 0);
        self.buffered.drain(0..len).collect()
    }

    /// Rebuffer previously [`flush`]ed [`Token`]s.
    #[inline]
    pub fn unflush(&mut self, toks: Vec<BufferedToken<'input>>) {
        self.consumed_c += toks.len();
        for tok in toks.into_iter().rev() {
            self.buffered.push_front(tok);
        }
    }

    /// Matches `target` against the least-recently buffered [`Token`], buffering one first if necessary.
    ///
    /// If there are no tokens to buffer or the match fails, returns [`Err`].
    #[inline]
    pub fn expect(&mut self, target: &Token) -> Result<(), Spanned<LexError<'input>, ByteOffset>> {
        match self.peek_n(0) {
            Some(((_, tok, _), _)) if target == tok => {
                self.consumed_c += 1;
                Ok(())
            }
            _ => Err((0.into(), LexError::Unknown, 0.into())),
        }
    }

    /// Returns a reference to the `i`th least-recently buffered [`Token`], buffering up to `i` [`Token`]s
    /// first if necessary.
    #[inline]
    pub fn peek_n(&mut self, i: usize) -> Option<&BufferedToken<'input>> {
        self.buffer(i + 1);
        self.get(i)
    }

    /// Returns a reference to the `i`th least-recently buffered [`Token`] or None if `i` [`Token`]s
    /// haven't been buffered.
    #[inline]
    fn get(&mut self, i: usize) -> Option<&BufferedToken<'input>> {
        self.buffered.get(self.consumed_c + i)
    }

    /// Buffer [`Token`]s until `upto` have are buffered; If at least `upto` are already buffered,
    /// this is a no-op.
    #[inline]
    fn buffer(&mut self, upto: usize) -> Option<Result<(), Spanned<LexError<'input>, ByteOffset>>> {
        while upto > self.buffered.len() - self.consumed_c {
            if let Some(tok) = self.lexer.next_internal() {
                match tok {
                    Ok(tok) => self.buffered.push_back((tok, self.lexer.slice())),
                    Err(e) => return Some(Err(e)),
                }
            } else {
                return None;
            }
        }
        Some(Ok(()))
    }
}
