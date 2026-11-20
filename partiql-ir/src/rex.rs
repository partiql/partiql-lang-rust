//! Typed scalar expressions (`Rex`).
//!
//! One [`RexOp`] variant per kind of scalar expression: literals, variables, paths, calls and
//! dispatches, casts, CASE/COALESCE/NULLIF, constructors (array, bag, struct, map, spread),
//! SELECT/PIVOT, subqueries, table references and errors.

use crate::refs::{QualifiedName, RoutineRef};
use crate::rel::Rel;
use partiql_types::PartiqlShape;
use partiql_value::Value;

/// A typed scalar expression: an operator and the shape of the value it produces.
///
/// Immutable: the fields are private and there are no setters. Build one with the constructor
/// for its operator (e.g. [`Rex::var`]); take one apart with [`Rex::into_parts`] to build a
/// changed copy (see [`crate::Rewriter`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rex {
    ty: PartiqlShape,
    op: RexOp,
}

/// The operator of a [`Rex`].
///
/// Variants are `#[non_exhaustive]` so that fields can be added; construct them through the
/// [`Rex`] constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RexOp {
    #[non_exhaustive]
    Array { values: Vec<Rex> },
    #[non_exhaustive]
    Bag { values: Vec<Rex> },
    /// A call of a resolved scalar function.
    #[non_exhaustive]
    Call {
        function: RoutineRef,
        args: Vec<Rex>,
    },
    /// `CASE [match] WHEN .. THEN .. [ELSE default] END`.
    #[non_exhaustive]
    Case {
        match_: Option<Box<Rex>>,
        branches: Vec<CaseBranch>,
        default: Option<Box<Rex>>,
    },
    #[non_exhaustive]
    Cast {
        operand: Box<Rex>,
        target: PartiqlShape,
    },
    #[non_exhaustive]
    Coalesce { args: Vec<Rex> },
    /// A call whose overload is chosen at run time from `functions`.
    #[non_exhaustive]
    Dispatch {
        name: String,
        functions: Vec<RoutineRef>,
        args: Vec<Rex>,
    },
    /// An expression that failed to plan; the problem is reported to the planner's listener.
    #[non_exhaustive]
    Error {},
    /// A literal, typed by its [`Rex`]'s shape.
    #[non_exhaustive]
    Lit { value: Value },
    /// A map constructor with declared key and value types.
    #[non_exhaustive]
    Map {
        key_type: PartiqlShape,
        value_type: PartiqlShape,
        entries: Vec<MapEntry>,
    },
    /// A map constructor whose key and value types are not declared.
    #[non_exhaustive]
    MapDynamic { entries: Vec<MapEntry> },
    #[non_exhaustive]
    NullIf { v1: Box<Rex>, v2: Box<Rex> },
    /// `operand[index]`.
    #[non_exhaustive]
    PathIndex { operand: Box<Rex>, index: Box<Rex> },
    /// `operand['key']`, a case-sensitive lookup.
    #[non_exhaustive]
    PathKey { operand: Box<Rex>, key: Box<Rex> },
    /// `operand.symbol`, a case-insensitive lookup.
    #[non_exhaustive]
    PathSymbol { operand: Box<Rex>, symbol: String },
    /// `PIVOT value AT key FROM ..`.
    #[non_exhaustive]
    Pivot {
        input: Box<Rel>,
        key: Box<Rex>,
        value: Box<Rex>,
    },
    /// `SELECT ..`: evaluates `constructor` for every row of `input`.
    #[non_exhaustive]
    Select {
        input: Box<Rel>,
        constructor: Box<Rex>,
    },
    /// Struct concatenation (TUPLEUNION).
    #[non_exhaustive]
    Spread { args: Vec<Rex> },
    #[non_exhaustive]
    Struct { fields: Vec<StructEntry> },
    /// A subquery used as an expression; `scalar` for a scalar (single value) subquery.
    #[non_exhaustive]
    Subquery {
        input: Box<Rel>,
        constructor: Box<Rex>,
        scalar: bool,
    },
    /// `args <comparison> <quantifier> (subquery)`.
    #[non_exhaustive]
    SubqueryComp {
        input: Box<Rel>,
        args: Vec<Rex>,
        comparison: Comparison,
        quantifier: Quantifier,
    },
    /// `args IN (subquery)`.
    #[non_exhaustive]
    SubqueryIn { input: Box<Rel>, args: Vec<Rex> },
    /// `EXISTS (subquery)` / `UNIQUE (subquery)`.
    #[non_exhaustive]
    SubqueryTest { input: Box<Rel>, test: SubqueryTest },
    /// A catalog table, by name; its shape is the [`Rex`]'s.
    #[non_exhaustive]
    Table { table: QualifiedName },
    /// A table by catalog and table id, indexes into a symbol table that the planner returns
    /// beside the plan (not part of this crate).
    #[non_exhaustive]
    TableRef { catalog_id: u32, table_id: u32 },
    /// A variable: field `offset` of the scope `scope` levels out (0 = innermost).
    #[non_exhaustive]
    Var { scope: u32, offset: u32 },
}

/// A `WHEN condition THEN result` branch of [`RexOp::Case`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseBranch {
    condition: Rex,
    result: Rex,
}

impl CaseBranch {
    pub fn new(condition: Rex, result: Rex) -> Self {
        CaseBranch { condition, result }
    }
    pub fn condition(&self) -> &Rex {
        &self.condition
    }
    pub fn result(&self) -> &Rex {
        &self.result
    }
    pub fn into_parts(self) -> (Rex, Rex) {
        (self.condition, self.result)
    }
}

/// A `key: value` field of [`RexOp::Struct`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructEntry {
    key: Rex,
    value: Rex,
}

impl StructEntry {
    pub fn new(key: Rex, value: Rex) -> Self {
        StructEntry { key, value }
    }
    pub fn key(&self) -> &Rex {
        &self.key
    }
    pub fn value(&self) -> &Rex {
        &self.value
    }
    pub fn into_parts(self) -> (Rex, Rex) {
        (self.key, self.value)
    }
}

/// A `key: value` entry of [`RexOp::Map`] and [`RexOp::MapDynamic`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEntry {
    key: Rex,
    value: Rex,
}

impl MapEntry {
    pub fn new(key: Rex, value: Rex) -> Self {
        MapEntry { key, value }
    }
    pub fn key(&self) -> &Rex {
        &self.key
    }
    pub fn value(&self) -> &Rex {
        &self.value
    }
    pub fn into_parts(self) -> (Rex, Rex) {
        (self.key, self.value)
    }
}

/// The comparison of [`RexOp::SubqueryComp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// The quantifier of [`RexOp::SubqueryComp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Quantifier {
    Any,
    All,
    Some,
}

/// The test of [`RexOp::SubqueryTest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SubqueryTest {
    Exists,
    Unique,
}

fn boxed(rex: Rex) -> Box<Rex> {
    Box::new(rex)
}

impl Rex {
    /// Builds a `Rex` from a shape and an operator, e.g. the parts returned by
    /// [`Rex::into_parts`].
    pub fn from_parts(ty: PartiqlShape, op: RexOp) -> Self {
        Rex { ty, op }
    }

    /// The shape of the value this expression produces.
    pub fn ty(&self) -> &PartiqlShape {
        &self.ty
    }

    pub fn op(&self) -> &RexOp {
        &self.op
    }

    /// Takes the expression apart, to build a changed copy with [`Rex::from_parts`].
    pub fn into_parts(self) -> (PartiqlShape, RexOp) {
        (self.ty, self.op)
    }

    pub fn array(ty: PartiqlShape, values: Vec<Rex>) -> Self {
        Rex::from_parts(ty, RexOp::Array { values })
    }

    pub fn bag(ty: PartiqlShape, values: Vec<Rex>) -> Self {
        Rex::from_parts(ty, RexOp::Bag { values })
    }

    pub fn call(ty: PartiqlShape, function: RoutineRef, args: Vec<Rex>) -> Self {
        Rex::from_parts(ty, RexOp::Call { function, args })
    }

    pub fn case(
        ty: PartiqlShape,
        match_: Option<Rex>,
        branches: Vec<CaseBranch>,
        default: Option<Rex>,
    ) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Case {
                match_: match_.map(boxed),
                branches,
                default: default.map(boxed),
            },
        )
    }

    pub fn cast(ty: PartiqlShape, operand: Rex, target: PartiqlShape) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Cast {
                operand: boxed(operand),
                target,
            },
        )
    }

    pub fn coalesce(ty: PartiqlShape, args: Vec<Rex>) -> Self {
        Rex::from_parts(ty, RexOp::Coalesce { args })
    }

    pub fn dispatch(
        ty: PartiqlShape,
        name: impl Into<String>,
        functions: Vec<RoutineRef>,
        args: Vec<Rex>,
    ) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Dispatch {
                name: name.into(),
                functions,
                args,
            },
        )
    }

    /// A failed expression, typed `Dynamic` (see [`crate::validate`]).
    pub fn error() -> Self {
        Rex::from_parts(PartiqlShape::Dynamic, RexOp::Error {})
    }

    pub fn lit(ty: PartiqlShape, value: Value) -> Self {
        Rex::from_parts(ty, RexOp::Lit { value })
    }

    pub fn map(
        ty: PartiqlShape,
        key_type: PartiqlShape,
        value_type: PartiqlShape,
        entries: Vec<MapEntry>,
    ) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Map {
                key_type,
                value_type,
                entries,
            },
        )
    }

    pub fn map_dynamic(ty: PartiqlShape, entries: Vec<MapEntry>) -> Self {
        Rex::from_parts(ty, RexOp::MapDynamic { entries })
    }

    pub fn null_if(ty: PartiqlShape, v1: Rex, v2: Rex) -> Self {
        Rex::from_parts(
            ty,
            RexOp::NullIf {
                v1: boxed(v1),
                v2: boxed(v2),
            },
        )
    }

    pub fn path_index(ty: PartiqlShape, operand: Rex, index: Rex) -> Self {
        Rex::from_parts(
            ty,
            RexOp::PathIndex {
                operand: boxed(operand),
                index: boxed(index),
            },
        )
    }

    pub fn path_key(ty: PartiqlShape, operand: Rex, key: Rex) -> Self {
        Rex::from_parts(
            ty,
            RexOp::PathKey {
                operand: boxed(operand),
                key: boxed(key),
            },
        )
    }

    pub fn path_symbol(ty: PartiqlShape, operand: Rex, symbol: impl Into<String>) -> Self {
        Rex::from_parts(
            ty,
            RexOp::PathSymbol {
                operand: boxed(operand),
                symbol: symbol.into(),
            },
        )
    }

    pub fn pivot(ty: PartiqlShape, input: Rel, key: Rex, value: Rex) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Pivot {
                input: Box::new(input),
                key: boxed(key),
                value: boxed(value),
            },
        )
    }

    pub fn select(ty: PartiqlShape, input: Rel, constructor: Rex) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Select {
                input: Box::new(input),
                constructor: boxed(constructor),
            },
        )
    }

    pub fn spread(ty: PartiqlShape, args: Vec<Rex>) -> Self {
        Rex::from_parts(ty, RexOp::Spread { args })
    }

    pub fn struct_(ty: PartiqlShape, fields: Vec<StructEntry>) -> Self {
        Rex::from_parts(ty, RexOp::Struct { fields })
    }

    pub fn subquery(ty: PartiqlShape, input: Rel, constructor: Rex, scalar: bool) -> Self {
        Rex::from_parts(
            ty,
            RexOp::Subquery {
                input: Box::new(input),
                constructor: boxed(constructor),
                scalar,
            },
        )
    }

    pub fn subquery_comp(
        ty: PartiqlShape,
        input: Rel,
        args: Vec<Rex>,
        comparison: Comparison,
        quantifier: Quantifier,
    ) -> Self {
        Rex::from_parts(
            ty,
            RexOp::SubqueryComp {
                input: Box::new(input),
                args,
                comparison,
                quantifier,
            },
        )
    }

    pub fn subquery_in(ty: PartiqlShape, input: Rel, args: Vec<Rex>) -> Self {
        Rex::from_parts(
            ty,
            RexOp::SubqueryIn {
                input: Box::new(input),
                args,
            },
        )
    }

    pub fn subquery_test(ty: PartiqlShape, input: Rel, test: SubqueryTest) -> Self {
        Rex::from_parts(
            ty,
            RexOp::SubqueryTest {
                input: Box::new(input),
                test,
            },
        )
    }

    pub fn table(ty: PartiqlShape, table: QualifiedName) -> Self {
        Rex::from_parts(ty, RexOp::Table { table })
    }

    pub fn table_ref(ty: PartiqlShape, catalog_id: u32, table_id: u32) -> Self {
        Rex::from_parts(
            ty,
            RexOp::TableRef {
                catalog_id,
                table_id,
            },
        )
    }

    pub fn var(ty: PartiqlShape, scope: u32, offset: u32) -> Self {
        Rex::from_parts(ty, RexOp::Var { scope, offset })
    }
}
