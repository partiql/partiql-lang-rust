use dyn_clone::DynClone;
use dyn_hash::DynHash;
use partiql_common::pretty::PrettyDoc;
use std::any::Any;
use std::cmp::Ordering;
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

pub trait DynBoxedVariantTypeFactory {
    fn to_dyn_type_tag(self) -> BoxedVariantTypeTag;
}

pub type BoxedVariantTypeTag = Box<dyn BoxedVariantType>;
pub trait BoxedVariantType: Debug + DynClone {
    fn construct(&self, bytes: Vec<u8>) -> BoxedVariantResult<DynBoxedVariant>;
    fn name(&self) -> &'static str;

    fn value_eq(&self, l: &DynBoxedVariant, r: &DynBoxedVariant) -> bool;
    fn value_cmp(&self, l: &DynBoxedVariant, r: &DynBoxedVariant) -> Ordering;
}

dyn_clone::clone_trait_object!(BoxedVariantType);

impl Eq for dyn BoxedVariantType {}
impl PartialEq for dyn BoxedVariantType {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
impl PartialOrd for dyn BoxedVariantType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn BoxedVariantType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl<T> DynBoxedVariantTypeFactory for T
where
    T: BoxedVariantType + 'static,
{
    fn to_dyn_type_tag(self) -> BoxedVariantTypeTag {
        Box::new(self)
    }
}

pub type DynBoxedVariant = Box<dyn BoxedVariant>;
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait BoxedVariant:
    Display + Debug + DynHash + DynClone + Datum<Value> + DatumLower<Value>
{
    fn type_tag(&self) -> BoxedVariantTypeTag;

    fn as_any(&self) -> &dyn Any;
    fn into_dyn_iter(self: Box<Self>) -> BoxedVariantResult<BoxedVariantValueIntoIterator>;

    fn category(&self) -> DatumCategoryRef<'_>;

    fn into_category(self: Box<Self>) -> DatumCategoryOwned;
}

dyn_hash::hash_trait_object!(BoxedVariant);
dyn_clone::clone_trait_object!(BoxedVariant);

impl Eq for DynBoxedVariant {}
impl PartialEq for DynBoxedVariant {
    fn eq(&self, other: &Self) -> bool {
        self.type_tag() == other.type_tag() && self.type_tag().value_eq(self, other)
    }
}
impl PartialOrd for DynBoxedVariant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DynBoxedVariant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_tag()
            .cmp(&other.type_tag())
            .then_with(|| self.type_tag().value_cmp(self, other))
    }
}

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
