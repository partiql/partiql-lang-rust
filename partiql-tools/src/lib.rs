// Public modules for use in benchmarks and binaries.
pub mod common;
// `pub` (not `pub(crate)`) because the `pqlite` binary in `src/bin/` is a
// separate crate from this library and imports `serialize_row` + the tag/
// version constants directly. Integration tests in `tests/` do the same.
pub mod row_codec;
pub mod storage;
