use crate::datum::Datum;
use crate::embedded_document::{
    DynEmbeddedDocument, EmbeddedDocResult, EmbeddedDocValueIntoIterator, EmbeddedDocValueIter,
    EmbeddedDocument,
};
use crate::{embedded_doc, Value, ValueIntoIterator, ValueIter};
use delegate::delegate;
use partiql_common::pretty::{pretty_surrounded, pretty_surrounded_doc, PrettyDoc};
use pretty::{DocAllocator, DocBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::{slice, vec};

#[derive(Clone, Debug)]
pub struct EmbeddedDoc(DynEmbeddedDocument);
/*
impl<'a> IntoIterator for &'a EmbeddedDoc {
    type Item = EmbeddedDocResult<&'a DynEmbeddedDocument>;
    type IntoIter = EmbeddedDocIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        EmbeddedDocIter(self.0.into_dyn_iter())
    }
}
*/

pub struct EmbeddedDocIter<'a>(EmbeddedDocValueIter<'a>);
impl<'a> Debug for EmbeddedDocIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> Clone for EmbeddedDocIter<'a> {
    fn clone(&self) -> Self {
        todo!()
    }
}

/*
impl<'a> Iterator for EmbeddedDocIter<'a> {
    type Item = EmbeddedDocResult<&'a DynEmbeddedDocument>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|res| res.map(|embed| unsafe { std::mem::transmute(embed) }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
 */

impl IntoIterator for EmbeddedDoc {
    type Item = EmbeddedDoc;
    type IntoIter = EmbeddedDocIntoIterator;

    fn into_iter(self) -> EmbeddedDocIntoIterator {
        // TODO [EMBDOC] handle error case
        EmbeddedDocIntoIterator(self.0.into_dyn_iter().expect("ion"))
    }
}

pub struct EmbeddedDocIntoIterator(EmbeddedDocValueIntoIterator);

impl Iterator for EmbeddedDocIntoIterator {
    type Item = EmbeddedDoc;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(EmbeddedDoc::new)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl EmbeddedDoc {
    pub fn new(doc: DynEmbeddedDocument) -> Self {
        EmbeddedDoc(doc)
    }
}

impl Datum for EmbeddedDoc {
    delegate! {
        to self.0 {
            fn is_null(&self) -> bool;
            fn is_missing(&self) -> bool;
            fn is_absent(&self) -> bool;
            fn is_present(&self) -> bool;
            fn is_sequence(&self) -> bool;
            fn is_ordered(&self) -> bool;
            //fn into_iter(self) -> ValueIntoIterator;
        }
    }
}

impl Hash for EmbeddedDoc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}

impl PartialEq<Self> for EmbeddedDoc {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for EmbeddedDoc {}

#[cfg(feature = "serde")]
impl Serialize for EmbeddedDoc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EmbeddedDoc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl PrettyDoc for EmbeddedDoc {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        // TODO [EMBDOC] write out type tag?
        // TODO [EMBDOC] handle backticks more generally.
        //let doc = self.bytes.pretty_doc(arena);
        let doc = arena.text("foo:\n\t-bar\n\t-baz");
        pretty_surrounded_doc(doc, "`````", "`````", arena)
    }
}
