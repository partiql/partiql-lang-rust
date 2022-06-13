use partiql_parser::Parser;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
/// Parses the given query and returns the json serialized String.
pub fn parse(query: &str) -> String {
    let parser = Parser::default();
    let res = parser.parse(query);
    match res {
        Ok(r) => serde_json::to_string_pretty(&r).unwrap(),
        Err(e) => serde_json::to_string_pretty(&e).unwrap(),
    }
}
