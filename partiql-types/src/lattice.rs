//! The type lattice of partiql-lang-kotlin (PLK) v1.6.1, as pure tables over [`ShapeKind`]:
//! coercion families, the precedence orders used for function and operator resolution, the SQL
//! type families used by the builtin operators, and the table of allowed `CAST`s.
//!
//! Each table is a port of one PLK table; the doc comment of each item names its source. The
//! tables are keyed by kind only, as in PLK, where they are keyed by `PType.code()`: type
//! parameters never take part.
//!
//! Only the tables keyed by kind are ported. Out of scope here, and owned by the typer and the
//! builtin signatures: PLK's `FnResolver` steps (exact match, dynamic dispatch, best match), the
//! builtin operators' widening, and the common-supertype rules of CASE, COALESCE and set
//! operations (`DynamicTyper.getCommonSuperType`, `coerceTypeBase` and `mergeParameterized`,
//! `DynamicTyper.kt:192-349`). Those rules widen along a directed relation
//! (`coerceTypeBase`) and depend on type parameters (e.g. `DECIMAL(5,2)` with `INTEGER` is
//! `DECIMAL(12,2)`, and `DATE` with `TIME` has no supertype), so neither [`can_coerce`] nor
//! [`TYPE_PRECEDENCE`] can stand in for them.

use crate::ShapeKind;
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
