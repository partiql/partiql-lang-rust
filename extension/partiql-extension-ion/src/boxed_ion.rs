use crate::util::{PartiqlValueTarget, ToPartiqlValue};
use ion_rs::{
    AnyEncoding, Element, ElementReader, IonData, IonError, IonResult, IonType,
    OwnedSequenceIterator, Reader, Sequence, Struct,
};
use partiql_value::boxed_variant::{
    BoxedVariant, BoxedVariantResult, BoxedVariantType, BoxedVariantTypeTag,
    BoxedVariantValueIntoIterator, DynBoxedVariant,
};
use partiql_value::datum::{
    Datum, DatumCategoryOwned, DatumCategoryRef, DatumLower, DatumLowerResult, DatumSeqOwned,
    DatumSeqRef, DatumTupleOwned, DatumTupleRef, DatumValueOwned, DatumValueRef, OwnedFieldView,
    OwnedSequenceView, OwnedTupleView, RefSequenceView, RefTupleView, SequenceDatum, TupleDatum,
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
use std::io::{BufReader, Cursor, Read};
use std::ops::DerefMut;
use std::rc::Rc;
use thiserror::Error;

#[derive(Default, Debug, Copy, Clone)]
pub struct BoxedIonType {}
impl BoxedVariantType for BoxedIonType {
    fn construct(&self, bytes: Vec<u8>) -> BoxedVariantResult<DynBoxedVariant> {
        self.value_from_bytes(bytes)
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

impl BoxedIonType {
    pub fn value_from_read<I: Read + 'static>(
        &self,
        mut input: BufReader<I>,
    ) -> BoxedIonResult<BoxedIon> {
        let mut output = Default::default();
        input.read_to_end(&mut output).expect("read");
        self.value_from_bytes(output)
    }
    pub fn value_from_str(&self, data: &str) -> BoxedIonResult<BoxedIon> {
        self.value_from_bytes(data.into())
    }

    pub fn value_from_bytes(&self, bytes: Vec<u8>) -> BoxedIonResult<BoxedIon> {
        let cursor = Box::new(Cursor::new(bytes));
        BoxedIon::parse(cursor, BoxedIonStreamType::SingleTLV)
    }

    pub fn stream_from_read<I: Read + 'static>(
        &self,
        input: BufReader<I>,
    ) -> BoxedIonResult<BoxedIon> {
        let buff = Box::new(input);
        BoxedIon::parse_unknown(buff)
    }
}

fn wrap_eq<const NULLS_EQUAL: bool, const NAN_EQUAL: bool>(
    l: &DynBoxedVariant,
    r: &DynBoxedVariant,
) -> Value {
    let (l, r) = dynvar_pair_to_boxed_ion(l, r);
    let wrap = IonEqualityValue::<'_, { NULLS_EQUAL }, { NAN_EQUAL }, _>;
    NullableEq::eq(&wrap(l), &wrap(r))
}

#[inline]
pub(crate) fn dynvar_to_boxed_ion(l: &DynBoxedVariant) -> &BoxedIon {
    l.as_any().downcast_ref::<BoxedIon>().expect("IonValue")
}

#[inline]
fn dynvar_pair_to_boxed_ion<'a, 'b>(
    l: &'a DynBoxedVariant,
    r: &'b DynBoxedVariant,
) -> (&'a BoxedIon, &'b BoxedIon) {
    debug_assert_eq!(*l.type_tag(), *r.type_tag());

    (dynvar_to_boxed_ion(l), dynvar_to_boxed_ion(r))
}

/// Errors in boxed Ion.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum BoxedIonError {
    /// Ion Read error.
    #[error("Error reading Ion `{0}`")]
    IonReadError(#[from] IonError),

    /// Expected a sequence error.
    #[error("Expected a sequence, but was `{elt}`")]
    NotASequence { elt: Box<Element> },
}

pub type BoxedIonResult<T> = std::result::Result<T, BoxedIonError>;

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
    reader: PeekMoreIterator<ElementIterator<Reader<AnyEncoding, Box<dyn Read>>>>,
}

impl IonContext {
    fn new_ctx(data: Box<dyn Read>) -> IonResult<Self> {
        let reader = Reader::new(AnyEncoding, data)?;
        let reader = ElementIterator { reader }.peekmore();
        Ok(Self { reader })
    }

    pub fn new_ptr(data: Box<dyn Read>) -> IonResult<IonContextPtr> {
        Ok(Rc::new(RefCell::new(Self::new_ctx(Box::new(data))?)))
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
            BoxedIonValue::Sequence(_seq) => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
            BoxedIonValue::Value(elt) => {
                if elt.is_null() {
                    DatumCategoryRef::Null
                } else {
                    match elt.ion_type() {
                        IonType::List => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
                        IonType::SExp => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
                        IonType::Null => DatumCategoryRef::Null,
                        IonType::Struct => DatumCategoryRef::Tuple(DatumTupleRef::Dynamic(self)),
                        _ => DatumCategoryRef::Scalar(DatumValueRef::Dynamic(self)),
                    }
                }
            }
        }
    }

    fn into_category(self: Box<Self>) -> DatumCategoryOwned {
        match &self.doc {
            BoxedIonValue::Stream() => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
            BoxedIonValue::Sequence(_seq) => {
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

        if let BoxedIonValue::Value(elt) = doc {
            if let Ok(strct) = elt.expect_struct() {
                for (k, elt) in strct {
                    if k.text().is_some_and(|k| matcher.matches(k)) {
                        return Some(Cow::Owned(Self::new_value(elt.clone(), ctx.clone())));
                    }
                }
            }
        }
        None
    }
}

impl OwnedTupleView<Value> for BoxedIon {
    fn take_val(self, target_key: &BindingsName<'_>) -> Option<Value> {
        let matcher = target_key.matcher();
        let Self { doc, ctx } = self;

        if let BoxedIonValue::Value(elt) = doc {
            if let Ok(strct) = elt.try_into_struct() {
                for (k, elt) in strct {
                    if k.text().is_some_and(|k| matcher.matches(k)) {
                        return Some(Self::new_value(elt, ctx));
                    }
                }
            }
        }
        None
    }

    fn take_val_boxed(self: Box<Self>, target_key: &BindingsName<'_>) -> Option<Value> {
        OwnedTupleView::take_val(*self, target_key)
    }

    fn into_iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = OwnedFieldView<Value>>> {
        let Self { doc, ctx } = *self;
        if let BoxedIonValue::Value(elt) = doc {
            if let Ok(strct) = elt.try_into_struct() {
                return Box::new(strct.into_iter().map(move |(name, value)| {
                    let name = name.text().unwrap_or("").to_string();
                    let value = Self::new_value(value, ctx.clone());
                    OwnedFieldView { name, value }
                }));
            }
        }
        Box::new(std::iter::empty())
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
    pub fn into_value(self) -> Value {
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

    pub(crate) fn parse(data: Box<dyn Read>, expected: BoxedIonStreamType) -> BoxedIonResult<Self> {
        let mut ctx = IonContext::new_ptr(data)?;
        let doc = Self::init_doc(&mut ctx, expected);
        Ok(Self::new(doc?, ctx))
    }

    #[allow(dead_code)]
    pub(crate) fn parse_unknown(data: Box<dyn Read>) -> BoxedIonResult<Self> {
        Self::parse(data, BoxedIonStreamType::Unknown)
    }

    #[allow(dead_code)]
    pub(crate) fn parse_tlv(data: Box<dyn Read>) -> BoxedIonResult<Self> {
        Self::parse(data, BoxedIonStreamType::SingleTLV)
    }

    #[allow(dead_code)]
    pub(crate) fn parse_stream(data: Box<dyn Read>) -> BoxedIonResult<Self> {
        Self::parse(data, BoxedIonStreamType::Stream)
    }

    fn init_doc(
        ctx: &mut IonContextPtr,
        expected: BoxedIonStreamType,
    ) -> BoxedIonResult<BoxedIonValue> {
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

    pub fn try_into_iter(self) -> BoxedIonResult<BoxedIonIterator> {
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

    pub(crate) fn try_into_element(&self) -> BoxedIonResult<Element> {
        let elt = if let BoxedIonValue::Value(elt) = &self.doc {
            elt.clone()
        } else if let BoxedIonValue::Sequence(seq) = &self.doc {
            Element::from(ion_rs::Value::List(seq.clone()))
        } else {
            todo!()
            /*
            let mut elts = Vec::new();
            for ion in self.try_into_iter()? {
                elts.push(ion?.try_into_element()?);
            }
            Element::from(ion_rs::Value::List(elts.into()))

             */
        };
        Ok(elt)
    }

    // TODO remove this double-encoding once encode/decode are upgraded
    // to latest ion-rs
    pub(crate) fn try_into_element_encoded(&self) -> BoxedIonResult<Vec<u8>> {
        Ok(self.try_into_element()?.encode_as(ion_rs::v1_0::Binary)?)
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

        let result = match (l.is_null(), r.is_null()) {
            (true, true) => NULLS_EQUAL,
            (false, false) => match (l.ion_type(), r.ion_type()) {
                (IonType::Float, IonType::Float) => {
                    let (l, r) = (l.as_float().unwrap(), r.as_float().unwrap());
                    if l.is_nan() && r.is_nan() {
                        NAN_EQUAL
                    } else {
                        l == r
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

                // There are some slight unexpected behavior in this equality
                // Check https://github.com/amazon-ion/ion-rust/issues/903 for fixes
                _ => l == r,
            },
            _ => false,
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
        let res: bool = if l.len() == r.len() {
            let l = l.iter().map(wrap);
            let r = r.iter().map(wrap);
            l.zip(r).all(|(l, r)| l.eqg(&r) == Value::Boolean(true))
        } else {
            false
        };
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
        let wrap = IonEqualityValue::<'a, { NULLS_EQUAL }, { NAN_EQUAL }, _>;

        let (l, r) = (self.0, other.0);
        let res: bool = if l.len() == r.len() {
            let l = l.iter().map(|(s, elt)| (s, wrap(elt)));
            let r = r.iter().map(|(s, elt)| (s, wrap(elt)));
            l.zip(r)
                .all(|((ls, lelt), (rs, relt))| ls == rs && lelt.eqg(&relt) == Value::Boolean(true))
        } else {
            false
        };
        Value::Boolean(res)
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

    fn is_missing(&self) -> bool {
        false
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

pub struct BoxedIonIterator {
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
    use partiql_value::datum::DatumCategory;
    use partiql_value::{Comparable, EqualityValue};
    use std::cmp::Ordering;

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

        let doc = BoxedIon::parse(Box::new(Cursor::new(data)), expected_stream_type)
            .expect("boxed ion create");

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

    #[test]
    fn equality() {
        let ty = BoxedIonType {};

        let one_elt: Vec<u8> =
            "[0, {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})}, [1,2,3,4,nan], null, (null nan)]"
                .into();
        let stream: Vec<u8> =
            "0 {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})} [1,2,3,4,nan] null.int (null nan)"
                .into();

        let one_elt = BoxedIon::parse_tlv(Box::new(Cursor::new(one_elt))).expect("elt");
        let stream = BoxedIon::parse_stream(Box::new(Cursor::new(stream))).expect("stream");

        let one_elt = Box::new(one_elt).into_dyn_iter().expect("elt");
        let stream = Box::new(stream).into_dyn_iter().expect("stream");
        for (x, y) in one_elt.zip(stream) {
            let (x, y) = (x.unwrap(), y.unwrap());

            let contains_null = contains_null(x.clone().into_lower_boxed().unwrap());
            let contains_nan = contains_nan(x.clone().into_lower_boxed().unwrap());

            let eq = ty.value_eq(&x, &y);

            let eqff = ty.value_eq_param(&x, &y, false, false);
            let eqtf = ty.value_eq_param(&x, &y, true, false);
            let eqft = ty.value_eq_param(&x, &y, false, true);
            let eqtt = ty.value_eq_param(&x, &y, true, true);

            assert_eq!(eq, eqtf);

            match (contains_null, contains_nan) {
                (false, false) => {
                    assert!(eqff);
                    assert!(eqtf);
                    assert!(eqft);
                    assert!(eqtt);
                }
                (true, false) => {
                    assert!(!eqff);
                    assert!(eqtf);
                    assert!(!eqft);
                    assert!(eqtt);
                }
                (false, true) => {
                    assert!(!eqff);
                    assert!(!eqtf);
                    assert!(eqft);
                    assert!(eqtt);
                }
                (true, true) => {
                    assert!(!eqff);
                    assert!(!eqtf);

                    // Note that although `eqft` says NULL_EQUAL is false, PartiQL's eqg ensures embedded
                    // NULLs are compared as equal
                    assert!(eqft);
                    assert!(eqtt);
                }
            }
        }
    }

    fn contains_null(doc: Value) -> bool {
        match doc.into_category() {
            DatumCategoryOwned::Null => true,
            DatumCategoryOwned::Tuple(t) => t
                .into_iter()
                .any(|OwnedFieldView { value, .. }| contains_null(value)),
            DatumCategoryOwned::Sequence(seq) => seq.into_iter().any(contains_null),
            DatumCategoryOwned::Scalar(s) => s.is_null(),
            _ => false,
        }
    }

    fn contains_nan(doc: Value) -> bool {
        match doc.into_category() {
            DatumCategoryOwned::Tuple(t) => t
                .into_iter()
                .any(|OwnedFieldView { value, .. }| contains_nan(value)),
            DatumCategoryOwned::Sequence(seq) => seq.into_iter().any(contains_nan),
            DatumCategoryOwned::Scalar(s) => {
                let value = s.into_lower().unwrap();
                match value {
                    Value::Real(f) => f.is_nan(),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    #[test]
    fn eqg() {
        let one_elt: Vec<u8> =
            "[0, {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})}, [1,2,3,4], (a b c)]".into();
        let stream: Vec<u8> =
            "0 {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})} [1,2,3,4] (a b c)".into();

        let one_elt = BoxedIon::parse_tlv(Box::new(Cursor::new(one_elt))).expect("elt");
        let stream = BoxedIon::parse_stream(Box::new(Cursor::new(stream))).expect("stream");

        let one_elt = one_elt.try_into_iter().expect("elt");
        let stream = stream.try_into_iter().expect("stream");
        for (x, y) in one_elt.zip(stream) {
            let (x, y) = (x.unwrap(), y.unwrap());

            let wrap = IonEqualityValue::<'_, true, true, BoxedIon>;
            let wrap_val = IonEqualityValue::<'_, true, true, BoxedIonValue>;
            let wrap_elt = IonEqualityValue::<'_, true, true, Element>;
            let wrap_seq = IonEqualityValue::<'_, true, true, Sequence>;
            let wrap_struct = IonEqualityValue::<'_, true, true, Struct>;

            let (wx, wy) = (wrap(&x), wrap(&y));
            assert_eq!(wx.eqg(&wy), Value::Boolean(true));
            let (wx, wy) = (wrap_val(&x.doc), wrap_val(&y.doc));
            assert_eq!(wx.eqg(&wy), Value::Boolean(true));

            match (x.clone().doc, y.clone().doc) {
                (BoxedIonValue::Value(vx), BoxedIonValue::Value(vy)) => {
                    let (wx, wy) = (wrap_elt(&vx), wrap_elt(&vy));
                    assert_eq!(wx.eqg(&wy), Value::Boolean(true));

                    match (vx.value(), vy.value()) {
                        (ion_rs::Value::List(lx), ion_rs::Value::List(ly)) => {
                            let (wx, wy) = (wrap_seq(lx), wrap_seq(ly));
                            assert_eq!(wx.eqg(&wy), Value::Boolean(true));
                        }
                        (ion_rs::Value::SExp(lx), ion_rs::Value::SExp(ly)) => {
                            let (wx, wy) = (wrap_seq(lx), wrap_seq(ly));
                            assert_eq!(wx.eqg(&wy), Value::Boolean(true));
                        }
                        (ion_rs::Value::Struct(sx), ion_rs::Value::Struct(sy)) => {
                            let (wx, wy) = (wrap_struct(sx), wrap_struct(sy));
                            assert_eq!(wx.eqg(&wy), Value::Boolean(true));
                        }
                        _ => (),
                    }
                }
                (BoxedIonValue::Sequence(sx), BoxedIonValue::Sequence(sy)) => {
                    let (wx, wy) = (wrap_seq(&sx), wrap_seq(&sy));
                    assert_eq!(wx.eqg(&wy), Value::Boolean(true));
                }
                _ => unreachable!(),
            }

            let (xv, yv) = (
                Value::Variant(Box::new(x.into())),
                Value::Variant(Box::new(y.into())),
            );
            match (xv, yv) {
                (Value::Variant(xv), Value::Variant(yv)) => {
                    assert_eq!(xv.partial_cmp(&yv), Some(Ordering::Equal));
                    assert!(xv.is_comparable_to(&yv));
                    let wrap = EqualityValue::<'_, true, false, Variant>;
                    let (xv, yv) = (wrap(&xv), wrap(&yv));
                    assert_eq!(NullableEq::eqg(&xv, &yv), Value::Boolean(true));
                }
                _ => unreachable!(),
            }
        }
    }
}
