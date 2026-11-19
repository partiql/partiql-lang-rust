//! [`ShapeKind`]: a field-less discriminant of a [`PartiqlShape`], one value per PartiQL type
//! kind.

use crate::{IntervalFamily, PartiqlShape, Static, StaticType};
use std::fmt::{Display, Formatter};

/// The kind of a shape, without its parameters: one value per PartiQL type kind (29 kinds).
///
/// It is a discriminant, not a second type model: the parameters (lengths, precisions, element
/// and field types) stay in the [`PartiqlShape`]. The tables in [`crate::lattice`] are keyed by
/// it. [`ShapeKind::ALL`] lists the kinds in a fixed order and [`Display`] prints the kind's type
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
    /// `ROW(fields)`: a closed [`Static::Struct`] (whatever its field constraint, or none). Every
    /// closed struct is a `ROW`, however it was built.
    ///
    /// Only a [`crate::StructConstraint::FieldList`] holds everything a `ROW` holds: a `ROW`'s
    /// fields are ordered, may repeat, and are never optional, and the row supertype merges
    /// fields by position. A `Fields` set ignores order in equality and merges equal fields, so
    /// a typer must fix an order, and decide what an optional field means, before it can merge
    /// such a struct by position.
    Row,
    /// `STRUCT`: an open [`Static::Struct`]. The `STRUCT` kind has no field parameters: an open
    /// struct's declared fields are not part of its kind.
    Struct,
    /// `UNKNOWN`, the static type of the absent values `NULL` and `MISSING` (which are values,
    /// not types): [`PartiqlShape::Undefined`]. It is the bottom of the lattice; see
    /// [`crate::lattice`].
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
    /// Every kind, in a fixed order (the order of the variants).
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

    /// The PartiQL type name of this kind (e.g. `TIMEZ`, `INTERVAL_YM`).
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
    /// The kind of this type, or `None` for the types that have no kind: `Int` (unbounded),
    /// `Decimal` (no precision), `DateTime` and `Graph`.
    ///
    /// The legacy logical-planner typer gives integer and decimal literals `Int` and `Decimal`,
    /// and uses the `Undefined` shape both for `NULL` / `MISSING` and for failures (an
    /// unresolved name, a missing field), so its output is not lattice input as it is: a user
    /// must type failures as `Dynamic` and narrow `Int` and `Decimal` by the literal's value: an
    /// integer literal is `Int32` if it fits in 32 bits, else `Int64` if it fits in 64, else
    /// `DecimalP(p, 0)`; an exact-numeric literal is `DecimalP` with its own precision and scale
    /// (the number of digits in its unscaled value, and the digits after the point), rounded to
    /// 38 digits, so `0.05` is `DecimalP(1, 2)`, scale above precision.
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
            Static::Variant(_) => ShapeKind::Variant,
            Static::Struct(s) if s.is_closed() => ShapeKind::Row,
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
    /// The kind of this shape: `Dynamic` for [`PartiqlShape::Dynamic`], `Unknown` for
    /// [`PartiqlShape::Undefined`] (the static type of `NULL` and `MISSING`, the `UNKNOWN`
    /// kind), the kind of a [`PartiqlShape::Static`] (see [`Static::kind`]), and `None` for
    /// the `Static` types listed at [`Static::kind`] and for [`PartiqlShape::AnyOf`].
    ///
    /// `Undefined` means only "no present value". A typing failure or an error node must be
    /// typed [`PartiqlShape::Dynamic`], never `Undefined`, so that `Undefined` stays unambiguous
    /// (the legacy logical-planner typer does not follow this yet; see [`Static::kind`]). This
    /// includes an unresolved variable and a path step that can never succeed.
    ///
    /// The lattice has no union kind: a typer collapses a set of types to the type itself when
    /// there is one distinct type and to `DYNAMIC` when there are several (an empty set has no
    /// type), and takes the kind of the result.
    pub fn kind(&self) -> Option<ShapeKind> {
        match self {
            PartiqlShape::Dynamic => Some(ShapeKind::Dynamic),
            PartiqlShape::Static(s) => s.kind(),
            PartiqlShape::Undefined => Some(ShapeKind::Unknown),
            PartiqlShape::AnyOf(_) => None,
        }
    }
}
