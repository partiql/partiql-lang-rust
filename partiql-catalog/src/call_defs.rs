use partiql_logical as logical;
use partiql_logical::ValueExpr;

use std::fmt::{Debug, Formatter};
use thiserror::Error;

use crate::scalar_fn::ScalarFnExpr;
use unicase::UniCase;

/// An error that can happen during call lookup
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CallLookupError {
    /// Invalid number of arguments to the function call.
    #[error("Invalid number of arguments: {0}")]
    InvalidNumberOfArguments(String),
}

#[derive(Debug)]
pub struct CallDef {
    pub names: Vec<&'static str>,
    pub overloads: Vec<CallSpec>,
}

impl CallDef {
    // Used when lowering AST -> plan
    pub fn lookup(&self, args: &[CallArgument], name: &str) -> Result<ValueExpr, CallLookupError> {
        'overload: for overload in &self.overloads {
            let formals = &overload.input;
            if formals.len() != args.len() {
                continue 'overload;
            }

            let mut actuals = vec![];
            for i in 0..formals.len() {
                let formal = &formals[i];
                let actual = &args[i];
                if let Some(vexpr) = formal.transform(actual) {
                    actuals.push(vexpr);
                } else {
                    continue 'overload;
                }
            }

            return Ok((overload.output)(actuals));
        }
        Err(CallLookupError::InvalidNumberOfArguments(name.into()))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CallArgument {
    Positional(ValueExpr),
    Named(String, ValueExpr),
    Star,
}

#[derive(Debug, Copy, Clone)]
pub enum CallSpecArg {
    Positional,
    Named(UniCase<&'static str>),
}

impl CallSpecArg {
    pub(crate) fn transform(&self, arg: &CallArgument) -> Option<ValueExpr> {
        match (self, arg) {
            (CallSpecArg::Positional, CallArgument::Positional(ve)) => Some(ve.clone()),
            (CallSpecArg::Named(formal_name), CallArgument::Named(arg_name, ve)) => {
                if formal_name == &UniCase::new(arg_name.as_str()) {
                    Some(ve.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct CallSpec {
    pub input: Vec<CallSpecArg>,
    pub output: Box<dyn Fn(Vec<ValueExpr>) -> logical::ValueExpr + Send + Sync>,
}

impl Debug for CallSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CallSpec [{:?}]", &self.input)
    }
}

#[derive(Debug, Clone)]
pub struct ScalarFnCallDef {
    pub names: Vec<&'static str>,
    pub overloads: ScalarFnCallSpecs,
}

pub type ScalarFnCallSpecs = Vec<ScalarFnCallSpec>;

#[derive(Clone)]
pub struct ScalarFnCallSpec {
    // TODO:  Include Scalar Function attributes (e.g., isNullCall and isMissingCall, etc.): https://github.com/partiql/partiql-lang-rust/issues/499
    pub input: Vec<CallSpecArg>,
    pub output: Box<dyn ScalarFnExpr>,
}

impl Debug for ScalarFnCallSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScalarFnCallSpec [{:?}]", &self.input)
    }
}
