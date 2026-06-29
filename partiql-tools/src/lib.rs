pub mod catalog;
pub mod common;
pub mod row_codec;
pub mod storage;

#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub mod test_support {
    use crate::storage::HeedDB;

    /// Inject a raw row bypassing the encoder so tests can construct malformed
    /// wire bytes without leaking heed types into the public API.
    pub fn inject_row(db: &HeedDB, table: &str, row_id: u64, payload: &[u8]) {
        HeedDB::inject_row_for_tests(db, table, row_id, payload);
    }

    /// Read the raw bytes the encoder wrote at `row_id`.
    pub fn read_row(db: &HeedDB, table: &str, row_id: u64) -> Vec<u8> {
        db.read_row_for_tests(table, row_id)
    }
}
