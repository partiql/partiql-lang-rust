use crate::call_defs::CallDef;
use crate::context::SessionContext;
use crate::extension::ExtensionResultError;
use partiql_value::Value;
use std::borrow::Cow;
use std::fmt::Debug;

pub type BaseTableExprResultValueIter<'a> =
    Box<dyn 'a + Iterator<Item = Result<Value, ExtensionResultError>>>;
pub type BaseTableExprResult<'a> = Result<BaseTableExprResultValueIter<'a>, ExtensionResultError>;

pub trait BaseTableExpr: Debug {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        ctx: &'c dyn SessionContext,
    ) -> BaseTableExprResult<'c>;
}

pub trait BaseTableFunctionInfo: Debug + Send + Sync {
    fn call_def(&self) -> &CallDef;
    fn plan_eval(&self) -> Box<dyn BaseTableExpr>;
}

#[derive(Debug)]
pub struct TableFunction {
    info: Box<dyn BaseTableFunctionInfo>,
}

impl TableFunction {
    #[must_use]
    pub fn new(info: Box<dyn BaseTableFunctionInfo>) -> Self {
        TableFunction { info }
    }

    pub fn call_def(&self) -> &CallDef {
        self.info.call_def()
    }

    pub fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        self.info.plan_eval()
    }
}
