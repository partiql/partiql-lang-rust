#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct CatalogId(u64);

impl From<u64> for CatalogId {
    fn from(value: u64) -> Self {
        CatalogId(value)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct EntryId(u64);

impl From<u64> for EntryId {
    fn from(value: u64) -> Self {
        EntryId(value)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct ObjectId {
    catalog_id: CatalogId,
    entry_id: EntryId,
}

impl ObjectId {
    pub fn new(catalog_id: CatalogId, entry_id: EntryId) -> Self {
        Self {
            catalog_id,
            entry_id,
        }
    }

    pub fn catalog_id(&self) -> CatalogId {
        self.catalog_id
    }
    pub fn entry_id(&self) -> EntryId {
        self.entry_id
    }
}

impl From<(CatalogId, EntryId)> for ObjectId {
    fn from((catalog_id, entry_id): (CatalogId, EntryId)) -> Self {
        ObjectId::new(catalog_id, entry_id)
    }
}
