use crate::buffer::{infer_buffer_type, BufferType};
use crate::{IonExtensionError, IonTableExprResult};
use ion_rs::IonError;
use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::context::SessionContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_catalog::table_fn::{BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo};
use partiql_extension_ion::boxed_ion::{BoxedIonError, BoxedIonType};
use partiql_logical as logical;
use partiql_value::Value;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

impl From<BoxedIonError> for IonExtensionError {
    fn from(err: BoxedIonError) -> IonExtensionError {
        match err {
            err @ BoxedIonError::IonReadError(_) => IonExtensionError::IonReadError(err.into()),
            err @ BoxedIonError::NotASequence { .. } => IonExtensionError::DataError(err.into()),
            _ => IonExtensionError::Unknown,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ScanIonFunction {
    call_def: CallDef,
}

/// `scan_ion` scans ion data lazily, wrapping it into PartiQL Boxed Variants
impl ScanIonFunction {
    pub fn new() -> Self {
        ScanIonFunction {
            call_def: CallDef {
                names: vec!["scan_ion"],
                overloads: vec![CallSpec {
                    input: vec![CallSpecArg::Positional],
                    output: Box::new(|args| {
                        logical::ValueExpr::Call(logical::CallExpr {
                            name: logical::CallName::ByName("scan_ion".to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl BaseTableFunctionInfo for ScanIonFunction {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(EvalFnScanIon {})
    }
}

#[derive(Debug)]
pub(crate) struct EvalFnScanIon {}

impl BaseTableExpr for EvalFnScanIon {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext<'c>,
    ) -> BaseTableExprResult<'c> {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(path) => Ok(Box::new(
                    parse_ion_file(path)?.map(|it| it.map_err(Into::into)),
                )),
                _ => {
                    let error = IonExtensionError::FunctionError(
                        "expected string path argument".to_string(),
                    );
                    Err(ExtensionResultError::ReadError(error.into()))
                }
            }
        } else {
            let error = IonExtensionError::FunctionError("expected path argument".to_string());
            Err(ExtensionResultError::ReadError(error.into()))
        }
    }
}

fn parse_ion_file<'a>(path: &str) -> IonTableExprResult<'a> {
    let path = PathBuf::from(path).canonicalize()?;
    let file = File::open(path)?;

    match infer_buffer_type(file) {
        BufferType::Gzip(gzip) => parse_ion_buff(gzip),
        BufferType::Zstd(zstd) => parse_ion_buff(zstd),
        BufferType::Unknown(buff) => parse_ion_buff(buff),
    }
}

fn parse_ion_buff<'a, I: 'a + Read + 'static>(input: BufReader<I>) -> IonTableExprResult<'a> {
    let iter = BoxedIonType {}.construct_buffered(input)?.try_into_iter()?;
    let iter = iter.map(|value| match value {
        Ok(v) => Ok(v.into_value()),
        Err(e) => Err(match e {
            err @ IonError::Conversion(_) => IonExtensionError::DataError(Box::new(err)),
            err => IonExtensionError::IonReadError(Box::new(err)),
        }),
    });
    Ok(Box::new(iter))
}
