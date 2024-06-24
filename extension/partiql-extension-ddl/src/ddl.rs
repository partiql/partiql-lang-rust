use ion_rs::IonError;
use miette::Diagnostic;
use partiql_types::{
    AnyOf, ArrayType, BagType, PartiqlShape, ShapeResultError, StaticType, StaticTypeVariant,
    StructType,
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
    #[error("IonEncodingError: {0}")]
    IonEncodingError(#[from] IonError),
    #[error("DateTimeEncodingError e: {0}")]
    DateTimeEncodingError(#[from] time::error::Format),
    #[error("Invalid Simulation Configuration e: {0}")]
    InvalidSimConfigError(String),
    #[error("Unexpected Type: {0}")]
    UnexpectedType(#[from] ShapeResultError),
}

/// Result of attempts to encode to Ion.
pub type ShapeDdlEncodeResult<T> = Result<T, ShapeEncodingError>;

const PARTIQL_DATA_TYPE_SYNTAX: &str = "partiql_datatype_syntax";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DdlFormat {
    Compact,
    Pretty,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlSyntaxVersion {
    major: u8,
    minor: u8,
}

pub trait PartiqlDdlEncoder {
    type Output;

    fn ddl(&self, ty: &PartiqlShape) -> ShapeDdlEncodeResult<Self::Output>;

    fn syntax(&self) -> DdlSyntax;
}

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
            StaticTypeVariant::Int => out.push_str("INT"),
            StaticTypeVariant::Int8 => out.push_str("TINYINT"),
            StaticTypeVariant::Int16 => out.push_str("SMALLINT"),
            StaticTypeVariant::Int32 => out.push_str("INTEGER"),
            StaticTypeVariant::Int64 => out.push_str("INT8"),
            StaticTypeVariant::Bool => out.push_str("BOOL"),
            StaticTypeVariant::Decimal => out.push_str("DECIMAL"),
            StaticTypeVariant::DecimalP(p, s) => out.push_str(&format!("DECIMAL({p}, {s})")),
            StaticTypeVariant::DateTime => out.push_str("TIMESTAMP"),
            StaticTypeVariant::Float32 => out.push_str("REAL"),
            StaticTypeVariant::Float64 => out.push_str("DOUBLE"),
            StaticTypeVariant::String => out.push_str("VARCHAR"),
            StaticTypeVariant::Struct(s) => out.push_str(&self.write_struct(&s)?),
            StaticTypeVariant::Bag(b) => out.push_str(&self.write_bag(&b)?),
            StaticTypeVariant::Array(a) => out.push_str(&self.write_array(&a)?),

            // non-exhaustive catch-all
            _ => todo!("handle type for {}", ty),
        }

        if !ty.is_nullable() {
            out.push_str(" NOT NULL")
        }

        Ok(out)
    }

    fn write_bag(&self, bag: &BagType) -> ShapeDdlEncodeResult<String> {
        Ok(format!("BAG<{}>", self.write_shape(bag.element_type())?))
    }

    fn write_array(&self, arr: &ArrayType) -> ShapeDdlEncodeResult<String> {
        Ok(format!("ARRAY<{}>", self.write_shape(arr.element_type())?))
    }

    fn write_struct(&self, strct: &StructType) -> ShapeDdlEncodeResult<String> {
        let mut struct_out = String::from("STRUCT<");

        let fields = strct.fields();
        let mut fields = fields.iter().peekable();
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

        if let StaticTypeVariant::Bag(bag) = ty.ty() {
            let s = bag.element_type().expect_struct()?;
            let fields = s.fields();
            let mut fields = fields.iter().peekable();
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
    use partiql_types::{array, bag, f64, int8, r#struct, str, struct_fields, StructConstraint};
    use std::collections::BTreeSet;

    #[test]
    fn ddl_test() {
        let nested_attrs = struct_fields![
            (
                "a",
                PartiqlShape::any_of(vec![
                    PartiqlShape::new(StaticTypeVariant::DecimalP(5, 4)),
                    PartiqlShape::new(StaticTypeVariant::Int8),
                ])
            ),
            ("b", array![str![]]),
            ("c", f64!()),
        ];
        let details = r#struct![BTreeSet::from([nested_attrs])];

        let fields = struct_fields![
            ("employee_id", int8![]),
            ("full_name", str![]),
            (
                "salary",
                PartiqlShape::new(StaticTypeVariant::DecimalP(8, 2))
            ),
            ("details", details),
            ("dependents", array![str![]])
        ];
        let ty = bag![r#struct![BTreeSet::from([
            fields,
            StructConstraint::Open(false)
        ])]];

        let expected_compact = r#""dependents" ARRAY<VARCHAR>,"details" STRUCT<"a": UNION<TINYINT,DECIMAL(5, 4)>,"b": ARRAY<VARCHAR>,"c": DOUBLE>,"employee_id" TINYINT,"full_name" VARCHAR,"salary" DECIMAL(8, 2)"#;
        let expected_pretty = r#""dependents" ARRAY<VARCHAR>,
"details" STRUCT<"a": UNION<TINYINT,DECIMAL(5, 4)>,"b": ARRAY<VARCHAR>,"c": DOUBLE>,
"employee_id" TINYINT,
"full_name" VARCHAR,
"salary" DECIMAL(8, 2)"#;

        let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
        assert_eq!(ddl_compact.ddl(&ty).expect("write shape"), expected_compact);

        dbg!(&expected_compact);

        let ddl_pretty = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
        assert_eq!(ddl_pretty.ddl(&ty).expect("write shape"), expected_pretty);

        dbg!(&expected_pretty);
    }
}
