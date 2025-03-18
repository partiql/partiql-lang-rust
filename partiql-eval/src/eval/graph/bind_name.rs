use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

/// A unicode non-character prefixed onto 'anonymous' bind names
const ANON_PREFIX: char = '\u{FDD0}';

pub trait BindNameExt {
    fn is_anon(&self) -> bool;
}

impl<S: AsRef<str>> BindNameExt for S {
    fn is_anon(&self) -> bool {
        self.as_ref().starts_with(ANON_PREFIX)
    }
}

/// Creates 'fresh' bind names
pub struct FreshBinder {
    #[allow(dead_code)] // TODO remove once graph planning is implemented
    node: AtomicU32,

    #[allow(dead_code)] // TODO remove once graph planning is implemented
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
    #[allow(dead_code)] // TODO remove once graph planning is implemented
    pub fn node(&self) -> String {
        format!("{ANON_PREFIX}🞎{}", self.node.fetch_add(1, Relaxed))
    }

    #[allow(dead_code)] // TODO remove once graph planning is implemented
    pub fn edge(&self) -> String {
        format!("{ANON_PREFIX}⁃{}", self.edge.fetch_add(1, Relaxed))
    }
}
