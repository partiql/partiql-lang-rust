use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

const PREFIX: char = '\u{FDD0}';

pub trait BindNameExt {
    fn is_anon(&self) -> bool;
}

impl<S: AsRef<str>> BindNameExt for S {
    fn is_anon(&self) -> bool {
        self.as_ref().starts_with(PREFIX)
    }
}

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
    pub fn node(&self) -> String {
        format!("{PREFIX}🞎{}", self.node.fetch_add(1, Relaxed))
    }

    pub fn edge(&self) -> String {
        format!("{PREFIX}⁃{}", self.edge.fetch_add(1, Relaxed))
    }
}
