use crate::buffer::{infer_buffer_type, BufferType};
use crate::{IonExtensionError, IonTableExprResult, IonTableExprResultValueIter};
use ion_rs_old::data_source::ToIonDataSource;
use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::context::SessionContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_catalog::table_fn::{BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo};
use partiql_extension_ion::decode::{IonDecodeError, IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical as logical;
use partiql_value::Value;
use std::borrow::Cow;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct ReadIonFunction {
    call_def: CallDef,
}

/// `read_ion` reads ion data eagerly and converts it to PartiQL Values at read time.
impl ReadIonFunction {
    pub fn new() -> Self {
        ReadIonFunction {
            call_def: CallDef {
                names: vec!["read_ion"],
                overloads: vec![CallSpec {
                    input: vec![CallSpecArg::Positional],
                    output: Box::new(|args| {
                        logical::ValueExpr::Call(logical::CallExpr {
                            name: logical::CallName::ByName("read_ion".to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl BaseTableFunctionInfo for ReadIonFunction {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(EvalFnReadIon {})
    }
}

#[derive(Debug)]
pub(crate) struct EvalFnReadIon {}

impl BaseTableExpr for EvalFnReadIon {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
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

fn parse_ion_buff<'a, I: 'a + ToIonDataSource>(input: I) -> IonTableExprResult<'a> {
    let err_map = |e: IonDecodeError| match e {
        err @ IonDecodeError::IonReaderError(_) => IonExtensionError::IonStreamError(err),
        err @ IonDecodeError::UnsupportedType(_) => IonExtensionError::DataError(err.into()),
        err @ IonDecodeError::ConversionError(_) => IonExtensionError::DataError(err.into()),
        err @ IonDecodeError::StreamError(_) => IonExtensionError::IonStreamError(err),
        err @ IonDecodeError::Unknown => IonExtensionError::IonStreamError(err),
        err => IonExtensionError::IonStreamError(err),
    };
    let reader = ion_rs_old::ReaderBuilder::new().build(input).unwrap();
    let decoder =
        IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion)).build(reader);
    let decoder = decoder.map_err(err_map)?.map(move |it| it.map_err(err_map));
    Ok(Box::new(decoder) as IonTableExprResultValueIter<'_>)
}
