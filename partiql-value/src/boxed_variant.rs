use dyn_clone::DynClone;
use dyn_hash::DynHash;
use partiql_common::pretty::PrettyDoc;
use std::error::Error;

use crate::datum::{Datum, DatumCategoryOwned, DatumCategoryRef, DatumLower};
use crate::Value;
use pretty::{DocAllocator, DocBuilder};
use std::fmt::{Debug, Display};

pub type BoxedVariantError = Box<dyn Error>;

pub type BoxedVariantResult<T> = Result<T, BoxedVariantError>;
pub type BoxedVariantValueIntoIterator = Box<dyn Iterator<Item = DynBoxedVariant>>;

pub type BoxedVariantValueIter<'a> =
    Box<dyn 'a + Iterator<Item = BoxedVariantResult<&'a DynBoxedVariant>>>;

// dyn

pub type DynBoxedVariantTypeTag = Box<dyn DynBoxedVariantType>;

pub trait DynBoxedVariantType: Debug + DynClone {
    fn construct(&self, bytes: Vec<u8>) -> BoxedVariantResult<Box<dyn BoxedVariant>>;
    fn name(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(DynBoxedVariantType);

pub trait DynBoxedVariantTypeFactory {
    fn to_dyn_type_tag(self) -> DynBoxedVariantTypeTag;
}

// typed

pub type BoxedVariantTypeTag<D> = Box<dyn BoxedVariantType<Doc = D>>;
pub trait BoxedVariantType: Debug + Clone {
    type Doc: BoxedVariant + 'static;

    fn construct(&self, bytes: Vec<u8>) -> BoxedVariantResult<Self::Doc>;
    fn name(&self) -> &'static str;
}

pub type DynBoxedVariant = Box<dyn BoxedVariant>;
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait BoxedVariant:
    Display + Debug + DynHash + DynClone + Datum<Value> + DatumLower<Value>
{
    fn into_dyn_iter(self: Box<Self>) -> BoxedVariantResult<BoxedVariantValueIntoIterator>;

    fn category(&self) -> DatumCategoryRef<'_>;

    fn into_category(self: Box<Self>) -> DatumCategoryOwned;
}

dyn_hash::hash_trait_object!(BoxedVariant);
dyn_clone::clone_trait_object!(BoxedVariant);

impl PrettyDoc for DynBoxedVariant {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        // todo!("impl PrettyDoc for BoxedVariant")
        arena.text(format!("{}", self))
    }
}

impl<T, D> DynBoxedVariantType for T
where
    T: BoxedVariantType<Doc = D>,
    D: BoxedVariant + 'static,
{
    fn construct(&self, bytes: Vec<u8>) -> BoxedVariantResult<Box<dyn BoxedVariant>> {
        BoxedVariantType::construct(self, bytes).map(|d| Box::new(d) as Box<dyn BoxedVariant>)
    }

    fn name(&self) -> &'static str {
        BoxedVariantType::name(self)
    }
}

impl<T, D> DynBoxedVariantTypeFactory for T
where
    T: BoxedVariantType<Doc = D> + 'static,
    D: BoxedVariant + 'static,
{
    fn to_dyn_type_tag(self) -> DynBoxedVariantTypeTag {
        Box::new(self)
    }
}
