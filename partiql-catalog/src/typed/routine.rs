//! Typed routine signatures: what a catalog says about its scalar functions and aggregations.
//!
//! Ported from partiql-lang-kotlin v1.6.1 `org.partiql.spi.function.{Parameter,
//! RoutineOverloadSignature, RoutineSignature, FnOverload, AggOverload}` and
//! `org.partiql.spi.catalog.RoutineBinding`, with [`PartiqlShape`] in place of `PType`. These
//! types carry signatures only; choosing an overload for a call is the planner's job.

use super::identifier::Name;
use partiql_types::PartiqlShape;
use std::fmt::Debug;
use std::sync::Arc;

/// A named, typed routine parameter.
#[derive(Debug, Clone)]
pub struct Parameter {
    name: String,
    ty: PartiqlShape,
}

impl Parameter {
    pub fn new(name: impl Into<String>, ty: PartiqlShape) -> Self {
        Parameter {
            name: name.into(),
            ty,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn ty(&self) -> &PartiqlShape {
        &self.ty
    }
}

/// The signature an overload is declared with: its name and parameter types.
#[derive(Debug, Clone)]
pub struct RoutineOverloadSignature {
    name: String,
    parameter_types: Vec<PartiqlShape>,
}

impl RoutineOverloadSignature {
    pub fn new(name: impl Into<String>, parameter_types: Vec<PartiqlShape>) -> Self {
        RoutineOverloadSignature {
            name: name.into(),
            parameter_types,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn arity(&self) -> usize {
        self.parameter_types.len()
    }

    #[must_use]
    pub fn parameter_types(&self) -> &[PartiqlShape] {
        &self.parameter_types
    }
}

/// The concrete signature of a routine instance: its parameters, its return type, and two flags
/// (partiql-lang-kotlin's IS NULL CALL and IS MISSING CALL). With `is_null_call`, a NULL argument
/// makes the call return NULL without invoking it; with `is_missing_call`, a MISSING argument
/// makes it return MISSING. NULL and MISSING are absent values, not types.
#[derive(Debug, Clone)]
pub struct RoutineSignature {
    name: String,
    parameters: Vec<Parameter>,
    returns: PartiqlShape,
    is_null_call: bool,
    is_missing_call: bool,
}

impl RoutineSignature {
    /// A signature that is both null-call and missing-call, as partiql-lang-kotlin's default.
    pub fn new(name: impl Into<String>, parameters: Vec<Parameter>, returns: PartiqlShape) -> Self {
        RoutineSignature {
            name: name.into(),
            parameters,
            returns,
            is_null_call: true,
            is_missing_call: true,
        }
    }

    /// Parameters named `arg0`, `arg1`, ... of the given types, as partiql-lang-kotlin's builders
    /// name them.
    pub fn with_types(
        name: impl Into<String>,
        parameter_types: impl IntoIterator<Item = PartiqlShape>,
        returns: PartiqlShape,
    ) -> Self {
        let parameters = parameter_types
            .into_iter()
            .enumerate()
            .map(|(i, ty)| Parameter::new(format!("arg{i}"), ty))
            .collect();
        RoutineSignature::new(name, parameters, returns)
    }

    #[must_use]
    pub fn with_null_call(mut self, is_null_call: bool) -> Self {
        self.is_null_call = is_null_call;
        self
    }

    #[must_use]
    pub fn with_missing_call(mut self, is_missing_call: bool) -> Self {
        self.is_missing_call = is_missing_call;
        self
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    #[must_use]
    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    #[must_use]
    pub fn returns(&self) -> &PartiqlShape {
        &self.returns
    }

    #[must_use]
    pub fn is_null_call(&self) -> bool {
        self.is_null_call
    }

    #[must_use]
    pub fn is_missing_call(&self) -> bool {
        self.is_missing_call
    }

    /// The overload signature this instance has: its name and parameter types.
    #[must_use]
    pub fn overload_signature(&self) -> RoutineOverloadSignature {
        RoutineOverloadSignature::new(
            self.name.clone(),
            self.parameters.iter().map(|p| p.ty.clone()).collect(),
        )
    }
}

/// A scalar function overload (partiql-lang-kotlin `FnOverload`).
pub trait FnOverload: Debug + Send + Sync {
    /// The signature the overload is declared with.
    fn signature(&self) -> &RoutineOverloadSignature;

    /// The concrete instance for arguments of types `args`, or `None` if there is none. An
    /// argument with no present value, such as a NULL or MISSING literal, has the type
    /// [`PartiqlShape::Undefined`].
    fn instance(&self, args: &[PartiqlShape]) -> Option<RoutineSignature>;
}

/// An aggregation overload (partiql-lang-kotlin `AggOverload`).
pub trait AggOverload: Debug + Send + Sync {
    /// The signature the overload is declared with.
    fn signature(&self) -> &RoutineOverloadSignature;

    /// The concrete instance for arguments of types `args`, or `None` if there is none. An
    /// argument with no present value, such as a NULL or MISSING literal, has the type
    /// [`PartiqlShape::Undefined`].
    fn instance(&self, args: &[PartiqlShape]) -> Option<RoutineSignature>;
}

/// An overload with one instance for any arguments, as partiql-lang-kotlin's
/// `FnOverload.Builder` and `AggOverload.Builder` build.
#[derive(Debug, Clone)]
pub struct FixedOverload {
    signature: RoutineOverloadSignature,
    instance: RoutineSignature,
}

impl FixedOverload {
    #[must_use]
    pub fn new(instance: RoutineSignature) -> Self {
        FixedOverload {
            signature: instance.overload_signature(),
            instance,
        }
    }
}

impl FnOverload for FixedOverload {
    fn signature(&self) -> &RoutineOverloadSignature {
        &self.signature
    }

    fn instance(&self, _args: &[PartiqlShape]) -> Option<RoutineSignature> {
        Some(self.instance.clone())
    }
}

impl AggOverload for FixedOverload {
    fn signature(&self) -> &RoutineOverloadSignature {
        &self.signature
    }

    fn instance(&self, _args: &[PartiqlShape]) -> Option<RoutineSignature> {
        Some(self.instance.clone())
    }
}

/// The errors of [`RoutineBinding::new`].
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RoutineBindingError {
    #[error("Routine binding canonical name cannot contain an empty part")]
    EmptyNamePart,
    #[error("Routine binding overloads cannot be empty")]
    NoOverloads,
}

/// The overloads a routine identifier resolves to, with the routine's canonical name.
#[derive(Debug)]
pub struct RoutineBinding<T: ?Sized> {
    canonical_name: Name,
    overloads: Vec<Arc<T>>,
}

impl<T: ?Sized> Clone for RoutineBinding<T> {
    fn clone(&self) -> Self {
        RoutineBinding {
            canonical_name: self.canonical_name.clone(),
            overloads: self.overloads.clone(),
        }
    }
}

impl<T: ?Sized> RoutineBinding<T> {
    /// A binding of `canonical_name`, none of whose parts may be empty, to one or more overloads.
    pub fn new(canonical_name: Name, overloads: Vec<Arc<T>>) -> Result<Self, RoutineBindingError> {
        if canonical_name.parts().any(str::is_empty) {
            return Err(RoutineBindingError::EmptyNamePart);
        }
        if overloads.is_empty() {
            return Err(RoutineBindingError::NoOverloads);
        }
        Ok(RoutineBinding {
            canonical_name,
            overloads,
        })
    }

    #[must_use]
    pub fn canonical_name(&self) -> &Name {
        &self.canonical_name
    }

    #[must_use]
    pub fn overloads(&self) -> &[Arc<T>] {
        &self.overloads
    }
}
