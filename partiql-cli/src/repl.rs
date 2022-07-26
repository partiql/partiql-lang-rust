#![deny(rustdoc::broken_intra_doc_links)]

use partiql_parser::ParseError;
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};

use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{ColorMode, Context, Helper};
use std::borrow::Cow;
use std::fs::OpenOptions;

use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};
use syntect::util::as_24_bit_terminal_escaped;

use miette::{IntoDiagnostic, Report};
use owo_colors::OwoColorize;

use crate::error::CLIErrors;

static ION_SYNTAX: &str = include_str!("ion.sublime-syntax");
static PARTIQL_SYNTAX: &str = include_str!("partiql.sublime-syntax");

struct PartiqlHelperConfig {
    dark_theme: bool,
}

impl PartiqlHelperConfig {
    pub fn infer() -> Self {
        const TERM_TIMEOUT_MILLIS: u64 = 20;
        let timeout = std::time::Duration::from_millis(TERM_TIMEOUT_MILLIS);
        let theme = termbg::theme(timeout);
        let dark_theme = match theme {
            Ok(termbg::Theme::Light) => false,
            Ok(termbg::Theme::Dark) => true,
            _ => true,
        };
        PartiqlHelperConfig { dark_theme }
    }
}
struct PartiqlHelper {
    config: PartiqlHelperConfig,
    syntaxes: SyntaxSet,
    themes: ThemeSet,
}

impl PartiqlHelper {
    pub fn new(config: PartiqlHelperConfig) -> Result<Self, ()> {
        let ion_def = SyntaxDefinition::load_from_str(ION_SYNTAX, false, Some("ion")).unwrap();
        let partiql_def =
            SyntaxDefinition::load_from_str(PARTIQL_SYNTAX, false, Some("partiql")).unwrap();
        let mut builder = SyntaxSetBuilder::new();
        builder.add(ion_def);
        builder.add(partiql_def);

        let syntaxes = builder.build();

        let _ps = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();
        Ok(PartiqlHelper {
            config,
            syntaxes,
            themes,
        })
    }
}

impl Helper for PartiqlHelper {}

impl Completer for PartiqlHelper {
    type Candidate = String;
}
impl Hinter for PartiqlHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        let hinter = HistoryHinter {};
        hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for PartiqlHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let syntax = self
            .syntaxes
            .find_syntax_by_extension("partiql")
            .unwrap()
            .clone();
        let theme = if self.config.dark_theme {
            &self.themes.themes["Solarized (dark)"]
        } else {
            &self.themes.themes["Solarized (light)"]
        };
        let mut highlighter = HighlightLines::new(&syntax, theme);

        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &self.syntaxes).unwrap();
        (as_24_bit_terminal_escaped(&ranges[..], true) + "\x1b[0m").into()
    }
    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        let _ = (line, pos);
        true
    }
}

impl Validator for PartiqlHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        // TODO remove this command parsing hack do something better
        let mut source = ctx.input();
        let flag_display = source.starts_with("\\ast");
        if flag_display {
            source = &source[4..];
        }

        let parser = partiql_parser::Parser::default();
        let result = parser.parse(source);
        match result {
            Ok(_parsed) => {
                #[cfg(feature = "visualize")]
                if flag_display {
                    use crate::visualize::render::display;
                    display(&_parsed.ast);
                }

                Ok(ValidationResult::Valid(None))
            }
            Err(e) => {
                if e.errors
                    .iter()
                    .any(|err| matches!(err, ParseError::UnexpectedEndOfInput))
                {
                    // TODO For now, this is what allows you to do things like hit `<ENTER>` and continue writing the query on the next line in the middle of a query.
                    // TODO we should probably do something more ergonomic. Perhaps require a `;` or two newlines to end?
                    Ok(ValidationResult::Incomplete)
                } else {
                    let err = Report::new(CLIErrors::from_parser_error(e));
                    Ok(ValidationResult::Invalid(Some(format!("\n\n{:?}", err))))
                }
            }
        }
    }
}

pub fn repl() -> miette::Result<()> {
    let mut rl = rustyline::Editor::<PartiqlHelper>::new().into_diagnostic()?;
    rl.set_color_mode(ColorMode::Forced);
    rl.set_helper(Some(
        PartiqlHelper::new(PartiqlHelperConfig::infer()).unwrap(),
    ));
    let expanded = shellexpand::tilde("~/partiql_cli.history").to_string();
    let history_path = Path::new(&expanded);
    OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(history_path)
        .expect("history file create if not exists");
    rl.load_history(history_path).expect("history load");

    println!("===============================");
    println!("PartiQL REPL");
    println!("CTRL-D on an empty line to quit");
    println!("===============================");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                println!("{}", "Parse OK!".green());
                rl.add_history_entry(line);
            }
            Err(_) => {
                println!("Exiting...");
                rl.append_history(history_path).expect("append history");
                break;
            }
        }
    }

    Ok(())
}
