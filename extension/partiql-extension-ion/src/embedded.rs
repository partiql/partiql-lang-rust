use core::fmt;
use delegate::delegate;
use ion_rs::{
    AnyEncoding, Element, ElementReader, IonDataSource, IonInput, IonResult, IonSlice, IonType,
    OwnedSequenceIterator, Reader, Sequence,
};
use ion_rs_old::{IonError, IonReader};
use partiql_common::pretty::{pretty_surrounded_doc, PrettyDoc};
use partiql_value::datum::Datum;
use partiql_value::embedded_document::{
    DynEmbeddedTypeTag, EmbeddedDocResult, EmbeddedDocValueIntoIterator, EmbeddedDocument,
    EmbeddedDocumentType,
};
use partiql_value::{EmbeddedDoc, Value, ValueIntoIterator};
use peekmore::{PeekMore, PeekMoreIterator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::slice;
use std::slice::Iter;
use std::sync::Arc;

use thiserror::Error;

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

struct EmbeddedIon {
    ctx: IonContextPtr,
    doc_type: RefCell<EmbeddedDocType>,
}

impl Debug for EmbeddedIon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EmbeddedIon").field(&self.doc_type).finish()
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
    fn force(&self) {
        let stream_type =
            if let EmbeddedDocType::Unexamined(expected) = self.doc_type.borrow().deref() {
                match *expected {
                    EmbeddedDocStreamType::Unknown => {
                        let reader = &mut self.ctx.borrow_mut().reader;
                        if reader.peek_nth(1).is_some() {
                            EmbeddedDocStreamType::Stream
                        } else {
                            EmbeddedDocStreamType::SingleTLV
                        }
                    }
                    other => other,
                }
            } else {
                return;
            };

        self.init_reader(stream_type);
    }

    fn init_reader(&self, expected: EmbeddedDocStreamType) {
        let reader = &mut self.ctx.borrow_mut().reader;
        let doc = match expected {
            EmbeddedDocStreamType::Unknown => {
                unreachable!("handled by `force`")
            }
            EmbeddedDocStreamType::Stream => EmbeddedDocType::Stream(),
            EmbeddedDocStreamType::SingleTLV => {
                let elt = reader.next().expect("ion value"); // TODO [EMBDOC]
                let elt = elt.expect("ion element"); // TODO [EMBDOC]
                if reader.peek().is_some() {
                    // TODO error on stream instead of TLV?
                }

                match elt.try_into_sequence() {
                    Err(elt) => EmbeddedDocType::Value(elt),
                    Ok(seq) => EmbeddedDocType::Sequence(seq.into_iter()),
                }
            }
        };

        self.doc_type.replace(doc);
    }

    fn try_into_iter(mut self) -> Result<EmbeddedIonIterator> {
        self.force();

        let EmbeddedIon { ctx, doc_type } = self;

        let inner = match doc_type.into_inner() {
            EmbeddedDocType::Unexamined(_) => unreachable!("handled by `force`"),
            EmbeddedDocType::Stream() => EmbeddedIterType::Stream(),
            EmbeddedDocType::Value(elt) => match elt.try_into_sequence() {
                Err(elt) => return Err(BoxedIonError::NotASequence { elt }),
                Ok(seq) => EmbeddedIterType::Sequence(seq.into_iter()),
            },
            EmbeddedDocType::Sequence(seq) => EmbeddedIterType::Sequence(seq.into_iter()),
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

#[derive(Debug)]
enum EmbeddedDocType {
    Unexamined(EmbeddedDocStreamType),
    Stream(),
    Value(Element),
    Sequence(OwnedSequenceIterator),
}

impl Datum for EmbeddedIon {
    fn is_null(&self) -> bool {
        self.force();
        match self.doc_type.borrow().deref() {
            EmbeddedDocType::Unexamined(_) => {
                unreachable!("already forced")
            }
            EmbeddedDocType::Value(elt) => elt.is_null(),
            EmbeddedDocType::Stream() => false,
            EmbeddedDocType::Sequence(_) => false,
        }
    }

    fn is_sequence(&self) -> bool {
        self.force();
        match self.doc_type.borrow().deref() {
            EmbeddedDocType::Unexamined(_) => {
                unreachable!("already forced")
            }
            EmbeddedDocType::Value(elt) => elt.as_sequence().is_some(),
            EmbeddedDocType::Stream() => true,
            EmbeddedDocType::Sequence(_) => true,
        }
    }

    fn is_ordered(&self) -> bool {
        self.force();
        match self.doc_type.borrow().deref() {
            EmbeddedDocType::Unexamined(_) => {
                unreachable!("already forced")
            }
            EmbeddedDocType::Value(_) => false,
            EmbeddedDocType::Stream() => false, // TODO [EMBDOC] is a top-level stream ordered?
            EmbeddedDocType::Sequence(_) => true,
        }
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
        elt.map(|elt| EmbeddedIon::new(EmbeddedDocType::Value(elt), self.ctx.clone()))
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
