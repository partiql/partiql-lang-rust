use ion_rs::{IonType, ReaderBuilder};
use miette::{miette, IntoDiagnostic};
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;

use crate::schema::spec::*;
use crate::schema::structure::*;
use ion_rs::element::reader::ElementReader;
use ion_rs::element::{Element, Struct};
use std::path::Path;

macro_rules! expect_value {
    ($val:expr, $msg:literal $(,)?) => {
        match $val {
            Some(v) => Ok(v),
            None => Err(miette!($msg))
        }?
    };
    ($val:expr, $fmt:expr, $($arg:tt)*) => {
        match $val {
            Some(v) => Ok(v),
            None => Err(miette!($fmt, $($arg)*))
        }?
    };
}
macro_rules! expect_str {
    ($val:expr, $msg:literal $(,)?) => {
        expect_value!(
            expect_value!($val, $msg).as_text(), "{} to be a string", $msg)
    };
    ($val:expr, $fmt:expr, $($arg:tt)*) => {
        expect_value!(
            expect_value!($val, $fmt, $($arg)*).as_str(),
            "{} to be a string", format!($fmt, $($arg)*))

    };
}
macro_rules! expect_struct {
    ($val:expr, $msg:literal $(,)?) => {
        expect_value!(
            expect_value!($val, $msg).as_struct(), "{} to be a struct", $msg)
    };
    ($val:expr, $fmt:expr, $($arg:tt)*) => {
        expect_value!(
            expect_value!($val, $fmt, $($arg)*).as_struct(),
            "{} to be a struct", format!($fmt, $($arg)*))

    };
}
macro_rules! expect_list {
    ($val:expr, $msg:literal $(,)?) => {
        expect_value!(
            expect_value!($val, $msg).as_sequence(), "{} to be a list", $msg)
    };
    ($val:expr, $fmt:expr, $($arg:tt)*) => {
        expect_value!(
            expect_value!($val, $fmt, $($arg)*).as_sequence(),
            "{} to be a list", format!($fmt, $($arg)*))

    };
}

pub fn read_schema(root: impl AsRef<Path>) -> miette::Result<TestRoot> {
    read_dir(&root).map(TestRoot)
}

fn read_dir(path: impl AsRef<Path>) -> miette::Result<Vec<TestEntry>> {
    let mut entries = vec![];
    for e in fs::read_dir(path).into_diagnostic()? {
        if let Some(entry) = read_entry(e.into_diagnostic()?)? {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn read_entry(entry: DirEntry) -> miette::Result<Option<TestEntry>> {
    let file_type = entry.file_type().into_diagnostic()?;
    let name = entry.file_name().into_string().unwrap();

    let ion_file_extension = OsStr::new("ion");
    let result = if file_type.is_file() {
        if entry.path().extension() == Some(ion_file_extension) {
            Some(TestEntry::Doc(TestFile {
                file_name: name,
                contents: read_test_doc(entry.path())?,
            }))
        } else {
            None
        }
    } else if file_type.is_dir() {
        Some(TestEntry::Dir(TestDir {
            dir_name: name,
            contents: read_dir(entry.path())?,
        }))
    } else {
        unreachable!()
    };

    Ok(result)
}

fn read_test_doc(path: impl AsRef<Path>) -> miette::Result<PartiQLTestDocument> {
    let buf = fs::read(path).into_diagnostic()?;
    buf.as_slice().try_into()
}

impl TryFrom<&[u8]> for PartiQLTestDocument {
    type Error = miette::Report;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = ReaderBuilder::new().build(value).into_diagnostic()?;
        let elements = reader.read_all_elements().into_diagnostic()?;
        elements.as_slice().try_into()
    }
}

impl TryFrom<&[Element]> for PartiQLTestDocument {
    type Error = miette::Report;

    fn try_from(value: &[Element]) -> Result<Self, Self::Error> {
        let inner: Result<Vec<_>, _> = value.iter().map(|e| e.try_into()).collect();
        Ok(PartiQLTestDocument(inner?))
    }
}

impl TryFrom<&Element> for TestVariant {
    type Error = miette::Report;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let typ = value.ion_type();
        if value.is_null() {
            Err(miette!("value was null when parsing TestVariant"))
        } else {
            match typ {
                IonType::List => {
                    let contents = value.try_into()?;
                    Ok(TestVariant::Namespace(contents))
                }
                IonType::Struct => {
                    let strct = value.as_struct().unwrap();
                    if value.has_annotation("envs") {
                        Ok(TestVariant::Environments(strct.try_into()?))
                    } else if value.has_annotation("equiv_class") {
                        Ok(TestVariant::EquivalenceClass(strct.try_into()?))
                    } else {
                        Ok(TestVariant::TestCase(strct.try_into()?))
                    }
                }
                _ => Err(miette!(
                    "unexpected type {:?} when parsing TestVariant",
                    typ
                )),
            }
        }
    }
}

impl TryFrom<&Element> for Namespace {
    type Error = miette::Report;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let name = if let Some(sym) = value.annotations().next() {
            sym.text().unwrap().to_string()
        } else {
            return Err(miette!(
                "expected annotation for name when parsing Namespace"
            ));
        };
        let list = expect_list!(Some(value), "Namespace");
        let contents: Result<Vec<_>, _> = list.elements().map(|e| e.try_into()).collect();
        Ok(Namespace {
            name,
            contents: contents?,
        })
    }
}

impl TryFrom<&Struct> for TestCase {
    type Error = miette::Report;

    fn try_from(value: &Struct) -> Result<Self, Self::Error> {
        let name = expect_str!(value.get("name"), "TestCase name").into();

        let statement = expect_value!(value.get("statement"), "TestCase statement");
        let statement = match statement.ion_type() {
            IonType::Symbol => TestStatement::EquivalenceClass(
                expect_str!(value.get("statement"), "TestCase statement").to_string(),
            ),
            IonType::String => TestStatement::Statement(
                expect_str!(value.get("statement"), "TestCase statement").to_string(),
            ),
            _ => {
                return Err(miette!(
                    "unexpected type {:?} when parsing TestCase statement",
                    statement.ion_type()
                ))
            }
        };

        let env = if let Some(v) = value.get("env") {
            Some(expect_struct!(Some(v), "TestCase envs").clone())
        } else {
            None
        };
        let assert_elt = expect_value!(value.get("assert"), "TestCase assert");
        let assert = match assert_elt.ion_type() {
            IonType::List => {
                let list = assert_elt.as_sequence().unwrap();
                let asserts: Result<Vec<_>, _> = list.elements().map(|e| e.try_into()).collect();
                asserts?
            }
            IonType::Struct => {
                vec![assert_elt.try_into()?]
            }
            _ => {
                return Err(miette!(
                    "unexpected type {:?} when parsing TestCase assert",
                    assert_elt.ion_type()
                ))
            }
        };

        Ok(TestCase {
            name,
            statement,
            env,
            assert,
        })
    }
}

impl TryFrom<&Element> for Assertion {
    type Error = miette::Report;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let val_struct = expect_struct!(Some(value), "Assertion");
        let result = expect_str!(val_struct.get("result"), "Assertion result field").to_string();

        let assertion = match result.as_str() {
            "SyntaxSuccess" => SyntaxSuccessAssertion { result }.into(),
            "SyntaxFail" => SyntaxFailAssertion { result }.into(),
            "StaticAnalysisFail" => StaticAnalysisFailAssertion { result }.into(),
            "EvaluationSuccess" => {
                let eval_mode = expect_value!(
                    val_struct.get("evalMode"),
                    "EvaluationSuccessAssertion evalMode"
                )
                .try_into()?;
                let output = expect_value!(
                    val_struct.get("output"),
                    "EvaluationSuccessAssertion output"
                )
                .clone();
                EvaluationSuccessAssertion {
                    result,
                    output,
                    eval_mode,
                }
                .into()
            }
            "EvaluationFail" => {
                let eval_mode = expect_value!(
                    val_struct.get("evalMode"),
                    "EvaluationFailAssertion evalMode"
                )
                .try_into()?;
                EvaluationFailAssertion { result, eval_mode }.into()
            }
            _ => return Err(miette!("unknown assertion type {:?}", result)),
        };
        Ok(assertion)
    }
}

impl TryFrom<&Element> for EvaluationModeList {
    type Error = miette::Report;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let eval_mode = |value: &Element| {
            let eval_mode = expect_str!(Some(value), "Eval Mode");
            match eval_mode {
                "EvalModeError" => Ok(EvaluationMode::EvalModeError),
                "EvalModeCoerce" => Ok(EvaluationMode::EvalModeCoerce),
                _ => Err(miette!("unknown Eval Mode type {:?}", eval_mode)),
            }
        };

        match value.ion_type() {
            IonType::Symbol | IonType::String => Ok(eval_mode(value)?.into()),
            IonType::List => {
                let list = value.as_sequence().unwrap();
                let eval_modes: Result<Vec<_>, _> = list.elements().map(|e| eval_mode(e)).collect();
                Ok(eval_modes?.into())
            }
            _ => Err(miette!(
                "unexpected type {:?} when parsing EvaluationModeSymbolOrList",
                value.ion_type()
            )),
        }
    }
}

impl TryFrom<&Struct> for Environments {
    type Error = miette::Report;

    fn try_from(value: &Struct) -> Result<Self, Self::Error> {
        Ok(Environments {
            envs: value.clone(),
        })
    }
}

impl TryFrom<&Struct> for EquivalenceClass {
    type Error = miette::Report;

    fn try_from(value: &Struct) -> Result<Self, Self::Error> {
        let id = expect_str!(value.get("id"), "EquivalenceClass id").into();
        let stmt_list = expect_list!(value.get("statements"), "EquivalenceClass statements");

        let mut statements = vec![];
        for stmt in stmt_list.elements() {
            statements.push(expect_str!(Some(stmt), "EquivalenceClass statement").into())
        }

        Ok(EquivalenceClass { id, statements })
    }
}

#[cfg(test)]
mod test {}
