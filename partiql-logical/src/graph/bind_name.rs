use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

/// A Unicode non-character prefixed onto 'anonymous' bind names
const ANON_PREFIX: char = '\u{FDD0}';

pub trait BindNameExt {
    /// `true` if a bind name is 'anonymous'. Anonymous bind names are stand-ins in places
    /// where the graph match expression doesn't explicitly include a bind name variable.
    fn is_anon(&self) -> bool;
}

impl<S: AsRef<str>> BindNameExt for S {
    fn is_anon(&self) -> bool {
        self.as_ref().starts_with(ANON_PREFIX)
    }
}

/// Creates 'fresh' bind names
#[derive(Debug)]
pub struct FreshBinder {
    node: AtomicU32,
    edge: AtomicU32,
}

impl Default for FreshBinder {
    fn default() -> Self {
        Self {
            node: AtomicU32::new(1),
            edge: AtomicU32::new(1),
        }
    }
}

impl FreshBinder {
    /// Creates an 'anonymous' bind name for a node. Anonymous bind names are stand-ins in places
    /// where the graph match expression doesn't explicitly include a bind name variable.
    pub fn node(&self) -> String {
        format!("{ANON_PREFIX}🞎{}", self.node.fetch_add(1, Relaxed))
    }

    /// Creates an 'anonymous' bind name for an edge. Anonymous bind names are stand-ins in places
    /// where the graph match expression doesn't explicitly include a bind name variable.
    pub fn edge(&self) -> String {
        format!("{ANON_PREFIX}⁃{}", self.edge.fetch_add(1, Relaxed))
    }
}
