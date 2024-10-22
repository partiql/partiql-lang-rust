use partiql_common::embedded_document::{EmbeddedDocument, LazyEmbeddedDocument};
use partiql_common::pretty::{pretty_surrounded, pretty_surrounded_doc, PrettyDoc};
use pretty::{DocAllocator, DocBuilder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

pub enum EmbeddedDoc {
    Lazy(LazyEmbeddedDocument),
}

impl Debug for EmbeddedDoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("<<TODO: Debug for EmbeddedDoc>>")
    }
}

impl Hash for EmbeddedDoc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}

impl Clone for EmbeddedDoc {
    fn clone(&self) -> Self {
        match self {
            EmbeddedDoc::Lazy(doc) => Self::Lazy(doc.clone()),
        }
    }
}

impl PartialEq<Self> for EmbeddedDoc {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for EmbeddedDoc {}

#[cfg(feature = "serde")]
impl Serialize for EmbeddedDoc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EmbeddedDoc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl PrettyDoc for EmbeddedDoc {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        todo!()
        /*
        //// TODO handle backticks better
        let doc = self.data.pretty_doc(arena);
        pretty_surrounded_doc(doc, "`````", "`````", arena)

         */
    }
}
