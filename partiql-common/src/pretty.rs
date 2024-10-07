use pretty::{Arena, DocAllocator, DocBuilder, Pretty};
use std::io;
use std::io::Write;
use std::string::FromUtf8Error;
use thiserror::Error;

pub const PRETTY_INDENT_MINOR_NEST: isize = 2;
pub const PRETTY_INDENT_SUBORDINATE_CLAUSE_NEST: isize = 6;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ToPrettyError {
    #[error("IO error: `{0}`")]
    IoError(#[from] std::io::Error),

    #[error("FromUtf8Error: `{0}`")]
    FromUtf8Error(#[from] FromUtf8Error),
}

pub type ToPrettyResult<T> = Result<T, ToPrettyError>;

pub trait ToPretty {
    /// Pretty-prints to a `String`.
    fn to_pretty_string(&self, width: usize) -> ToPrettyResult<String> {
        let mut out = Vec::new();
        self.to_pretty(width, &mut out)?;
        Ok(String::from_utf8(out)?)
    }

    /// Pretty-prints to a `std::io::Write` object.
    fn to_pretty<W>(&self, width: usize, out: &mut W) -> ToPrettyResult<()>
    where
        W: ?Sized + io::Write;
}

impl<T> ToPretty for T
where
    T: PrettyDoc,
{
    fn to_pretty<W>(&self, width: usize, out: &mut W) -> ToPrettyResult<()>
    where
        W: ?Sized + Write,
    {
        let arena = Arena::new();
        let DocBuilder(_, doc) = self.pretty_doc::<_, ()>(&arena);
        Ok(doc.render(width, out)?)
    }
}

pub trait PrettyDoc {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone;
}

impl<T> PrettyDoc for &T
where
    T: PrettyDoc,
{
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        (*self).pretty_doc(arena)
    }
}

impl<T> PrettyDoc for Box<T>
where
    T: PrettyDoc,
{
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.as_ref().pretty_doc(arena)
    }
}

impl PrettyDoc for str {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        arena.concat(["'", self, "'"])
    }
}

impl PrettyDoc for String {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        arena.concat(["'", self, "'"])
    }
}

impl PrettyDoc for Vec<u8> {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let y = String::from_utf8_lossy(self.as_slice());
        arena.text(y)
    }
}

impl PrettyDoc for rust_decimal::Decimal {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        arena.text(self.to_string())
    }
}

#[inline]
pub fn pretty_prefixed_doc<'b, E, D, A>(
    annot: &'static str,
    doc: E,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    E: Pretty<'b, D, A>,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    arena.text(annot).append(arena.space()).append(doc).group()
}

#[inline]
pub fn pretty_surrounded<'b, P, D, A>(
    inner: &'b P,
    start: &'static str,
    end: &'static str,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    P: PrettyDoc + 'b,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_surrounded_doc(inner.pretty_doc(arena), start, end, arena)
}

#[inline]
pub fn pretty_surrounded_doc<'b, E, D, A>(
    doc: E,
    start: &'static str,
    end: &'static str,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    E: Pretty<'b, D, A>,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    arena
        .text(start)
        .append(doc)
        .append(arena.text(end))
        .group()
}

#[inline]
pub fn pretty_parenthesized_doc<'b, E, D, A>(doc: E, arena: &'b D) -> DocBuilder<'b, D, A>
where
    E: Pretty<'b, D, A>,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_surrounded_doc(doc, "(", ")", arena)
}

#[inline]
pub fn pretty_seq_doc<'i, 'b, I, E, D, A>(
    seq: I,
    start: &'static str,
    qualifier: Option<E>,
    end: &'static str,
    sep: &'static str,
    nest: isize,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    E: Pretty<'b, D, A>,
    I: IntoIterator<Item = E>,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    let start = arena.text(start);
    let end = arena.text(end);
    let sep = arena.text(sep).append(arena.line());
    let start = if let Some(qual) = qualifier {
        start.append(arena.space()).append(qual)
    } else {
        start
    };
    let body = arena
        .line()
        .append(arena.intersperse(seq, sep))
        .append(arena.line())
        .group();
    start.append(body.nest(nest)).append(end).group()
}

#[inline]
pub fn pretty_seq<'i, 'b, I, P, D, A>(
    list: I,
    start: &'static str,
    end: &'static str,
    sep: &'static str,
    nest: isize,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    I: IntoIterator<Item = &'b P>,
    P: PrettyDoc + 'b,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    let seq = list.into_iter().map(|l| l.pretty_doc(arena));
    pretty_seq_doc(seq, start, None, end, sep, nest, arena)
}

#[inline]
pub fn pretty_list<'b, I, P, D, A>(list: I, nest: isize, arena: &'b D) -> DocBuilder<'b, D, A>
where
    I: IntoIterator<Item = &'b P>,
    P: PrettyDoc + 'b,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    let sep = arena.text(",").append(arena.softline());
    pretty_seperated(sep, list, nest, arena)
}

#[inline]
pub fn pretty_seperated<'b, I, E, P, D, A>(
    sep: E,
    list: I,
    nest: isize,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    I: IntoIterator<Item = &'b P>,
    E: Pretty<'b, D, A>,
    P: PrettyDoc + 'b,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    let list = list.into_iter().map(|l| l.pretty_doc(arena));
    pretty_seperated_doc(sep, list, nest, arena)
}

#[inline]
pub fn pretty_seperated_doc<'b, I, E, D, A>(
    sep: E,
    list: I,
    nest: isize,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    I: IntoIterator<Item = DocBuilder<'b, D, A>>,
    E: Pretty<'b, D, A>,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    let sep = sep.pretty(arena);
    arena.intersperse(list, sep).nest(nest).group()
}
