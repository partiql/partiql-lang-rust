use partiql_catalog::call_defs::{CallArgument, CallLookupError, CallSpecArg, ScalarFnCallSpecs};
use partiql_catalog::catalog::{FunctionEntry, FunctionEntryFunction};
use partiql_common::catalog::ObjectId;
use partiql_logical::{CallExpr, CallName, ValueExpr};
use unicase::UniCase;

pub(crate) trait Function {
    fn resolve(&self, name: &str, args: &[CallArgument]) -> Result<ValueExpr, CallLookupError>;
}

impl Function for FunctionEntry<'_> {
    fn resolve(&self, name: &str, args: &[CallArgument]) -> Result<ValueExpr, CallLookupError> {
        let oid = self.id();
        match self.entry() {
            FunctionEntryFunction::Table(tbl) => {
                tbl.call_def().lookup(args, name).map_err(Into::into)
            }
            FunctionEntryFunction::Scalar(scfn) => {
                ScalarFnResolver { oid, scfn }.resolve(name, args)
            }
            FunctionEntryFunction::Aggregate() => {
                todo!("Aggregate function resolution")
            }
        }
    }
}

struct ScalarFnResolver<'a> {
    pub oid: &'a ObjectId,
    pub scfn: &'a ScalarFnCallSpecs,
}

impl Function for ScalarFnResolver<'_> {
    fn resolve(&self, name: &str, args: &[CallArgument]) -> Result<ValueExpr, CallLookupError> {
        let oid = self.oid;
        let overloads = self.scfn;
        'overload: for (idx, overload) in overloads.iter().enumerate() {
            let formals = &overload.input;
            if formals.len() != args.len() {
                continue 'overload;
            }

            let mut actuals = vec![];
            for i in 0..formals.len() {
                let formal = &formals[i];
                let actual = &args[i];
                if let Some(vexpr) = formal.resolve_argument(actual) {
                    actuals.push(vexpr);
                } else {
                    continue 'overload;
                }
            }

            // the current overload matches the argument arity and types
            return Ok(ValueExpr::Call(CallExpr {
                name: CallName::ById(name.to_string(), *oid, idx),
                arguments: actuals.into_iter().cloned().collect(),
            }));
        }
        Err(CallLookupError::InvalidNumberOfArguments(name.into()))
    }
}

pub(crate) trait FormalArg {
    fn resolve_argument<'a>(&self, arg: &'a CallArgument) -> Option<&'a ValueExpr>;
}

impl FormalArg for CallSpecArg {
    fn resolve_argument<'a>(&self, arg: &'a CallArgument) -> Option<&'a ValueExpr> {
        match (self, arg) {
            (CallSpecArg::Positional, CallArgument::Positional(ve)) => Some(ve),
            (CallSpecArg::Named(formal_name), CallArgument::Named(arg_name, ve)) => {
                if formal_name == &UniCase::new(arg_name.as_str()) {
                    Some(ve)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
