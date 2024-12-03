use ion_rs::{
    AnyEncoding, Element, ElementReader, IonResult, IonType, OwnedSequenceIterator, Reader,
    Sequence,
};
use ion_rs_old::IonReader;
use partiql_value::datum::{
    BoxedOwnedSequenceView, Datum, DatumCategoryOwned, DatumCategoryRef, DatumSeqOwned,
    DatumSeqRef, DatumTupleOwned, DatumTupleRef, OwnedTupleView, RefSequenceView, RefTupleView,
};
use partiql_value::embedded_document::{
    EmbeddedDocResult, EmbeddedDocValueIntoIterator, EmbeddedDocument, EmbeddedDocumentType,
};
use partiql_value::{BindingsName, EmbeddedDoc, Value};
use peekmore::{PeekMore, PeekMoreIterator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use std::rc::Rc;
use thiserror::Error;

#[derive(Default, Debug, Copy, Clone)]
pub struct BoxedIonType {}
impl EmbeddedDocumentType for BoxedIonType {
    type Doc = BoxedIon;

    fn construct(&self, bytes: Vec<u8>) -> EmbeddedDocResult<Self::Doc> {
        BoxedIon::parse(bytes, BoxedIonStreamType::SingleTLV).map_err(Into::into)
    }

    fn name(&self) -> &'static str {
        "ion"
    }
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

pub type IonContextPtr = Rc<RefCell<IonContext>>;

// TODO [EMBDOC] does this serialization work?
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
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for BoxedIon {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl Hash for BoxedIon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!("BoxedIon.hash")
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl EmbeddedDocument for BoxedIon {
    fn into_dyn_iter(self: Box<Self>) -> EmbeddedDocResult<EmbeddedDocValueIntoIterator> {
        let iter = self.try_into_iter()?;

        Ok(
            Box::new(iter.map(|d| Box::new(d) as Box<dyn EmbeddedDocument>))
                as EmbeddedDocValueIntoIterator,
        )
    }

    fn category(&self) -> DatumCategoryRef<'_> {
        match &self.doc {
            BoxedIonValue::Stream() => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
            BoxedIonValue::Sequence(seq) => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
            BoxedIonValue::Value(elt) => match elt.ion_type() {
                IonType::List => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
                IonType::SExp => DatumCategoryRef::Sequence(DatumSeqRef::Dynamic(self)),
                IonType::Null => DatumCategoryRef::Null,
                IonType::Struct => DatumCategoryRef::Tuple(DatumTupleRef::Dynamic(self)),
                _ => DatumCategoryRef::Value(todo!()),
            },
        }
    }

    fn into_category(self: Box<Self>) -> DatumCategoryOwned {
        match &self.doc {
            BoxedIonValue::Stream() => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
            BoxedIonValue::Sequence(seq) => {
                DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self))
            }
            BoxedIonValue::Value(elt) => match elt.ion_type() {
                IonType::List => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
                IonType::SExp => DatumCategoryOwned::Sequence(DatumSeqOwned::Dynamic(self)),
                IonType::Null => DatumCategoryOwned::Null,
                IonType::Struct => DatumCategoryOwned::Tuple(DatumTupleOwned::Dynamic(self)),
                _ => DatumCategoryOwned::Value(todo!()),
            },
        }
    }
}
impl<'a> RefSequenceView<'a, Value> for BoxedIon {
    fn is_ordered(&self) -> bool {
        true
    }

    fn get(&self, k: i64) -> Option<Cow<'a, Value>> {
        match &self.doc {
            BoxedIonValue::Stream() => {
                todo!()
            }
            BoxedIonValue::Sequence(seq) => seq
                .clone() // TODO remove clone by holding vecdeque directly?
                .nth(k as usize)
                .map(|elt| Cow::Owned(self.child_value(elt))),
            BoxedIonValue::Value(elt) => match elt.expect_sequence() {
                Ok(seq) => seq
                    .iter()
                    .nth(k as usize)
                    .map(|elt| Cow::Owned(self.child_value(elt.clone()))), // TODO remove clone
                Err(e) => todo!(),
            },
        }
    }
}

impl BoxedOwnedSequenceView<Value> for BoxedIon {
    fn is_ordered(&self) -> bool {
        true
    }

    fn take_val(self: Box<Self>, k: i64) -> Option<Value> {
        let Self { doc, ctx } = *self;
        match doc {
            BoxedIonValue::Stream() => {
                todo!()
            }
            BoxedIonValue::Sequence(mut seq) => {
                seq.nth(k as usize).map(|elt| Self::new_value(elt, ctx))
            }
            BoxedIonValue::Value(elt) => match elt.try_into_sequence() {
                Ok(seq) => seq
                    .into_iter()
                    .nth(k as usize)
                    .map(|elt| Self::new_value(elt, ctx)),
                Err(e) => todo!(),
            },
        }
    }
}

impl<'a> RefTupleView<'a, Value> for BoxedIon {
    fn get(&self, target_key: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        let matcher = target_key.matcher();
        let Self { doc, ctx } = self;
        match doc {
            BoxedIonValue::Stream() => {
                todo!()
            }
            BoxedIonValue::Sequence(seq) => {
                todo!()
            }
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
                Err(e) => todo!(),
            },
        }
    }
}

impl OwnedTupleView<Value> for BoxedIon {
    fn take_val(self, k: &BindingsName<'_>) -> Option<Value> {
        todo!()
    }

    fn take_val_boxed(self: Box<Self>, target_key: &BindingsName<'_>) -> Option<Value> {
        let matcher = target_key.matcher();
        let Self { doc, ctx } = *self;
        match doc {
            BoxedIonValue::Stream() => {
                todo!()
            }
            BoxedIonValue::Sequence(seq) => {
                todo!()
            }
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
                Err(e) => todo!(),
            },
        }
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
        Value::EmbeddedDoc(Box::new(EmbeddedDoc::Boxed(Box::new(self))))
    }

    pub fn new(doc: impl Into<BoxedIonValue>, ctx: IonContextPtr) -> Self {
        Self {
            ctx,
            doc: doc.into(),
        }
    }
    pub fn new_value(doc: impl Into<BoxedIonValue>, ctx: IonContextPtr) -> Value {
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

    pub fn parse(data: Vec<u8>, expected: BoxedIonStreamType) -> IonResult<Self> {
        let mut ctx = IonContext::new_ptr(data)?;
        let doc = Self::init_doc(&mut ctx, expected);
        Ok(Self::new(doc, ctx))
    }

    pub fn parse_unknown(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, BoxedIonStreamType::Unknown)
    }
    pub fn parse_tlv(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, BoxedIonStreamType::SingleTLV)
    }

    pub fn parse_stream(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, BoxedIonStreamType::Stream)
    }

    fn init_doc(ctx: &mut IonContextPtr, expected: BoxedIonStreamType) -> BoxedIonValue {
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
        match expected {
            BoxedIonStreamType::Unknown => {
                unreachable!()
            }
            BoxedIonStreamType::Stream => BoxedIonValue::Stream(),
            BoxedIonStreamType::SingleTLV => {
                let elt = reader.next().expect("ion value"); // TODO [EMBDOC]
                let elt = elt.expect("ion element"); // TODO [EMBDOC]
                if reader.peek().is_some() {
                    // TODO error on stream instead of TLV?
                }

                match elt.try_into_sequence() {
                    Err(err) => BoxedIonValue::Value(err.original_value()),
                    Ok(seq) => BoxedIonValue::Sequence(seq.into_iter()),
                }
            }
        }
    }

    fn try_into_iter(self) -> Result<BoxedIonIterator> {
        let BoxedIon { ctx, doc } = self;

        let inner = match doc {
            BoxedIonValue::Stream() => BoxedIonIterType::Stream(),
            BoxedIonValue::Value(elt) => match elt.try_into_sequence() {
                Err(err) => {
                    // TODO [EMBDOC]
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
enum BoxedIonStreamType {
    Unknown,
    Stream,
    SingleTLV,
}

#[derive(Debug)]
enum BoxedIonValue {
    Stream(),
    Value(Element),
    Sequence(OwnedSequenceIterator),
}

impl From<Element> for BoxedIonValue {
    fn from(value: Element) -> Self {
        BoxedIonValue::Value(value)
    }
}

impl From<OwnedSequenceIterator> for BoxedIonValue {
    fn from(value: OwnedSequenceIterator) -> Self {
        BoxedIonValue::Sequence(value)
    }
}

impl Clone for BoxedIonValue {
    fn clone(&self) -> Self {
        // TODO [EMBDOC]
        match self {
            BoxedIonValue::Stream() => {
                todo!("stream not cloneable? ")
            }
            BoxedIonValue::Value(val) => BoxedIonValue::Value(val.clone()),
            BoxedIonValue::Sequence(seq) => {
                todo!("clone for Seq")
            }
        }
    }
}

impl Display for BoxedIonValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO [EMBDOC]
        match self {
            BoxedIonValue::Stream() => {
                todo!("stream not displayable? ")
            }
            BoxedIonValue::Value(val) => std::fmt::Display::fmt(val, f),
            BoxedIonValue::Sequence(seq) => {
                todo!("display for Seq")
            }
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
        match &self.doc {
            BoxedIonValue::Value(_) => false,
            BoxedIonValue::Stream() => false, // TODO [EMBDOC] is a top-level stream ordered?
            BoxedIonValue::Sequence(_) => true,
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
    type Item = BoxedIon;

    fn next(&mut self) -> Option<Self::Item> {
        let elt = match self.inner.borrow_mut().deref_mut() {
            BoxedIonIterType::Stream() => {
                let elt = self.ctx.borrow_mut().deref_mut().reader.next();
                // TODO [EMBDOC]
                elt.transpose().expect("ion not error")
            }
            BoxedIonIterType::Sequence(seq) => seq.next(),
        };
        elt.map(|elt| BoxedIon::new(BoxedIonValue::Value(elt), self.ctx.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flatten_dump(doc: BoxedIon) {
        if doc.is_sequence() {
            for c in doc.try_into_iter().expect("TODO [EMBDOC]") {
                flatten_dump(c)
            }
        } else {
            println!("{:?}", doc);
        }
    }

    fn dump(data: Vec<u8>, expected_stream_type: BoxedIonStreamType) {
        println!("\n===========\n");

        let doc = BoxedIon::parse(data, expected_stream_type).expect("embedded ion create");

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
