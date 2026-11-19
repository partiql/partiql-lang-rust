//! The PartiQL type lattice, as pure tables over [`ShapeKind`]: coercion families, the
//! precedence orders used for function and operator resolution, the SQL type families used by
//! the builtin operators, and the table of allowed `CAST`s.
//!
//! The tables are keyed by kind only: type parameters never take part.
//!
//! It also gives the common supertype of two types ([`common_supertype`]). Out of scope here, and
//! owned by the typer and the builtin signatures: the steps of function resolution (exact match,
//! dynamic dispatch, best match), the builtin operators' widening, and what the typer does around
//! the supertype (the `CASE` / `COALESCE` result step and the set operations' check that their
//! result contains no `DYNAMIC`; see [`common_supertype`]). Neither [`can_coerce`] nor
//! [`TYPE_PRECEDENCE`] is a supertype relation (`DATE` with `TIME` has no supertype; `DECIMAL(5,2)`
//! with `INTEGER` is `DECIMAL(12,2)`); use [`common_supertype`].
//!
//! The absent values `NULL` and `MISSING` are not types. Their static type is
//! [`PartiqlShape::Undefined`], whose kind is [`ShapeKind::Unknown`] (`UNKNOWN`), and every type
//! admits them. `UNKNOWN` is the bottom of the lattice: it coerces to every type without a
//! cast ([`implicit_coercion`]), and its supertype with any shape is that shape.

use crate::{
    ArrayType, BagType, PartiqlNoIdShapeBuilder, PartiqlShape, ShapeKind, Static, StructField,
    StructType,
};
use std::cmp::Ordering;

/// A group of kinds whose values may be implicitly coerced to one another (SQL:1999 section
/// 4.1.2), as used by function resolution. `DATE` and `TIMESTAMP` share the `DateTimestamp`
/// family; there are no separate `DATE` or `TIMESTAMP` families.
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
    /// The family of `UNKNOWN` and of `VARIANT`, which has no family of its own.
    Unknown,
    Dynamic,
    IntervalYm,
    IntervalDt,
}

/// The coercion family of a kind. `CLOB` is a `String` (not a `Binary`) and `VARIANT` is in the
/// `Unknown` family.
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
/// [`CoercionFamily`]. The rule has no direction (`DOUBLE` coerces to `TINYINT`).
///
/// A resolver must apply it only after an exact match fails and only when no argument is
/// `DYNAMIC` or `VARIANT`: a call with such an argument is dispatched dynamically over every
/// candidate first. So `false` for a `VARIANT` argument does not mean that no overload
/// applies.
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
    /// An exact match: the argument has the parameter's kind and is passed as it is. A resolver
    /// ranks candidates by their number of exact matches. This compares kinds only,
    /// not parameters (so `VARIANT('ion')` to `VARIANT('json')` is `Exact` too).
    Exact,
    /// The argument is `UNKNOWN` (an absent value, the bottom): it is passed as it is and is not
    /// counted as an exact match.
    Absent,
    /// Passed through an implicit `CAST` to the parameter type.
    Cast,
}

/// The implicit coercion of an argument of kind `from` to a parameter of kind `to`, or `None` if
/// there is none: [`Coercion::Absent`] for an `UNKNOWN` argument (the bottom),
/// [`Coercion::Exact`] for equal kinds, and [`Coercion::Cast`] when [`can_coerce`] allows one.
///
/// This is the per-argument rule of overload matching: an `UNKNOWN` argument is skipped, an
/// argument of the parameter's kind is an exact match, and any other argument needs a coercion.
/// It applies only once dynamic dispatch has been ruled out (see [`can_coerce`]).
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
/// `UNKNOWN` and no `VARIANT`, which have no precedence.
///
/// It is not part of the PartiQL specification and does not imply that a `CAST` exists.
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
/// (see [`common_supertype`]).
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
/// no precedence (`UNKNOWN` or `VARIANT`).
///
/// This is not a total order over all kinds: a caller that sorts with it must leave out
/// `UNKNOWN` and `VARIANT` (no builtin signature uses them as parameters) or handle `None`.
/// Equal kinds give `Equal`, whatever their parameters.
///
/// A resolver orders overloads by arity first, then by this comparison of their parameters,
/// left to right.
pub fn compare_fn_precedence(lhs: ShapeKind, rhs: ShapeKind) -> Option<Ordering> {
    if lhs == rhs {
        return Some(Ordering::Equal);
    }
    Some(fn_precedence(lhs)?.cmp(&fn_precedence(rhs)?))
}

/// A set of kinds that the builtin operators treat as one category, such as the numbers or the
/// character strings. Equality compares members.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SqlTypeFamily {
    members: &'static [ShapeKind],
}

impl SqlTypeFamily {
    /// `BOOL`.
    pub const BOOLEAN: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Bool],
    };
    /// `CHAR`, `VARCHAR`, `STRING`, `CLOB`.
    pub const TEXT: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::Char,
            ShapeKind::Varchar,
            ShapeKind::String,
            ShapeKind::Clob,
        ],
    };
    /// `ARRAY`, `BAG`.
    pub const COLLECTION: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Array, ShapeKind::Bag],
    };
    /// The eight numeric kinds.
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
    /// `DATE`.
    pub const DATE: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Date],
    };
    /// `TIME`, `TIMEZ`.
    pub const TIME: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Time, ShapeKind::TimeTz],
    };
    /// `TIMESTAMP`, `TIMESTAMPZ`.
    pub const TIMESTAMP: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::Timestamp, ShapeKind::TimestampTz],
    };
    /// `DATE`, `TIMESTAMP`, `TIMESTAMPZ`.
    pub const DATE_TIMESTAMP: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::Date,
            ShapeKind::Timestamp,
            ShapeKind::TimestampTz,
        ],
    };
    /// `DATE`, `TIME`, `TIMEZ`, `TIMESTAMP`, `TIMESTAMPZ`.
    pub const DATETIME: SqlTypeFamily = SqlTypeFamily {
        members: &[
            ShapeKind::Date,
            ShapeKind::Time,
            ShapeKind::TimeTz,
            ShapeKind::Timestamp,
            ShapeKind::TimestampTz,
        ],
    };
    /// `INTERVAL_YM`, `INTERVAL_DT`.
    pub const INTERVAL: SqlTypeFamily = SqlTypeFamily {
        members: &[ShapeKind::IntervalYm, ShapeKind::IntervalDt],
    };

    /// The family of a kind: [`Self::NUMBER`], [`Self::TEXT`] or [`Self::COLLECTION`] for their
    /// members, otherwise a family whose only member is `kind`.
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

    /// The kinds in this family, in a fixed order.
    #[inline]
    pub fn members(&self) -> &'static [ShapeKind] {
        self.members
    }

    /// Whether `kind` is in this family.
    #[inline]
    pub fn contains(&self, kind: ShapeKind) -> bool {
        self.members.contains(&kind)
    }
}

/// Whether the planner accepts a `CAST` from kind `operand` to kind `target`. This is the
/// planner's table, used to resolve a `CAST`; an evaluator's runtime casts are a separate
/// matter.
///
/// A cast is either allowed or not. The table has known gaps, kept as they are: e.g. `DATE`
/// casts to `TIMEZ` but not to `TIME`, the character strings do not cast to `DYNAMIC`, and
/// `BLOB` casts to nothing, not even itself.
pub fn can_cast(operand: ShapeKind, target: ShapeKind) -> bool {
    use ShapeKind as K;
    // Rows of `BOOL` and of every number.
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
    // Row of `TIME` and `TIMEZ`.
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
    // Row of `TIMESTAMP` and `TIMESTAMPZ`.
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
    // Row of `DATE`.
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
        // `UNKNOWN`, `VARIANT` and `DYNAMIC` cast to every kind.
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
        // Every kind except `BLOB`, `UNKNOWN`, `DYNAMIC`, `ARRAY`, `BAG`, `ROW`.
        K::Char | K::String | K::Varchar | K::Clob => {
            return !matches!(
                target,
                K::Blob | K::Unknown | K::Dynamic | K::Array | K::Bag | K::Row
            )
        }
        K::Blob => &[],
        K::Date => DATE,
        K::Time | K::TimeTz => TIMES,
        K::Timestamp | K::TimestampTz => TIMESTAMPS,
        K::Bag | K::Array => &[K::Bag, K::Array],
        K::Struct => &[K::Struct, K::Map],
        K::Map => &[K::Map],
        K::IntervalYm => &[K::IntervalYm],
        K::IntervalDt => &[K::IntervalDt],
        K::Row => &[K::Row, K::Map],
    };
    targets.contains(&target)
}

/// The common supertype of two shapes, or `None` if they have none.
///
/// This is the pairwise supertype only. A typer must do more around it: for `CASE` and
/// `COALESCE` it skips `NULL` and `MISSING` literals, types the result `DYNAMIC` when no branch
/// is left or no supertype exists, and then gives the union of the branch types (one type, or
/// `DYNAMIC`) when the supertype is a row, struct or collection, `DECIMAL(max p, max s)` when
/// every branch is a `DECIMAL`, and its own rule for text. A set operation is an error when the
/// supertype is `None` or contains `DYNAMIC`.
///
/// The rules, in order:
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
///    `BAG<DYNAMIC>`), and the result merged as in 1.
///
/// [`PartiqlShape::Undefined`] is the bottom for every shape, also one without a kind; apart from
/// that, shapes without a kind (see [`PartiqlShape::kind`]) have no supertype. Rows merge by
/// position only when both are rows ([`crate::StructType::is_row`]); any other closed struct (e.g.
/// one whose fields are an unordered `Fields` set) has no field order to merge by, so it only has a
/// supertype with an equal struct, where equality includes node ids and nullability (shapes built
/// with [`crate::PartiqlNoIdShapeBuilder`] have no ids). A merged field is optional if either input
/// field is. Two `VARIANT`s have a supertype only if their encodings are equal, since the encoding
/// is part of the type. Every result is nullable at the top level, since every type admits the
/// absent values. A result may be one of the inputs (with its node id and unspecified-parameter
/// flags), e.g. for the bottom; a shape built here is fresh, with no node id and no flags.
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
    // The bottom first, so that it holds also for shapes without a kind (and `UNKNOWN` with
    // `UNKNOWN` is `UNKNOWN`, as a same-kind merge would give).
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

/// `source` widened to kind `target` along the directed widening relation, or `None`.
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

/// The merge of the parameters of two shapes of one kind.
fn merge_parameterized(a: &PartiqlShape, b: &PartiqlShape) -> Option<PartiqlShape> {
    let (Some(sa), Some(sb)) = (static_of(a), static_of(b)) else {
        return Some(a.clone());
    };
    // `min(38, max(p1 - s1, p2 - s2) + max(s1, s2))`, in signed arithmetic, since a scale may
    // exceed its precision (`0.05` is `DECIMAL(1,2)`).
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
        // VARIANTs of different encodings are different types.
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
