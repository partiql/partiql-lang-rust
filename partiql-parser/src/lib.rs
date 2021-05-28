// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//!
//! An API to interact with PartiQL tokens is the [`mod@scanner`] module.
//! The [`scanner()`] function creates a [`Scanner`](scanner::Scanner) instance
//! that one can use to parse tokens incrementally from some input slice.
//!
//! ```
//! use partiql_parser::prelude::*;
//! use partiql_parser::scanner;
//!
//! fn main() -> ParserResult<()> {
//!     use partiql_parser::scanner::Content::*;
//!
//!     let mut scanner = scanner("SELECT '🦄💩'");
//!     let first = scanner.next_token()?;
//!
//!     // get the parsed variant of the token
//!     match first.content() {
//!         Keyword(kw) => assert_eq!("SELECT", kw),
//!         _ => panic!("Didn't get a keyword!"),
//!     }
//!     // the entire text of a token can be fetched--which looks the roughly the
//!     // same for a keyword.
//!     assert_eq!("SELECT", first.text());
//!     
//!     let second = scanner.next_token()?;
//!     // get the parsed variant of the token
//!     match second.content() {
//!         StringLiteral(text) => assert_eq!("🦄💩", text),
//!         _ => panic!("Didn't get a string literal!"),
//!     }
//!     // the other thing we can do is get line/column information from a token
//!     assert_eq!(LineAndColumn::try_at(1, 8)?, second.start());
//!     assert_eq!(LineAndColumn::try_at(1, 12)?, second.end());
//!
//!     // this API is built on immutable slices, so we can restart scanning from any token
//!     scanner = first.into();
//!     let second_again = scanner.next_token()?;
//!     assert_eq!(second, second_again);
//!     
//!     Ok(())
//! }
//! ```
//!
//! [partiql]: https://partiql.org

mod peg;
pub mod prelude;
pub mod result;
pub mod scanner;

pub use peg::recognize_partiql;
pub use scanner::scanner;
