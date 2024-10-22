use dyn_clone::DynClone;
use dyn_hash::DynHash;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone)]
pub struct LazyEmbeddedDocument {
    bytes: Arc<Vec<u8>>,
    typ: DynEmbeddedTypeTag,
}

impl LazyEmbeddedDocument {
    pub fn new<B: Into<Vec<u8>>, T: Into<DynEmbeddedTypeTag>>(bytes: B, typ: T) -> Self {
        let bytes = Arc::new(bytes.into());
        let typ = typ.into();
        Self { bytes, typ }
    }
}

// dyn

pub type DynEmbeddedTypeTag = Box<dyn DynEmbeddedDocumentType>;

pub trait DynEmbeddedDocumentType: DynClone {
    fn construct(&self, bytes: &[u8]) -> Box<dyn EmbeddedDocument>;
}

dyn_clone::clone_trait_object!(DynEmbeddedDocumentType);

pub trait DynEmbeddedDocumentTypeFactory {
    fn to_dyn_type_tag(self) -> DynEmbeddedTypeTag;
}

// typed

pub type EmbeddedTypeTag<D> = Box<dyn EmbeddedDocumentType<Doc = D>>;
pub trait EmbeddedDocumentType: Clone {
    type Doc: EmbeddedDocument + 'static;

    fn construct(&self, bytes: &[u8]) -> Self::Doc;
}

#[cfg_attr(feature = "serde", typetag::serde)]
pub trait EmbeddedDocument: Debug + DynHash + DynClone {}

dyn_hash::hash_trait_object!(EmbeddedDocument);
dyn_clone::clone_trait_object!(EmbeddedDocument);

impl<T, D> DynEmbeddedDocumentType for T
where
    T: EmbeddedDocumentType<Doc = D>,
    D: EmbeddedDocument + 'static,
{
    fn construct(&self, bytes: &[u8]) -> Box<dyn EmbeddedDocument> {
        Box::new(EmbeddedDocumentType::construct(self, bytes))
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
