use miette::Diagnostic;
use partiql_types::{
    AnyOf, ArrayType, BagType, PartiqlShape, ShapeResultError, Static, StaticType, StructType,
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
            Static::DecimalP(p, s) => out.push_str(&format!("DECIMAL({p}, {s})")),
            Static::DateTime => out.push_str("TIMESTAMP"),
            Static::Float32 => out.push_str("REAL"),
            Static::Float64 => out.push_str("DOUBLE"),
            Static::String => out.push_str("VARCHAR"),
            Static::Struct(s) => out.push_str(&self.write_struct(s)?),
            Static::Bag(b) => out.push_str(&self.write_type_bag(b)?),
            Static::Array(a) => out.push_str(&self.write_type_array(a)?),
            // non-exhaustive catch-all
            _ => todo!("handle type for {}", ty),
        }

        if !ty.is_nullable() {
            out.push_str(" NOT NULL")
        }

        Ok(out)
    }

    fn write_type_bag(&self, type_bag: &BagType) -> ShapeDdlEncodeResult<String> {
        Ok(format!(
            "type_bag<{}>",
            self.write_shape(type_bag.element_type())?
        ))
    }

    fn write_type_array(&self, arr: &ArrayType) -> ShapeDdlEncodeResult<String> {
        Ok(format!(
            "type_array<{}>",
            self.write_shape(arr.element_type())?
        ))
    }

    fn write_struct(&self, strct: &StructType) -> ShapeDdlEncodeResult<String> {
        let mut struct_out = String::from("STRUCT<");

        let mut fields = strct.fields().peekable();
        while let Some(field) = fields.next() {
            struct_out.push_str(&format!("\"{}\": ", field.name()));
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

impl PartiqlDdlEncoder for PartiqlBasicDdlEncoder {
    type Output = String;

    fn ddl(&self, ty: &PartiqlShape) -> ShapeDdlEncodeResult<String> {
        let mut output = String::new();
        let ty = ty.expect_static()?;

        if let Static::Bag(type_bag) = ty.ty() {
            let s = type_bag.element_type().expect_struct()?;
            let mut fields = s.fields().peekable();
            while let Some(field) = fields.next() {
                output.push_str(&format!("\"{}\" ", field.name()));

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
                "Unsupported top level type {:?}",
                ty
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
        PartiqlShapeBuilder, StructConstraint,
    };

    #[test]
    fn ddl_test() {
        let nested_attrs = struct_fields![
            (
                "a",
                PartiqlShapeBuilder::init_or_get().any_of(vec![
                    PartiqlShapeBuilder::init_or_get().new_static(Static::DecimalP(5, 4)),
                    PartiqlShapeBuilder::init_or_get().new_static(Static::Int8),
                ])
            ),
            ("b", type_array![type_string![]]),
            ("c", type_float64!()),
        ];
        let details = type_struct![IndexSet::from([nested_attrs])];

        let fields = struct_fields![
            ("employee_id", type_int8![]),
            ("full_name", type_string![]),
            (
                "salary",
                PartiqlShapeBuilder::init_or_get().new_static(Static::DecimalP(8, 2))
            ),
            ("details", details),
            ("dependents", type_array![type_string![]])
        ];
        let ty = type_bag![type_struct![IndexSet::from([
            fields,
            StructConstraint::Open(false)
        ])]];

        let expected_compact = r#""employee_id" TINYINT,"full_name" VARCHAR,"salary" DECIMAL(8, 2),"details" STRUCT<"a": UNION<DECIMAL(5, 4),TINYINT>,"b": type_array<VARCHAR>,"c": DOUBLE>,"dependents" type_array<VARCHAR>"#;
        let expected_pretty = r#""employee_id" TINYINT,
"full_name" VARCHAR,
"salary" DECIMAL(8, 2),
"details" STRUCT<"a": UNION<DECIMAL(5, 4),TINYINT>,"b": type_array<VARCHAR>,"c": DOUBLE>,
"dependents" type_array<VARCHAR>"#;

        let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
        assert_eq!(ddl_compact.ddl(&ty).expect("write shape"), expected_compact);

        dbg!(&expected_compact);

        let ddl_pretty = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
        assert_eq!(ddl_pretty.ddl(&ty).expect("write shape"), expected_pretty);

        dbg!(&expected_pretty);
    }
}
