use crate::call_defs::{CallSpecArg, ScalarFnCallDef, ScalarFnCallSpec};
use crate::context::SessionContext;

use crate::extension::ExtensionResultError;
use dyn_clone::DynClone;
use partiql_common::FN_VAR_ARG_MAX;
use partiql_value::Value;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

pub type ScalarFnExprResultValue<'a> = Cow<'a, Value>;
pub type ScalarFnExprResult<'a> = Result<ScalarFnExprResultValue<'a>, ExtensionResultError>;

pub trait ScalarFnExpr: DynClone + Debug {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        ctx: &'c dyn SessionContext,
    ) -> ScalarFnExprResult<'c>;
}

dyn_clone::clone_trait_object!(ScalarFnExpr);

pub trait ScalarFunctionInfo: Debug {
    fn call_def(&self) -> &ScalarFnCallDef;

    fn into_call_def(self: Box<Self>) -> ScalarFnCallDef;
}

pub struct SimpleScalarFunctionInfo {
    call_def: ScalarFnCallDef,
}

impl SimpleScalarFunctionInfo {
    pub fn new(call_def: ScalarFnCallDef) -> Self {
        Self { call_def }
    }
}

impl ScalarFunctionInfo for SimpleScalarFunctionInfo {
    fn call_def(&self) -> &ScalarFnCallDef {
        &self.call_def
    }

    fn into_call_def(self: Box<Self>) -> ScalarFnCallDef {
        self.call_def
    }
}

impl Debug for SimpleScalarFunctionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.call_def.fmt(f)
    }
}

#[derive(Debug)]
pub struct ScalarFunction {
    info: Box<dyn ScalarFunctionInfo>,
}

impl ScalarFunction {
    pub fn new(info: Box<dyn ScalarFunctionInfo>) -> Self {
        ScalarFunction { info }
    }

    pub fn call_def(&self) -> &ScalarFnCallDef {
        self.info.call_def()
    }

    pub fn into_call_def(self) -> ScalarFnCallDef {
        self.info.into_call_def()
    }
}

pub fn vararg_scalar_fn_overloads(scalar_fn_expr: Box<dyn ScalarFnExpr>) -> Vec<ScalarFnCallSpec> {
    (1..=FN_VAR_ARG_MAX)
        .map(|n| ScalarFnCallSpec {
            input: std::iter::repeat_n(CallSpecArg::Positional, n).collect(),
            output: scalar_fn_expr.clone(),
        })
        .collect()
}
