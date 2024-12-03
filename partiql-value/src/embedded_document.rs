use dyn_clone::DynClone;
use dyn_hash::DynHash;
use partiql_common::pretty::PrettyDoc;
use std::error::Error;

use crate::datum::{Datum, DatumCategoryOwned, DatumCategoryRef};
use crate::Value;
use pretty::{DocAllocator, DocBuilder};
use std::fmt::{Debug, Display};

pub type EmbeddedDocError = Box<dyn Error>;

pub type EmbeddedDocResult<T> = Result<T, EmbeddedDocError>;
pub type EmbeddedDocValueIntoIterator = Box<dyn Iterator<Item = DynEmbeddedDocument>>;

pub type EmbeddedDocValueIter<'a> =
    Box<dyn 'a + Iterator<Item = EmbeddedDocResult<&'a DynEmbeddedDocument>>>;

// dyn

pub type DynEmbeddedTypeTag = Box<dyn DynEmbeddedDocumentType>;

pub trait DynEmbeddedDocumentType: Debug + DynClone {
    fn construct(&self, bytes: Vec<u8>) -> EmbeddedDocResult<Box<dyn EmbeddedDocument>>;
    fn name(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(DynEmbeddedDocumentType);

pub trait DynEmbeddedDocumentTypeFactory {
    fn to_dyn_type_tag(self) -> DynEmbeddedTypeTag;
}

// typed

pub type EmbeddedTypeTag<D> = Box<dyn EmbeddedDocumentType<Doc = D>>;
pub trait EmbeddedDocumentType: Debug + Clone {
    type Doc: EmbeddedDocument + 'static;

    fn construct(&self, bytes: Vec<u8>) -> EmbeddedDocResult<Self::Doc>;
    fn name(&self) -> &'static str;
}

pub type DynEmbeddedDocument = Box<dyn EmbeddedDocument>;
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait EmbeddedDocument: Display + Debug + DynHash + DynClone + Datum<Value>
/*+ for<'a> DatumCattt<'a>*/
{
    fn into_dyn_iter(self: Box<Self>) -> EmbeddedDocResult<EmbeddedDocValueIntoIterator>;

    fn category(&self) -> DatumCategoryRef<'_>;

    fn into_category(self: Box<Self>) -> DatumCategoryOwned;
}

dyn_hash::hash_trait_object!(EmbeddedDocument);
dyn_clone::clone_trait_object!(EmbeddedDocument);

impl PrettyDoc for DynEmbeddedDocument {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        todo!("impl PrettyDoc for EmbeddedDocument")
    }
}

impl<T, D> DynEmbeddedDocumentType for T
where
    T: EmbeddedDocumentType<Doc = D>,
    D: EmbeddedDocument + 'static,
{
    fn construct(&self, bytes: Vec<u8>) -> EmbeddedDocResult<Box<dyn EmbeddedDocument>> {
        EmbeddedDocumentType::construct(self, bytes)
            .map(|d| Box::new(d) as Box<dyn EmbeddedDocument>)
    }

    fn name(&self) -> &'static str {
        EmbeddedDocumentType::name(self)
    }
}

impl<T, D> DynEmbeddedDocumentTypeFactory for T
where
    T: EmbeddedDocumentType<Doc = D> + 'static,
    D: EmbeddedDocument + 'static,
{
    fn to_dyn_type_tag(self) -> DynEmbeddedTypeTag {
        Box::new(self)
    }
}
