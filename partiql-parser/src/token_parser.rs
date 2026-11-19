use crate::error::LexError;
use crate::lexer::{PartiqlLexer, Spanned, Token};
use partiql_common::syntax::location::ByteOffset;
use std::collections::VecDeque;

/// A [`Token`] and its associated `&str` slice; buffered from the lexer for parsing/matching.
pub(crate) type BufferedToken<'input> = (Spanned<Token<'input>, ByteOffset>, &'input str);

/// A minimal pratt-style parser base over [`Token`]s
pub(crate) struct TokenParser<'input, 'tracker> {
    lexer: PartiqlLexer<'input, 'tracker>,
    buffered: VecDeque<BufferedToken<'input>>,
    consumed_c: usize,
    /// A lexing error met while peeking ahead, kept until a [`consume`](Self::consume) reaches
    /// it, so that a look-ahead cannot drop it.
    error: Option<Spanned<LexError<'input>, ByteOffset>>,
}

const TOKEN_BUFFER_INIT_CAPACITY: usize = 20;

impl<'input, 'tracker> TokenParser<'input, 'tracker> {
    pub fn new(lexer: PartiqlLexer<'input, 'tracker>) -> Self {
        Self {
            lexer,
            buffered: VecDeque::with_capacity(TOKEN_BUFFER_INIT_CAPACITY),
            consumed_c: 0,
            error: None,
        }
    }

    /// Consume the least-recently buffered [`Token`], buffering one first if necessary.
    #[inline]
    pub fn consume(&mut self) -> Option<Result<(), Spanned<LexError<'input>, ByteOffset>>> {
        match self.buffer(1, true) {
            Some(Ok(())) => {
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
    pub fn expect(
        &mut self,
        target: &Token<'_>,
    ) -> Result<(), Spanned<LexError<'input>, ByteOffset>> {
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
        let _ = self.buffer(i + 1, false);
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
    ///
    /// A lexing error stops the buffering and is returned; it is also kept and returned again
    /// by later calls that need more tokens, until a call with `take_error` returns it.
    #[inline]
    fn buffer(
        &mut self,
        upto: usize,
        take_error: bool,
    ) -> Option<Result<(), Spanned<LexError<'input>, ByteOffset>>> {
        while upto > self.buffered.len() - self.consumed_c {
            if self.error.is_some() {
                return if take_error {
                    self.error.take().map(Err)
                } else {
                    self.error.clone().map(Err)
                };
            }
            match self.lexer.next_internal()? {
                Ok(tok) => self.buffered.push_back((tok, self.lexer.slice())),
                Err(e) => {
                    if !take_error {
                        self.error = Some(e.clone());
                    }
                    return Some(Err(e));
                }
            }
        }
        Some(Ok(()))
    }
}
