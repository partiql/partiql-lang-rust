use miette::{Diagnostic, LabeledSpan, SourceCode};
use partiql_parser::ParseError;
use partiql_source_map::location::{BytePosition, Location};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CLIError {
    #[error("PartiQL syntax error:")]
    SyntaxError {
        src: String,
        msg: String,
        loc: Location<BytePosition>,
    },
    // TODO add github issue link
    #[error("Internal Compiler Error - please report this.")]
    InternalCompilerError { src: String },
}

impl Diagnostic for CLIError {
    fn source_code(&self) -> Option<&dyn SourceCode> {
        match self {
            CLIError::SyntaxError { src, .. } => Some(src),
            CLIError::InternalCompilerError { src, .. } => Some(src),
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        match self {
            CLIError::SyntaxError { msg, loc, .. } => {
                Some(Box::new(std::iter::once(LabeledSpan::new(
                    Some(msg.to_string()),
                    loc.start.0 .0 as usize,
                    loc.end.0 .0 as usize - loc.start.0 .0 as usize,
                ))))
            }
            CLIError::InternalCompilerError { .. } => None,
        }
    }
}

impl CLIError {
    pub fn from_parser_error(err: ParseError, source: &str) -> CLIError {
        match err {
            ParseError::SyntaxError(partiql_source_map::location::Located { inner, location }) => {
                CLIError::SyntaxError {
                    src: source.to_string(),
                    msg: format!("Syntax error `{}`", inner),
                    loc: location,
                }
            }
            ParseError::UnexpectedToken(partiql_source_map::location::Located {
                inner,
                location,
            }) => CLIError::SyntaxError {
                src: source.to_string(),
                msg: format!("Unexpected token `{}`", inner.token),
                loc: location,
            },
            ParseError::LexicalError(partiql_source_map::location::Located { inner, location }) => {
                CLIError::SyntaxError {
                    src: source.to_string(),
                    msg: format!("Lexical error `{}`", inner),
                    loc: location,
                }
            }
            ParseError::Unknown(location) => CLIError::SyntaxError {
                src: source.to_string(),
                msg: "Unknown parser error".to_string(),
                loc: Location {
                    start: location,
                    end: location,
                },
            },
            ParseError::IllegalState(_location) => CLIError::InternalCompilerError {
                src: source.to_string(),
            },
            _ => {
                todo!("Not yet handled {:?}", err);
            }
        }
    }
}
