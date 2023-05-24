use partiql_logical as logical;
use partiql_logical::ValueExpr;

use std::fmt::{Debug, Formatter};
use thiserror::Error;

use unicase::UniCase;

/// An error that can happen during call lookup
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CallLookupError {
    /// Invalid number of arguments to the function call.
    #[error("Invalid number of arguments: {0}")]
    InvalidNumberOfArguments(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CallArgument {
    Positional(ValueExpr),
    Named(String, ValueExpr),
}

#[derive(Debug)]
pub struct CallDef {
    pub names: Vec<&'static str>,
    pub overloads: Vec<CallSpec>,
}

impl CallDef {
    pub fn lookup(
        &self,
        args: &Vec<CallArgument>,
        name: &str,
    ) -> Result<ValueExpr, CallLookupError> {
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

impl CallSpecArg {}

pub struct CallSpec {
    pub input: Vec<CallSpecArg>,
    pub output: Box<dyn Fn(Vec<ValueExpr>) -> logical::ValueExpr + Send + Sync>,
}

impl Debug for CallSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CallSpec [{:?}]", &self.input)
    }
}
