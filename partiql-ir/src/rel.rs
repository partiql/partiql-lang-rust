//! Typed relational operators (`Rel`).
//!
//! One [`RelOp`] variant per public `Rel*` class of partiql-lang-kotlin v1.6.1's
//! `org.partiql.plan.rel` package; each variant cites its class as `rel/<Class>.java:<line>`,
//! relative to `partiql-plan/src/main/java/org/partiql/plan/`.

use crate::refs::RoutineRef;
use crate::rex::Rex;
use partiql_types::PartiqlShape;

/// A typed relational operator: an operator and the schema of the rows it produces.
///
/// Immutable: the fields are private and there are no setters (see [`Rex`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rel {
    ty: RelType,
    op: RelOp,
}

/// The schema of a [`Rel`]: its fields in order, and whether its rows are ordered.
/// `rel/RelType.java:9`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelType {
    fields: Vec<RelField>,
    ordered: bool,
}

impl RelType {
    pub fn new(fields: Vec<RelField>, ordered: bool) -> Self {
        RelType { fields, ordered }
    }

    pub fn fields(&self) -> &[RelField] {
        &self.fields
    }

    /// The number of fields.
    pub fn degree(&self) -> usize {
        self.fields.len()
    }

    pub fn is_ordered(&self) -> bool {
        self.ordered
    }

    pub fn into_parts(self) -> (Vec<RelField>, bool) {
        (self.fields, self.ordered)
    }
}

/// A named field of a [`RelType`] (partiql-lang-kotlin's `PTypeField`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelField {
    name: String,
    ty: PartiqlShape,
}

impl RelField {
    pub fn new(name: impl Into<String>, ty: PartiqlShape) -> Self {
        RelField {
            name: name.into(),
            ty,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ty(&self) -> &PartiqlShape {
        &self.ty
    }
    pub fn into_parts(self) -> (String, PartiqlShape) {
        (self.name, self.ty)
    }
}

/// The operator of a [`Rel`].
///
/// Variants are `#[non_exhaustive]` so that fields can be added; construct them through the
/// [`Rel`] constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelOp {
    /// `GROUP BY` and aggregation: `measures` then `groups`. `rel/RelAggregate.java:33`
    #[non_exhaustive]
    Aggregate {
        input: Box<Rel>,
        measures: Vec<Measure>,
        groups: Vec<Rex>,
    },
    /// A lateral join: `right` may refer to `left`'s fields. `rel/RelCorrelate.java:27`
    #[non_exhaustive]
    Correlate {
        left: Box<Rel>,
        right: Box<Rel>,
        join_type: JoinType,
    },
    /// `rel/RelDistinct.java:12`
    #[non_exhaustive]
    Distinct { input: Box<Rel> },
    /// `rel/RelExcept.java:12`
    #[non_exhaustive]
    Except {
        left: Box<Rel>,
        right: Box<Rel>,
        all: bool,
    },
    /// `EXCLUDE` paths. `rel/RelExclude.java:13`
    #[non_exhaustive]
    Exclude {
        input: Box<Rel>,
        exclusions: Vec<Exclusion>,
    },
    /// `rel/RelFilter.java:13`
    #[non_exhaustive]
    Filter { input: Box<Rel>, predicate: Rex },
    /// `rel/RelIntersect.java:12`
    #[non_exhaustive]
    Intersect {
        left: Box<Rel>,
        right: Box<Rel>,
        all: bool,
    },
    /// Iterates a collection, binding each element (`FROM x AS v` over a non-table value).
    /// `rel/RelIterate.java:13`
    #[non_exhaustive]
    Iterate { rex: Rex },
    /// `rel/RelJoin.java:20`
    #[non_exhaustive]
    Join {
        left: Box<Rel>,
        right: Box<Rel>,
        condition: Rex,
        join_type: JoinType,
    },
    /// `rel/RelLimit.java:13`
    #[non_exhaustive]
    Limit { input: Box<Rel>, limit: Rex },
    /// `rel/RelOffset.java:13`
    #[non_exhaustive]
    Offset { input: Box<Rel>, offset: Rex },
    /// `rel/RelProject.java:13`
    #[non_exhaustive]
    Project {
        input: Box<Rel>,
        projections: Vec<Rex>,
    },
    /// Scans a collection, binding each element. `rel/RelScan.java:13`
    #[non_exhaustive]
    Scan { rex: Rex },
    /// `rel/RelSort.java:13`
    #[non_exhaustive]
    Sort {
        input: Box<Rel>,
        collations: Vec<Collation>,
    },
    /// `rel/RelUnion.java:12`
    #[non_exhaustive]
    Union {
        left: Box<Rel>,
        right: Box<Rel>,
        all: bool,
    },
    /// `UNPIVOT`: binds each (value, key) of a struct. `rel/RelUnpivot.java:13`
    #[non_exhaustive]
    Unpivot { rex: Rex },
    /// Window functions over `input`, partitioned by `partitions` and sorted by `collations`;
    /// `bindings[i]` names the output of `functions[i]`. Deprecated in partiql-lang-kotlin
    /// (experimental). `rel/RelWindow.java:29`
    #[non_exhaustive]
    Window {
        input: Box<Rel>,
        name: Option<String>,
        bindings: Vec<String>,
        functions: Vec<WindowFunction>,
        collations: Vec<Collation>,
        partitions: Vec<Rex>,
    },
    /// `WITH name AS (..), .. input`. `rel/RelWith.java:19`
    #[non_exhaustive]
    With {
        input: Box<Rel>,
        elements: Vec<WithElement>,
    },
}

/// The join type of [`RelOp::Join`] and [`RelOp::Correlate`] (`JoinType.java:10`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// A sort key (`Collation.java:11`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collation {
    column: Rex,
    order: Order,
    nulls: Nulls,
}

impl Collation {
    pub fn new(column: Rex, order: Order, nulls: Nulls) -> Self {
        Collation {
            column,
            order,
            nulls,
        }
    }
    pub fn column(&self) -> &Rex {
        &self.column
    }
    pub fn order(&self) -> Order {
        self.order
    }
    pub fn nulls(&self) -> Nulls {
        self.nulls
    }
    pub fn into_parts(self) -> (Rex, Order, Nulls) {
        (self.column, self.order, self.nulls)
    }
}

/// `Collation.java:34`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Order {
    Asc,
    Desc,
}

/// `Collation.java:71`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Nulls {
    First,
    Last,
}

/// An aggregation of [`RelOp::Aggregate`] (`rel/RelAggregate.java:134`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measure {
    agg: RoutineRef,
    args: Vec<Rex>,
    distinct: bool,
}

impl Measure {
    pub fn new(agg: RoutineRef, args: Vec<Rex>, distinct: bool) -> Self {
        Measure {
            agg,
            args,
            distinct,
        }
    }
    /// The resolved aggregation.
    pub fn agg(&self) -> &RoutineRef {
        &self.agg
    }
    pub fn args(&self) -> &[Rex] {
        &self.args
    }
    pub fn is_distinct(&self) -> bool {
        self.distinct
    }
    pub fn into_parts(self) -> (RoutineRef, Vec<Rex>, bool) {
        (self.agg, self.args, self.distinct)
    }
}

/// One `EXCLUDE` path: a variable and the steps below it (`Exclusion.kt:8`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exclusion {
    var: Rex,
    items: Vec<ExclusionItem>,
}

impl Exclusion {
    /// `var` is a [`crate::RexOp::Var`] (checked by [`crate::validate`]).
    pub fn new(var: Rex, items: Vec<ExclusionItem>) -> Self {
        Exclusion { var, items }
    }
    pub fn var(&self) -> &Rex {
        &self.var
    }
    pub fn items(&self) -> &[ExclusionItem] {
        &self.items
    }
    pub fn into_parts(self) -> (Rex, Vec<ExclusionItem>) {
        (self.var, self.items)
    }
}

/// A step of an [`Exclusion`] path and the steps below it (`Exclusion.kt:60-210`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExclusionItem {
    /// `[index]`
    #[non_exhaustive]
    CollIndex {
        index: i32,
        items: Vec<ExclusionItem>,
    },
    /// `[*]`
    #[non_exhaustive]
    CollWildcard { items: Vec<ExclusionItem> },
    /// `['key']` (case-sensitive)
    #[non_exhaustive]
    StructKey {
        key: String,
        items: Vec<ExclusionItem>,
    },
    /// `.symbol` (case-insensitive)
    #[non_exhaustive]
    StructSymbol {
        symbol: String,
        items: Vec<ExclusionItem>,
    },
    /// `.*`
    #[non_exhaustive]
    StructWildcard { items: Vec<ExclusionItem> },
}

impl ExclusionItem {
    pub fn coll_index(index: i32, items: Vec<ExclusionItem>) -> Self {
        ExclusionItem::CollIndex { index, items }
    }
    pub fn coll_wildcard(items: Vec<ExclusionItem>) -> Self {
        ExclusionItem::CollWildcard { items }
    }
    pub fn struct_key(key: impl Into<String>, items: Vec<ExclusionItem>) -> Self {
        ExclusionItem::StructKey {
            key: key.into(),
            items,
        }
    }
    pub fn struct_symbol(symbol: impl Into<String>, items: Vec<ExclusionItem>) -> Self {
        ExclusionItem::StructSymbol {
            symbol: symbol.into(),
            items,
        }
    }
    pub fn struct_wildcard(items: Vec<ExclusionItem>) -> Self {
        ExclusionItem::StructWildcard { items }
    }

    /// The steps below this one.
    pub fn items(&self) -> &[ExclusionItem] {
        match self {
            ExclusionItem::CollIndex { items, .. }
            | ExclusionItem::CollWildcard { items }
            | ExclusionItem::StructKey { items, .. }
            | ExclusionItem::StructSymbol { items, .. }
            | ExclusionItem::StructWildcard { items } => items,
        }
    }
}

/// A window function call of [`RelOp::Window`] (`WindowFunctionNode.java:14`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowFunction {
    signature: WindowSignature,
    args: Vec<Rex>,
}

impl WindowFunction {
    pub fn new(signature: WindowSignature, args: Vec<Rex>) -> Self {
        WindowFunction { signature, args }
    }
    pub fn signature(&self) -> &WindowSignature {
        &self.signature
    }
    pub fn args(&self) -> &[Rex] {
        &self.args
    }
    pub fn into_parts(self) -> (WindowSignature, Vec<Rex>) {
        (self.signature, self.args)
    }
}

/// The signature of a [`WindowFunction`] (`WindowFunctionSignature.java:14`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSignature {
    name: String,
    params: Vec<PartiqlShape>,
    returns: PartiqlShape,
    ignore_nulls: bool,
}

impl WindowSignature {
    pub fn new(
        name: impl Into<String>,
        params: Vec<PartiqlShape>,
        returns: PartiqlShape,
        ignore_nulls: bool,
    ) -> Self {
        WindowSignature {
            name: name.into(),
            params,
            returns,
            ignore_nulls,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn params(&self) -> &[PartiqlShape] {
        &self.params
    }
    pub fn returns(&self) -> &PartiqlShape {
        &self.returns
    }
    pub fn is_ignore_nulls(&self) -> bool {
        self.ignore_nulls
    }
    pub fn into_parts(self) -> (String, Vec<PartiqlShape>, PartiqlShape, bool) {
        (self.name, self.params, self.returns, self.ignore_nulls)
    }
}

/// A `name AS (representation)` element of [`RelOp::With`] (`WithListElement.java:15`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithElement {
    name: String,
    representation: Rex,
}

impl WithElement {
    pub fn new(name: impl Into<String>, representation: Rex) -> Self {
        WithElement {
            name: name.into(),
            representation,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn representation(&self) -> &Rex {
        &self.representation
    }
    pub fn into_parts(self) -> (String, Rex) {
        (self.name, self.representation)
    }
}

fn boxed(rel: Rel) -> Box<Rel> {
    Box::new(rel)
}

impl Rel {
    /// Builds a `Rel` from a schema and an operator, e.g. the parts returned by
    /// [`Rel::into_parts`].
    pub fn from_parts(ty: RelType, op: RelOp) -> Self {
        Rel { ty, op }
    }

    /// The schema of the rows this operator produces.
    pub fn ty(&self) -> &RelType {
        &self.ty
    }

    pub fn op(&self) -> &RelOp {
        &self.op
    }

    /// Takes the operator apart, to build a changed copy with [`Rel::from_parts`].
    pub fn into_parts(self) -> (RelType, RelOp) {
        (self.ty, self.op)
    }

    pub fn aggregate(ty: RelType, input: Rel, measures: Vec<Measure>, groups: Vec<Rex>) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Aggregate {
                input: boxed(input),
                measures,
                groups,
            },
        )
    }

    pub fn correlate(ty: RelType, left: Rel, right: Rel, join_type: JoinType) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Correlate {
                left: boxed(left),
                right: boxed(right),
                join_type,
            },
        )
    }

    pub fn distinct(ty: RelType, input: Rel) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Distinct {
                input: boxed(input),
            },
        )
    }

    pub fn except(ty: RelType, left: Rel, right: Rel, all: bool) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Except {
                left: boxed(left),
                right: boxed(right),
                all,
            },
        )
    }

    pub fn exclude(ty: RelType, input: Rel, exclusions: Vec<Exclusion>) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Exclude {
                input: boxed(input),
                exclusions,
            },
        )
    }

    pub fn filter(ty: RelType, input: Rel, predicate: Rex) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Filter {
                input: boxed(input),
                predicate,
            },
        )
    }

    pub fn intersect(ty: RelType, left: Rel, right: Rel, all: bool) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Intersect {
                left: boxed(left),
                right: boxed(right),
                all,
            },
        )
    }

    pub fn iterate(ty: RelType, rex: Rex) -> Self {
        Rel::from_parts(ty, RelOp::Iterate { rex })
    }

    pub fn join(ty: RelType, left: Rel, right: Rel, condition: Rex, join_type: JoinType) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Join {
                left: boxed(left),
                right: boxed(right),
                condition,
                join_type,
            },
        )
    }

    pub fn limit(ty: RelType, input: Rel, limit: Rex) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Limit {
                input: boxed(input),
                limit,
            },
        )
    }

    pub fn offset(ty: RelType, input: Rel, offset: Rex) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Offset {
                input: boxed(input),
                offset,
            },
        )
    }

    pub fn project(ty: RelType, input: Rel, projections: Vec<Rex>) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Project {
                input: boxed(input),
                projections,
            },
        )
    }

    pub fn scan(ty: RelType, rex: Rex) -> Self {
        Rel::from_parts(ty, RelOp::Scan { rex })
    }

    pub fn sort(ty: RelType, input: Rel, collations: Vec<Collation>) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Sort {
                input: boxed(input),
                collations,
            },
        )
    }

    pub fn union(ty: RelType, left: Rel, right: Rel, all: bool) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Union {
                left: boxed(left),
                right: boxed(right),
                all,
            },
        )
    }

    pub fn unpivot(ty: RelType, rex: Rex) -> Self {
        Rel::from_parts(ty, RelOp::Unpivot { rex })
    }

    /// Argument order as partiql-lang-kotlin's `RelWindow.create` (`rel/RelWindow.java:41`).
    pub fn window(
        ty: RelType,
        name: Option<String>,
        input: Rel,
        bindings: Vec<String>,
        functions: Vec<WindowFunction>,
        collations: Vec<Collation>,
        partitions: Vec<Rex>,
    ) -> Self {
        Rel::from_parts(
            ty,
            RelOp::Window {
                input: boxed(input),
                name,
                bindings,
                functions,
                collations,
                partitions,
            },
        )
    }

    pub fn with(ty: RelType, input: Rel, elements: Vec<WithElement>) -> Self {
        Rel::from_parts(
            ty,
            RelOp::With {
                input: boxed(input),
                elements,
            },
        )
    }
}
