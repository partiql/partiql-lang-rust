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
use std::io::{BufReader, Read, Seek, SeekFrom};
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

    parse_ion_scan(file)
}

fn parse_ion_scan<'a>(mut reader: impl 'a + Read + Seek + 'static) -> IonTableExprResult<'a> {
    let mut header: [u8; 4] = [0; 4];
    reader.read_exact(&mut header).expect("file header");
    reader.seek(SeekFrom::Start(0)).expect("file seek");

    if header.starts_with(&[0x1f, 0x8b]) {
        let decoder = flate2::read::GzDecoder::new(reader);
        let buffered = BufReader::new(decoder);
        parse_ion_buff(buffered)
    } else if header.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
        let decoder = zstd::Decoder::new(reader).expect("zstd reader creation");
        let buffered = BufReader::new(decoder);
        parse_ion_buff(buffered)
    } else {
        let buffered = BufReader::new(reader);
        parse_ion_buff(buffered)
    }
}

fn parse_ion_buff<'a, I: 'a + Read + 'static>(input: BufReader<I>) -> IonTableExprResult<'a> {
    let iter = BoxedIonType {}.stream_from_read(input)?.try_into_iter()?;
    let iter = iter.map(|value| match value {
        Ok(v) => Ok(v.into_value()),
        Err(e) => Err(match e {
            err @ IonError::Conversion(_) => IonExtensionError::DataError(Box::new(err)),
            err => IonExtensionError::IonReadError(Box::new(err)),
        }),
    });
    Ok(Box::new(iter))
}
