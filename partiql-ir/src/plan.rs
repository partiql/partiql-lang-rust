//! The plan root (`Plan.java:8`, `Action.java:9`, `Version.java:10`, relative to
//! `partiql-plan/src/main/java/org/partiql/plan/` in partiql-lang-kotlin v1.6.1).

use crate::rex::Rex;

/// A typed plan: a version and the action it performs.
///
/// Equality (`==`) is structural and, as `PartiqlShape`'s, ignores the unspecified-parameter
/// flags of a shape (`TypeParams`) and the encoding of a `Variant`; compare [`Plan::dump`] or
/// `Debug` to see those.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    version: Version,
    action: Action,
}

impl Plan {
    pub fn new(version: Version, action: Action) -> Self {
        Plan { version, action }
    }

    /// A [`Version::V1`] plan of a query whose result is `rex`.
    pub fn query(rex: Rex) -> Self {
        Plan::new(Version::V1, Action::Query { rex })
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    /// Takes the plan apart, to build a changed copy with [`Plan::new`].
    pub fn into_parts(self) -> (Version, Action) {
        (self.version, self.action)
    }
}

/// What a plan does.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Action {
    /// A query; its result is the value of `rex` (e.g. a [`crate::RexOp::Select`]).
    #[non_exhaustive]
    Query { rex: Rex },
}

impl Action {
    pub fn query(rex: Rex) -> Self {
        Action::Query { rex }
    }
}

/// The plan format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Version {
    V1,
}
