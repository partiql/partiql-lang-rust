//! The type lattice of partiql-lang-kotlin (PLK) v1.6.1, as pure tables over [`ShapeKind`]:
//! coercion families, the precedence orders used for function and operator resolution, the SQL
//! type families used by the builtin operators, and the table of allowed `CAST`s.
//!
//! Each table is a port of one PLK table; the doc comment of each item names its source. The
//! tables are keyed by kind only, as in PLK, where they are keyed by `PType.code()`: type
//! parameters never take part.
//!
//! It also ports PLK's common supertype of two types ([`common_supertype`], PLK's
//! `DynamicTyper.getCommonSuperType`). Out of scope here, and owned by the typer and the builtin
//! signatures: PLK's `FnResolver` steps (exact match, dynamic dispatch, best match), the builtin
//! operators' widening, and what the typer does around the supertype (the `CASE` / `COALESCE`
//! result step `DynamicTyper.mapping` and the set operations' check that their result contains
//! no `DYNAMIC`; see [`common_supertype`]). Neither [`can_coerce`] nor [`TYPE_PRECEDENCE`] is a
//! supertype relation (`DATE` with `TIME` has no supertype; `DECIMAL(5,2)` with `INTEGER` is
//! `DECIMAL(12,2)`); use [`common_supertype`].
//!
//! The absent values `NULL` and `MISSING` are not types. Their static type is
//! [`PartiqlShape::Undefined`], whose kind is [`ShapeKind::Unknown`] (PLK's `UNKNOWN`), and every
//! type admits them. `UNKNOWN` is the bottom of the lattice: it coerces to every type without a
//! cast ([`implicit_coercion`]), and its supertype with any shape is that shape.

use crate::{
    ArrayType, BagType, PartiqlNoIdShapeBuilder, PartiqlShape, ShapeKind, Static, StructField,
    StructType,
};
use std::cmp::Ordering;

/// A group of kinds whose values may be implicitly coerced to one another (SQL:1999 section
/// 4.1.2), as used by PLK's function resolution.
///
/// Port of PLK's planner `CoercionFamily` (`partiql-planner/.../internal/CoercionFamily.kt`).
/// That enum also declares `DATE` and `TIMESTAMP`, which its `family` function never returns;
/// they are left out here, which makes this the same set as PLK's `partiql-spi` copy of the
/// enum (`partiql-spi/.../internal/CoercionFamily.kt:12-26`), whose `family` mapping is
/// identical.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CoercionFamily {
    Number,
    String,
    Binary,
    Boolean,
    Structure,
    DateTimestamp,
    Time,
    Collection,
    Map,
    /// The family of `UNKNOWN` and (as in PLK, which does not handle it yet) `VARIANT`.
    Unknown,
    Dynamic,
    IntervalYm,
    IntervalDt,
}

/// The coercion family of a kind.
///
/// Port of `CoercionFamily.family` (`partiql-planner/.../internal/CoercionFamily.kt:39-72`).
/// As there, `CLOB` is a `String` (not a `Binary`) and `VARIANT` is in the `Unknown` family.
pub fn coercion_family(kind: ShapeKind) -> CoercionFamily {
    match kind {
        ShapeKind::TinyInt
        | ShapeKind::SmallInt
        | ShapeKind::Integer
        | ShapeKind::Numeric
        | ShapeKind::BigInt
        | ShapeKind::Real
        | ShapeKind::Double
        | ShapeKind::Decimal => CoercionFamily::Number,
        ShapeKind::String | ShapeKind::Char | ShapeKind::Varchar | ShapeKind::Clob => {
            CoercionFamily::String
        }
        ShapeKind::Bool => CoercionFamily::Boolean,
        ShapeKind::TimeTz | ShapeKind::Time => CoercionFamily::Time,
        ShapeKind::TimestampTz | ShapeKind::Timestamp | ShapeKind::Date => {
            CoercionFamily::DateTimestamp
        }
        ShapeKind::Struct | ShapeKind::Row => CoercionFamily::Structure,
        ShapeKind::Array | ShapeKind::Bag => CoercionFamily::Collection,
        ShapeKind::Map => CoercionFamily::Map,
        ShapeKind::Dynamic => CoercionFamily::Dynamic,
        ShapeKind::Blob => CoercionFamily::Binary,
        ShapeKind::Unknown | ShapeKind::Variant => CoercionFamily::Unknown,
        ShapeKind::IntervalYm => CoercionFamily::IntervalYm,
        ShapeKind::IntervalDt => CoercionFamily::IntervalDt,
    }
}

/// Whether a value of kind `from` may be implicitly coerced to kind `to` when resolving a
/// function: always from `UNKNOWN`, always to `DYNAMIC`, and otherwise when both are in the same
/// [`CoercionFamily`]. As in PLK the rule has no direction (`DOUBLE` coerces to `TINYINT`).
///
/// PLK applies it only after an exact match fails and only when no argument is `DYNAMIC` or
/// `VARIANT`: a call with such an argument is dispatched dynamically over every candidate first
/// (`FnResolver.kt:47-52`). So `false` for a `VARIANT` argument does not mean that no overload
/// applies.
///
/// Port of `CoercionFamily.canCoerce` (`partiql-planner/.../internal/CoercionFamily.kt:82-90`).
pub fn can_coerce(from: ShapeKind, to: ShapeKind) -> bool {
    from == ShapeKind::Unknown
        || to == ShapeKind::Dynamic
        || coercion_family(from) == coercion_family(to)
}

/// How an argument of kind `from` is passed to a parameter of kind `to` (see
/// [`implicit_coercion`]).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Coercion {
    /// An exact match: the argument has the parameter's kind and is passed as it is. PLK counts
    /// these to rank candidates (`FnResolver.kt:154`, `:68`). Like PLK, this compares kinds
    /// only, not parameters (so `VARIANT('ion')` to `VARIANT('json')` is `Exact` too).
    Exact,
    /// The argument is `UNKNOWN` (an absent value, the bottom): it is passed as it is and is not
    /// counted as an exact match (`FnResolver.kt:148-150`).
    Absent,
    /// Passed through an implicit `CAST` to the parameter type.
    Cast,
}

/// The implicit coercion of an argument of kind `from` to a parameter of kind `to`, or `None` if
/// there is none: [`Coercion::Absent`] for an `UNKNOWN` argument (the bottom),
/// [`Coercion::Exact`] for equal kinds, and [`Coercion::Cast`] when [`can_coerce`] allows one.
///
/// Port of the per-argument rule of `FnResolver.match` (`FnResolver.kt:146-157`): an `UNKNOWN`
/// argument is skipped (`:148-150`), an argument of the parameter's kind is an exact match
/// (`:154`), and any other argument needs a coercion (`:155`, `:165-170`). As there, it applies
/// only once dynamic dispatch has been ruled out (see [`can_coerce`]).
pub fn implicit_coercion(from: ShapeKind, to: ShapeKind) -> Option<Coercion> {
    if from == ShapeKind::Unknown {
        Some(Coercion::Absent)
    } else if from == to {
        Some(Coercion::Exact)
    } else if can_coerce(from, to) {
        Some(Coercion::Cast)
    } else {
        None
    }
}

/// The precedence of parameter kinds when ordering function overloads, lowest first. It has no
/// `UNKNOWN` and no `VARIANT`; PLK fails on them (a `!!` on a missing map entry).
///
/// Port of `FnComparator.precedence` (`partiql-planner/.../internal/FnComparator.kt:45-73`). It
/// is not part of the PartiQL specification and does not imply that a `CAST` exists.
pub const FN_PRECEDENCE: &[ShapeKind] = &[
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
    ShapeKind::Clob,
    ShapeKind::Blob,
    ShapeKind::Date,
    ShapeKind::Time,
    ShapeKind::TimeTz,
    ShapeKind::Timestamp,
    ShapeKind::TimestampTz,
    ShapeKind::IntervalYm,
    ShapeKind::IntervalDt,
    ShapeKind::Array,
    ShapeKind::Bag,
    ShapeKind::Row,
    ShapeKind::Struct,
    ShapeKind::Map,
    ShapeKind::Dynamic,
];

/// The precedence of operand kinds used by the builtin operators to pick the wider operand
/// (e.g. `INTEGER + BIGINT` is computed as `BIGINT`), lowest first. Unlike [`FN_PRECEDENCE`] it
/// starts with `UNKNOWN` and has no `MAP` (and no `VARIANT`). It orders every kind it lists and
/// is not a supertype relation: it ranks `TIME` above `DATE`, which have no common supertype
/// in PLK.
///
/// Port of `TypePrecedence.TYPE_PRECEDENCE`
/// (`partiql-spi/.../function/builtins/TypePrecedence.kt:11-39`).
pub const TYPE_PRECEDENCE: &[ShapeKind] = &[
    ShapeKind::Unknown,
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
    ShapeKind::Clob,
    ShapeKind::Blob,
    ShapeKind::Date,
    ShapeKind::Time,
    ShapeKind::TimeTz,
    ShapeKind::Timestamp,
    ShapeKind::TimestampTz,
    ShapeKind::IntervalYm,
    ShapeKind::IntervalDt,
    ShapeKind::Array,
    ShapeKind::Bag,
    ShapeKind::Row,
    ShapeKind::Struct,
    ShapeKind::Dynamic,
];

/// The position of `kind` in [`FN_PRECEDENCE`], or `None` for `UNKNOWN` and `VARIANT`.
pub fn fn_precedence(kind: ShapeKind) -> Option<usize> {
    FN_PRECEDENCE.iter().position(|k| *k == kind)
}

/// The position of `kind` in [`TYPE_PRECEDENCE`], or `None` for `MAP` and `VARIANT`.
pub fn type_precedence(kind: ShapeKind) -> Option<usize> {
    TYPE_PRECEDENCE.iter().position(|k| *k == kind)
}

/// Compares two parameter kinds of function overloads by [`FN_PRECEDENCE`]: equal kinds are
/// `Equal`, otherwise the order of their precedences. `None` when the kinds differ and one has
/// no precedence (`UNKNOWN` or `VARIANT`), where PLK fails.
///
/// This is not a total order over all kinds: a caller that sorts with it must leave out
/// `UNKNOWN` and `VARIANT` (PLK's builtin signatures never use them as parameters) or handle
/// `None`. Equal kinds give `Equal` as in PLK, where two types of one code are equal
/// (`Enum.equals` compares codes) or have the same precedence.
///
/// Port of `FnComparator.comparePrecedence`
/// (`partiql-planner/.../internal/FnComparator.kt:32-37`), which PLK applies to the parameters
/// left to right after comparing arities (`:14-30`).
pub fn compare_fn_precedence(lhs: ShapeKind, rhs: ShapeKind) -> Option<Ordering> {
    if lhs == rhs {
        return Some(Ordering::Equal);
    }
    Some(fn_precedence(lhs)?.cmp(&fn_precedence(rhs)?))
}

/// A set of kinds that the builtin operators treat as one category, such as the numbers or the
/// character strings (after Calcite's `SqlTypeFamily`).
///
/// Port of PLK's `SqlTypeFamily` (`partiql-spi/.../internal/SqlTypeFamily.kt:26-142`). PLK also
/// records a `preferred` type for each family; nothing in PLK reads it, so it is left out.
/// Equality compares members (PLK's class compares by identity, but PLK never compares
/// families).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SqlTypeFamily {
    members: &'static [ShapeKind],
}

impl SqlTypeFamily {
    /// `BOOL` (`SqlTypeFamily.kt:47-48`).
    pub const BOOLEAN: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Bool],
    };
    /// `CHAR`, `VARCHAR`, `STRING`, `CLOB` (`SqlTypeFamily.kt:50-59`).
    pub const TEXT: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::Char,
            ShapeKind::Varchar,
            ShapeKind::String,
            ShapeKind::Clob,
        ],
    };
    /// `ARRAY`, `BAG` (`SqlTypeFamily.kt:61-68`).
    pub const COLLECTION: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Array, ShapeKind::Bag],
    };
    /// The eight numeric kinds (`SqlTypeFamily.kt:70-83`).
    pub const NUMBER: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::TinyInt,
            ShapeKind::SmallInt,
            ShapeKind::Integer,
            ShapeKind::BigInt,
            ShapeKind::Numeric,
            ShapeKind::Real,
            ShapeKind::Double,
            ShapeKind::Decimal,
        ],
    };
    /// `DATE` (`SqlTypeFamily.kt:85-91`).
    pub const DATE: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Date],
    };
    /// `TIME`, `TIMEZ` (`SqlTypeFamily.kt:93-100`).
    pub const TIME: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Time, ShapeKind::TimeTz],
    };
    /// `TIMESTAMP`, `TIMESTAMPZ` (`SqlTypeFamily.kt:102-109`).
    pub const TIMESTAMP: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Timestamp, ShapeKind::TimestampTz],
    };
    /// `DATE`, `TIMESTAMP`, `TIMESTAMPZ` (`SqlTypeFamily.kt:111-119`).
    pub const DATE_TIMESTAMP: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::Date,
            ShapeKind::Timestamp,
            ShapeKind::TimestampTz,
        ],
    };
    /// `DATE`, `TIME`, `TIMEZ`, `TIMESTAMP`, `TIMESTAMPZ` (`SqlTypeFamily.kt:121-131`).
    pub const DATETIME: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::Date,
            ShapeKind::Time,
            ShapeKind::TimeTz,
            ShapeKind::Timestamp,
            ShapeKind::TimestampTz,
        ],
    };
    /// `INTERVAL_YM`, `INTERVAL_DT` (`SqlTypeFamily.kt:133-140`).
    pub const INTERVAL: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::IntervalYm, ShapeKind::IntervalDt],
    };

    /// The family of a kind: [`Self::NUMBER`], [`Self::TEXT`] or [`Self::COLLECTION`] for their
    /// members, otherwise a family whose only member is `kind`.
    ///
    /// Port of `SqlTypeFamily.of` (`SqlTypeFamily.kt:40-45`).
    pub fn of(kind: ShapeKind) -> SqlTypeFamily {
        if Self::NUMBER.contains(kind) {
            Self::NUMBER
        } else if Self::TEXT.contains(kind) {
            Self::TEXT
        } else if Self::COLLECTION.contains(kind) {
            Self::COLLECTION
        } else {
            SqlTypeFamily {
                members: std::slice::from_ref(&ShapeKind::ALL[kind as usize]),
            }
        }
    }

    /// The kinds in this family, in PLK's declaration order.
    #[inline]
    pub fn members(&self) -> &'static [ShapeKind] {
        self.members
    }

    /// Whether `kind` is in this family (`SqlTypeFamily.kt:36`).
    #[inline]
    pub fn contains(&self, kind: ShapeKind) -> bool {
        self.members.contains(&kind)
    }
}

/// Whether the planner accepts a `CAST` from kind `operand` to kind `target`. This is the
/// planner's table; PLK's evaluator has its own runtime cast table.
///
/// Port of `CastTable.partiql` (`partiql-planner/.../internal/casts/CastTable.kt:54-311`),
/// which PLK's planner uses to resolve a `CAST` (`Env.resolveCast`, `Env.kt:250-254`). PLK
/// marks every allowed cast `YES` (it has no `MODIFIED` entries), so a cast is either allowed or
/// not. PLK's table has gaps that are kept here: e.g. `DATE` casts to `TIMEZ` but not to `TIME`,
/// the character strings do not cast to `DYNAMIC`, and `BLOB` casts to nothing, not even itself.
pub fn can_cast(operand: ShapeKind, target: ShapeKind) -> bool {
    use ShapeKind as K;
    // Rows of `BOOL` and of every number (`CastTable.kt:78-203`).
    const BOOL_AND_NUMBERS: &[ShapeKind] = &[
        K::Bool,
        K::TinyInt,
        K::SmallInt,
        K::Integer,
        K::BigInt,
        K::Numeric,
        K::Decimal,
        K::Real,
        K::Double,
        K::Char,
        K::String,
        K::Varchar,
    ];
    // Row of `TIME` and `TIMEZ` (`CastTable.kt:243-262`).
    const TIMES: &[ShapeKind] = &[
        K::TimestampTz,
        K::Timestamp,
        K::TimeTz,
        K::Time,
        K::String,
        K::Varchar,
        K::Char,
        K::Clob,
    ];
    // Row of `TIMESTAMP` and `TIMESTAMPZ` (`CastTable.kt:263-284`).
    const TIMESTAMPS: &[ShapeKind] = &[
        K::TimestampTz,
        K::Timestamp,
        K::Date,
        K::TimeTz,
        K::Time,
        K::String,
        K::Varchar,
        K::Char,
        K::Clob,
    ];
    // Row of `DATE` (`CastTable.kt:233-242`).
    const DATE: &[ShapeKind] = &[
        K::TimestampTz,
        K::Timestamp,
        K::TimeTz,
        K::Date,
        K::String,
        K::Varchar,
        K::Char,
        K::Clob,
    ];
    let targets: &[ShapeKind] = match operand {
        // `UNKNOWN`, `VARIANT` and `DYNAMIC` cast to every kind (`CastTable.kt:61-77`).
        K::Unknown | K::Variant | K::Dynamic => return true,
        K::Bool
        | K::TinyInt
        | K::SmallInt
        | K::Integer
        | K::BigInt
        | K::Numeric
        | K::Decimal
        | K::Real
        | K::Double => BOOL_AND_NUMBERS,
        // Every kind except `BLOB`, `UNKNOWN`, `DYNAMIC`, `ARRAY`, `BAG`, `ROW`
        // (`CastTable.kt:204-231`).
        K::Char | K::String | K::Varchar | K::Clob => {
            return !matches!(
                target,
                K::Blob | K::Unknown | K::Dynamic | K::Array | K::Bag | K::Row
            )
        }
        K::Blob => &[], // `CastTable.kt:232`
        K::Date => DATE,
        K::Time | K::TimeTz => TIMES,
        K::Timestamp | K::TimestampTz => TIMESTAMPS,
        K::Bag | K::Array => &[K::Bag, K::Array], // `CastTable.kt:285-292`
        K::Struct => &[K::Struct, K::Map],        // `CastTable.kt:293-296`
        K::Map => &[K::Map],                      // `CastTable.kt:297-299`
        K::IntervalYm => &[K::IntervalYm],        // `CastTable.kt:300-302`
        K::IntervalDt => &[K::IntervalDt],        // `CastTable.kt:303-305`
        K::Row => &[K::Row, K::Map],              // `CastTable.kt:306-309`
    };
    targets.contains(&target)
}

/// The common supertype of two shapes, or `None` if they have none.
///
/// This is PLK's `getCommonSuperType` only. PLK's typer does more around it, and a typer that
/// matches PLK must too: for `CASE` and `COALESCE` it skips `NULL` and `MISSING` literals, types
/// the result `DYNAMIC` when no branch is left or no supertype exists, and then
/// (`DynamicTyper.mapping`, `DynamicTyper.kt:78-114`) gives the union of the branch types
/// (one type, or `DYNAMIC`) when the supertype is a row, struct or collection, `DECIMAL(max p,
/// max s)` when every branch is a `DECIMAL`, and its own rule for text. A set operation is an
/// error when the supertype is `None` or contains `DYNAMIC` (`PlanTyper.kt:565-570`).
///
/// Port of `DynamicTyper.getCommonSuperType` (`DynamicTyper.kt:192-210`) with
/// `coerceTypeBase` (`:220-306`) and `mergeParameterized` (`:311-349`):
/// 1. Shapes of the same kind merge their parameters: the larger length or precision, the
///    decimal rule `DECIMAL(min(38, max(p1 - s1, p2 - s2) + max(s1, s2)), max(s1, s2))`, the
///    supertype of collection elements, and for rows of equal arity the field-wise supertype
///    (a field whose names differ is named `_`). Any other kind keeps the left shape.
/// 2. `UNKNOWN` ([`PartiqlShape::Undefined`], the bottom) gives the other shape.
/// 3. `DYNAMIC` on either side gives `DYNAMIC`.
/// 4. Otherwise one side is widened to the other's kind (in either direction) along
///    `TINYINT < SMALLINT < INTEGER < BIGINT < NUMERIC/DECIMAL(p,0) < REAL < DOUBLE`,
///    `NUMERIC < DECIMAL`, `CHAR < VARCHAR < STRING < CLOB`, `DATE < TIMESTAMP(0)`,
///    `DATE < TIMESTAMPZ(0)`, `TIME < TIMEZ`, `TIMESTAMP < TIMESTAMPZ` and `ARRAY < BAG` (as
///    `BAG<DYNAMIC>`, as in PLK), and the result merged as in 1.
///
/// [`PartiqlShape::Undefined`] is the bottom for every shape, also one without a kind; apart from
/// that, shapes without a kind (see [`PartiqlShape::kind`]) have no supertype. Rows merge by
/// position only when both are rows ([`crate::StructType::is_row`]); any other closed struct (e.g.
/// one whose fields are an unordered `Fields` set) has no field order to merge by, so it only has a
/// supertype with an equal struct, where equality includes node ids and nullability (shapes built
/// with [`crate::PartiqlNoIdShapeBuilder`] have no ids). A merged field is optional if either input
/// field is (PLK has no optional fields). Two `VARIANT`s have a supertype only if their encodings
/// are equal, since the encoding is part of the type here (PLK keeps the left one, as its `VARIANT`
/// compares only its code). Every result is nullable at the top level, since every type admits the
/// absent values. A result may be one of the inputs (with its node id and unspecified-parameter
/// flags), e.g. for the bottom; a shape built here has no node id and no flags, as PLK builds fresh
/// types.
pub fn common_supertype(lhs: &PartiqlShape, rhs: &PartiqlShape) -> Option<PartiqlShape> {
    supertype(lhs, rhs).map(nullable)
}

/// `shape`, nullable at the top level (nested shapes are not changed here).
fn nullable(shape: PartiqlShape) -> PartiqlShape {
    match &shape {
        PartiqlShape::Static(st) if st.is_not_nullable() => {
            PartiqlNoIdShapeBuilder::default().new_static_with_params(st.ty().clone(), *st.params())
        }
        _ => shape,
    }
}

fn supertype(lhs: &PartiqlShape, rhs: &PartiqlShape) -> Option<PartiqlShape> {
    // The bottom first, so that it holds also for shapes without a kind. (PLK checks equal codes
    // first, which for `UNKNOWN` with `UNKNOWN` gives the same result.)
    if lhs.is_undefined() {
        return Some(rhs.clone());
    }
    if rhs.is_undefined() {
        return Some(lhs.clone());
    }
    let (lk, rk) = (lhs.kind()?, rhs.kind()?);
    if lk == rk {
        return merge_parameterized(lhs, rhs);
    }
    if lk == ShapeKind::Dynamic || rk == ShapeKind::Dynamic {
        return Some(PartiqlShape::Dynamic);
    }
    if let Some(coerced) = coerce_type_base(lhs, rk) {
        return supertype(&coerced, rhs);
    }
    if let Some(coerced) = coerce_type_base(rhs, lk) {
        return supertype(lhs, &coerced);
    }
    None
}

fn new_shape(ty: Static) -> PartiqlShape {
    PartiqlNoIdShapeBuilder::default().new_static(ty)
}

fn static_of(shape: &PartiqlShape) -> Option<&Static> {
    match shape {
        PartiqlShape::Static(s) => Some(s.ty()),
        _ => None,
    }
}

/// `DynamicTyper.coerceTypeBase` (`DynamicTyper.kt:220-306`): `source` widened to kind
/// `target`, or `None`.
fn coerce_type_base(source: &PartiqlShape, target: ShapeKind) -> Option<PartiqlShape> {
    use ShapeKind as K;
    if source.kind()? == target {
        return Some(source.clone());
    }
    let ty = match (static_of(source)?, target) {
        (Static::Int8, K::SmallInt) => Static::Int16,
        (Static::Int8 | Static::Int16, K::Integer) => Static::Int32,
        (Static::Int8 | Static::Int16 | Static::Int32, K::BigInt) => Static::Int64,
        (Static::Int8, K::Numeric) => Static::NumericP(3, 0),
        (Static::Int8, K::Decimal) => Static::DecimalP(3, 0),
        (Static::Int16, K::Numeric) => Static::NumericP(5, 0),
        (Static::Int16, K::Decimal) => Static::DecimalP(5, 0),
        (Static::Int32, K::Numeric) => Static::NumericP(10, 0),
        (Static::Int32, K::Decimal) => Static::DecimalP(10, 0),
        (Static::Int64, K::Numeric) => Static::NumericP(19, 0),
        (Static::Int64, K::Decimal) => Static::DecimalP(19, 0),
        (Static::NumericP(p, s), K::Decimal) => Static::DecimalP(*p, *s),
        (
            Static::Int8
            | Static::Int16
            | Static::Int32
            | Static::Int64
            | Static::NumericP(..)
            | Static::DecimalP(..),
            K::Real,
        ) => Static::Float32,
        (
            Static::Int8
            | Static::Int16
            | Static::Int32
            | Static::Int64
            | Static::NumericP(..)
            | Static::DecimalP(..)
            | Static::Float32,
            K::Double,
        ) => Static::Float64,
        (Static::StringFixed(n), K::Varchar) => Static::StringVarying(*n),
        (Static::StringFixed(_) | Static::StringVarying(_), K::String) => Static::String,
        (Static::StringFixed(n) | Static::StringVarying(n), K::Clob) => Static::Clob(*n),
        (Static::String, K::Clob) => Static::Clob(i32::MAX as usize),
        (Static::Date, K::Timestamp) => Static::Timestamp(0),
        (Static::Date, K::TimestampTz) => Static::TimestampTz(0),
        (Static::Time(p), K::TimeTz) => Static::TimeTz(*p),
        (Static::Timestamp(p), K::TimestampTz) => Static::TimestampTz(*p),
        (Static::Array(_), K::Bag) => Static::Bag(BagType::new_any()),
        _ => return None,
    };
    Some(new_shape(ty))
}

/// `DynamicTyper.mergeParameterized` (`DynamicTyper.kt:311-349`) for two shapes of one kind.
fn merge_parameterized(a: &PartiqlShape, b: &PartiqlShape) -> Option<PartiqlShape> {
    let (Some(sa), Some(sb)) = (static_of(a), static_of(b)) else {
        return Some(a.clone());
    };
    // `min(38, max(p1 - s1, p2 - s2) + max(s1, s2))`, in signed arithmetic as in PLK, where a
    // scale may exceed its precision (`0.05` is `DECIMAL(1,2)`).
    let decimal = |(p1, s1): (usize, usize), (p2, s2): (usize, usize)| {
        let scale = s1.max(s2);
        let digits = (p1 as i128 - s1 as i128).max(p2 as i128 - s2 as i128) + scale as i128;
        (digits.clamp(0, 38) as usize, scale)
    };
    let ty = match (sa, sb) {
        (Static::DecimalP(p1, s1), Static::DecimalP(p2, s2)) => {
            let (p, s) = decimal((*p1, *s1), (*p2, *s2));
            Static::DecimalP(p, s)
        }
        (Static::NumericP(p1, s1), Static::NumericP(p2, s2)) => {
            let (p, s) = decimal((*p1, *s1), (*p2, *s2));
            Static::NumericP(p, s)
        }
        (Static::StringFixed(x), Static::StringFixed(y)) => Static::StringFixed(*x.max(y)),
        (Static::StringVarying(x), Static::StringVarying(y)) => Static::StringVarying(*x.max(y)),
        (Static::Clob(x), Static::Clob(y)) => Static::Clob(*x.max(y)),
        (Static::Blob(x), Static::Blob(y)) => Static::Blob(*x.max(y)),
        (Static::Time(x), Static::Time(y)) => Static::Time(*x.max(y)),
        (Static::TimeTz(x), Static::TimeTz(y)) => Static::TimeTz(*x.max(y)),
        (Static::Timestamp(x), Static::Timestamp(y)) => Static::Timestamp(*x.max(y)),
        (Static::TimestampTz(x), Static::TimestampTz(y)) => Static::TimestampTz(*x.max(y)),
        (Static::Array(x), Static::Array(y)) => {
            let e = common_supertype(x.element_type(), y.element_type())?;
            Static::Array(ArrayType::new(Box::new(e)))
        }
        (Static::Bag(x), Static::Bag(y)) => {
            let e = common_supertype(x.element_type(), y.element_type())?;
            Static::Bag(BagType::new(Box::new(e)))
        }
        // VARIANTs of different encodings are different types here (PLK compares only the code).
        (Static::Variant(x), Static::Variant(y)) => return (x == y).then(|| a.clone()),
        (Static::Struct(x), Static::Struct(y)) if x.is_closed() && y.is_closed() => {
            if !(x.is_row() && y.is_row()) {
                // A `Fields` set has no order to merge by.
                return (x == y).then(|| a.clone());
            }
            let (xf, yf): (Vec<_>, Vec<_>) = (x.fields().collect(), y.fields().collect());
            if xf.len() != yf.len() {
                return None;
            }
            let mut fields = Vec::with_capacity(xf.len());
            for (f, g) in xf.into_iter().zip(yf) {
                let ty = common_supertype(f.ty(), g.ty())?;
                let name = if f.name() == g.name() { f.name() } else { "_" };
                fields.push(if f.is_optional() || g.is_optional() {
                    StructField::new_optional(name, ty)
                } else {
                    StructField::new(name, ty)
                });
            }
            Static::Struct(StructType::new_row(fields))
        }
        _ => return Some(a.clone()),
    };
    Some(new_shape(ty))
}

#[cfg(test)]
mod tests {
    use crate::ShapeKind;

    /// `SqlTypeFamily::of` indexes `ShapeKind::ALL` by discriminant.
    #[test]
    fn all_is_in_declaration_order() {
        for (i, k) in ShapeKind::ALL.iter().enumerate() {
            assert_eq!(*k as usize, i, "{k}");
        }
        assert_eq!(ShapeKind::ALL.len(), 29);
    }
}
