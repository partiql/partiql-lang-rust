#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use educe::Educe;
use indexmap::IndexSet;
use itertools::Itertools;
use miette::Diagnostic;
use partiql_common::node::{AutoNodeIdGenerator, NodeId, NodeIdGenerator, NullIdGenerator};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Error, Diagnostic)]
#[error("ShapeResult Error")]
#[non_exhaustive]
pub enum ShapeResultError {
    #[error("Unexpected type `{0:?}` for static type bool")]
    UnexpectedType(String),
    /// A map key type was [`PartiqlShape::Dynamic`], which partiql-lang-kotlin also rejects.
    #[error("MAP key type must not be DYNAMIC")]
    DynamicMapKey,
}

/// Result of attempts to encode to Ion.
pub type ShapeResult<T> = Result<T, ShapeResultError>;

pub trait Type {}

impl Type for StaticType {}

/// Hashes an `IndexSet` independently of insertion order, as `IndexSet`'s `PartialEq` ignores
/// order: each element is hashed on its own, and the sorted element hashes are fed to `state`
/// (so the caller's hasher, keyed or not, still sees every element hash).
fn indexset_hash<H, T>(set: &IndexSet<T>, state: &mut H)
where
    H: Hasher,
    T: Hash,
{
    let mut hashes: Vec<u64> = set
        .iter()
        .map(|val| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            val.hash(&mut hasher);
            hasher.finish()
        })
        .collect();
    hashes.sort_unstable();
    state.write_usize(hashes.len());
    for h in hashes {
        state.write_u64(h);
    }
}

#[macro_export]
macro_rules! type_dynamic {
    ($bld:expr) => {
        $bld.new_dynamic()
    };
}

#[macro_export]
macro_rules! type_int {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Int)
    };
}

#[macro_export]
macro_rules! type_int8 {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Int8)
    };
}

#[macro_export]
macro_rules! type_int16 {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Int16)
    };
}

#[macro_export]
macro_rules! type_int32 {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Int32)
    };
}

#[macro_export]
macro_rules! type_int64 {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Int64)
    };
}

#[macro_export]
macro_rules! type_decimal {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Decimal)
    };
}

// TODO add macro_rule for Decimal with precision and scale

#[macro_export]
macro_rules! type_float32 {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Float32)
    };
}

#[macro_export]
macro_rules! type_float64 {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Float64)
    };
}

#[macro_export]
macro_rules! type_string {
    ($bld:expr) => {
        $bld.new_static($crate::Static::String)
    };
}

#[macro_export]
macro_rules! type_bool {
    ($bld:expr) => {
        $bld.new_static($crate::Static::Bool)
    };
}

#[macro_export]
macro_rules! type_numeric {
    ($bld:expr) => {
        [
            $crate::type_int!($bld),
            $crate::type_float32!($bld),
            $crate::type_float64!($bld),
            $crate::type_decimal!($bld),
        ]
        .into_any_of($bld)
    };
}

#[macro_export]
macro_rules! type_datetime {
    ($bld:expr) => {
        $bld.new_static($crate::Static::DateTime)
    };
}

#[macro_export]
macro_rules! type_struct {
    ($bld:expr) => {
        $bld.new_struct(StructType::new_any())
    };
    ($bld:expr, $elem:expr) => {{
        let elem = $elem;
        $bld.new_struct(StructType::new(elem))
    }};
}

#[macro_export]
macro_rules! type_graph {
    ($bld:expr) => {
        $bld.new_graph()
    };
}

#[macro_export]
macro_rules! struct_fields {
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        $crate::StructConstraint::Fields([$(($x, $y).into()),+].into())
    );
}

#[macro_export]
macro_rules! type_bag {
    ($bld:expr) => {
        $bld.new_bag(BagType::new_any())
    };
    ($bld:expr, $elem:expr) => {{
        let elem = $elem;
        $bld.new_bag($crate::BagType::new(Box::new(elem)))
    }};
}

#[macro_export]
macro_rules! type_array {
    ($bld:expr) => {
        $bld.new_array(ArrayType::new_any())
    };
    ($bld:expr, $elem:expr) => {{
        let elem = $elem;
        $bld.new_array($crate::ArrayType::new(Box::new(elem)))
    }};
}

/// Represents a PartiQL Shape
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
// With this implementation `Dynamic` and `AnyOf` cannot have `nullability`; this does not mean their
// `null` value at runtime cannot belong to their domain.
// TODO adopt the correct model Pending PartiQL Types semantics finalization: https://github.com/partiql/partiql-lang/issues/18
#[non_exhaustive]
pub enum PartiqlShape {
    Dynamic,
    AnyOf(AnyOf),
    Static(StaticType),
    Undefined,
}

#[allow(dead_code)]
impl PartiqlShape {
    #[inline]
    pub fn static_type_id(&self) -> Option<NodeId> {
        if let PartiqlShape::Static(StaticType { id, .. }) = self {
            Some(*id)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(
            &self,
            PartiqlShape::Static(StaticType {
                ty: Static::String,
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Struct(_),
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_collection(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Bag(_),
                nullable: true,
                ..
            })
        ) || matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_unordered_collection(&self) -> bool {
        !self.is_ordered_collection()
    }

    #[inline]
    pub fn is_ordered_collection(&self) -> bool {
        // TODO Add Sexp when added
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_graph(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Graph(),
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_bag(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Bag(_),
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true,
                ..
            })
        )
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool {
        matches!(*self, PartiqlShape::Dynamic)
    }

    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(*self, PartiqlShape::Undefined)
    }

    pub fn expect_bool(&self) -> ShapeResult<StaticType> {
        if let PartiqlShape::Static(StaticType {
            id,
            ty: Static::Bool,
            nullable: n,
            ..
        }) = self
        {
            Ok(StaticType {
                id: *id,
                ty: Static::Bool,
                nullable: *n,
                params: TypeParams::default(),
            })
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_bag(&self) -> ShapeResult<BagType> {
        if let PartiqlShape::Static(StaticType {
            ty: Static::Bag(bag),
            ..
        }) = self
        {
            Ok(bag.clone())
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_struct(&self) -> ShapeResult<StructType> {
        if let PartiqlShape::Static(StaticType {
            ty: Static::Struct(stct),
            ..
        }) = self
        {
            Ok(stct.clone())
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_static(&self) -> ShapeResult<StaticType> {
        if let PartiqlShape::Static(s) = self {
            Ok(s.clone())
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_dynamic_type(&self) -> ShapeResult<PartiqlShape> {
        if let PartiqlShape::Dynamic = self {
            Ok(PartiqlShape::Dynamic)
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_undefined(&self) -> ShapeResult<PartiqlShape> {
        if let PartiqlShape::Undefined = self {
            Ok(PartiqlShape::Undefined)
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }
}

impl Display for PartiqlShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            PartiqlShape::Dynamic => "Dynamic".to_string(),
            PartiqlShape::AnyOf(anyof) => {
                format!("AnyOf({})", anyof.types.iter().cloned().join(", "))
            }
            PartiqlShape::Static(s) => format!("{s}"),
            PartiqlShape::Undefined => "Undefined".to_string(),
        };
        write!(f, "{x}")
    }
}

#[derive(Default, Debug)]
pub struct ShapeBuilder<Id: NodeIdGenerator> {
    id_gen: Id,
}

pub type PartiqlShapeBuilder = ShapeBuilder<AutoNodeIdGenerator>;
pub type PartiqlNoIdShapeBuilder = ShapeBuilder<NullIdGenerator>;

impl<Id: NodeIdGenerator + Default> ShapeBuilder<Id> {
    /// A thread-safe method for creating PartiQL shapes with guaranteed uniqueness over
    /// generated `NodeId`s.
    #[track_caller]
    pub fn singleton() -> &'static PartiqlShapeBuilder {
        static SHAPE_BUILDER: OnceLock<PartiqlShapeBuilder> = OnceLock::new();
        SHAPE_BUILDER.get_or_init(PartiqlShapeBuilder::default)
    }

    #[track_caller]
    pub fn dummy_singleton() -> &'static PartiqlNoIdShapeBuilder {
        static SHAPE_BUILDER: OnceLock<PartiqlNoIdShapeBuilder> = OnceLock::new();
        SHAPE_BUILDER.get_or_init(PartiqlNoIdShapeBuilder::default)
    }
}

impl<Id: NodeIdGenerator> ShapeBuilder<Id> {
    #[inline]
    pub fn new_static(&mut self, ty: Static) -> PartiqlShape {
        let id = self.id_gen.next_id();
        PartiqlShape::Static(StaticType {
            id,
            ty,
            nullable: true,
            params: TypeParams::default(),
        })
    }

    /// Creates a non-nullable static type. The absent-value types [`Static::Null`] and
    /// [`Static::Missing`] have no non-nullable form and stay nullable.
    #[inline]
    pub fn new_non_nullable_static(&mut self, ty: Static) -> PartiqlShape {
        let id = self.id_gen.next_id();
        PartiqlShape::Static(StaticType {
            nullable: ty.is_absent(),
            id,
            ty,
            params: TypeParams::default(),
        })
    }

    /// Creates a nullable static type and records in `params` which of `ty`'s parameters were
    /// left unspecified by the user (e.g. `VARCHAR` rather than `VARCHAR(1)`). `ty` carries the
    /// concrete value, which for an unspecified parameter is its default (see [`TypeParams`]).
    ///
    /// A flag is kept only if `ty` has that parameter and carries its default value; any other
    /// flag is cleared, so a specified value is never mistaken for an unspecified one.
    #[inline]
    pub fn new_static_with_params(&mut self, ty: Static, params: TypeParams) -> PartiqlShape {
        let id = self.id_gen.next_id();
        PartiqlShape::Static(StaticType {
            params: params.normalized_for(&ty),
            id,
            ty,
            nullable: true,
        })
    }

    /// Creates a closed, ordered struct whose fields keep their declaration order and any
    /// duplicate names (see [`StructType::new_row`]).
    #[inline]
    pub fn new_row<I>(&mut self, fields: I) -> PartiqlShape
    where
        I: IntoIterator<Item = StructField>,
    {
        self.new_struct(StructType::new_row(fields))
    }

    /// Creates a nullable map type with the given key and value shapes; an error if the key is
    /// [`PartiqlShape::Dynamic`] (see [`MapType::new`]).
    #[inline]
    pub fn new_map(&mut self, key: PartiqlShape, value: PartiqlShape) -> ShapeResult<PartiqlShape> {
        Ok(self.new_static(Static::Map(MapType::new(key, value)?)))
    }

    #[inline]
    pub fn new_dynamic(&mut self) -> PartiqlShape {
        PartiqlShape::Dynamic
    }

    #[inline]
    pub fn new_undefined(&mut self) -> PartiqlShape {
        PartiqlShape::Undefined
    }

    #[inline]
    pub fn new_struct(&mut self, s: StructType) -> PartiqlShape {
        self.new_static(Static::Struct(s))
    }

    #[inline]
    pub fn new_graph(&mut self) -> PartiqlShape {
        self.new_static(Static::Graph())
    }

    #[inline]
    pub fn new_struct_of_dyn(&mut self) -> PartiqlShape {
        self.new_struct(StructType::new_any())
    }

    #[inline]
    pub fn new_bag(&mut self, b: BagType) -> PartiqlShape {
        self.new_static(Static::Bag(b))
    }

    #[inline]
    pub fn new_bag_of<E>(&mut self, element_type: E) -> PartiqlShape
    where
        E: Into<PartiqlShape>,
    {
        self.new_bag(BagType::new_of(element_type))
    }

    #[inline]
    pub fn new_bag_of_dyn(&mut self) -> PartiqlShape {
        let element_type = self.new_dynamic();
        self.new_bag_of(element_type)
    }

    #[inline]
    pub fn new_bag_of_static(&mut self, ty: Static) -> PartiqlShape {
        let element_type = self.new_static(ty);
        self.new_bag_of(element_type)
    }

    #[inline]
    pub fn new_bag_any_of<I>(&mut self, types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let shape = self.any_of(types);
        let bag_type = BagType::new(Box::new(shape));
        self.new_bag(bag_type)
    }

    #[inline]
    pub fn new_array(&mut self, a: ArrayType) -> PartiqlShape {
        self.new_static(Static::Array(a))
    }

    #[inline]
    pub fn new_array_of<E>(&mut self, element_type: E) -> PartiqlShape
    where
        E: Into<PartiqlShape>,
    {
        self.new_array(ArrayType::new_of(element_type))
    }

    #[inline]
    pub fn new_array_of_dyn(&mut self) -> PartiqlShape {
        let element_type = self.new_dynamic();
        self.new_array_of(element_type)
    }

    #[inline]
    pub fn new_array_of_static(&mut self, ty: Static) -> PartiqlShape {
        let element_type = self.new_static(ty);
        self.new_array_of(element_type)
    }

    #[inline]
    pub fn new_array_any_of<I>(&mut self, types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let shape = self.any_of(types);
        let array_type = ArrayType::new(Box::new(shape));
        self.new_array(array_type)
    }

    // The AnyOf::from_iter(types) uses an IndexSet internally to
    // deduplicate types, thus the match on any_of.types.len() could
    // "flatten" AnyOfs that had duplicates.
    // With the addition of IDs, this deduplication no longer happens.
    // TODO revisit the current implementaion and consider an implementation
    // that allows merging of the `metas` for the same type, e.g., with a
    // user-defined control.
    pub fn any_of<I>(&mut self, types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let any_of = AnyOf::from_iter(types);
        match any_of.types.len() {
            0 => self.new_dynamic(),
            1 => {
                let AnyOf { types } = any_of;
                types.into_iter().next().unwrap()
            }
            // TODO figure out what does it mean for a Union to be nullable or not
            _ => PartiqlShape::AnyOf(any_of),
        }
    }

    #[inline]
    pub fn union(&mut self, lhs: PartiqlShape, rhs: PartiqlShape) -> PartiqlShape {
        match (lhs, rhs) {
            (PartiqlShape::Dynamic, _) | (_, PartiqlShape::Dynamic) => PartiqlShape::Dynamic,
            (PartiqlShape::AnyOf(lhs), PartiqlShape::AnyOf(rhs)) => {
                self.any_of(lhs.types.into_iter().chain(rhs.types))
            }
            (PartiqlShape::AnyOf(anyof), other) | (other, PartiqlShape::AnyOf(anyof)) => {
                let mut types = anyof.types;
                types.insert(other);
                self.any_of(types)
            }
            (l, r) => {
                let types = [l, r];
                self.any_of(types)
            }
        }
    }

    pub fn union_of<I>(&mut self, types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let types: Vec<_> = types.into_iter().collect();
        match types.len() {
            0 => self.new_undefined(),
            1 => types.into_iter().next().unwrap(),
            _ => types.into_iter().reduce(|l, r| self.union(l, r)).unwrap(),
        }
    }

    /// A non-nullable copy of a static shape, keeping its parameter flags; `None` for other
    /// shapes. [`Static::Null`] and [`Static::Missing`] stay nullable.
    #[inline]
    pub fn as_non_nullable(&mut self, shape: &PartiqlShape) -> Option<PartiqlShape> {
        if let PartiqlShape::Static(stype) = shape {
            let id = self.id_gen.next_id();
            Some(PartiqlShape::Static(StaticType {
                id,
                ty: stype.ty.clone(),
                nullable: stype.ty.is_absent(),
                params: stype.params,
            }))
        } else {
            None
        }
    }
}

pub trait ShapeBuilderExtensions<Id: NodeIdGenerator> {
    fn into_union(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape;

    fn into_any_of(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape;

    fn into_array(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape;

    fn into_bag(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape;
}

impl<const N: usize, Id: NodeIdGenerator, E: Into<PartiqlShape>> ShapeBuilderExtensions<Id>
    for [E; N]
{
    #[inline]
    fn into_union(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        bld.union_of(self.into_iter().map(|e| e.into()))
    }

    #[inline]
    fn into_any_of(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        bld.any_of(self.into_iter().map(|e| e.into()))
    }

    #[inline]
    fn into_array(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        let ty = self.into_any_of(bld);
        bld.new_array_of(ty)
    }

    #[inline]
    fn into_bag(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        let ty = self.into_any_of(bld);
        bld.new_bag_of(ty)
    }
}

impl<Id: NodeIdGenerator, E: Into<PartiqlShape>> ShapeBuilderExtensions<Id> for E {
    #[inline]
    fn into_union(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        [self].into_union(bld)
    }

    #[inline]
    fn into_any_of(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        [self].into_any_of(bld)
    }

    #[inline]
    fn into_array(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        bld.new_array_of(self)
    }

    #[inline]
    fn into_bag(self, bld: &mut ShapeBuilder<Id>) -> PartiqlShape {
        bld.new_bag_of(self)
    }
}

#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
#[allow(dead_code)]
pub struct AnyOf {
    #[educe(Hash(method(indexset_hash)))]
    types: IndexSet<PartiqlShape>,
}

impl AnyOf {
    #[inline]
    pub const fn new(types: IndexSet<PartiqlShape>) -> Self {
        AnyOf { types }
    }

    pub fn types(&self) -> impl Iterator<Item = &PartiqlShape> {
        self.types.iter()
    }
}

impl FromIterator<PartiqlShape> for AnyOf {
    fn from_iter<T: IntoIterator<Item = PartiqlShape>>(iter: T) -> Self {
        AnyOf {
            types: iter.into_iter().collect(),
        }
    }
}

#[derive(Educe, Clone, Eq)]
#[educe(PartialEq, Hash)]
pub struct StaticType {
    id: NodeId,
    ty: Static,
    nullable: bool,
    /// Which of `ty`'s parameters the user left unspecified. Like PLK's PType metas, these do
    /// not take part in equality or hashing.
    #[educe(PartialEq(ignore), Hash(ignore))]
    params: TypeParams,
}

/// Records which parameters of a parameterized type were not written by the user, so that a
/// consumer can tell `VARCHAR` from `VARCHAR(1)` although both carry length 1.
///
/// The [`Static`] payload always carries the concrete (default) values; these flags are
/// metadata only and are ignored by [`StaticType`]'s `PartialEq` and `Hash`. A consequence is that
/// collections that de-duplicate shapes (e.g. [`AnyOf`] or a set of [`StructField`]s) treat
/// `VARCHAR` and `VARCHAR(1)` as the same shape and keep whichever was inserted first.
///
/// A set flag means the payload holds the default for that parameter. The defaults are the
/// ones partiql-lang-kotlin's planner uses for a type written without the parameter:
///
/// | Kind | Default |
/// |---|---|
/// | `StringFixed`, `StringVarying` (`CHAR`, `VARCHAR`) | length 1 |
/// | `Blob`, `Clob` | length 2147483647 (`i32::MAX`) |
/// | `DecimalP`, `NumericP` | precision 38, scale 0 |
/// | `Time`, `TimeTz` | precision 0 |
/// | `Timestamp`, `TimestampTz` | precision 6 |
/// | `Interval` | leading precision 2, fractional seconds precision 6 |
///
/// [`ShapeBuilder::new_static_with_params`] keeps a flag only where the kind has that parameter
/// and carries its default, so consumers such as DDL encoders can omit a flagged parameter
/// without losing a specified value.
///
/// Build one from [`TypeParams::default`] with the `with_*` methods; more flags may be added.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub struct TypeParams {
    /// The length of `CHAR(n)`, `VARCHAR(n)`, `BLOB(n)` or `CLOB(n)` was not specified.
    pub unspecified_length: bool,
    /// The precision of a decimal, numeric, time, timestamp or interval type was not specified.
    pub unspecified_precision: bool,
    /// The scale of `DECIMAL(p,s)` or `NUMERIC(p,s)` was not specified.
    pub unspecified_scale: bool,
    /// The fractional seconds precision of an interval type was not specified.
    pub unspecified_fractional_precision: bool,
}

const DEFAULT_LENGTH: usize = 1;
const DEFAULT_LOB_LENGTH: usize = i32::MAX as usize;
const DEFAULT_DECIMAL_PRECISION: usize = 38;
const DEFAULT_DECIMAL_SCALE: usize = 0;
const DEFAULT_TIME_PRECISION: usize = 0;
const DEFAULT_TIMESTAMP_PRECISION: usize = 6;
const DEFAULT_INTERVAL_PRECISION: usize = 2;
const DEFAULT_INTERVAL_FRACTIONAL_PRECISION: usize = 6;

impl TypeParams {
    /// These flags with every flag cleared that `ty` has no such parameter for, or whose
    /// parameter does not hold its default (see the table above).
    fn normalized_for(self, ty: &Static) -> TypeParams {
        let (length, precision, scale, fraction) = match ty {
            Static::StringFixed(n) | Static::StringVarying(n) => {
                (*n == DEFAULT_LENGTH, false, false, false)
            }
            Static::Blob(n) | Static::Clob(n) => (*n == DEFAULT_LOB_LENGTH, false, false, false),
            Static::DecimalP(p, s) | Static::NumericP(p, s) => (
                false,
                *p == DEFAULT_DECIMAL_PRECISION,
                *s == DEFAULT_DECIMAL_SCALE,
                false,
            ),
            Static::Time(p) | Static::TimeTz(p) => {
                (false, *p == DEFAULT_TIME_PRECISION, false, false)
            }
            Static::Timestamp(p) | Static::TimestampTz(p) => {
                (false, *p == DEFAULT_TIMESTAMP_PRECISION, false, false)
            }
            // As in partiql-lang-kotlin, an interval written without a fractional precision is
            // flagged whatever its qualifier; without a SECOND field the stored value is 0.
            Static::Interval(i) => (
                false,
                i.precision() == DEFAULT_INTERVAL_PRECISION,
                false,
                !i.qualifier().has_seconds()
                    || i.fractional_precision() == DEFAULT_INTERVAL_FRACTIONAL_PRECISION,
            ),
            _ => (false, false, false, false),
        };
        TypeParams {
            unspecified_length: self.unspecified_length && length,
            unspecified_precision: self.unspecified_precision && precision,
            unspecified_scale: self.unspecified_scale && scale,
            unspecified_fractional_precision: self.unspecified_fractional_precision && fraction,
        }
    }

    /// Returns `true` if every parameter was specified (all flags `false`).
    #[inline]
    pub fn is_default(&self) -> bool {
        *self == TypeParams::default()
    }

    /// Sets [`TypeParams::unspecified_length`].
    #[inline]
    pub const fn with_unspecified_length(mut self, unspecified: bool) -> Self {
        self.unspecified_length = unspecified;
        self
    }

    /// Sets [`TypeParams::unspecified_precision`].
    #[inline]
    pub const fn with_unspecified_precision(mut self, unspecified: bool) -> Self {
        self.unspecified_precision = unspecified;
        self
    }

    /// Sets [`TypeParams::unspecified_scale`].
    #[inline]
    pub const fn with_unspecified_scale(mut self, unspecified: bool) -> Self {
        self.unspecified_scale = unspecified;
        self
    }

    /// Sets [`TypeParams::unspecified_fractional_precision`].
    #[inline]
    pub const fn with_unspecified_fractional_precision(mut self, unspecified: bool) -> Self {
        self.unspecified_fractional_precision = unspecified;
        self
    }
}

// Hand-written so that the Debug text of every pre-existing shape is unchanged.
impl Debug for StaticType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("StaticType");
        d.field("id", &self.id)
            .field("ty", &self.ty)
            .field("nullable", &self.nullable);
        if !self.params.is_default() {
            d.field("params", &self.params);
        }
        d.finish()
    }
}

impl StaticType {
    #[inline]
    pub fn ty(&self) -> &Static {
        &self.ty
    }

    pub fn ty_id(&self) -> &NodeId {
        &self.id
    }

    #[inline]
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Which of this type's parameters were left unspecified by the user.
    #[inline]
    pub fn params(&self) -> &TypeParams {
        &self.params
    }

    #[inline]
    pub fn is_not_nullable(&self) -> bool {
        !self.nullable
    }

    pub fn is_scalar(&self) -> bool {
        self.ty.is_scalar()
    }

    pub fn is_sequence(&self) -> bool {
        self.ty.is_sequence()
    }

    pub fn is_struct(&self) -> bool {
        self.ty.is_struct()
    }
}

impl From<StaticType> for PartiqlShape {
    fn from(value: StaticType) -> Self {
        PartiqlShape::Static(value)
    }
}

impl Display for StaticType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ty = &self.ty;
        if self.nullable {
            write!(f, "{ty}")
        } else {
            write!(f, "NOT NULL {ty}")
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Static {
    // Scalar Types
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Bool,
    Decimal,
    DecimalP(usize, usize),

    Float32,
    Float64,

    String,
    StringFixed(usize),
    StringVarying(usize),

    DateTime,

    /// SQL `NUMERIC(p,s)`, distinct from `DECIMAL(p,s)` as in SQL and PLK's type codes. Note that
    /// partiql-lang-kotlin's planner (v1.6.1) types a `NUMERIC` written in a query as `DECIMAL`.
    NumericP(usize, usize),
    /// `BLOB(n)`
    Blob(usize),
    /// `CLOB(n)`
    Clob(usize),

    Date,
    /// `TIME(p)` without time zone
    Time(usize),
    /// `TIME(p) WITH TIME ZONE`
    TimeTz(usize),
    /// `TIMESTAMP(p)` without time zone
    Timestamp(usize),
    /// `TIMESTAMP(p) WITH TIME ZONE`
    TimestampTz(usize),
    /// `INTERVAL <qualifier>`, a year-month or day-time interval.
    Interval(IntervalType),

    /// The type of the `NULL` literal (an absent value).
    ///
    /// partiql-lang-kotlin types both the `NULL` and the `MISSING` literal as `UNKNOWN`; its
    /// counterpart here is `Null` for a `NULL` literal and [`Static::Missing`] for a `MISSING`
    /// literal (and for expressions that evaluate to `MISSING`, such as an unresolved path). Both
    /// map back to `UNKNOWN`. They are scalars (`is_scalar()` is `true`) and always nullable.
    Null,
    /// The type of the `MISSING` literal (an absent value); see [`Static::Null`].
    Missing,
    /// A value of an embedded document format, named by its encoding (e.g. `ion`), whose type
    /// is not known statically. As in partiql-lang-kotlin, the encoding does not take part in
    /// equality or hashing (see [`VariantType`]).
    Variant(VariantType),

    // Container Types
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    /// A map from keys of one shape to values of another.
    Map(MapType),

    Graph(
        /* TODO: https://github.com/partiql/partiql-lang/blob/main/RFCs/0025-graph-data-model.md */
    ),
    // TODO Add BitString and ByteString types
}

#[non_exhaustive]
pub enum StaticCategory<'a> {
    Graph(),
    Tuple(),
    Map(&'a MapType),
    Sequence(&'a PartiqlShape),
    Scalar(&'a Static),
}

impl Static {
    pub fn is_scalar(&self) -> bool {
        !matches!(
            self,
            Static::Struct(_) | Static::Bag(_) | Static::Array(_) | Static::Map(_)
        )
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Static::Bag(_) | Static::Array(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Static::Struct(_))
    }

    /// The absent-value types `Null` and `Missing`.
    fn is_absent(&self) -> bool {
        matches!(self, Static::Null | Static::Missing)
    }

    pub fn category(&self) -> StaticCategory<'_> {
        match self {
            Static::Struct(_) => StaticCategory::Tuple(),
            Static::Bag(b) => StaticCategory::Sequence(b.element_type()),
            Static::Array(b) => StaticCategory::Sequence(b.element_type()),
            Static::Graph() => StaticCategory::Graph(),
            Static::Map(m) => StaticCategory::Map(m),
            _ => StaticCategory::Scalar(self),
        }
    }
}

// TODO, this should probably be via a prettyprint...
impl Display for Static {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Static::Int => write!(f, "Int"),
            Static::Int8 => write!(f, "Int8"),
            Static::Int16 => write!(f, "Int16"),
            Static::Int32 => write!(f, "Int32"),
            Static::Int64 => write!(f, "Int64"),
            Static::Bool => write!(f, "Bool"),
            Static::Decimal => write!(f, "Decimal"),
            Static::DecimalP(p, s) => {
                write!(f, "Decimal({p},{s})")
            }
            Static::Float32 => write!(f, "Float32"),
            Static::Float64 => write!(f, "Float64"),
            Static::String => write!(f, "String"),
            Static::StringFixed(n) => write!(f, "Char({n})"),
            Static::StringVarying(n) => write!(f, "Varchar({n})"),
            Static::DateTime => write!(f, "DateTime"),
            Static::NumericP(p, s) => write!(f, "Numeric({p},{s})"),
            Static::Blob(n) => write!(f, "Blob({n})"),
            Static::Clob(n) => write!(f, "Clob({n})"),
            Static::Date => write!(f, "Date"),
            Static::Time(p) => write!(f, "Time({p})"),
            Static::TimeTz(p) => write!(f, "TimeTz({p})"),
            Static::Timestamp(p) => write!(f, "Timestamp({p})"),
            Static::TimestampTz(p) => write!(f, "TimestampTz({p})"),
            Static::Interval(i) => std::fmt::Display::fmt(i, f),
            Static::Null => write!(f, "Null"),
            Static::Missing => write!(f, "Missing"),
            Static::Variant(v) => std::fmt::Display::fmt(v, f),
            Static::Map(m) => std::fmt::Display::fmt(m, f),
            Static::Struct(inner) => std::fmt::Display::fmt(inner, f),
            Static::Bag(inner) => std::fmt::Display::fmt(inner, f),
            Static::Array(inner) => std::fmt::Display::fmt(inner, f),
            Static::Graph() => write!(f, "Graph"),
        }
    }
}

pub const TYPE_DYNAMIC: PartiqlShape = PartiqlShape::Dynamic;

#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
#[allow(dead_code)]
pub struct StructType {
    #[educe(Hash(method(indexset_hash)))]
    constraints: IndexSet<StructConstraint>,
}

impl StructType {
    #[inline]
    pub fn new(constraints: IndexSet<StructConstraint>) -> Self {
        StructType { constraints }
    }

    #[inline]
    pub fn new_any() -> Self {
        StructType {
            constraints: Default::default(),
        }
    }

    /// The struct's fields as a set: equal fields (same name and shape) appear once, so a row
    /// with two equal no-id fields yields one. Use [`StructType::fields`] to see every field.
    pub fn fields_set(&self) -> IndexSet<StructField> {
        self.fields().cloned().collect()
    }

    /// The struct's fields in constraint order. Names may repeat: a row (see
    /// [`StructType::new_row`]) keeps duplicate names, and a [`StructConstraint::Fields`] set may
    /// hold two fields with the same name but different shapes.
    pub fn fields(&self) -> impl Iterator<Item = &StructField> {
        use itertools::Either;
        self.constraints.iter().flat_map(|c| match c {
            StructConstraint::Fields(fields) => Either::Left(fields.iter()),
            StructConstraint::FieldList(fields) => Either::Right(fields.iter()),
            _ => Either::Right([].iter()),
        })
    }

    /// A closed, ordered struct whose fields keep their declaration order and allow duplicate
    /// names, e.g. the type of the rows of `SELECT a, a, b FROM t`. Its fields are held in a
    /// [`StructConstraint::FieldList`], so no-id shapes with the same name and type are kept.
    pub fn new_row<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = StructField>,
    {
        StructType {
            constraints: IndexSet::from([
                StructConstraint::Open(false),
                StructConstraint::Ordered(true),
                StructConstraint::DuplicateAttrs(true),
                StructConstraint::FieldList(fields.into_iter().collect()),
            ]),
        }
    }

    /// Returns `true` if the struct has row semantics: closed, ordered, and its fields held in a
    /// [`StructConstraint::FieldList`] (so their order and duplicates are kept and take part in
    /// equality). Every struct built by [`StructType::new_row`] is a row; a closed, ordered
    /// struct whose fields are a [`StructConstraint::Fields`] set is not.
    #[inline]
    pub fn is_row(&self) -> bool {
        self.is_closed()
            && self.constraints.contains(&StructConstraint::Ordered(true))
            && self
                .constraints
                .iter()
                .any(|c| matches!(c, StructConstraint::FieldList(_)))
    }

    #[inline]
    pub fn is_partial(&self) -> bool {
        !self.is_closed()
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.constraints.contains(&StructConstraint::Open(false))
    }
}

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let partial = self.is_partial();
        if self.is_row() {
            write!(f, "Row")?;
        }
        write!(f, "{{")?;
        let mut first = true;
        for StructField { name, ty, optional } in self.fields() {
            if !first {
                write!(f, ", ")?;
            }
            if *optional {
                write!(f, "{name}?: {ty}")?;
            } else {
                write!(f, "{name}: {ty}")?;
            }
            first = false
        }
        if partial {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "...")?;
        }
        write!(f, "}}")
    }
}

#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum StructConstraint {
    Open(bool),
    Ordered(bool),
    DuplicateAttrs(bool),
    Fields(#[educe(Hash(method(indexset_hash)))] IndexSet<StructField>),
    /// Fields in declaration order, duplicate names kept (see [`StructType::new_row`]).
    FieldList(Vec<StructField>),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[allow(dead_code)]
pub struct StructField {
    optional: bool,
    name: String,
    ty: PartiqlShape,
}

impl StructField {
    #[inline]
    pub fn new(name: &str, ty: PartiqlShape) -> Self {
        StructField {
            name: name.to_string(),
            ty,
            optional: false,
        }
    }

    #[inline]
    pub fn new_optional(name: &str, ty: PartiqlShape) -> Self {
        StructField {
            name: name.to_string(),
            ty,
            optional: true,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn ty(&self) -> &PartiqlShape {
        &self.ty
    }

    #[inline]
    pub fn is_optional(&self) -> bool {
        self.optional
    }
}

impl From<(&str, PartiqlShape)> for StructField {
    fn from(value: (&str, PartiqlShape)) -> Self {
        StructField {
            name: value.0.to_string(),
            ty: value.1,
            optional: false,
        }
    }
}

impl From<(&str, PartiqlShape, bool)> for StructField {
    fn from(value: (&str, PartiqlShape, bool)) -> Self {
        StructField {
            name: value.0.to_string(),
            ty: value.1,
            optional: value.2,
        }
    }
}

#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
#[allow(dead_code)]
pub struct BagType {
    element_type: Box<PartiqlShape>,
}

impl BagType {
    #[inline]
    pub fn new(typ: Box<PartiqlShape>) -> Self {
        BagType { element_type: typ }
    }

    #[inline]
    pub fn new_of<E>(element_type: E) -> Self
    where
        E: Into<PartiqlShape>,
    {
        Self::new(Box::new(element_type.into()))
    }

    #[inline]
    pub fn new_any() -> Self {
        Self::new_of(PartiqlShape::Dynamic)
    }

    pub fn element_type(&self) -> &PartiqlShape {
        &self.element_type
    }
}

impl Display for BagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ty = &self.element_type;
        write!(f, "<<{ty}>>")
    }
}

#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
#[allow(dead_code)]
pub struct ArrayType {
    element_type: Box<PartiqlShape>,
    // TODO Add Array constraint once we have Schema Specification:
    // https://github.com/partiql/partiql-spec/issues/49
}

impl ArrayType {
    #[inline]
    pub fn new(typ: Box<PartiqlShape>) -> Self {
        ArrayType { element_type: typ }
    }

    #[inline]
    pub fn new_of<E>(element_type: E) -> Self
    where
        E: Into<PartiqlShape>,
    {
        Self::new(Box::new(element_type.into()))
    }

    #[inline]
    pub fn new_any() -> Self {
        Self::new_of(PartiqlShape::Dynamic)
    }

    #[inline]
    pub fn element_type(&self) -> &PartiqlShape {
        &self.element_type
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ty = &self.element_type;
        write!(f, "[{ty}]")
    }
}

/// The type of a map: keys of one shape, values of another.
#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
pub struct MapType {
    key: Box<PartiqlShape>,
    value: Box<PartiqlShape>,
}

impl MapType {
    /// A map type. As in partiql-lang-kotlin, the key shape must not be
    /// [`PartiqlShape::Dynamic`]; that is an error.
    #[inline]
    pub fn new(key: PartiqlShape, value: PartiqlShape) -> ShapeResult<Self> {
        if key.is_dynamic() {
            return Err(ShapeResultError::DynamicMapKey);
        }
        Ok(MapType {
            key: Box::new(key),
            value: Box::new(value),
        })
    }

    #[inline]
    pub fn key_type(&self) -> &PartiqlShape {
        &self.key
    }

    #[inline]
    pub fn value_type(&self) -> &PartiqlShape {
        &self.value
    }
}

impl Display for MapType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Map<{}, {}>", self.key, self.value)
    }
}

/// The SQL interval qualifiers.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum IntervalQualifier {
    Year,
    Month,
    YearToMonth,
    Day,
    Hour,
    Minute,
    Second,
    DayToHour,
    DayToMinute,
    DayToSecond,
    HourToMinute,
    HourToSecond,
    MinuteToSecond,
}

/// Year-month and day-time intervals are different, mutually incomparable types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum IntervalFamily {
    YearMonth,
    DayTime,
}

impl IntervalQualifier {
    /// The interval family (year-month or day-time) this qualifier belongs to.
    pub fn family(&self) -> IntervalFamily {
        match self {
            IntervalQualifier::Year | IntervalQualifier::Month | IntervalQualifier::YearToMonth => {
                IntervalFamily::YearMonth
            }
            IntervalQualifier::Day
            | IntervalQualifier::Hour
            | IntervalQualifier::Minute
            | IntervalQualifier::Second
            | IntervalQualifier::DayToHour
            | IntervalQualifier::DayToMinute
            | IntervalQualifier::DayToSecond
            | IntervalQualifier::HourToMinute
            | IntervalQualifier::HourToSecond
            | IntervalQualifier::MinuteToSecond => IntervalFamily::DayTime,
        }
    }

    /// Returns `true` if the qualifier ends in a SECOND field (which may have a fractional
    /// precision).
    pub fn has_seconds(&self) -> bool {
        matches!(
            self,
            IntervalQualifier::Second
                | IntervalQualifier::DayToSecond
                | IntervalQualifier::HourToSecond
                | IntervalQualifier::MinuteToSecond
        )
    }
}

/// The type of an interval value: its qualifier and precisions.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct IntervalType {
    qualifier: IntervalQualifier,
    /// Leading field precision.
    precision: usize,
    /// Fractional seconds precision; 0 when the qualifier has no SECOND field.
    fractional_precision: usize,
}

impl IntervalType {
    /// An interval type. `fractional_precision` is ignored (stored as 0) when `qualifier` has no
    /// SECOND field, so that e.g. `INTERVAL YEAR(2)` has a single representation.
    #[inline]
    pub fn new(
        qualifier: IntervalQualifier,
        precision: usize,
        fractional_precision: usize,
    ) -> Self {
        let fractional_precision = if qualifier.has_seconds() {
            fractional_precision
        } else {
            0
        };
        IntervalType {
            qualifier,
            precision,
            fractional_precision,
        }
    }

    #[inline]
    pub fn qualifier(&self) -> IntervalQualifier {
        self.qualifier
    }

    #[inline]
    pub fn precision(&self) -> usize {
        self.precision
    }

    #[inline]
    pub fn fractional_precision(&self) -> usize {
        self.fractional_precision
    }

    #[inline]
    pub fn family(&self) -> IntervalFamily {
        self.qualifier.family()
    }
}

impl Display for IntervalType {
    /// SQL-like text, e.g. `Interval(Day(2) To Second(6))`, `Interval(Year(2) To Month)`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (p, fp) = (self.precision, self.fractional_precision);
        let qualifier = match self.qualifier {
            IntervalQualifier::Year => format!("Year({p})"),
            IntervalQualifier::Month => format!("Month({p})"),
            IntervalQualifier::YearToMonth => format!("Year({p}) To Month"),
            IntervalQualifier::Day => format!("Day({p})"),
            IntervalQualifier::Hour => format!("Hour({p})"),
            IntervalQualifier::Minute => format!("Minute({p})"),
            IntervalQualifier::Second => format!("Second({p},{fp})"),
            IntervalQualifier::DayToHour => format!("Day({p}) To Hour"),
            IntervalQualifier::DayToMinute => format!("Day({p}) To Minute"),
            IntervalQualifier::DayToSecond => format!("Day({p}) To Second({fp})"),
            IntervalQualifier::HourToMinute => format!("Hour({p}) To Minute"),
            IntervalQualifier::HourToSecond => format!("Hour({p}) To Second({fp})"),
            IntervalQualifier::MinuteToSecond => format!("Minute({p}) To Second({fp})"),
        };
        write!(f, "Interval({qualifier})")
    }
}

/// The payload of [`Static::Variant`]: the name of the value's encoding (e.g. `ion`).
///
/// As in partiql-lang-kotlin (whose `VARIANT` type compares only its type code), the encoding
/// does not take part in equality or hashing: every `VARIANT` type equals every other.
#[derive(Educe, Eq, Debug, Clone)]
#[educe(PartialEq, Hash)]
pub struct VariantType {
    #[educe(PartialEq(ignore), Hash(ignore))]
    encoding: String,
}

impl VariantType {
    /// A variant type for values in the given encoding.
    #[inline]
    pub fn new(encoding: impl Into<String>) -> Self {
        VariantType {
            encoding: encoding.into(),
        }
    }

    /// The name of the encoding.
    #[inline]
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
}

impl Display for VariantType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variant({})", self.encoding)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        PartiqlNoIdShapeBuilder, PartiqlShape, PartiqlShapeBuilder, ShapeBuilderExtensions, Static,
        StructConstraint, StructField, StructType,
    };
    use indexmap::IndexSet;

    #[test]
    fn union() {
        let mut bld = PartiqlNoIdShapeBuilder::default();

        let expect_int = bld.new_static(Static::Int);
        assert_eq!(
            expect_int,
            [type_int!(bld), type_int!(bld)].into_union(&mut bld)
        );

        let numbers = [bld.new_static(Static::Int), bld.new_static(Static::Float32)];
        let expect_nums = bld.any_of(numbers);
        assert_eq!(
            expect_nums,
            [type_int!(bld), type_float32!(bld)].into_union(&mut bld)
        );
        assert_eq!(
            expect_nums,
            [
                [type_int!(bld), type_float32!(bld)].into_union(&mut bld),
                [type_int!(bld), type_float32!(bld)].into_union(&mut bld),
            ]
            .into_any_of(&mut bld)
        );
        assert_eq!(
            expect_nums,
            [
                [type_int!(bld), type_float32!(bld)].into_union(&mut bld),
                [type_int!(bld), type_float32!(bld)].into_union(&mut bld),
                [
                    [type_int!(bld), type_float32!(bld)].into_union(&mut bld),
                    [type_int!(bld), type_float32!(bld)].into_union(&mut bld)
                ]
                .into_any_of(&mut bld)
            ]
            .into_any_of(&mut bld)
        );
    }

    #[test]
    fn unique_node_ids() {
        let mut bld = PartiqlShapeBuilder::default();
        let age_field = struct_fields![("age", type_int!(bld))];
        let details = type_struct![bld, IndexSet::from([age_field])];

        let fields = [
            StructField::new("id", type_int!(bld)),
            StructField::new("name", type_string!(bld)),
            StructField::new("details", details.clone()),
        ];

        let row = type_struct![
            bld,
            IndexSet::from([
                StructConstraint::Fields(IndexSet::from(fields)),
                StructConstraint::Open(false)
            ])
        ];

        let shape = type_bag![bld, row.clone()];

        let mut ids = collect_ids(shape);
        ids.sort_unstable();
        assert!(ids.windows(2).all(|w| w[0] != w[1]));
    }

    fn collect_ids(row: PartiqlShape) -> Vec<u32> {
        let mut out = vec![];
        match row {
            PartiqlShape::Dynamic => {}
            PartiqlShape::AnyOf(anyof) => {
                for shape in anyof.types {
                    out.push(collect_ids(shape));
                }
            }
            PartiqlShape::Static(static_type) => {
                out.push(vec![static_type.id.0]);
                match static_type.ty {
                    Static::Struct(struct_type) => {
                        for f in struct_type.fields() {
                            out.push(collect_ids(f.ty.clone()));
                        }
                    }
                    Static::Bag(bag_type) => out.push(collect_ids(*bag_type.element_type)),
                    Static::Array(array_type) => out.push(collect_ids(*array_type.element_type)),
                    _ => {}
                }
            }
            PartiqlShape::Undefined => {}
        }
        out.into_iter().flatten().collect()
    }
}
