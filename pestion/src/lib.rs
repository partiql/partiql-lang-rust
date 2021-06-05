// Copyright Amazon.com, Inc. or its affiliates.

//! Provides simple conversion from [Pest] grammar syntax to Amazon [Ion].
//!
//! ## Example
//!
//! The easiest way to convert [Pest] grammars to Ion is from a `str` slice:
//!
//! ```
//! use pestion::*;
//! use ion_rs::value::*;
//!
//! fn main() -> PestionResult<()> {
//!     // parse a Pest grammar and convert it to Ion element
//!     let element = r#"a = @{ "a" | "b" ~ "c" }"#.pest_to_element()?;
//!
//!     // the grammar is a struct containing a field for each rule
//!     let a_rule = element
//!         .as_struct()
//!         .and_then(|s| s.get("a"))
//!         .and_then(|r| r.as_struct()).unwrap();
//!
//!     // The '@' in the start of the rule means it is atomic
//!     assert_eq!("atomic", a_rule.get("type").and_then(|t| t.as_str()).unwrap());
//!
//!     // the first node in the expression tree is a `choice` operator
//!     assert_eq!(
//!         "choice",
//!         a_rule
//!             .get("expression")
//!             .and_then(|e| e.as_sequence())
//!             .and_then(|s| s.get(0))
//!             .and_then(|h| h.as_str()).unwrap()
//!     );
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

/// Converts a representation of a Pest grammar (or part of a grammar) into Ion [`Element`].
pub trait PestToElement {
    type Element: Element;

    /// Converts this into [`Element`] which may imply parsing Pest syntax.
    fn pest_to_element(&self) -> PestionResult<Self::Element>;
}

impl PestToElement for &str {
    type Element = OwnedElement;

    /// Parses a `str` slice as a Pest grammar and serializes the AST into [`Element`].
    fn pest_to_element(&self) -> PestionResult<Self::Element> {
        let pairs = PestParser::parse(Rule::grammar_rules, *self)?;
        let ast = match consume_rules(pairs) {
            Ok(ast) => ast,
            Err(errors) => {
                return if errors.is_empty() {
                    invalid("Error converting Pest grammar to AST with no context")
                } else {
                    // TODO deal with more than one error..
                    let err = errors.into_iter().next().unwrap();
                    Err(err.into())
                };
            }
        };

        ast.pest_to_element()
    }
}

impl PestToElement for Vec<AstRule> {
    type Element = OwnedElement;

    /// Converts a body of rules into a `struct` that has a rule for each field.
    fn pest_to_element(&self) -> PestionResult<Self::Element> {
        let mut fields = vec![];
        for rule in self.iter() {
            let rule_name = text_token(rule.name.clone());
            let rule_value = rule.pest_to_element()?;
            fields.push((rule_name, rule_value));
        }
        Ok(Self::Element::new_struct(fields))
    }
}

impl PestToElement for AstRule {
    type Element = OwnedElement;

    /// Converts a Pest Rule into a `struct` that has the field for [`RuleType`] as a symbol
    /// and a field for the [`Expr`].
    fn pest_to_element(&self) -> PestionResult<Self::Element> {
        let fields = vec![
            (text_token("type"), self.ty.pest_to_element()?),
            (text_token("expression"), self.expr.pest_to_element()?),
        ];
        Ok(Self::Element::new_struct(fields))
    }
}

impl PestToElement for AstRuleType {
    type Element = OwnedElement;

    /// Serializes the enum into a symbolic value.
    fn pest_to_element(&self) -> PestionResult<Self::Element> {
        let sym_tok = text_token(match self {
            RuleType::Normal => "normal",
            RuleType::Silent => "silent",
            RuleType::Atomic => "atomic",
            RuleType::CompoundAtomic => "compound_atomic",
            RuleType::NonAtomic => "non_atomic",
        });

        Ok(sym_tok.into())
    }
}

impl PestToElement for Expr {
    type Element = OwnedElement;

    /// Generates a `sexp` representation of the rule expression.
    fn pest_to_element(&self) -> PestionResult<Self::Element> {
        use OwnedValue::*;

        let element = Self::Element::new_sexp(match self.clone() {
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
                expr.pest_to_element()?,
            ],
            Expr::NegPred(expr) => vec![
                text_token("predicate").into(),
                text_token("negative").into(),
                expr.pest_to_element()?,
            ],
            Expr::Seq(left, right) => vec![
                text_token("sequence").into(),
                left.pest_to_element()?,
                right.pest_to_element()?,
            ],
            Expr::Choice(left, right) => vec![
                text_token("choice").into(),
                left.pest_to_element()?,
                right.pest_to_element()?,
            ],
            Expr::Opt(expr) => vec![text_token("optional").into(), expr.pest_to_element()?],
            Expr::Rep(expr) => vec![
                text_token("repeat_min").into(),
                0.into(),
                expr.pest_to_element()?,
            ],
            Expr::RepOnce(expr) => vec![
                text_token("repeat_min").into(),
                1.into(),
                expr.pest_to_element()?,
            ],
            Expr::RepMin(expr, min) => vec![
                text_token("repeat_min").into(),
                (min as i64).into(),
                expr.pest_to_element()?,
            ],
            Expr::RepMax(expr, max) => vec![
                text_token("repeat_max").into(),
                (max as i64).into(),
                expr.pest_to_element()?,
            ],
            Expr::RepExact(expr, exact) => vec![
                text_token("repeat_range").into(),
                (exact as i64).into(),
                (exact as i64).into(),
                expr.pest_to_element()?,
            ],
            Expr::RepMinMax(expr, min, max) => vec![
                text_token("repeat_range").into(),
                (min as i64).into(),
                (max as i64).into(),
                expr.pest_to_element()?,
            ],
            // TODO implement these
            Expr::Skip(_) => unimplemented!(),
            Expr::Push(_) => unimplemented!(),
            Expr::PeekSlice(_, _) => unimplemented!(),
        });

        Ok(element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ion_rs::value::reader::*;
    use ion_rs::value::writer::*;
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
                        (sequence
                            (string exact "a")
                            (string insensitive "b")
                        )
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
                        (choice
                            (string exact "a")
                            (string insensitive "b")
                        )
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
                        (choice
                            (sequence
                                (string exact "a")
                                (string insensitive "b")
                            )
                            (sequence
                                (sequence
                                    (string exact "c")
                                    (string insensitive "d")
                                )
                                (string exact "e")
                            )
                        )
                        (sequence
                            (string exact "f")
                            (string exact "g")
                        )
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
    fn good<T, S>(#[case] input: T, #[case] ion_literal: S) -> PestionResult<()>
    where
        T: PestToElement<Element = OwnedElement> + Debug,
        S: AsRef<str>,
    {
        let actual = input.pest_to_element()?;
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
    fn pest_errors<T: PestToElement>(#[case] input: T) -> PestionResult<()> {
        match input.pest_to_element() {
            Err(PestionError::Pest(_)) => {}
            something => {
                unreachable!("Got result we did not expect: {:?}", something);
            }
        }
        Ok(())
    }
}
