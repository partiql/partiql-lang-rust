//! [`ShapeKind`]: a field-less discriminant of a [`PartiqlShape`], one value per type code of
//! partiql-lang-kotlin's `PType`.

use crate::{IntervalFamily, PartiqlShape, Static, StaticType};
use std::fmt::{Display, Formatter};

/// The kind of a shape, without its parameters: one value per type code of
/// partiql-lang-kotlin's (PLK) `PType` (v1.6.1, `PType.java:307-681`).
///
/// It is a discriminant, not a second type model: the parameters (lengths, precisions, element
/// and field types) stay in the [`PartiqlShape`]. The tables in [`crate::lattice`] are keyed by
/// it. [`ShapeKind::ALL`] lists the kinds in PLK's code order and [`Display`] prints PLK's type
/// name (e.g. `TIMESTAMPZ`, `INTERVAL_DT`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ShapeKind {
    /// `DYNAMIC`: [`PartiqlShape::Dynamic`].
    Dynamic,
    /// `BOOL`: [`Static::Bool`].
    Bool,
    /// `TINYINT`: [`Static::Int8`].
    TinyInt,
    /// `SMALLINT`: [`Static::Int16`].
    SmallInt,
    /// `INTEGER`: [`Static::Int32`].
    Integer,
    /// `BIGINT`: [`Static::Int64`].
    BigInt,
    /// `NUMERIC(p,s)`: [`Static::NumericP`].
    Numeric,
    /// `DECIMAL(p,s)`: [`Static::DecimalP`].
    Decimal,
    /// `REAL`: [`Static::Float32`].
    Real,
    /// `DOUBLE`: [`Static::Float64`].
    Double,
    /// `CHAR(n)`: [`Static::StringFixed`].
    Char,
    /// `VARCHAR(n)`: [`Static::StringVarying`].
    Varchar,
    /// `STRING`: [`Static::String`].
    String,
    /// `BLOB(n)`: [`Static::Blob`].
    Blob,
    /// `CLOB(n)`: [`Static::Clob`].
    Clob,
    /// `DATE`: [`Static::Date`].
    Date,
    /// `TIME(p)`: [`Static::Time`].
    Time,
    /// `TIMEZ(p)` (`TIME(p) WITH TIME ZONE`): [`Static::TimeTz`].
    TimeTz,
    /// `TIMESTAMP(p)`: [`Static::Timestamp`].
    Timestamp,
    /// `TIMESTAMPZ(p)` (`TIMESTAMP(p) WITH TIME ZONE`): [`Static::TimestampTz`].
    TimestampTz,
    /// `ARRAY<t>`: [`Static::Array`].
    Array,
    /// `BAG<t>`: [`Static::Bag`].
    Bag,
    /// `ROW(fields)`: a [`Static::Struct`] whose [`crate::StructType::is_row`] is `true`.
    Row,
    /// `STRUCT`: any other [`Static::Struct`].
    Struct,
    /// `UNKNOWN`, the type of the `NULL` and `MISSING` literals: [`Static::Null`] and
    /// [`Static::Missing`].
    Unknown,
    /// `VARIANT(encoding)`: [`Static::Variant`].
    Variant,
    /// `INTERVAL_YM`: a [`Static::Interval`] of the year-month family.
    IntervalYm,
    /// `INTERVAL_DT`: a [`Static::Interval`] of the day-time family.
    IntervalDt,
    /// `MAP<k,v>`: [`Static::Map`].
    Map,
}

impl ShapeKind {
    /// Every kind, in partiql-lang-kotlin's code order (`PType.codes()`, `PType.java:198-230`).
    ///
    /// A slice, not an array, so that adding a kind is not a breaking change.
    pub const ALL: &'static [ShapeKind] = &[
        ShapeKind::Dynamic,
        ShapeKind::Bool,
        ShapeKind::TinyInt,
        ShapeKind::SmallInt,
        ShapeKind::Integer,
        ShapeKind::BigInt,
        ShapeKind::Numeric,
        ShapeKind::Decimal,
        ShapeKind::Real,
        ShapeKind::Double,
        ShapeKind::Char,
        ShapeKind::Varchar,
        ShapeKind::String,
        ShapeKind::Blob,
        ShapeKind::Clob,
        ShapeKind::Date,
        ShapeKind::Time,
        ShapeKind::TimeTz,
        ShapeKind::Timestamp,
        ShapeKind::TimestampTz,
        ShapeKind::Array,
        ShapeKind::Bag,
        ShapeKind::Row,
        ShapeKind::Struct,
        ShapeKind::Unknown,
        ShapeKind::Variant,
        ShapeKind::IntervalYm,
        ShapeKind::IntervalDt,
        ShapeKind::Map,
    ];

    /// partiql-lang-kotlin's name for this kind (`PType.name()`, `PType.java:232-297`).
    pub fn name(&self) -> &'static str {
        match self {
            ShapeKind::Dynamic => "DYNAMIC",
            ShapeKind::Bool => "BOOL",
            ShapeKind::TinyInt => "TINYINT",
            ShapeKind::SmallInt => "SMALLINT",
            ShapeKind::Integer => "INTEGER",
            ShapeKind::BigInt => "BIGINT",
            ShapeKind::Numeric => "NUMERIC",
            ShapeKind::Decimal => "DECIMAL",
            ShapeKind::Real => "REAL",
            ShapeKind::Double => "DOUBLE",
            ShapeKind::Char => "CHAR",
            ShapeKind::Varchar => "VARCHAR",
            ShapeKind::String => "STRING",
            ShapeKind::Blob => "BLOB",
            ShapeKind::Clob => "CLOB",
            ShapeKind::Date => "DATE",
            ShapeKind::Time => "TIME",
            ShapeKind::TimeTz => "TIMEZ",
            ShapeKind::Timestamp => "TIMESTAMP",
            ShapeKind::TimestampTz => "TIMESTAMPZ",
            ShapeKind::Array => "ARRAY",
            ShapeKind::Bag => "BAG",
            ShapeKind::Row => "ROW",
            ShapeKind::Struct => "STRUCT",
            ShapeKind::Unknown => "UNKNOWN",
            ShapeKind::Variant => "VARIANT",
            ShapeKind::IntervalYm => "INTERVAL_YM",
            ShapeKind::IntervalDt => "INTERVAL_DT",
            ShapeKind::Map => "MAP",
        }
    }
}

impl Display for ShapeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Static {
    /// The kind of this type, or `None` for the types that have no partiql-lang-kotlin
    /// counterpart: `Int` (unbounded), `Decimal` (no precision), `DateTime` and `Graph`.
    ///
    /// The legacy logical-planner typer gives integer and decimal literals `Int` and `Decimal`
    /// (and `NULL` / `MISSING` the `Undefined` shape); a user of the lattice must first narrow
    /// such shapes to a typed-planner type (e.g. `Int64`, `DecimalP`, `Null`), which is what
    /// partiql-lang-kotlin's planner assigns.
    pub fn kind(&self) -> Option<ShapeKind> {
        let kind = match self {
            Static::Int | Static::Decimal | Static::DateTime | Static::Graph() => return None,
            Static::Bool => ShapeKind::Bool,
            Static::Int8 => ShapeKind::TinyInt,
            Static::Int16 => ShapeKind::SmallInt,
            Static::Int32 => ShapeKind::Integer,
            Static::Int64 => ShapeKind::BigInt,
            Static::NumericP(_, _) => ShapeKind::Numeric,
            Static::DecimalP(_, _) => ShapeKind::Decimal,
            Static::Float32 => ShapeKind::Real,
            Static::Float64 => ShapeKind::Double,
            Static::StringFixed(_) => ShapeKind::Char,
            Static::StringVarying(_) => ShapeKind::Varchar,
            Static::String => ShapeKind::String,
            Static::Blob(_) => ShapeKind::Blob,
            Static::Clob(_) => ShapeKind::Clob,
            Static::Date => ShapeKind::Date,
            Static::Time(_) => ShapeKind::Time,
            Static::TimeTz(_) => ShapeKind::TimeTz,
            Static::Timestamp(_) => ShapeKind::Timestamp,
            Static::TimestampTz(_) => ShapeKind::TimestampTz,
            Static::Interval(i) => match i.family() {
                IntervalFamily::YearMonth => ShapeKind::IntervalYm,
                IntervalFamily::DayTime => ShapeKind::IntervalDt,
            },
            Static::Null | Static::Missing => ShapeKind::Unknown,
            Static::Variant(_) => ShapeKind::Variant,
            Static::Struct(s) if s.is_row() => ShapeKind::Row,
            Static::Struct(_) => ShapeKind::Struct,
            Static::Bag(_) => ShapeKind::Bag,
            Static::Array(_) => ShapeKind::Array,
            Static::Map(_) => ShapeKind::Map,
        };
        Some(kind)
    }
}

impl StaticType {
    /// The kind of this type's [`Static`] (see [`Static::kind`]); nullability is not part of a
    /// kind.
    #[inline]
    pub fn kind(&self) -> Option<ShapeKind> {
        self.ty().kind()
    }
}

impl PartiqlShape {
    /// The kind of this shape: `Dynamic` for [`PartiqlShape::Dynamic`], the kind of a
    /// [`PartiqlShape::Static`] (see [`Static::kind`]), and `None` for the shapes that have no
    /// partiql-lang-kotlin counterpart: [`PartiqlShape::AnyOf`], [`PartiqlShape::Undefined`]
    /// and the `Static` types listed at [`Static::kind`].
    pub fn kind(&self) -> Option<ShapeKind> {
        match self {
            PartiqlShape::Dynamic => Some(ShapeKind::Dynamic),
            PartiqlShape::Static(s) => s.kind(),
            PartiqlShape::AnyOf(_) | PartiqlShape::Undefined => None,
        }
    }
}
