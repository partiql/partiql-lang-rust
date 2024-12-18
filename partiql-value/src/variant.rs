use crate::boxed_variant::{
    BoxedVariant, BoxedVariantError, BoxedVariantResult, BoxedVariantValueIntoIterator,
    BoxedVariantValueIter, DynBoxedVariant, DynBoxedVariantTypeTag,
};
use crate::datum::{
    Datum, DatumCategory, DatumCategoryOwned, DatumCategoryRef, DatumLowerResult, DatumValue,
};
use delegate::delegate;
use partiql_common::pretty::{pretty_surrounded_doc, PrettyDoc};
use pretty::{DocAllocator, DocBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Variant {
    variant: DynBoxedVariant,
}

impl Variant {
    pub fn new<B: Into<Vec<u8>>>(
        contents: B,
        type_tag: DynBoxedVariantTypeTag,
    ) -> BoxedVariantResult<Self> {
        let variant = Unparsed::new(contents, type_tag)?.parse()?;
        Ok(Self { variant })
    }
}

impl<T> From<T> for Variant
where
    T: BoxedVariant + 'static,
{
    fn from(variant: T) -> Self {
        let variant = Box::new(variant) as DynBoxedVariant;
        Self { variant }
    }
}

impl From<DynBoxedVariant> for Variant {
    fn from(variant: DynBoxedVariant) -> Self {
        Self { variant }
    }
}

impl DatumValue<Variant> for Variant {
    fn into_lower(self) -> DatumLowerResult<Variant> {
        // TODO lower
        Ok(self)
    }
}

impl<'a> DatumCategory<'a> for Variant {
    fn category(&'a self) -> DatumCategoryRef<'a> {
        self.variant.category()
    }

    fn into_category(self) -> DatumCategoryOwned {
        self.variant.into_category()
    }
}

#[derive(Debug, Clone)]
pub struct Unparsed {
    contents: Vec<u8>,
    type_tag: DynBoxedVariantTypeTag,
}

impl Unparsed {
    pub fn new<B: Into<Vec<u8>>>(
        contents: B,
        type_tag: DynBoxedVariantTypeTag,
    ) -> BoxedVariantResult<Self> {
        Ok(Unparsed {
            contents: contents.into(),
            type_tag,
        })
    }
}

#[derive(Error, Debug)]
pub enum VariantError {
    #[error("Latent Type Error for Boxed Document {0}")]
    LatentTypeError(BoxedVariantError, DynBoxedVariantTypeTag),
}

pub type VariantResult<T> = Result<T, VariantError>;

impl Unparsed {
    fn parse(self) -> VariantResult<DynBoxedVariant> {
        let Self { contents, type_tag } = self;
        match type_tag.construct(contents) {
            Ok(doc) => Ok(doc),
            Err(err) => Err(VariantError::LatentTypeError(err, type_tag.clone())),
        }
    }
}

pub struct VariantIter<'a>(BoxedVariantValueIter<'a>);
impl Debug for VariantIter<'_> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Clone for VariantIter<'_> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl IntoIterator for Variant {
    type Item = Variant;
    type IntoIter = VariantIntoIterator;

    fn into_iter(self) -> VariantIntoIterator {
        let iter = self.variant.into_dyn_iter().expect("TODO [EMBDOC]");
        VariantIntoIterator(iter)
    }
}

pub struct VariantIntoIterator(BoxedVariantValueIntoIterator);

impl Iterator for VariantIntoIterator {
    type Item = Variant;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Variant::from)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl Datum<Variant> for Variant {
    delegate! {
        to self.variant {
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

impl Hash for Variant {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        todo!()
    }
}

impl PartialEq<Self> for Variant {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl Eq for Variant {}

#[cfg(feature = "serde")]
impl Serialize for Variant {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Variant {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl PrettyDoc for Variant {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        // TODO [EMBDOC] write out type tag?
        // TODO [EMBDOC] handle backticks more generally.
        let doc = self.variant.pretty_doc(arena);

        pretty_surrounded_doc(doc, "`", "`", arena)
            .append(arena.text("::"))
            .append(arena.text("ion"))
    }
}
