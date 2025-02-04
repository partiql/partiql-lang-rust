use crate::boxed_variant::{
    BoxedVariant, BoxedVariantError, BoxedVariantResult, BoxedVariantTypeTag,
    BoxedVariantValueIntoIterator, BoxedVariantValueIter, DynBoxedVariant,
};
use crate::datum::{
    Datum, DatumCategory, DatumCategoryOwned, DatumCategoryRef, DatumLower, DatumLowerResult,
    DatumValue,
};

use crate::{Comparable, EqualityValue, NullSortedValue, NullableEq, Value};
use delegate::delegate;
use partiql_common::pretty::{pretty_surrounded_doc, PrettyDoc};
use pretty::{DocAllocator, DocBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;

use thiserror::Error;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Variant {
    variant: DynBoxedVariant,
}

impl Variant {
    pub fn new<B: Into<Vec<u8>>>(
        contents: B,
        type_tag: BoxedVariantTypeTag,
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

impl DatumValue<Value> for Variant {}

impl DatumLower<Value> for Variant {
    fn into_lower(self) -> DatumLowerResult<Value> {
        self.variant.into_lower_boxed()
    }

    fn into_lower_boxed(self: Box<Self>) -> DatumLowerResult<Value> {
        self.into_lower()
    }

    fn lower(&self) -> DatumLowerResult<Cow<'_, Value>> {
        self.variant.lower()
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
    type_tag: BoxedVariantTypeTag,
}

impl Unparsed {
    pub fn new<B: Into<Vec<u8>>>(
        contents: B,
        type_tag: BoxedVariantTypeTag,
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
    LatentTypeError(BoxedVariantError, BoxedVariantTypeTag),
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

#[allow(dead_code)]
pub struct VariantIter<'a>(BoxedVariantValueIter<'a>);

impl IntoIterator for Variant {
    type Item = BoxedVariantResult<Variant>;
    type IntoIter = VariantIntoIterator;

    fn into_iter(self) -> VariantIntoIterator {
        let iter = self.variant.into_dyn_iter().expect("into_dyn_iter");
        VariantIntoIterator(iter)
    }
}

pub struct VariantIntoIterator(BoxedVariantValueIntoIterator);

impl Iterator for VariantIntoIterator {
    type Item = BoxedVariantResult<Variant>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(Variant::from))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl Datum<Value> for Variant {
    delegate! {
        to self.variant {
            fn is_null(&self) -> bool;
            fn is_missing(&self) -> bool;
            fn is_absent(&self) -> bool;
            fn is_present(&self) -> bool;
            fn is_sequence(&self) -> bool;
            fn is_ordered(&self) -> bool;
        }
    }
}

impl PartialOrd<Self> for Variant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Variant {
    fn cmp(&self, other: &Self) -> Ordering {
        let l = &self.variant;
        let r = &other.variant;
        l.type_tag().cmp(&r.type_tag()).then_with(|| l.cmp(r))
    }
}

impl PartialEq<Self> for Variant {
    fn eq(&self, other: &Self) -> bool {
        let lty = self.variant.type_tag();
        let rty = other.variant.type_tag();
        lty.eq(&rty) && self.variant.eq(&other.variant)
    }
}

impl Eq for Variant {}

impl Hash for Variant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.variant.type_tag().name().hash(state);
        self.variant.hash(state);
    }
}

impl<const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for EqualityValue<'_, NULLS_EQUAL, NAN_EQUAL, Variant>
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> Value {
        let l = &self.0.variant;
        let r = &other.0.variant;
        let lty = l.type_tag();
        let rty = r.type_tag();

        let res = lty == rty && lty.value_eq_param(l, r, NULLS_EQUAL, NAN_EQUAL);
        Value::Boolean(res)
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = EqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl PrettyDoc for Variant {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let doc = self.variant.pretty_doc(arena);

        pretty_surrounded_doc(doc, "`", "`", arena)
        // TODO eventually support suffixing quoted documents with `::<type>`
        //let ty = self.variant.type_tag().name();
        //.append(arena.text("::"))
        //.append(arena.text(ty))
    }
}

impl Comparable for Variant {
    fn is_comparable_to(&self, rhs: &Self) -> bool {
        self.variant.type_tag().name() == rhs.variant.type_tag().name()
    }
}

impl<const NULLS_FIRST: bool> Ord for NullSortedValue<'_, NULLS_FIRST, Variant> {
    fn cmp(&self, other: &Self) -> Ordering {
        let wrap = NullSortedValue::<{ NULLS_FIRST }, _>;

        let l = self.0.lower().expect("lower");
        let l = wrap(l.as_ref());
        let r = other.0.lower().expect("lower");
        let r = wrap(r.as_ref());

        l.cmp(&r)
    }
}
