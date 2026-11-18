use miette::Diagnostic;
use partiql_types::{
    AnyOf, ArrayType, BagType, IntervalQualifier, IntervalType, MapType, PartiqlShape,
    ShapeResultError, Static, StaticType, StructType, TypeParams,
};
use std::fmt::{Display, Formatter};
use std::string::ToString;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("ShapeEncodingError Error")]
#[non_exhaustive]
pub enum ShapeEncodingError {
    #[error("UnsupportedEncoding: {0}")]
    UnsupportedEncoding(String),
    #[error("DateTimeEncodingError e: {0}")]
    DateTimeEncodingError(#[from] time::error::Format),
    #[error("Invalid Simulation Configuration e: {0}")]
    InvalidSimConfigError(String),
    #[error("Unexpected Type: {0}")]
    UnexpectedType(#[from] ShapeResultError),
}

/// Result of attempts to encode as data definition language (DDL_.
pub type ShapeDdlEncodeResult<T> = Result<T, ShapeEncodingError>;

const PARTIQL_DATA_TYPE_SYNTAX: &str = "partiql_datatype_syntax";

/// Represents s PartiQL DDL Format
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DdlFormat {
    Compact,
    Pretty,
}

/// Represents s PartiQL DDL Syntax
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlSyntax {
    name: String,
    version: DdlSyntaxVersion,
}

impl Display for DdlSyntax {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}.{}",
            self.name, self.version.major, self.version.minor
        )
    }
}

impl DdlSyntax {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn version(&self) -> String {
        format!("{}.{}", self.version.major, self.version.minor)
    }
}

/// Represents a PartiQL DDL Syntax Version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlSyntaxVersion {
    major: u8,
    minor: u8,
}

/// Represents a PartiQL DDL Encoder
pub trait PartiqlDdlEncoder {
    type Output;

    fn ddl(&self, ty: &PartiqlShape) -> ShapeDdlEncodeResult<Self::Output>;

    fn syntax(&self) -> DdlSyntax;
}

/// Represents a PartiQL Basic DDL Encoder
#[derive(Debug, Clone)]
pub struct PartiqlBasicDdlEncoder {
    format: DdlFormat,
    syntax: DdlSyntax,
}

impl PartiqlBasicDdlEncoder {
    pub fn new(format: DdlFormat) -> Self {
        PartiqlBasicDdlEncoder {
            format,
            syntax: DdlSyntax {
                name: PARTIQL_DATA_TYPE_SYNTAX.to_string(),
                version: DdlSyntaxVersion { major: 0, minor: 1 },
            },
        }
    }

    fn write_shape(&self, shape: &PartiqlShape) -> ShapeDdlEncodeResult<String> {
        Ok(match shape {
            PartiqlShape::AnyOf(any_of) => self.write_union(any_of)?,
            PartiqlShape::Static(stype) => self.write_attribute(stype)?,
            _ => Err(ShapeEncodingError::UnsupportedEncoding(format!(
                "`{shape}` is unsupported"
            )))?,
        })
    }

    fn write_attribute(&self, ty: &StaticType) -> ShapeDdlEncodeResult<String> {
        let mut out = String::new();

        match ty.ty() {
            Static::Int => out.push_str("INT"),
            Static::Int8 => out.push_str("TINYINT"),
            Static::Int16 => out.push_str("SMALLINT"),
            Static::Int32 => out.push_str("INTEGER"),
            Static::Int64 => out.push_str("INT8"),
            Static::Bool => out.push_str("BOOL"),
            Static::Decimal => out.push_str("DECIMAL"),
            Static::DecimalP(p, s) => {
                out.push_str(&with_precision_scale("DECIMAL", *p, *s, ty.params()))
            }
            Static::DateTime => out.push_str("TIMESTAMP"),
            Static::Float32 => out.push_str("REAL"),
            Static::Float64 => out.push_str("DOUBLE"),
            Static::String => out.push_str("VARCHAR"),
            Static::StringFixed(n) => out.push_str(&with_length("CHAR", *n, ty.params())),
            Static::StringVarying(n) => out.push_str(&with_length("VARCHAR", *n, ty.params())),
            Static::NumericP(p, s) => {
                out.push_str(&with_precision_scale("NUMERIC", *p, *s, ty.params()))
            }
            Static::Blob(n) => out.push_str(&with_length("BLOB", *n, ty.params())),
            Static::Clob(n) => out.push_str(&with_length("CLOB", *n, ty.params())),
            Static::Date => out.push_str("DATE"),
            Static::Time(p) => out.push_str(&with_precision("TIME", *p, ty.params())),
            Static::TimeTz(p) => {
                out.push_str(&with_precision("TIME", *p, ty.params()));
                out.push_str(" WITH TIME ZONE");
            }
            Static::Timestamp(p) => out.push_str(&with_precision("TIMESTAMP", *p, ty.params())),
            Static::TimestampTz(p) => {
                out.push_str(&with_precision("TIMESTAMP", *p, ty.params()));
                out.push_str(" WITH TIME ZONE");
            }
            Static::Interval(i) => out.push_str(&write_interval(i, ty.params())?),
            Static::Variant(encoding) => {
                out.push_str(&format!("VARIANT('{}')", checked_text(encoding, '\'')?))
            }
            Static::Struct(s) => out.push_str(&self.write_struct(s)?),
            Static::Bag(b) => out.push_str(&self.write_type_bag(b)?),
            Static::Array(a) => out.push_str(&self.write_type_array(a)?),
            Static::Map(m) => out.push_str(&self.write_type_map(m)?),
            // Also `Null` and `Missing`, which have no DDL form, and `Graph`; plus the
            // non-exhaustive catch-all.
            _ => Err(ShapeEncodingError::UnsupportedEncoding(format!(
                "`{ty}` is unsupported"
            )))?,
        }

        if !ty.is_nullable() {
            out.push_str(" NOT NULL")
        }

        Ok(out)
    }

    fn write_type_bag(&self, type_bag: &BagType) -> ShapeDdlEncodeResult<String> {
        Ok(format!(
            "BAG<{}>",
            self.write_shape(type_bag.element_type())?
        ))
    }

    fn write_type_array(&self, arr: &ArrayType) -> ShapeDdlEncodeResult<String> {
        Ok(format!("ARRAY<{}>", self.write_shape(arr.element_type())?))
    }

    fn write_type_map(&self, map: &MapType) -> ShapeDdlEncodeResult<String> {
        Ok(format!(
            "MAP<{},{}>",
            self.write_shape(map.key_type())?,
            self.write_shape(map.value_type())?
        ))
    }

    // Duplicate names are allowed in a nested struct type (a row may have them); only table
    // columns must be unique (see `ddl`).
    fn write_struct(&self, strct: &StructType) -> ShapeDdlEncodeResult<String> {
        let mut struct_out = String::from("STRUCT<");

        let mut fields = strct.fields().peekable();
        while let Some(field) = fields.next() {
            struct_out.push_str(&format!("{}: ", quoted_identifier(field.name())?));
            struct_out.push_str(&self.write_shape(field.ty())?);
            if fields.peek().is_some() {
                struct_out.push(',');
            }
        }

        struct_out.push('>');
        Ok(struct_out)
    }

    fn write_union(&self, any_of: &AnyOf) -> ShapeDdlEncodeResult<String> {
        let mut union_out = String::from("UNION<");
        let mut types = any_of.types().peekable();
        while let Some(ty) = types.next() {
            union_out.push_str(&self.write_shape(ty)?);
            if types.peek().is_some() {
                union_out.push(',');
            }
        }
        union_out.push('>');
        Ok(union_out)
    }

    fn write_line(&self) -> ShapeDdlEncodeResult<String> {
        Ok(if self.format == DdlFormat::Pretty {
            "\n".to_string()
        } else {
            "".to_string()
        })
    }
}

/// A delimited identifier: `name` in double quotes.
fn quoted_identifier(name: &str) -> ShapeDdlEncodeResult<String> {
    Ok(format!("\"{}\"", checked_text(name, '"')?))
}

/// `text` unchanged, or an error if it holds `quote`, a backslash or a control character.
///
/// SQL escapes a quote by doubling it while PartiQL's lexer uses backslash escapes, so no
/// escaping reads back the same under both; such text is rejected rather than risk the output
/// being read as different (e.g. extra) columns.
fn checked_text(text: &str, quote: char) -> ShapeDdlEncodeResult<&str> {
    if text
        .chars()
        .any(|c| c == quote || c == '\\' || c.is_control())
    {
        Err(ShapeEncodingError::UnsupportedEncoding(format!(
            "{text:?} contains a quote, backslash or control character"
        )))
    } else {
        Ok(text)
    }
}

/// `NAME(n)`, or `NAME` when the length was not specified.
fn with_length(name: &str, n: usize, params: &TypeParams) -> String {
    if params.unspecified_length {
        name.to_string()
    } else {
        format!("{name}({n})")
    }
}

/// `NAME(p)`, or `NAME` when the precision was not specified.
fn with_precision(name: &str, p: usize, params: &TypeParams) -> String {
    if params.unspecified_precision {
        name.to_string()
    } else {
        format!("{name}({p})")
    }
}

/// `NAME(p, s)`, `NAME(p)` when the scale was not specified, or `NAME` when neither was. A
/// specified scale cannot be written without a precision, so then both are written.
fn with_precision_scale(name: &str, p: usize, s: usize, params: &TypeParams) -> String {
    match (params.unspecified_precision, params.unspecified_scale) {
        (true, true) => name.to_string(),
        (false, true) => format!("{name}({p})"),
        (_, false) => format!("{name}({p}, {s})"),
    }
}

/// `INTERVAL <qualifier>` in SQL syntax, e.g. `INTERVAL DAY(2) TO SECOND(6)`.
fn write_interval(i: &IntervalType, params: &TypeParams) -> ShapeDdlEncodeResult<String> {
    let lead = |name: &str| {
        if params.unspecified_precision {
            name.to_string()
        } else {
            format!("{name}({})", i.precision())
        }
    };
    let fraction = || {
        if params.unspecified_fractional_precision {
            "SECOND".to_string()
        } else {
            format!("SECOND({})", i.fractional_precision())
        }
    };
    let qualifier = match i.qualifier() {
        IntervalQualifier::Year => lead("YEAR"),
        IntervalQualifier::Month => lead("MONTH"),
        IntervalQualifier::YearToMonth => format!("{} TO MONTH", lead("YEAR")),
        IntervalQualifier::Day => lead("DAY"),
        IntervalQualifier::Hour => lead("HOUR"),
        IntervalQualifier::Minute => lead("MINUTE"),
        IntervalQualifier::Second => match (
            params.unspecified_precision,
            params.unspecified_fractional_precision,
        ) {
            // A specified fractional precision cannot be written without the leading one.
            (true, true) => "SECOND".to_string(),
            (false, true) => format!("SECOND({})", i.precision()),
            (_, false) => format!("SECOND({}, {})", i.precision(), i.fractional_precision()),
        },
        IntervalQualifier::DayToHour => format!("{} TO HOUR", lead("DAY")),
        IntervalQualifier::DayToMinute => format!("{} TO MINUTE", lead("DAY")),
        IntervalQualifier::DayToSecond => format!("{} TO {}", lead("DAY"), fraction()),
        IntervalQualifier::HourToMinute => format!("{} TO MINUTE", lead("HOUR")),
        IntervalQualifier::HourToSecond => format!("{} TO {}", lead("HOUR"), fraction()),
        IntervalQualifier::MinuteToSecond => format!("{} TO {}", lead("MINUTE"), fraction()),
        // non-exhaustive catch-all
        q => Err(ShapeEncodingError::UnsupportedEncoding(format!(
            "interval qualifier `{q:?}` is unsupported"
        )))?,
    };
    Ok(format!("INTERVAL {qualifier}"))
}

impl PartiqlDdlEncoder for PartiqlBasicDdlEncoder {
    type Output = String;

    fn ddl(&self, ty: &PartiqlShape) -> ShapeDdlEncodeResult<String> {
        let mut output = String::new();
        let ty = ty.expect_static()?;

        if let Static::Bag(type_bag) = ty.ty() {
            let s = type_bag.element_type().expect_struct()?;
            let mut names = std::collections::HashSet::new();
            let mut fields = s.fields().peekable();
            while let Some(field) = fields.next() {
                if !names.insert(field.name()) {
                    Err(ShapeEncodingError::UnsupportedEncoding(format!(
                        "duplicate column name {:?}",
                        field.name()
                    )))?;
                }
                output.push_str(&format!("{} ", quoted_identifier(field.name())?));

                if field.is_optional() {
                    output.push_str("OPTIONAL ");
                }

                output.push_str(&self.write_shape(field.ty())?);
                if fields.peek().is_some() {
                    output.push(',');
                    output.push_str(&self.write_line()?);
                }
            }
            Ok(output)
        } else {
            Err(ShapeEncodingError::UnsupportedEncoding(format!(
                "Unsupported top level type {ty:?}"
            )))
        }
    }

    fn syntax(&self) -> DdlSyntax {
        self.syntax.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexSet;
    use partiql_types::{
        struct_fields, type_array, type_bag, type_float64, type_int8, type_string, type_struct,
        PartiqlShapeBuilder, ShapeBuilderExtensions, StructConstraint,
    };

    #[test]
    fn ddl_test() {
        let mut bld = PartiqlShapeBuilder::default();
        let nested_attrs = struct_fields![
            (
                "a",
                [
                    bld.new_static(Static::DecimalP(5, 4)),
                    bld.new_static(Static::Int8),
                ]
                .into_any_of(&mut bld)
            ),
            ("b", type_string![bld].into_array(&mut bld)),
            ("c", type_float64!(bld)),
        ];
        let details = type_struct![bld, IndexSet::from([nested_attrs])];

        let fields = struct_fields![
            ("employee_id", type_int8![bld]),
            ("full_name", type_string![bld]),
            ("salary", bld.new_static(Static::DecimalP(8, 2))),
            ("details", details),
            ("dependents", type_array![bld, type_string![bld]])
        ];
        let ty = type_bag![
            bld,
            type_struct![bld, IndexSet::from([fields, StructConstraint::Open(false)])]
        ];

        let expected_compact = r#""employee_id" TINYINT,"full_name" VARCHAR,"salary" DECIMAL(8, 2),"details" STRUCT<"a": UNION<DECIMAL(5, 4),TINYINT>,"b": ARRAY<VARCHAR>,"c": DOUBLE>,"dependents" ARRAY<VARCHAR>"#;
        let expected_pretty = r#""employee_id" TINYINT,
"full_name" VARCHAR,
"salary" DECIMAL(8, 2),
"details" STRUCT<"a": UNION<DECIMAL(5, 4),TINYINT>,"b": ARRAY<VARCHAR>,"c": DOUBLE>,
"dependents" ARRAY<VARCHAR>"#;

        let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
        assert_eq!(ddl_compact.ddl(&ty).expect("write shape"), expected_compact);

        dbg!(&expected_compact);

        let ddl_pretty = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
        assert_eq!(ddl_pretty.ddl(&ty).expect("write shape"), expected_pretty);

        dbg!(&expected_pretty);
    }
}
