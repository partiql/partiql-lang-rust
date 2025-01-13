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
}

/// Result of attempts to encode to Ion.
pub type ShapeResult<T> = Result<T, ShapeResultError>;

pub trait Type {}

impl Type for StaticType {}

fn indexset_hash<H, T>(set: &IndexSet<T>, state: &mut H)
where
    H: Hasher,
    T: Hash,
{
    for val in set {
        val.hash(state)
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
macro_rules! struct_fields {
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        $crate::StructConstraint::Fields([$(($x, $y).into()),+].into())
    );
}

#[macro_export]
macro_rules! type_bag {
    ($bld:expr) => {
        $bld.new_bag(BagType::new_any());
    };
    ($bld:expr, $elem:expr) => {{
        let elem = $elem;
        $bld.new_bag($crate::BagType::new(Box::new(elem)))
    }};
}

#[macro_export]
macro_rules! type_array {
    ($bld:expr) => {
        $bld.new_array(ArrayType::new_any());
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
        }) = self
        {
            Ok(StaticType {
                id: *id,
                ty: Static::Bool,
                nullable: *n,
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
        })
    }

    #[inline]
    pub fn new_non_nullable_static(&mut self, ty: Static) -> PartiqlShape {
        let id = self.id_gen.next_id();
        PartiqlShape::Static(StaticType {
            id,
            ty,
            nullable: false,
        })
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

    #[inline]
    pub fn as_non_nullable(&mut self, shape: &PartiqlShape) -> Option<PartiqlShape> {
        if let PartiqlShape::Static(stype) = shape {
            Some(self.new_non_nullable_static(stype.ty.clone()))
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StaticType {
    id: NodeId,
    ty: Static,
    nullable: bool,
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

    // Container Types
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    // TODO Add BitString, ByteString, Blob, Clob, and Graph types
}

impl Static {
    pub fn is_scalar(&self) -> bool {
        !matches!(self, Static::Struct(_) | Static::Bag(_) | Static::Array(_))
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Static::Bag(_) | Static::Array(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Static::Struct(_))
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
            Static::StringFixed(_) => {
                todo!()
            }
            Static::StringVarying(_) => {
                todo!()
            }
            Static::DateTime => write!(f, "DateTime"),
            Static::Struct(inner) => std::fmt::Display::fmt(inner, f),
            Static::Bag(inner) => std::fmt::Display::fmt(inner, f),
            Static::Array(inner) => std::fmt::Display::fmt(inner, f),
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

    pub fn fields_set(&self) -> IndexSet<StructField> {
        self.constraints
            .iter()
            .flat_map(|c| {
                if let StructConstraint::Fields(fields) = c.clone() {
                    fields
                } else {
                    Default::default()
                }
            })
            .collect()
    }

    pub fn fields(&self) -> impl Iterator<Item = &StructField> {
        self.constraints
            .iter()
            .filter_map(|c| {
                if let StructConstraint::Fields(fields) = c {
                    Some(fields)
                } else {
                    None
                }
            })
            .flat_map(|f| f.iter())
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
