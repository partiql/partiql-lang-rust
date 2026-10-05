use std::fs;
use std::path::{Path, PathBuf};

use ion_rs::element::Element;
use ion_rs::IonType;

const EXPECT_ERROR_FIELDS: &[&str] = &["error"];
const EXPECT_SUCCESS_FIELDS: &[&str] = &["rows", "affected_rows", "created_table"];

#[derive(Debug)]
pub struct StepDef {
    pub sql: String,
    pub expect: Option<Element>,
    /// `skip::` annotation on the top-level Ion value marks a step that the
    /// runner must load but must NOT execute. Lets fixture authors keep
    /// known-broken cases in-tree while the compiler is still catching up.
    pub skip: bool,
}

#[derive(Debug)]
pub enum LoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Ion {
        path: PathBuf,
        source: ion_rs::IonError,
    },
    NotAStruct {
        path: PathBuf,
        index: usize,
        actual: IonType,
    },
    MissingSql {
        path: PathBuf,
        index: usize,
    },
    /// `field` had the wrong Ion type; e.g. `sql` was an int, `expect` was a list.
    TypeMismatch {
        path: PathBuf,
        index: usize,
        field: &'static str,
        expected: &'static str,
        actual: IonType,
    },
    /// Catches typos like `expects:` or `row:` that would silently no-op the step.
    UnknownField {
        path: PathBuf,
        index: usize,
        location: &'static str,
        name: String,
    },
    ConflictingExpect {
        path: PathBuf,
        index: usize,
    },
    /// A `.test.ion` file with zero top-level structs would pass with nothing checked.
    EmptyFile {
        path: PathBuf,
    },
    /// Ion allows repeat field names; `Struct::get` returns only one value.
    DuplicateField {
        path: PathBuf,
        index: usize,
        location: &'static str,
        name: String,
    },
    /// `expect: { error: "" }` is a fixture-author bug — `msg.contains("")`
    /// is trivially true, so any failure would satisfy the assertion.
    EmptyErrorSubstring {
        path: PathBuf,
        index: usize,
    },
    /// Only bare `skip::` is recognized; chained annotations like
    /// `skip::foo::"..."` or an unknown annotation would silently no-op.
    UnknownAnnotation {
        path: PathBuf,
        index: usize,
        annotations: Vec<String>,
    },
    /// `skip::` may only wrap a string or a struct.
    SkipInvalidPayload {
        path: PathBuf,
        index: usize,
        actual: IonType,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LoadError::*;
        match self {
            Io { path, source } => write!(f, "{}: {}", path.display(), source),
            Ion { path, source } => write!(f, "{}: ion parse error: {}", path.display(), source),
            NotAStruct {
                path,
                index,
                actual,
            } => write!(
                f,
                "{}: top-level value #{} must be a struct, was {}",
                path.display(),
                index,
                actual
            ),
            MissingSql { path, index } => write!(
                f,
                "{}: step #{} missing required field `sql`",
                path.display(),
                index
            ),
            TypeMismatch {
                path,
                index,
                field,
                expected,
                actual,
            } => write!(
                f,
                "{}: step #{} field `{}` must be {}, was {}",
                path.display(),
                index,
                field,
                expected,
                actual
            ),
            UnknownField {
                path,
                index,
                location,
                name,
            } => write!(
                f,
                "{}: step #{} unknown field `{}` on {}",
                path.display(),
                index,
                name,
                location
            ),
            ConflictingExpect { path, index } => write!(
                f,
                "{}: step #{} `expect` may contain either `error` or a success-shape key, not both",
                path.display(),
                index,
            ),
            EmptyFile { path } => {
                write!(f, "{}: file contains no top-level structs", path.display())
            }
            DuplicateField {
                path,
                index,
                location,
                name,
            } => write!(
                f,
                "{}: step #{} field `{}` appears more than once on {}",
                path.display(),
                index,
                name,
                location
            ),
            EmptyErrorSubstring { path, index } => write!(
                f,
                "{}: step #{} `expect.error` must be a non-empty substring",
                path.display(),
                index
            ),
            UnknownAnnotation {
                path,
                index,
                annotations,
            } => write!(
                f,
                "{}: top-level value #{} has unsupported annotations {:?}; \
                 only bare `skip::` is recognized",
                path.display(),
                index,
                annotations,
            ),
            SkipInvalidPayload {
                path,
                index,
                actual,
            } => write!(
                f,
                "{}: top-level value #{} `skip::` must wrap a string or a struct, was {}",
                path.display(),
                index,
                actual
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// Load every top-level Ion value in `path`; each must be a struct with at
/// least `sql: <string>` and optionally `expect: <struct>`. Unknown fields at
/// either level are rejected so a typo can't silently disable a check.
pub fn load_test_case(path: &Path) -> Result<Vec<StepDef>, LoadError> {
    let bytes = fs::read(path).map_err(|e| LoadError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let elements = Element::read_all(&bytes).map_err(|e| LoadError::Ion {
        path: path.to_path_buf(),
        source: e,
    })?;
    if elements.is_empty() {
        return Err(LoadError::EmptyFile {
            path: path.to_path_buf(),
        });
    }
    let mut steps = Vec::with_capacity(elements.len());
    for (index, el) in elements.into_iter().enumerate() {
        steps.push(parse_step(path, index, &el)?);
    }
    Ok(steps)
}

fn parse_step(path: &Path, index: usize, el: &Element) -> Result<StepDef, LoadError> {
    // Only `skip::` (alone) is recognized. Anything else — including chained
    // annotations like `skip::foo::"..."` — must be a hard load error so
    // typos can't silently disable a step. An annotation whose symbol has no
    // resolvable text (e.g. `$0::`) must NOT be silently dropped — dropping
    // it would let an unrecognized annotation slip past the check below.
    let mut annotations: Vec<String> = Vec::new();
    for sym in el.annotations().iter() {
        match sym.text() {
            Some(t) => annotations.push(t.to_string()),
            None => {
                return Err(LoadError::UnknownAnnotation {
                    path: path.to_path_buf(),
                    index,
                    annotations: vec!["<unresolved symbol>".to_string()],
                });
            }
        }
    }
    let skip = match annotations.as_slice() {
        [] => false,
        [only] if only == "skip" => true,
        _ => {
            return Err(LoadError::UnknownAnnotation {
                path: path.to_path_buf(),
                index,
                annotations,
            });
        }
    };

    if skip && el.ion_type() == IonType::String {
        // `skip::"INSERT INTO ..."` — a skipped bare SQL statement with no
        // assertion. The runner will not execute it.
        let sql = el
            .as_string()
            .expect("annotated string element must expose as_string")
            .to_owned();
        return Ok(StepDef {
            sql,
            expect: None,
            skip: true,
        });
    }

    if skip && el.ion_type() != IonType::Struct {
        return Err(LoadError::SkipInvalidPayload {
            path: path.to_path_buf(),
            index,
            actual: el.ion_type(),
        });
    }

    let s = el.as_struct().ok_or_else(|| LoadError::NotAStruct {
        path: path.to_path_buf(),
        index,
        actual: el.ion_type(),
    })?;

    let mut sql_count = 0;
    let mut expect_count = 0;
    for (name, _) in s.iter() {
        let text = name.text().unwrap_or("");
        match text {
            "sql" => sql_count += 1,
            "expect" => expect_count += 1,
            _ => {
                return Err(LoadError::UnknownField {
                    path: path.to_path_buf(),
                    index,
                    location: "step",
                    name: text.to_string(),
                });
            }
        }
    }
    if sql_count > 1 {
        return Err(LoadError::DuplicateField {
            path: path.to_path_buf(),
            index,
            location: "step",
            name: "sql".to_string(),
        });
    }
    if expect_count > 1 {
        return Err(LoadError::DuplicateField {
            path: path.to_path_buf(),
            index,
            location: "step",
            name: "expect".to_string(),
        });
    }

    let sql_el = s.get("sql").ok_or(LoadError::MissingSql {
        path: path.to_path_buf(),
        index,
    })?;
    let sql = sql_el
        .as_string()
        .ok_or_else(|| LoadError::TypeMismatch {
            path: path.to_path_buf(),
            index,
            field: "sql",
            expected: "string",
            actual: sql_el.ion_type(),
        })?
        .to_owned();

    let expect = match s.get("expect") {
        None => None,
        Some(e) if e.ion_type() == IonType::Struct => {
            validate_expect(path, index, e)?;
            Some(e.clone())
        }
        Some(e) => {
            return Err(LoadError::TypeMismatch {
                path: path.to_path_buf(),
                index,
                field: "expect",
                expected: "struct",
                actual: e.ion_type(),
            })
        }
    };

    Ok(StepDef { sql, expect, skip })
}

fn validate_expect(path: &Path, index: usize, expect: &Element) -> Result<(), LoadError> {
    let s = expect.as_struct().expect("caller guarantees struct");
    let mut error_count = 0u32;
    let mut success_count = 0u32;
    let mut seen_names: Vec<String> = Vec::new();
    for (name, value) in s.iter() {
        let text = name.text().unwrap_or("");
        if EXPECT_ERROR_FIELDS.contains(&text) {
            error_count += 1;
            if value.as_string() == Some("") {
                return Err(LoadError::EmptyErrorSubstring {
                    path: path.to_path_buf(),
                    index,
                });
            }
        } else if EXPECT_SUCCESS_FIELDS.contains(&text) {
            success_count += 1;
        } else {
            return Err(LoadError::UnknownField {
                path: path.to_path_buf(),
                index,
                location: "expect",
                name: text.to_string(),
            });
        }
        if seen_names.iter().any(|n| n == text) {
            return Err(LoadError::DuplicateField {
                path: path.to_path_buf(),
                index,
                location: "expect",
                name: text.to_string(),
            });
        }
        seen_names.push(text.to_string());
    }
    if error_count > 0 && success_count > 0 {
        return Err(LoadError::ConflictingExpect {
            path: path.to_path_buf(),
            index,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every malformed-fixture rejection lives in one table so a new variant
    /// costs one row instead of a whole test. Each row: filename, fixture
    /// body, predicate over the returned `LoadError`.
    #[test]
    fn rejects_malformed_fixtures() {
        #[allow(clippy::type_complexity)]
        const CASES: &[(&str, &str, fn(&LoadError) -> bool)] = &[
            ("bad.test.ion", r#" 42 "#, |e| {
                matches!(e, LoadError::NotAStruct { .. })
            }),
            ("bad2.test.ion", r#" { expect: { rows: [] } } "#, |e| {
                matches!(e, LoadError::MissingSql { .. })
            }),
            (
                "typo.test.ion",
                r#" { sql: "SELECT * FROM t", expects: { rows: $bag::[] } } "#,
                |e| {
                    matches!(
                        e,
                        LoadError::UnknownField {
                            location: "step",
                            ..
                        }
                    )
                },
            ),
            (
                "typo2.test.ion",
                r#" { sql: "SELECT * FROM t", expect: { row: $bag::[] } } "#,
                |e| {
                    matches!(
                        e,
                        LoadError::UnknownField {
                            location: "expect",
                            ..
                        }
                    )
                },
            ),
            (
                "mixed.test.ion",
                r#" { sql: "SELECT * FROM t", expect: { error: "boom", rows: $bag::[] } } "#,
                |e| matches!(e, LoadError::ConflictingExpect { .. }),
            ),
            (
                "empty_file.test.ion",
                "// just a comment, no cases\n",
                |e| matches!(e, LoadError::EmptyFile { .. }),
            ),
            (
                "dup_sql.test.ion",
                r#" { sql: "SELECT 1", sql: "SELECT 2", expect: { rows: [1] } } "#,
                |e| {
                    matches!(
                        e,
                        LoadError::DuplicateField {
                            location: "step",
                            ..
                        }
                    )
                },
            ),
            (
                "empty_err.test.ion",
                r#" { sql: "SELECT * FROM ghost", expect: { error: "" } } "#,
                |e| matches!(e, LoadError::EmptyErrorSubstring { .. }),
            ),
            (
                "dup_rows.test.ion",
                r#" { sql: "SELECT * FROM t", expect: { rows: $bag::[], rows: $bag::[] } } "#,
                |e| {
                    matches!(
                        e,
                        LoadError::DuplicateField {
                            location: "expect",
                            ..
                        }
                    )
                },
            ),
            (
                "sql_int.test.ion",
                r#" { sql: 42, expect: { rows: $bag::[] } } "#,
                |e| matches!(e, LoadError::TypeMismatch { .. }),
            ),
            ("bad_annot.test.ion", "wrong::\"CREATE TABLE t\"\n", |e| {
                matches!(e, LoadError::UnknownAnnotation { .. })
            }),
            ("skip_int.test.ion", "skip::42\n", |e| {
                matches!(e, LoadError::SkipInvalidPayload { .. })
            }),
        ];

        for (filename, body, predicate) in CASES {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join(filename);
            std::fs::write(&path, body).unwrap();
            let err = load_test_case(&path).unwrap_err();
            assert!(
                predicate(&err),
                "case {filename}: predicate rejected LoadError: {err:?}"
            );
        }
    }

    #[test]
    fn parses_skip_annotated_string_as_skipped_bare_sql() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skip_str.test.ion");
        std::fs::write(&path, r#" skip::"INSERT INTO t VALUES (1)" "#).unwrap();
        let steps = load_test_case(&path).unwrap();
        assert_eq!(steps.len(), 1);
        assert!(steps[0].skip, "expected skip=true, got {:?}", steps[0]);
        assert_eq!(steps[0].sql, "INSERT INTO t VALUES (1)");
        assert!(steps[0].expect.is_none());
    }

    #[test]
    fn parses_skip_annotated_struct_as_skipped_step() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skip_struct.test.ion");
        std::fs::write(
            &path,
            r#" skip::{ sql: "SELECT * FROM t", expect: { rows: $bag::[] } } "#,
        )
        .unwrap();
        let steps = load_test_case(&path).unwrap();
        assert_eq!(steps.len(), 1);
        assert!(steps[0].skip, "expected skip=true, got {:?}", steps[0]);
        assert_eq!(steps[0].sql, "SELECT * FROM t");
        assert!(steps[0].expect.is_some(), "expect should still be parsed");
    }

    #[test]
    fn bare_sql_step_supported_when_skip_annotated() {
        // An un-annotated top-level string must be REJECTED. Only skip::"..."
        // may be a bare SQL step; regular steps must still be structs.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bare.test.ion");
        std::fs::write(&path, r#" "CREATE TABLE t" "#).unwrap();
        let err = load_test_case(&path).unwrap_err();
        assert!(matches!(err, LoadError::NotAStruct { .. }), "got: {err:?}");
    }
}
