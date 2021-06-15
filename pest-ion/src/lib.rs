// Copyright Amazon.com, Inc. or its affiliates.

//! Provides simple conversion from [Pest] grammar syntax to Amazon [Ion].
//!
//! ## Example
//!
//! The easiest way to convert [Pest] grammars to Ion is from a `str` slice:
//!
//! ```
//! use pest_ion::*;
//! use ion_rs::value::*;
//! use ion_rs::value::reader::*;
//!
//! fn main() -> PestToIonResult<()> {
//!     // parse a Pest grammar and convert it to Ion element
//!     let actual = r#"a = @{ "a" | "b" ~ "c" }"#.try_pest_to_element()?;
//!
//!     // here is the equivalent Ion element
//!     let ion_text = r#"{
//!         a: {
//!             type: atomic,
//!             expression:
//!                 (choice
//!                     (string exact "a")
//!                     (sequence
//!                         (string exact "b")
//!                         (string exact "c")
//!                     )
//!                 )
//!         }
//!     }"#;
//!     let expected = element_reader().read_one(ion_text.as_bytes())?;
//!
//!     // these should be equivalent
//!     assert_eq!(expected, actual);
//!     
//!     Ok(())
//! }
//! ```
//!
//! [Pest]: https://pest.rs/
//! [Ion]: https://amzn.github.io/ion-docs/

pub mod result;

pub use result::*;

use ion_rs::value::owned::{text_token, OwnedElement, OwnedValue};
use ion_rs::value::{Builder, Element};
use pest::Parser;
use pest_meta::ast::{Expr, Rule as AstRule, RuleType as AstRuleType, RuleType};
use pest_meta::parser::{consume_rules, PestParser, Rule};
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Converts representation of a Pest grammar (or part of a grammar) into Ion [`Element`].
pub trait TryPestToElement {
    type Element: Element;

    /// Converts this into [`Element`] which may imply parsing Pest syntax.
    ///
    /// This returns `Err` if the the conversion fails. For example, this can happen if the
    /// source is not a valid Pest grammar.
    fn try_pest_to_element(&self) -> PestToIonResult<Self::Element>;
}

/// Infallible conversion of a Pest grammar (or part of a grammar) into Ion [`Element`].
pub trait PestToElement {
    type Element: Element;

    /// Converts this into an [`Element`] representation.
    ///
    /// This operation cannot fail and therefore it is implied that it represents some
    /// well formed Pest grammar or component thereof.
    fn pest_to_element(&self) -> Self::Element;
}

impl<T> TryPestToElement for &T
where
    T: TryPestToElement + ?Sized,
{
    type Element = T::Element;

    fn try_pest_to_element(&self) -> PestToIonResult<Self::Element> {
        (*self).try_pest_to_element()
    }
}

impl TryPestToElement for str {
    type Element = OwnedElement;

    /// Parses a `str` slice as a Pest grammar.
    fn try_pest_to_element(&self) -> PestToIonResult<Self::Element> {
        let pairs = PestParser::parse(Rule::grammar_rules, self)?;
        let ast = match consume_rules(pairs) {
            Ok(ast) => ast,
            Err(errors) => {
                return if errors.is_empty() {
                    invalid(
                        "Parser indicated an error happened but didn't return any error messages",
                    )
                } else {
                    // TODO deal with more than one error..
                    let err = errors.into_iter().next().unwrap();
                    Err(err.into())
                };
            }
        };

        Ok(ast.pest_to_element())
    }
}

impl TryPestToElement for String {
    type Element = OwnedElement;

    /// Parses a `String` as a Pest grammar.
    #[inline]
    fn try_pest_to_element(&self) -> PestToIonResult<Self::Element> {
        self.as_str().try_pest_to_element()
    }
}

impl TryPestToElement for Path {
    type Element = OwnedElement;

    /// Reads a file specified by the given path as a Pest Grammar.
    #[inline]
    fn try_pest_to_element(&self) -> PestToIonResult<Self::Element> {
        let file = from_read(File::open(self)?);
        file.try_pest_to_element()
    }
}

/// Wrapper over an arbitrary [`Read`] that can be converted to `Element`.
///
/// [`TryPestToElement`] is designed to work on `&self` which makes sense
/// for data types like `&str`, but cannot work for [`Read`] without interior mutability.
struct ReadWrapper<R: Read>(RefCell<R>);

impl<R: Read> TryPestToElement for ReadWrapper<R> {
    type Element = OwnedElement;

    fn try_pest_to_element(&self) -> PestToIonResult<Self::Element> {
        let mut read = self.0.borrow_mut();
        let mut buffer = String::new();
        read.read_to_string(&mut buffer)?;

        buffer.try_pest_to_element()
    }
}

/// Construct a [`TryPestToElement`] from an arbitrary [`Read`].
///
/// Note that the lack of specialization and the requirement for interior mutability
/// for the [`TryPestToElement`] API requires this wrapper as opposed to a blanket
/// trait implementation.
pub fn from_read<R: Read>(source: R) -> impl TryPestToElement<Element = OwnedElement> {
    ReadWrapper(RefCell::new(source))
}

impl PestToElement for Vec<AstRule> {
    type Element = OwnedElement;

    /// Converts a body of rules into a `struct` that has a rule for each field.
    fn pest_to_element(&self) -> Self::Element {
        let fields = self.iter().map(|rule| {
            let rule_name = text_token(rule.name.clone());
            let rule_value = rule.pest_to_element();
            (rule_name, rule_value)
        });
        Self::Element::new_struct(fields)
    }
}

impl PestToElement for AstRule {
    type Element = OwnedElement;

    /// Converts a Pest Rule into a `struct` that has the field for [`RuleType`] as a symbol
    /// and a field for the [`Expr`].
    fn pest_to_element(&self) -> Self::Element {
        let fields = std::array::IntoIter::new([
            (text_token("type"), self.ty.pest_to_element()),
            (text_token("expression"), self.expr.pest_to_element()),
        ]);
        Self::Element::new_struct(fields)
    }
}

impl PestToElement for AstRuleType {
    type Element = OwnedElement;

    /// Serializes the enum into a symbolic value.
    fn pest_to_element(&self) -> Self::Element {
        let sym_tok = text_token(match self {
            RuleType::Normal => "normal",
            RuleType::Silent => "silent",
            RuleType::Atomic => "atomic",
            RuleType::CompoundAtomic => "compound_atomic",
            RuleType::NonAtomic => "non_atomic",
        });

        sym_tok.into()
    }
}

/// Signalling result for the callback in [`flatten`].
enum ShouldFlatten {
    /// Indicates that the children of the operand should be flattened recursively.
    Yes(Box<Expr>, Box<Expr>),
    /// Indicates that the operand should not be flattened and is transfered back to the caller.
    No(Box<Expr>),
}

/// High order function to flatten associative binary nodes in a Pest expression.
///
/// Certain nodes like the `choice` and `sequence` nodes are associative
/// (though not commutative) and can be flattened in to a variadic node instead
/// of the fixed binary one that the Pest AST has.
///
/// The caller is responsible for seeding the vector with the tag (e.g. `choice`).
///
/// The `determine_flatten` function parameter returns [`ShouldFlatten::Yes`] when the underlying
/// binary operator for an operand should be flattened recursively (for its underlying children),
/// and returns [`ShouldFlatten::No`] when it should not be flattened and the operand is moved
/// back to the caller to covert to [`Element`] normally.
fn flatten<F>(
    sexp_elements: &mut Vec<OwnedElement>,
    left: Box<Expr>,
    right: Box<Expr>,
    determine_flatten: F,
) where
    F: Fn(Box<Expr>) -> ShouldFlatten + Copy,
{
    for operand in std::array::IntoIter::new([left, right]) {
        match determine_flatten(operand) {
            ShouldFlatten::Yes(child_left, child_right) => {
                flatten(sexp_elements, child_left, child_right, determine_flatten);
            }
            ShouldFlatten::No(original) => {
                sexp_elements.push(original.pest_to_element());
            }
        }
    }
}

impl PestToElement for Expr {
    type Element = OwnedElement;

    /// Generates a `sexp` representation of the rule expression.
    fn pest_to_element(&self) -> Self::Element {
        use OwnedValue::*;

        let values = match self.clone() {
            Expr::Str(text) => vec![
                text_token("string").into(),
                text_token("exact").into(),
                String(text).into(),
            ],
            Expr::Insens(text) => vec![
                text_token("string").into(),
                text_token("insensitive").into(),
                String(text).into(),
            ],
            Expr::Range(begin, end) => vec![
                text_token("character_range").into(),
                String(begin).into(),
                String(end).into(),
            ],
            Expr::Ident(name) => vec![text_token("identifier").into(), String(name).into()],
            Expr::PosPred(expr) => vec![
                text_token("predicate").into(),
                text_token("positive").into(),
                expr.pest_to_element(),
            ],
            Expr::NegPred(expr) => vec![
                text_token("predicate").into(),
                text_token("negative").into(),
                expr.pest_to_element(),
            ],
            Expr::Seq(left, right) => {
                let mut elements = vec![text_token("sequence").into()];
                flatten(
                    &mut elements,
                    left,
                    right,
                    |operand: Box<_>| match *operand {
                        Expr::Seq(child_left, child_right) => {
                            ShouldFlatten::Yes(child_left, child_right)
                        }
                        _ => ShouldFlatten::No(operand),
                    },
                );
                elements
            }
            Expr::Choice(left, right) => {
                let mut elements = vec![text_token("choice").into()];
                flatten(
                    &mut elements,
                    left,
                    right,
                    |operand: Box<_>| match *operand {
                        Expr::Choice(child_left, child_right) => {
                            ShouldFlatten::Yes(child_left, child_right)
                        }
                        _ => ShouldFlatten::No(operand),
                    },
                );
                elements
            }
            Expr::Opt(expr) => {
                vec![text_token("optional").into(), expr.pest_to_element()]
            }
            Expr::Rep(expr) => vec![
                text_token("repeat_min").into(),
                0.into(),
                expr.pest_to_element(),
            ],
            Expr::RepOnce(expr) => vec![
                text_token("repeat_min").into(),
                1.into(),
                expr.pest_to_element(),
            ],
            Expr::RepMin(expr, min) => vec![
                text_token("repeat_min").into(),
                (min as i64).into(),
                expr.pest_to_element(),
            ],
            Expr::RepMax(expr, max) => vec![
                text_token("repeat_max").into(),
                (max as i64).into(),
                expr.pest_to_element(),
            ],
            Expr::RepExact(expr, exact) => vec![
                text_token("repeat_range").into(),
                (exact as i64).into(),
                (exact as i64).into(),
                expr.pest_to_element(),
            ],
            Expr::RepMinMax(expr, min, max) => vec![
                text_token("repeat_range").into(),
                (min as i64).into(),
                (max as i64).into(),
                expr.pest_to_element(),
            ],
            // TODO implement these
            Expr::Skip(_) => unimplemented!(),
            Expr::Push(_) => unimplemented!(),
            Expr::PeekSlice(_, _) => unimplemented!(),
        };

        let element = Self::Element::new_sexp(values);

        element
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ion_rs::value::reader::*;
    use ion_rs::value::writer::*;
    use ion_rs::value::*;
    use rstest::*;
    use std::fmt::Debug;
    use std::str::from_utf8;

    #[rstest]
    #[case::string(
        r#"a = { "hello" }"#,
        r#"
        {
            a: {
                type: normal,
                expression: (string exact "hello")
            }
        }"#
    )]
    #[case::case_insensitive_string_atomic(
        r#"a = @{ ^"world" }"#,
        r#"
        {
            a: {
                type: atomic,
                expression: (string insensitive "world")
            }
        }"#
    )]
    #[case::range_silent(
        r#"a = _{ 'a'..'z' }"#,
        r#"
        {
            a: {
                type: silent,
                expression: (character_range "a" "z")
            }
        }"#
    )]
    #[case::range_identifier_compound(
        r#"a = ${ ANY }"#,
        r#"
        {
            a: {
                type: compound_atomic,
                expression: (identifier "ANY")
            }
        }"#
    )]
    #[case::predicates_non_atomic(
        r#"a = !{ &(b) }
           b = !{ !"hi" }"#,
        r#"
        {
            a: {
                type: non_atomic,
                expression: (predicate positive (identifier "b"))
            },
            b: {
                type: non_atomic,
                expression: (predicate negative (string exact "hi"))
            }
        }"#
    )]
    #[case::sequence(
        r#"a = { "a" ~ ^"b" ~ "c" }"#,
        r#"
        {
            a: {
                type: normal,
                expression:
                    (sequence
                        (string exact "a")
                        (string insensitive "b")
                        (string exact "c")
                    )
            }
        }"#
    )]
    #[case::choice(
        r#"a = { "a" | ^"b" | "c" }"#,
        r#"
        {
            a: {
                type: normal,
                expression:
                    (choice
                        (string exact "a")
                        (string insensitive "b")
                        (string exact "c")
                    )
            }
        }"#
    )]
    #[case::mix_choice_seq(
        r#"a = { "a" ~ ^"b" | "c" ~ ^"d" ~ "e" | "f" ~ "g" }"#,
        r#"
        {
            a: {
                type: normal,
                expression:
                    (choice
                        (sequence
                            (string exact "a")
                            (string insensitive "b")
                        )
                        (sequence
                            (string exact "c")
                            (string insensitive "d")
                            (string exact "e")
                        )
                        (sequence
                            (string exact "f")
                            (string exact "g")
                        )
                    )
            }
        }"#
    )]
    #[case::mix_choice_grouping_1(
        r#"a = { "a" ~ (^"b" | "c") ~ ^"d" ~ ("e" | "f") ~ "g" }"#,
        r#"
        {
            a: {
                type: normal,
                expression:
                    (sequence
                        (string exact "a")
                        (choice
                            (string insensitive "b")
                            (string exact "c")
                        )
                        (string insensitive "d")
                        (choice
                            (string exact "e")
                            (string exact "f")
                        )
                        (string exact "g")
                    )
            }
        }"#
    )]
    #[case::all_choice_grouping(
        r#"a = { "a" | (^"b" | "c") | ^"d" | ("e" | "f") | "g" }"#,
        r#"
        {
            a: {
                type: normal,
                expression:
                    (choice
                        (string exact "a")
                        (string insensitive "b")
                        (string exact "c")
                        (string insensitive "d")
                        (string exact "e")
                        (string exact "f")
                        (string exact "g")
                    )
            }
        }"#
    )]
    #[case::optional(
        r#"a = { "a"? }"#,
        r#"
        {
            a: {
                type: normal,
                expression: (optional (string exact "a"))
            }
        }"#
    )]
    #[case::repeat_min(
        r#"a = { "a"* }
           b = { "b"+ }
           c = { "c"{1,} }
           d = { "d"{2,} }"#,
        r#"
        {
            a: {
                type: normal,
                expression: (repeat_min 0 (string exact "a"))
            },
            b: {
                type: normal,
                expression: (repeat_min 1 (string exact "b"))
            },
            c: {
                type: normal,
                expression: (repeat_min 1 (string exact "c"))
            },
            d: {
                type: normal,
                expression: (repeat_min 2 (string exact "d"))
            },
        }"#
    )]
    #[case::repeat_max(
        r#"a = { "a"{,100} }"#,
        r#"
        {
            a: {
                type: normal,
                expression: (repeat_max 100 (string exact "a"))
            },
        }"#
    )]
    #[case::repeat_range(
        r#"a = { "a"{5} ~ "b"{7, 10} }"#,
        r#"
        {
            a: {
                type: normal,
                expression:
                    (sequence
                        (repeat_range 5 5 (string exact "a"))
                        (repeat_range 7 10 (string exact "b"))
                    )
            },
        }"#
    )]
    fn good<T, S>(#[case] input: T, #[case] ion_literal: S) -> PestToIonResult<()>
    where
        T: TryPestToElement<Element = OwnedElement> + Debug,
        S: AsRef<str>,
    {
        let actual = input.try_pest_to_element()?;
        let expected = element_reader().read_one(ion_literal.as_ref().as_bytes())?;

        const BUF_SIZE: usize = 16 * 1024 * 1024;
        let mut buf = vec![0u8; BUF_SIZE];
        let mut writer = Format::Text(TextKind::Pretty).element_writer_for_slice(&mut buf)?;
        writer.write(&actual)?;
        let actual_converted_text = from_utf8(writer.finish()?).unwrap();

        assert_eq!(
            expected,
            actual,
            "Expected \n{}\nbut was\n{}",
            ion_literal.as_ref(),
            actual_converted_text
        );
        Ok(())
    }

    /// The goal here is not to test Pest's meta parsing, but just to ensure that we get errors
    /// from our APIs when we expect to.
    #[rstest]
    #[case::empty_rule(r#"a = {}"#)]
    #[case::self_reference(r#"a = { a }"#)]
    #[case::double_rule(r#"a = { "a" }\n a = { "b" }"#)]
    fn pest_errors<T: TryPestToElement>(#[case] input: T) -> PestToIonResult<()> {
        match input.try_pest_to_element() {
            Err(PestToIonError::Pest(_)) => {}
            something => {
                unreachable!("Got result we did not expect: {:?}", something);
            }
        }
        Ok(())
    }

    /// Simple test case to make sure we can convert the PartiQL grammar
    #[test]
    fn convert_partiql() -> PestToIonResult<()> {
        let grammar_path = Path::new("../partiql-parser/src/peg/partiql.pest");
        let grammar_element = grammar_path.try_pest_to_element()?;
        let grammar_struct = grammar_element
            .as_struct()
            .expect("top-level value must be struct");

        // some sanity check on the generated Ion--the goal here is not to test the
        // serialization of the grammar because that can be refactored over time

        // entry point for parser
        assert!(grammar_struct.get("Query").is_some());

        // entry point for scanner
        assert!(grammar_struct.get("Scanner").is_some());

        Ok(())
    }
}
