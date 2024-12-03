use crate::datum::{Datum, DatumCategory, DatumCategoryOwned, DatumCategoryRef};
use crate::embedded_document::{
    DynEmbeddedDocument, DynEmbeddedTypeTag, EmbeddedDocError, EmbeddedDocResult,
    EmbeddedDocValueIntoIterator, EmbeddedDocValueIter, EmbeddedDocument,
};
use crate::Value;
use delegate::delegate;
use partiql_common::pretty::{pretty_surrounded_doc, PrettyDoc};
use pretty::{DocAllocator, DocBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use thiserror::Error;

macro_rules! ice_must_lower {
    () => {
        todo!("ICE Error; must be lowered first")
    };
}

#[derive(Clone, Debug)]
pub enum EmbeddedDoc {
    Raw(RawEmbeddedDoc),
    Boxed(DynEmbeddedDocument),
}

impl EmbeddedDoc {
    pub fn new<B: Into<Vec<u8>>>(
        contents: B,
        type_tag: DynEmbeddedTypeTag,
    ) -> EmbeddedDocResult<Self> {
        Ok(EmbeddedDoc::Raw(RawEmbeddedDoc::new(contents, type_tag)?))
    }

    pub fn force_into(self) -> EmbeddedResult<DynEmbeddedDocument> {
        match self {
            EmbeddedDoc::Raw(raw) => raw.parse(),
            EmbeddedDoc::Boxed(doc) => Ok(doc),
        }
    }

    pub fn force(&self) -> EmbeddedResult<&DynEmbeddedDocument> {
        match self {
            EmbeddedDoc::Raw(_) => {
                ice_must_lower!();
            }
            EmbeddedDoc::Boxed(doc) => Ok(doc),
        }
    }
}

impl<'a> DatumCategory<'a> for EmbeddedDoc {
    fn category(&'a self) -> DatumCategoryRef<'a> {
        match self {
            EmbeddedDoc::Raw(_) => {
                ice_must_lower!()
            }
            EmbeddedDoc::Boxed(doc) => doc.category(),
        }
    }

    fn into_category(self) -> DatumCategoryOwned {
        match self {
            EmbeddedDoc::Raw(_) => {
                ice_must_lower!()
            }
            EmbeddedDoc::Boxed(doc) => doc.into_category(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawEmbeddedDoc {
    contents: Vec<u8>,
    type_tag: DynEmbeddedTypeTag,
}

impl RawEmbeddedDoc {
    pub fn new<B: Into<Vec<u8>>>(
        contents: B,
        type_tag: DynEmbeddedTypeTag,
    ) -> EmbeddedDocResult<Self> {
        Ok(RawEmbeddedDoc {
            contents: contents.into(),
            type_tag,
        })
    }
}

#[derive(Error, Debug)]
pub enum EmbeddedError {
    #[error("Latent Type Error for Boxed Document {0}")]
    LatentTypeError(EmbeddedDocError, DynEmbeddedTypeTag),
}

pub type EmbeddedResult<T> = Result<T, EmbeddedError>;

impl RawEmbeddedDoc {
    fn parse(self) -> EmbeddedResult<DynEmbeddedDocument> {
        let Self { contents, type_tag } = self;
        match type_tag.construct(contents) {
            Ok(doc) => Ok(doc),
            Err(err) => Err(EmbeddedError::LatentTypeError(err, type_tag.clone())),
        }
    }
}

pub struct EmbeddedDocIter<'a>(EmbeddedDocValueIter<'a>);
impl Debug for EmbeddedDocIter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Clone for EmbeddedDocIter<'_> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl IntoIterator for EmbeddedDoc {
    type Item = EmbeddedDoc;
    type IntoIter = EmbeddedDocIntoIterator;

    fn into_iter(self) -> EmbeddedDocIntoIterator {
        let iter = self
            .force_into()
            .expect("TODO [EMBDOC]")
            .into_dyn_iter()
            .expect("TODO [EMBDOC]");
        EmbeddedDocIntoIterator(iter)
    }
}

pub struct EmbeddedDocIntoIterator(EmbeddedDocValueIntoIterator);

impl Iterator for EmbeddedDocIntoIterator {
    type Item = EmbeddedDoc;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(EmbeddedDoc::Boxed)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl Datum<Value> for EmbeddedDoc {
    delegate! {
        to self.force().expect("handle errors for datum") {
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

impl Datum<Value> for Rc<dyn Error> {
    fn is_null(&self) -> bool {
        todo!()
    }

    fn is_missing(&self) -> bool {
        todo!()
    }

    fn is_absent(&self) -> bool {
        todo!()
    }

    fn is_present(&self) -> bool {
        todo!()
    }

    fn is_sequence(&self) -> bool {
        todo!()
    }

    fn is_ordered(&self) -> bool {
        todo!()
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
        let doc = match self {
            EmbeddedDoc::Raw(RawEmbeddedDoc { contents, type_tag }) => {
                String::from_utf8_lossy(contents).into_owned()
            }
            EmbeddedDoc::Boxed(doc) => format!("{}", doc),
        };

        pretty_surrounded_doc(doc, "`", "`", arena)
            .append(arena.text("::"))
            .append(arena.text("ion"))
    }
}
