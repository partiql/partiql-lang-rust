#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate partiql_parser;

use partiql_parser::{parse_partiql, ParserError, ParserResult};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_partiql(s);
    }
});
