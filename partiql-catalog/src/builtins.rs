use crate::call_defs::{CallDef, CallSpec, CallSpecArg};
use crate::{ScalarExpr, ScalarExprResult, ScalarFunctionInfo};
use partiql_logical::{CallExpr, CallName, ValueExpr};
use partiql_value::Value;
use std::borrow::Cow;

#[inline]
#[track_caller]
fn string_transform<FnTransform>(value: &Value, transform_fn: FnTransform) -> Value
where
    FnTransform: Fn(&str) -> Value,
{
    match value {
        Value::Null => Value::Null,
        Value::String(s) => transform_fn(s.as_ref()),
        _ => Value::Missing,
    }
}

#[derive(Debug)]
pub(crate) struct CharLenFunction {
    call_def: CallDef,
}

impl CharLenFunction {
    pub fn new() -> Self {
        CharLenFunction {
            call_def: CallDef {
                names: vec!["char_length", "character_length"],
                overloads: vec![CallSpec {
                    input: vec![CallSpecArg::Positional],
                    output: Box::new(|args| {
                        ValueExpr::Call(CallExpr {
                            name: CallName::ByName("char_length".to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl ScalarFunctionInfo for CharLenFunction {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn ScalarExpr> {
        Box::new(EvalFnCharLength {})
    }
}

/// Represents a built-in character length string function, e.g. `char_length('123456789')`.
#[derive(Debug)]
pub(crate) struct EvalFnCharLength {}

impl ScalarExpr for EvalFnCharLength {
    fn evaluate(&self, args: &[Cow<Value>]) -> ScalarExprResult {
        if args.len() != 1 {
            todo!("Error plumbing")
        }
        let value = args.first().unwrap();
        let transformed = string_transform(value.as_ref(), |s| s.chars().count().into());
        Ok(Cow::Owned(transformed))
    }
}
