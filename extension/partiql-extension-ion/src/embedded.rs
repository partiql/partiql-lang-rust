use core::fmt;
use delegate::delegate;
use ion_rs::{
    AnyEncoding, ConversionOperationError, Element, ElementReader, IonInput, IonResult, IonSlice,
    IonType, OwnedSequenceIterator, Reader, Sequence,
};
use ion_rs_old::{IonError, IonReader};
use partiql_common::pretty::{pretty_surrounded_doc, PrettyDoc};
use partiql_value::datum::{
    Datum, DatumCategoryOwned, DatumCategoryRef, DatumCattt, DatumLowerResult,
};
use partiql_value::embedded_document::{
    DynEmbeddedTypeTag, EmbeddedDocResult, EmbeddedDocValueIntoIterator, EmbeddedDocument,
    EmbeddedDocumentType,
};
use partiql_value::{EmbeddedDoc, Value, ValueIntoIterator};
use peekmore::{PeekMore, PeekMoreIterator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::ops::{ControlFlow, Deref, DerefMut};
use std::rc::Rc;
use std::slice;
use std::slice::Iter;
use std::sync::Arc;
use thiserror::Error;

#[derive(Default, Debug, Copy, Clone)]
pub struct EmbeddedIonType {}
impl EmbeddedDocumentType for EmbeddedIonType {
    type Doc = EmbeddedIon;

    fn construct(&self, bytes: Vec<u8>) -> EmbeddedDocResult<Self::Doc> {
        EmbeddedIon::parse(bytes.into(), EmbeddedDocStreamType::SingleTLV).map_err(Into::into)
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
    NotASequence { elt: Element },
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
}

pub type IonContextPtr = Rc<RefCell<IonContext>>;

// TODO [EMBDOC] does this serialization work?
pub struct EmbeddedIon {
    ctx: IonContextPtr,
    doc_type: RefCell<EmbeddedDocType>,
}

#[cfg(feature = "serde")]
impl Serialize for EmbeddedIon {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EmbeddedIon {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl Clone for EmbeddedIon {
    // TODO [EMBDOC] make clone of doc less expensive
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            doc_type: self.doc_type.clone(),
        }
    }
}

impl Hash for EmbeddedIon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!("EmbeddedIon.hash")
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl EmbeddedDocument for EmbeddedIon {
    fn into_dyn_iter(self: Box<Self>) -> EmbeddedDocResult<EmbeddedDocValueIntoIterator> {
        let iter = self.try_into_iter()?;

        Ok(
            Box::new(iter.map(|d| Box::new(d) as Box<dyn EmbeddedDocument>))
                as EmbeddedDocValueIntoIterator,
        )
    }

    fn category<'a>(&'a self) -> DatumCategoryRef<'a> {
        todo!() //self.doc_type.borrow().category()
    }

    fn into_category(self: Box<Self>) -> DatumCategoryOwned {
        todo!()
    }
}

impl Debug for EmbeddedIon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EmbeddedIon").field(&self.doc_type).finish()
    }
}

impl Display for EmbeddedIon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.force(), f)
    }
}

impl EmbeddedIon {
    pub fn new(doc: EmbeddedDocType, ctx: IonContextPtr) -> Self {
        let doc_type = RefCell::new(doc);
        Self { ctx, doc_type }
    }

    pub fn parse(data: Vec<u8>, expected: EmbeddedDocStreamType) -> IonResult<Self> {
        let ctx = Rc::new(RefCell::new(IonContext::new(data)?));
        Ok(Self::new(EmbeddedDocType::Unexamined(expected), ctx))
    }

    pub fn parse_unknown(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, EmbeddedDocStreamType::Unknown)
    }
    pub fn parse_tlv(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, EmbeddedDocStreamType::SingleTLV)
    }

    pub fn parse_stream(data: Vec<u8>) -> IonResult<Self> {
        Self::parse(data, EmbeddedDocStreamType::Stream)
    }

    #[inline]
    fn force(&self) -> RefMut<'_, EmbeddedValue> {
        let doc = self.doc_type.borrow_mut();
        RefMut::map(doc, |mut doc| match doc {
            EmbeddedDocType::Unexamined(expected) => {
                *doc = EmbeddedDocType::Forced(self.init_reader(*expected));
                match doc {
                    EmbeddedDocType::Unexamined(_) => {
                        unreachable!()
                    }
                    EmbeddedDocType::Forced(doc) => doc,
                }
            }
            EmbeddedDocType::Forced(doc) => doc,
        })
    }

    fn init_reader(&self, expected: EmbeddedDocStreamType) -> EmbeddedValue {
        let reader = &mut self.ctx.borrow_mut().reader;
        let expected = match expected {
            EmbeddedDocStreamType::Unknown => {
                if reader.peek_nth(1).is_some() {
                    EmbeddedDocStreamType::Stream
                } else {
                    EmbeddedDocStreamType::SingleTLV
                }
            }
            other => other,
        };
        match expected {
            EmbeddedDocStreamType::Unknown => {
                unreachable!()
            }
            EmbeddedDocStreamType::Stream => EmbeddedValue::Stream(),
            EmbeddedDocStreamType::SingleTLV => {
                let elt = reader.next().expect("ion value"); // TODO [EMBDOC]
                let elt = elt.expect("ion element"); // TODO [EMBDOC]
                if reader.peek().is_some() {
                    // TODO error on stream instead of TLV?
                }

                match elt.try_into_sequence() {
                    Err(err) => EmbeddedValue::Value(err.original_value()),
                    Ok(seq) => EmbeddedValue::Sequence(seq.into_iter()),
                }
            }
        }
    }

    fn try_into_iter(mut self) -> Result<EmbeddedIonIterator> {
        self.force();
        let EmbeddedIon { ctx, doc_type } = self;

        let inner = match doc_type.into_inner() {
            EmbeddedDocType::Unexamined(_) => unreachable!("handled by `force`"),
            EmbeddedDocType::Forced(EmbeddedValue::Stream()) => EmbeddedIterType::Stream(),
            EmbeddedDocType::Forced(EmbeddedValue::Value(elt)) => match elt.try_into_sequence() {
                Err(err) => {
                    // TODO [EMBDOC]
                    // We could error? But generally PartiQL coerces to a singleton collection...
                    //Err(BoxedIonError::NotASequence { elt }),
                    EmbeddedIterType::Sequence(Sequence::new([err.original_value()]).into_iter())
                }
                Ok(seq) => EmbeddedIterType::Sequence(seq.into_iter()),
            },
            EmbeddedDocType::Forced(EmbeddedValue::Sequence(seq)) => {
                EmbeddedIterType::Sequence(seq.into_iter())
            }
        }
        .into();

        Ok(EmbeddedIonIterator { ctx, inner })
    }
}

#[derive(Debug, Copy, Clone)]
enum EmbeddedDocStreamType {
    Unknown,
    Stream,
    SingleTLV,
}

#[derive(Debug, Clone)]
enum EmbeddedDocType {
    Unexamined(EmbeddedDocStreamType),
    Forced(EmbeddedValue),
}

#[derive(Debug)]
enum EmbeddedValue {
    Stream(),
    Value(Element),
    Sequence(OwnedSequenceIterator),
}

impl Clone for EmbeddedValue {
    fn clone(&self) -> Self {
        // TODO [EMBDOC]
        match self {
            EmbeddedValue::Stream() => {
                todo!("stream not cloneable? ")
            }
            EmbeddedValue::Value(val) => EmbeddedValue::Value(val.clone()),
            EmbeddedValue::Sequence(seq) => {
                todo!("clone for Seq")
            }
        }
    }
}

impl Display for EmbeddedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO [EMBDOC]
        match self {
            EmbeddedValue::Stream() => {
                todo!("stream not displayable? ")
            }
            EmbeddedValue::Value(val) => std::fmt::Display::fmt(val, f),
            EmbeddedValue::Sequence(seq) => {
                todo!("display for Seq")
            }
        }
    }
}

impl Datum<Value> for EmbeddedIon {
    fn is_null(&self) -> bool {
        match self.force().deref() {
            EmbeddedValue::Value(elt) => elt.is_null(),
            EmbeddedValue::Stream() => false,
            EmbeddedValue::Sequence(_) => false,
        }
    }

    fn is_sequence(&self) -> bool {
        match self.force().deref() {
            EmbeddedValue::Value(elt) => elt.as_sequence().is_some(),
            EmbeddedValue::Stream() => true,
            EmbeddedValue::Sequence(_) => true,
        }
    }

    fn is_ordered(&self) -> bool {
        match self.force().deref() {
            EmbeddedValue::Value(_) => false,
            EmbeddedValue::Stream() => false, // TODO [EMBDOC] is a top-level stream ordered?
            EmbeddedValue::Sequence(_) => true,
        }
    }

    fn lower(self) -> DatumLowerResult<Value> {
        todo!("lower for EmbeddedIon")
    }
}

impl<'a> DatumCattt<'a> for EmbeddedDocType {
    fn category(&'a self) -> DatumCategoryRef<'a> {
        match self {
            EmbeddedDocType::Unexamined(_) => {
                todo!("TODO [EMBDOC]")
            }
            EmbeddedDocType::Forced(doc) => doc.category(),
        }
    }

    fn into_category(self) -> DatumCategoryOwned {
        todo!()
    }
}

impl<'a> DatumCattt<'a> for EmbeddedValue {
    fn category(&'a self) -> DatumCategoryRef<'a> {
        todo!()
    }

    fn into_category(self) -> DatumCategoryOwned {
        todo!()
    }
}

#[derive(Debug)]
enum EmbeddedIterType {
    Stream(),
    Sequence(OwnedSequenceIterator),
}

struct EmbeddedIonIterator {
    ctx: IonContextPtr,
    inner: RefCell<EmbeddedIterType>,
}

impl Iterator for EmbeddedIonIterator {
    type Item = EmbeddedIon;

    fn next(&mut self) -> Option<Self::Item> {
        let elt = match self.inner.borrow_mut().deref_mut() {
            EmbeddedIterType::Stream() => {
                let elt = self.ctx.borrow_mut().deref_mut().reader.next();
                let elt = elt.transpose().expect("ion not error"); // TODO [EMBDOC]
                elt
            }
            EmbeddedIterType::Sequence(seq) => seq.next(),
        };
        elt.map(|elt| {
            EmbeddedIon::new(
                EmbeddedDocType::Forced(EmbeddedValue::Value(elt)),
                self.ctx.clone(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ion_rs::{ElementReader, WriteAsIon};
    use std::io::Read;

    fn flatten_dump(doc: EmbeddedIon) {
        if doc.is_sequence() {
            for c in doc.try_into_iter().expect("TODO [EMBDOC]") {
                flatten_dump(c)
            }
        } else {
            println!("{:?}", doc);
        }
    }

    fn dump(data: Vec<u8>, expected_embedded_doc_type: EmbeddedDocStreamType) {
        println!("\n===========\n");

        let doc =
            EmbeddedIon::parse(data, expected_embedded_doc_type).expect("embedded ion create");

        flatten_dump(doc);
    }

    #[test]
    fn simple() {
        let one_elt: Vec<u8> =
            "[0, {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})}, [1,2,3,4]]".into();
        let stream: Vec<u8> = "0 {a: 1, b:2, c: [], d: foo::(SYMBOL 3 2 1 {})} [1,2,3,4]".into();

        dump(one_elt.clone(), EmbeddedDocStreamType::SingleTLV);
        dump(one_elt, EmbeddedDocStreamType::Unknown);
        dump(stream.clone(), EmbeddedDocStreamType::Stream);
        dump(stream, EmbeddedDocStreamType::Unknown);
    }
}
