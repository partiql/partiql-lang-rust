//! Catalog, session and routine-signature API for typed planning.
//!
//! The catalog interface the typed planner resolves names against, and the signatures of
//! routines, with [`partiql_types::PartiqlShape`] as the type model. It is additive: the existing [`crate::catalog`] traits are unchanged, and
//! [`LegacyCatalogAdapter`] presents any [`crate::catalog::SharedCatalog`] as a [`TypedCatalog`].
//!
//! - [`Identifier`]: a name as written in a query, whose [`IdentPart`]s are regular (match
//!   ignoring case) or delimited (match exactly); [`Name`]: a catalog's canonical, exact name.
//! - [`TypedCatalog`] / [`RoutineCatalog`]: tables ([`Table`]) and routine overloads
//!   ([`FnOverload`], [`AggOverload`], [`RoutineBinding`]).
//! - [`Session`]: the current catalog and namespace, the [`Catalogs`], and the routine [`Path`].
//! - [`resolve_table`]: the planner's lookup order for a table identifier.
//!
//! Routine lookup (walking [`Session::path`], qualified calls, the by-text fallback, scalar-versus-aggregate classification) and overload selection are not here
//! yet; they come with the builtin routines. [`Session::path`] and [`RoutineCatalog`] are the
//! inputs they will use.

mod catalog;
mod identifier;
mod legacy;
mod resolve;
mod routine;
mod session;

pub use catalog::{
    CatalogNameAmbiguous, Catalogs, RoutineCatalog, SystemCatalog, Table, TypedCatalog,
    SYSTEM_CATALOG_NAME,
};
pub use identifier::{
    equals_ignore_case, IdentPart, Identifier, Name, Namespace, Path, PathEntryNotPresent,
};
pub use legacy::LegacyCatalogAdapter;
pub use resolve::{resolve_table, ResolveError, ResolvedTable};
pub use routine::{
    AggOverload, FixedAggOverload, FixedFnOverload, FnOverload, Parameter, RoutineBinding,
    RoutineBindingError, RoutineOverloadSignature, RoutineSignature,
};
pub use session::{Session, SessionBuilder, SessionError};
