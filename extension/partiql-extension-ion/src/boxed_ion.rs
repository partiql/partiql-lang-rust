use crate::util::{PartiqlValueTarget, ToPartiqlValue};
use ion_rs::{
    AnyEncoding, Element, ElementReader, IonData, IonResult, IonType, OwnedSequenceIterator,
    Reader, Sequence, Struct, Symbol,
};
use itertools::Itertools;
use partiql_value::boxed_variant::{
    BoxedVariant, BoxedVariantResult, BoxedVariantType, BoxedVariantTypeTag,
    BoxedVariantValueIntoIterator, DynBoxedVariant,
};
use partiql_value::datum::{
    Datum, DatumCategoryOwned, DatumCategoryRef, DatumLower, DatumLowerResult, DatumSeqOwned,
    DatumSeqRef, DatumTupleOwned, DatumTupleRef, DatumValueOwned, DatumValueRef, OwnedSequenceView,
    OwnedTupleView, RefSequenceView, RefTupleView, SequenceDatum, TupleDatum,
};
use partiql_value::{Bag, BindingsName, List, NullableEq, Tuple, Value, Variant};
use peekmore::{PeekMore, PeekMoreIterator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use std::rc::Rc;
use thiserror::Error;

#[derive(Default, Debug, Copy, Clone)]
pub struct BoxedIonType {}
impl BoxedVariantType for BoxedIonType {
    fn construct(&self, bytes: Vec<u8>) -> BoxedVariantResult<DynBoxedVariant> {
        BoxedIon::parse(bytes, BoxedIonStreamType::SingleTLV)
            .map_err(Into::into)
            .map(|b| Box::new(b) as DynBoxedVariant)
    }

    fn name(&self) -> &'static str {
        "ion"
    }

    fn value_eq(&self, l: &DynBoxedVariant, r: &DynBoxedVariant) -> bool {
        wrap_eq::<true, false>(l, r) == Value::Boolean(true)
    }

    fn value_eq_param(
        &self,
        l: &DynBoxedVariant,
        r: &DynBoxedVariant,
        nulls_eq: bool,
        nans_eq: bool,
    ) -> bool {
        let res = match (nulls_eq, nans_eq) {
            (true, true) => wrap_eq::<true, true>(l, r),
            (true, false) => wrap_eq::<true, false>(l, r),
            (false, true) => wrap_eq::<false, true>(l, r),
            (false, false) => wrap_eq::<false, false>(l, r),
        };
        res == Value::Boolean(true)
    }
}

fn wrap_eq<const NULLS_EQUAL: bool, const NAN_EQUAL: bool>(
    l: &DynBoxedVariant,
    r: &DynBoxedVariant,
) -> Value {
    let (l, r) = get_values(l, r);
    let wrap = IonEqualityValue::<'_, { NULLS_EQUAL }, { NAN_EQUAL }, _>;
    NullableEq::eq(&wrap(l), &wrap(r))
}

#[inline]
fn get_value(l: &DynBoxedVariant) -> &BoxedIon {
    l.as_any().downcast_ref::<BoxedIon>().expect("IonValue")
}

#[inline]
fn get_values<'a, 'b>(
    l: &'a DynBoxedVariant,
    r: &'b DynBoxedVariant,
) -> (&'a BoxedIon, &'b BoxedIon) {
    debug_assert_eq!(*l.type_tag(), *r.type_tag());

    (get_value(l), get_value(r))
}

/// Errors in boxed Ion.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum BoxedIonError {
    /// Ion Writer error.
    #[error("Expected a sequence, but was `{elt}`")]
    NotASequence { elt: Box<Element> },
}

pub type Result<T> = std::result::Result<T, BoxedIonError>;

pub struct ElementIterator<R: ElementReader> {
    reader: R,
}

impl<R: ElementReader> Iterator for ElementIterator<R> {
    type Item = IonResult<Element>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.read_next_element().transpose()
    }
}

struct IonContext {
    reader: PeekMoreIterator<ElementIterator<Reader<AnyEncoding, Vec<u8>>>>,
}

impl IonContext {
    pub fn new(data: Vec<u8>) -> IonResult<Self> {
        let reader = Reader::new(AnyEncoding, data)?;
        let reader = ElementIterator { reader }.peekmore();
        Ok(Self { reader })
    }

    pub fn new_ptr(data: Vec<u8>) -> IonResult<IonContextPtr> {
        Ok(Rc::new(RefCell::new(Self::new(data)?)))
    }
}

type IonContextPtr = Rc<RefCell<IonContext>>;

#[derive(Clone)]
pub struct BoxedIon {
    ctx: IonContextPtr,
    doc: BoxedIonValue,
}

#[cfg(feature = "serde")]
impl Serialize for BoxedIon {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!("Serialize for BoxedIon")
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for BoxedIon {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!("Deserialize for BoxedIon")
    }
}

impl Hash for BoxedIon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.doc.hash(state);
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl BoxedVariant for BoxedIon {
    fn type_tag(&self) -> BoxedVariantTypeTag {
        Box::new(BoxedIonType {})
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_dyn_iter(self: Box<Self>) -> BoxedVariantResult<BoxedVariantValueIntoIterator> {
        let iter = self.try_into_iter()?;

        Ok(Box::new(iter.map(|res| {
            res.map(|d| Box::new(d) as Box<dyn BoxedVariant>)
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        })) as BoxedVariantValueIntoIterator)
    }

    fn category(&self) -> DatumCategoryRef<'_> {
        match &self.doc {
            BoxedIonValue::Stream() => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
            BoxedIonValue::Sequence(_) => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
            BoxedIonValue::Value(elt) => {
                if elt.is_null() {
                    DatumCategoryRef::Null
                } else {
                    match elt.ion_type() {
                        IonType::List => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
                        IonType::SExp => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
                        IonType::Null => DatumCategoryRef::Null,
                        IonType::Struct => DatumCategoryRef::Tuple(DatumTupleRef::Dynamic(self)),
                        _ => DatumCategoryRef::Scalar(DatumValueRef::Lower(self)),
                    }
                }
            }
        }
    }

    fn into_category(self: Box<Self>) -> DatumCategoryOwned {
        match &self.doc {
            BoxedIonValue::Stream() => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
            BoxedIonValue::Sequence(_) => {
                DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self))
            }
            BoxedIonValue::Value(elt) => {
                if elt.is_null() {
                    DatumCategoryOwned::Null
                } else {
                    match elt.ion_type() {
                        IonType::List => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
                        IonType::SExp => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
                        IonType::Null => DatumCategoryOwned::Null,
                        IonType::Struct => {
                            DatumCategoryOwned::Tuple(DatumTupleOwned::Dynamic(self))
                        }
                        _ => DatumCategoryOwned::Scalar(DatumValueOwned::Value(self.into_value())),
                    }
                }
            }
        }
    }
}

/// A wrapper on [`T`] that specifies if missing and null values should be equal.
#[derive(Eq, PartialEq)]
pub struct IonEqualityValue<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool, T>(pub &'a T);

impl<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for IonEqualityValue<'a, NULLS_EQUAL, NAN_EQUAL, BoxedIon>
{
    fn eq(&self, rhs: &Self) -> Value {
        let wrap = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(&self.0.doc), &wrap(&rhs.0.doc))
    }
    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = IonEqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl DatumLower<Value> for BoxedIon {
    fn into_lower(self) -> DatumLowerResult<Value> {
        let Self { ctx, doc } = self;
        let pval = match doc {
            BoxedIonValue::Stream() => todo!("DatumLower::into_lower for BoxedIonValue::Stream"),
            BoxedIonValue::Sequence(seq) => seq.into_partiql_value()?,
            BoxedIonValue::Value(elt) => elt.into_partiql_value()?,
        };
        Ok(match pval {
            PartiqlValueTarget::Atom(val) => val,
            PartiqlValueTarget::List(l) => {
                let vals = l.into_iter().map(|elt| Self::new_value(elt, ctx.clone()));
                List::from_iter(vals).into()
            }
            PartiqlValueTarget::Bag(b) => {
                let vals = b.into_iter().map(|elt| Self::new_value(elt, ctx.clone()));
                Bag::from_iter(vals).into()
            }
            PartiqlValueTarget::Struct(s) => {
                let vals = s
                    .into_iter()
                    .map(|(key, elt)| (key, Self::new_value(elt, ctx.clone())));
                Tuple::from_iter(vals).into()
            }
        })
    }

    fn into_lower_boxed(self: Box<Self>) -> DatumLowerResult<Value> {
        self.into_lower()
    }

    fn lower(&self) -> DatumLowerResult<Cow<'_, Value>> {
        self.clone().into_lower().map(Cow::Owned)
    }
}

impl SequenceDatum for BoxedIon {
    fn is_ordered(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        match &self.doc {
            BoxedIonValue::Stream() => {
                todo!("SequenceDatum::len for BoxedIonValue::Stream")
            }
            BoxedIonValue::Sequence(seq) => seq.len(),
            BoxedIonValue::Value(elt) => match elt.expect_sequence() {
                Ok(seq) => seq.len(),
                Err(_) => 0,
            },
        }
    }
}

impl<'a> RefSequenceView<'a, Value> for BoxedIon {
    fn get_val(&self, k: i64) -> Option<Cow<'a, Value>> {
        match &self.doc {
            BoxedIonValue::Stream() => {
                todo!("RefSequenceView::get_val for BoxedIonValue::Stream")
            }
            BoxedIonValue::Sequence(seq) => seq
                .get(k as usize)
                .map(|elt| Cow::Owned(self.child_value(elt.clone()))), // TODO find a way to remove clone
            BoxedIonValue::Value(elt) => match elt.expect_sequence() {
                Ok(seq) => seq
                    .iter()
                    .nth(k as usize)
                    .map(|elt| Cow::Owned(self.child_value(elt.clone()))), // TODO find a way to remove clone
                Err(_) => None,
            },
        }
    }

    fn into_iter(self) -> Box<dyn Iterator<Item = Cow<'a, Value>> + 'a> {
        todo!()
    }
}

impl OwnedSequenceView<Value> for BoxedIon {
    fn take_val(self, k: i64) -> Option<Value> {
        let Self { doc, ctx } = self;
        match doc {
            BoxedIonValue::Stream() => {
                todo!("OwnedSequenceView::take_val for BoxedIonValue::Stream")
            }
            BoxedIonValue::Sequence(seq) => seq
                .into_iter()
                .nth(k as usize)
                .map(|elt| Self::new_value(elt, ctx)),
            BoxedIonValue::Value(elt) => match elt.try_into_sequence() {
                Ok(seq) => seq
                    .into_iter()
                    .nth(k as usize)
                    .map(|elt| Self::new_value(elt, ctx)),
                Err(_) => None,
            },
        }
    }

    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<Value> {
        OwnedSequenceView::take_val(*self, k)
    }

    fn into_iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = Value>> {
        Box::new(
            self.into_dyn_iter()
                .expect("BoxedIon::into_iter_boxed")
                .map(|r| r.expect("BoxedIon::into_iter_boxed"))
                .map(|v| Value::from(Variant::from(v))),
        )
    }
}

impl TupleDatum for BoxedIon {
    fn len(&self) -> usize {
        match &self.doc {
            BoxedIonValue::Stream() => {
                todo!("TupleDatum::len for BoxedIonValue::Stream")
            }
            BoxedIonValue::Sequence(_seq) => 0,
            BoxedIonValue::Value(elt) => match elt.expect_struct() {
                Ok(strct) => strct.len(),
                Err(_) => 0,
            },
        }
    }
}

impl<'a> RefTupleView<'a, Value> for BoxedIon {
    fn get_val(&self, target_key: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        let matcher = target_key.matcher();
        let Self { doc, ctx } = self;
        match doc {
            BoxedIonValue::Stream() => {
                todo!("RefTupleView::get_val for BoxedIonValue::Stream")
            }
            BoxedIonValue::Sequence(_seq) => None,
            BoxedIonValue::Value(elt) => match elt.expect_struct() {
                Ok(strct) => {
                    for (k, elt) in strct {
                        if let Some(k) = k.text() {
                            if matcher.matches(k) {
                                return Some(Cow::Owned(Self::new_value(elt.clone(), ctx.clone())));
                            }
                        }
                    }
                    None
                }
                Err(_) => None,
            },
        }
    }
}

impl OwnedTupleView<Value> for BoxedIon {
    fn take_val(self, target_key: &BindingsName<'_>) -> Option<Value> {
        let matcher = target_key.matcher();
        let Self { doc, ctx } = self;
        match doc {
            BoxedIonValue::Stream() => {
                todo!("OwnedTupleView::take_val for BoxedIonValue::Stream")
            }
            BoxedIonValue::Sequence(_) => None,
            BoxedIonValue::Value(elt) => match elt.try_into_struct() {
                Ok(strct) => {
                    for (k, elt) in strct {
                        if let Some(k) = k.text() {
                            if matcher.matches(k) {
                                return Some(Self::new_value(elt, ctx));
                            }
                        }
                    }
                    None
                }
                Err(_) => None,
            },
        }
    }

    fn take_val_boxed(self: Box<Self>, target_key: &BindingsName<'_>) -> Option<Value> {
        OwnedTupleView::take_val(*self, target_key)
    }
}

impl Debug for BoxedIon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoxedIon").field(&self.doc).finish()
    }
}

impl Display for BoxedIon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.doc, f)
    }
}

impl BoxedIon {
    fn into_value(self) -> Value {
        Value::from(Variant::from(self))
    }

    fn new(doc: impl Into<BoxedIonValue>, ctx: IonContextPtr) -> Self {
        Self {
            ctx,
            doc: doc.into(),
        }
    }
    fn new_value(doc: impl Into<BoxedIonValue>, ctx: IonContextPtr) -> Value {
        Self::new(doc, ctx).into_value()
    }

    fn child(&self, child: impl Into<BoxedIonValue>) -> Self {
        Self {
            ctx: self.ctx.clone(),
            doc: child.into(),
        }
    }

    fn child_value(&self, child: impl Into<BoxedIonValue>) -> Value {
        self.child(child).into_value()
    }

    pub(crate) fn parse(data: Vec<u8>, expected: BoxedIonStreamType) -> IonResult<Self> {
        let mut ctx = IonContext::new_ptr(data)?;
        let doc = Self::init_doc(&mut ctx, expected);
        Ok(Self::new(doc?, ctx))
    }

    #[allow(dead_code)]
    pub(crate) fn parse_unknown(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, BoxedIonStreamType::Unknown)
    }

    #[allow(dead_code)]
    pub(crate) fn parse_tlv(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, BoxedIonStreamType::SingleTLV)
    }

    #[allow(dead_code)]
    pub(crate) fn parse_stream(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, BoxedIonStreamType::Stream)
    }

    fn init_doc(ctx: &mut IonContextPtr, expected: BoxedIonStreamType) -> IonResult<BoxedIonValue> {
        let reader = &mut ctx.borrow_mut().reader;
        let expected = match expected {
            BoxedIonStreamType::Unknown => {
                if reader.peek_nth(1).is_some() {
                    BoxedIonStreamType::Stream
                } else {
                    BoxedIonStreamType::SingleTLV
                }
            }
            other => other,
        };
        Ok(match expected {
            BoxedIonStreamType::Unknown => {
                unreachable!()
            }
            BoxedIonStreamType::Stream => BoxedIonValue::Stream(),
            BoxedIonStreamType::SingleTLV => {
                let elt = reader.next().expect("ion value")?;
                if reader.peek().is_some() {
                    // TODO error on stream instead of TLV?
                }
                BoxedIonValue::Value(elt)
            }
        })
    }

    fn try_into_iter(self) -> Result<BoxedIonIterator> {
        let BoxedIon { ctx, doc } = self;

        let inner = match doc {
            BoxedIonValue::Stream() => BoxedIonIterType::Stream(),
            BoxedIonValue::Value(elt) => match elt.try_into_sequence() {
                Err(err) => {
                    // We could error? But generally PartiQL coerces to a singleton collection...
                    //Err(BoxedIonError::NotASequence { elt }),
                    BoxedIonIterType::Sequence(Sequence::new([err.original_value()]).into_iter())
                }
                Ok(seq) => BoxedIonIterType::Sequence(seq.into_iter()),
            },
            BoxedIonValue::Sequence(seq) => BoxedIonIterType::Sequence(seq.into_iter()),
        }
        .into();

        Ok(BoxedIonIterator { ctx, inner })
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum BoxedIonStreamType {
    Unknown,
    Stream,
    SingleTLV,
}

#[derive(Debug)]
enum BoxedIonValue {
    Stream(),
    Value(Element),
    Sequence(Sequence),
}

impl Hash for BoxedIonValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BoxedIonValue::Stream() => {
                todo!("Hash::hash for BoxedIonValue::Stream")
            }
            BoxedIonValue::Value(val) => {
                let sha = ion_rs::ion_hash::sha256(val).expect("ion hash");
                state.write(&sha);
            }
            BoxedIonValue::Sequence(seq) => {
                for elt in seq {
                    let sha = ion_rs::ion_hash::sha256(elt).expect("ion hash");
                    state.write(&sha);
                }
            }
        }
    }
}

impl<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for IonEqualityValue<'a, NULLS_EQUAL, NAN_EQUAL, BoxedIonValue>
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> Value {
        let wrap = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, Element>;
        let wrap_seq = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, Sequence>;
        match (self.0, other.0) {
            (BoxedIonValue::Value(l), BoxedIonValue::Value(r)) => {
                NullableEq::eq(&wrap(l), &wrap(r))
            }
            (BoxedIonValue::Sequence(l), BoxedIonValue::Sequence(r)) => {
                NullableEq::eq(&wrap_seq(l), &wrap_seq(r))
            }
            _ => Value::Boolean(false),
        }
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = IonEqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for IonEqualityValue<'a, NULLS_EQUAL, NAN_EQUAL, Element>
{
    fn eq(&self, other: &Self) -> Value {
        let wrap_seq = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, Sequence>;
        let wrap_struct = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, Struct>;
        let (l, r) = (self.0, other.0);
        let (lty, rty) = (l.ion_type(), r.ion_type());

        let result = if l.is_null() && r.is_null() {
            NULLS_EQUAL && l.annotations().eq(r.annotations())
        } else {
            match (lty, rty) {
                (IonType::Float, IonType::Float) => {
                    let (lf, rf) = (l.as_float().unwrap(), r.as_float().unwrap());
                    if lf.is_nan() && rf.is_nan() {
                        NAN_EQUAL && l.annotations().eq(r.annotations())
                    } else {
                        lf == rf
                    }
                }

                (IonType::List, IonType::List) => {
                    let (ls, rs) = (l.as_list().unwrap(), r.as_list().unwrap());
                    l.annotations().eq(r.annotations())
                        && NullableEq::eq(&wrap_seq(ls), &wrap_seq(rs)) == Value::Boolean(true)
                }
                (IonType::SExp, IonType::SExp) => {
                    let (ls, rs) = (l.as_sexp().unwrap(), r.as_sexp().unwrap());
                    l.annotations().eq(r.annotations())
                        && NullableEq::eq(&wrap_seq(ls), &wrap_seq(rs)) == Value::Boolean(true)
                }

                (IonType::Struct, IonType::Struct) => {
                    let (ls, rs) = (l.as_struct().unwrap(), r.as_struct().unwrap());
                    l.annotations().eq(r.annotations())
                        && NullableEq::eq(&wrap_struct(ls), &wrap_struct(rs))
                            == Value::Boolean(true)
                }

                _ => l == r,
            }
        };

        Value::Boolean(result)
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = IonEqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for IonEqualityValue<'a, NULLS_EQUAL, NAN_EQUAL, Sequence>
{
    fn eq(&self, other: &Self) -> Value {
        let wrap = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, _>;
        let (l, r) = (self.0, other.0);
        let l = l.iter().map(wrap);
        let r = r.iter().map(wrap);
        let res = l.zip(r).all(|(l, r)| l.eqg(&r) == Value::Boolean(true));
        Value::Boolean(res)
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = IonEqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for IonEqualityValue<'a, NULLS_EQUAL, NAN_EQUAL, Struct>
{
    fn eq(&self, other: &Self) -> Value {
        if self.0.len() != other.0.len() {
            return Value::Boolean(false);
        }

        let (l, r) = (self.0, other.0);
        let l = l.iter();
        let r = r.iter();

        let sort_fn = |(ls, le): &(&Symbol, &Element), (rs, re): &(&Symbol, &Element)| {
            ls.cmp(rs).then(IonData::from(le).cmp(&IonData::from(re)))
        };
        for ((ls, lv), (rs, rv)) in l.sorted_by(sort_fn).zip(r.sorted_by(sort_fn)) {
            if ls != rs {
                return Value::Boolean(false);
            }

            let wrap = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, _>;
            if NullableEq::eqg(&wrap(lv), &wrap(rv)) != Value::Boolean(true) {
                return Value::Boolean(false);
            }
        }
        Value::Boolean(true)
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = IonEqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl From<Element> for BoxedIonValue {
    fn from(value: Element) -> Self {
        BoxedIonValue::Value(value)
    }
}

impl From<Sequence> for BoxedIonValue {
    fn from(value: Sequence) -> Self {
        BoxedIonValue::Sequence(value)
    }
}

impl Clone for BoxedIonValue {
    fn clone(&self) -> Self {
        match self {
            BoxedIonValue::Stream() => {
                todo!("Clone::clone for BoxedIonValue::Stream")
            }
            BoxedIonValue::Value(val) => BoxedIonValue::Value(val.clone()),
            BoxedIonValue::Sequence(seq) => BoxedIonValue::Sequence(seq.clone()),
        }
    }
}

impl Display for BoxedIonValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoxedIonValue::Stream() => {
                todo!("Display::fmt for BoxedIonValue::Stream")
            }
            BoxedIonValue::Value(val) => std::fmt::Display::fmt(val, f),
            BoxedIonValue::Sequence(seq) => std::fmt::Debug::fmt(&seq, f),
        }
    }
}

impl Datum<Value> for BoxedIon {
    fn is_null(&self) -> bool {
        match &self.doc {
            BoxedIonValue::Value(elt) => elt.is_null(),
            BoxedIonValue::Stream() => false,
            BoxedIonValue::Sequence(_) => false,
        }
    }

    fn is_sequence(&self) -> bool {
        match &self.doc {
            BoxedIonValue::Value(elt) => elt.as_sequence().is_some(),
            BoxedIonValue::Stream() => true,
            BoxedIonValue::Sequence(_) => true,
        }
    }

    fn is_ordered(&self) -> bool {
        match self.category() {
            DatumCategoryRef::Sequence(seq) => seq.is_ordered(),
            _ => false,
        }
    }
}

#[derive(Debug)]
enum BoxedIonIterType {
    Stream(),
    Sequence(OwnedSequenceIterator),
}

struct BoxedIonIterator {
    ctx: IonContextPtr,
    inner: RefCell<BoxedIonIterType>,
}

impl Iterator for BoxedIonIterator {
    type Item = IonResult<BoxedIon>;

    fn next(&mut self) -> Option<Self::Item> {
        let elt = match self.inner.borrow_mut().deref_mut() {
            BoxedIonIterType::Stream() => self.ctx.borrow_mut().deref_mut().reader.next(),
            BoxedIonIterType::Sequence(seq) => Ok(seq.next()).transpose(),
        };
        elt.map(|res| res.map(|elt| BoxedIon::new(BoxedIonValue::Value(elt), self.ctx.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flatten_dump(doc: BoxedIon) {
        if doc.is_sequence() {
            for c in doc.try_into_iter().expect("boxed ion iterate") {
                flatten_dump(c.expect("boxed ion element"))
            }
        } else {
            println!("{:?}", doc);
        }
    }

    fn dump(data: Vec<u8>, expected_stream_type: BoxedIonStreamType) {
        println!("\n===========\n");

        let doc = BoxedIon::parse(data, expected_stream_type).expect("boxed ion create");

        flatten_dump(doc);
    }

    #[test]
    fn simple() {
        let one_elt: Vec<u8> =
            "[0, {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})}, [1,2,3,4]]".into();
        let stream: Vec<u8> = "0 {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})} [1,2,3,4]".into();

        dump(one_elt.clone(), BoxedIonStreamType::SingleTLV);
        dump(one_elt, BoxedIonStreamType::Unknown);
        dump(stream.clone(), BoxedIonStreamType::Stream);
        dump(stream, BoxedIonStreamType::Unknown);
    }
}
