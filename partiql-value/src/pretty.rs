use crate::{Bag, DateTime, List, Tuple, Value};
use partiql_common::pretty::{
    pretty_prefixed_doc, pretty_seq, pretty_seq_doc, pretty_surrounded, PrettyDoc,
    PRETTY_INDENT_MINOR_NEST,
};
use pretty::{DocAllocator, DocBuilder};

impl PrettyDoc for Value {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Value::Null => arena.text("NULL"),
            Value::Missing => arena.text("MISSING"),
            Value::Boolean(inner) => arena.text(inner.to_string()),
            Value::Integer(inner) => arena.text(inner.to_string()),
            Value::Real(inner) => arena.text(inner.0.to_string()),
            Value::Decimal(inner) => inner.pretty_doc(arena),
            Value::String(inner) => inner.pretty_doc(arena),
            Value::Blob(inner) => pretty_string(inner, arena),
            Value::DateTime(inner) => inner.pretty_doc(arena),
            Value::List(inner) => inner.pretty_doc(arena),
            Value::Bag(inner) => inner.pretty_doc(arena),
            Value::Tuple(inner) => inner.pretty_doc(arena),
        }
    }
}

impl PrettyDoc for DateTime {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            DateTime::Date(d) => pretty_prefixed_doc("DATE", format!("'{d:?}'"), arena),

            DateTime::Time(t) => pretty_prefixed_doc("TIME", format!("'{t:?}'"), arena),
            DateTime::TimeWithTz(t, tz) => {
                pretty_prefixed_doc("TIME WITH TIME ZONE", format!("'{t:?} {tz:?}'"), arena)
            }
            DateTime::Timestamp(dt) => pretty_prefixed_doc("TIMESTAMP", format!("'{dt:?}'"), arena),
            DateTime::TimestampWithTz(dt) => {
                pretty_prefixed_doc("TIMESTAMP WITH TIME ZONE", format!("'{dt:?}'"), arena)
            }
        }
    }
}

impl PrettyDoc for List {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seq(self.iter(), "[", "]", ",", PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for Bag {
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seq(
            self.iter(),
            "<<",
            ">>",
            ",",
            PRETTY_INDENT_MINOR_NEST,
            arena,
        )
    }
}

impl PrettyDoc for Tuple {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let seq = self.pairs().map(|(k, v)| {
            let k = k.pretty_doc(arena);
            let v = v.pretty_doc(arena);
            let sep = arena.text(": ");

            k.append(sep).group().append(v).group()
        });
        pretty_seq_doc(seq, "{", None, "}", ",", PRETTY_INDENT_MINOR_NEST, arena)
    }
}

fn pretty_string<'b, P, D, A>(contents: &'b P, arena: &'b D) -> DocBuilder<'b, D, A>
where
    P: PrettyDoc + 'b,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_surrounded(contents, "'", "'", arena)
}
