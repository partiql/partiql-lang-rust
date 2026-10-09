//! Syntax highlighting for the `pqlite` REPL.
//!
//! [`PqliteHighlighter`] re-lexes the whole reedline buffer on every repaint
//! with the parser's own lexer (see [`crate::lexer`]) and maps each token's
//! [`Category`] to a [`Theme`] style.

use std::ffi::OsStr;
use std::io::IsTerminal;
use std::ops::Range;

use nu_ansi_term::{Color, Style};
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use reedline::{Highlighter, StyledText};

use crate::error::LexError;
use crate::lexer::{PartiqlLexer, Token};

/// REPL meta-commands; anything else starting with `.` is highlighted as invalid.
pub const META_COMMANDS: &[&str] = &[".help", ".quit", ".exit"];

/// Coarse lexical category of a span of PartiQL text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Category {
    Keyword,
    /// Keywords that are also valid identifiers (`ANY`, `LABEL`, ...).
    NonReservedKeyword,
    /// `TRUE`, `FALSE`, `NULL`, `MISSING`
    Constant,
    Identifier,
    QuotedIdentifier,
    /// `@x`, `@"x"`
    Variable,
    String,
    Number,
    /// Backtick-quoted embedded Ion
    IonLiteral,
    Operator,
    Punctuation,
    Comment,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Classified {
    /// Byte offsets into the lexed text.
    pub span: Range<usize>,
    pub category: Category,
    /// `false` for an unterminated string, quoted identifier, comment, or Ion
    /// literal; its span then runs to the end of the text and lexing stops.
    pub terminated: bool,
}

/// Lex `text` into categorized spans. Never fails; whitespace is not emitted.
pub(crate) fn classify(text: &str) -> Vec<Classified> {
    let mut tracker = LineOffsetTracker::default();
    let mut out = Vec::new();
    for res in PartiqlLexer::new(text, &mut tracker) {
        match res {
            Ok((start, tok, end)) => out.push(Classified {
                span: start.to_usize()..end.to_usize(),
                category: category(&tok),
                terminated: true,
            }),
            Err((start, err, end)) => {
                let (start, end) = (start.to_usize(), end.to_usize());
                let unterminated = match &err {
                    // The sub-lexers report these to end of input, then the
                    // outer lexer resumes *inside* them; stop instead.
                    LexError::UnterminatedComment => Some(Category::Comment),
                    LexError::UnterminatedDocLiteral => Some(Category::IonLiteral),
                    // An unclosed quote fails the token regex through to EOF.
                    LexError::InvalidInput(input) if end == text.len() => {
                        if input.starts_with('\'') {
                            Some(Category::String)
                        } else if input.starts_with('"') {
                            Some(Category::QuotedIdentifier)
                        } else if input.starts_with("@\"") {
                            Some(Category::Variable)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                match unterminated {
                    Some(category) => {
                        out.push(Classified {
                            span: start..text.len(),
                            category,
                            terminated: false,
                        });
                        break;
                    }
                    None => out.push(Classified {
                        span: start..end,
                        category: Category::Invalid,
                        terminated: true,
                    }),
                }
            }
        }
    }
    out
}

/// Exhaustive on purpose: a new lexer token fails to compile until classified.
fn category(tok: &Token<'_>) -> Category {
    use Token::*;
    match tok {
        All | Asc | And | As | At | Between | By | Case | Columns | Create | Cross | Cycle
        | Date | Desc | Distinct | Element | Else | End | Escape | Except | Exclude | Export
        | First | For | Full | From | Group | Groups | Having | In | Inner | Insert | Into | Is
        | Intersect | Join | Keep | Last | Lateral | Left | Like | Limit | Match | Natural | No
        | Not | Nulls | Offset | On | One | Or | Order | Outer | Path | Partial | Per | Pivot
        | Preserve | Recursive | Repeatable | Right | Row | Select | Search | Table | Time
        | Timestamp | Then | Union | Unpivot | Using | Value | Values | When | Where | With
        | Without | Zone | AllDifferent | BindingCount | ElementId | ElementNumber | Graph
        | GraphTable | MatchNum | PathLength | PathName | PropertyExists | Same => {
            Category::Keyword
        }
        Any(_)
        | Simple(_)
        | Acyclic(_)
        | Bindings(_)
        | Bound(_)
        | Destination(_)
        | Different(_)
        | Directed(_)
        | Edge(_)
        | Edges(_)
        | Elements(_)
        | Label(_)
        | Labeled(_)
        | Node(_)
        | Paths(_)
        | Properties(_)
        | Property(_)
        | PropertyGraphCatalog(_)
        | PropertyGraphName(_)
        | PropertyGraphSchema(_)
        | Relationship(_)
        | Relationships(_)
        | Shortest(_)
        | Singletons(_)
        | Step(_)
        | Tables(_)
        | Trail(_)
        | Vertex(_)
        | Walk(_) => Category::NonReservedKeyword,
        True | False | Null | Missing => Category::Constant,
        UnquotedIdent(_) => Category::Identifier,
        QuotedIdent(_) => Category::QuotedIdentifier,
        UnquotedAtIdentifier(_) | QuotedAtIdentifier(_) => Category::Variable,
        String(_) => Category::String,
        Int(_) | Real(_) | ExpReal(_) => Category::Number,
        EmbeddedDoc(_) => Category::IonLiteral,
        CommentLine(_) | CommentBlock(_) => Category::Comment,
        OpenSquare | CloseSquare | OpenCurly | CloseCurly | OpenParen | CloseParen
        | OpenDblAngle | CloseDblAngle | Comma | Semicolon | Colon | Period => {
            Category::Punctuation
        }
        EqualEqual
        | BangEqual
        | LessGreater
        | LessEqual
        | GreaterEqual
        | Equal
        | LessThan
        | GreaterThan
        | Minus
        | Plus
        | Star
        | Percent
        | Slash
        | Caret
        | DblPipe
        | Pipe
        | Ampersand
        | Bang
        | QuestionMark
        | PipePlusPipe
        | LeftArrow
        | Tilde
        | RightArrow
        | LeftArrowTilde
        | TildeRightArrow
        | LeftArrowBracket
        | RightBracketMinus
        | TildeLeftBracket
        | RightBracketTilde
        | MinusLeftBracket
        | BracketRightArrow
        | LeftArrowTildeBracket
        | BracketTildeRightArrow
        | LeftMinusRight
        | LeftArrowSlash
        | RightSlashMinus
        | TildeLeftSlash
        | RightSlashTilde
        | MinusLeftSlash
        | SlashRightArrow
        | LeftArrowTildeSlash
        | SlashTildeRightArrow => Category::Operator,
        // Consumed inside the lexer; never surfaced.
        Newline | CommentBlockStart | EmbeddedDocQuote | EmptyEmbeddedDocQuote => Category::Invalid,
    }
}

/// Styles per category. Defaults use the 16 ANSI colors so they follow the
/// terminal's palette on light and dark backgrounds.
#[derive(Debug, Clone)]
pub struct Theme {
    pub keyword: Style,
    pub non_reserved_keyword: Style,
    pub constant: Style,
    pub identifier: Style,
    /// An identifier immediately followed by `(`.
    pub function: Style,
    pub quoted_identifier: Style,
    pub variable: Style,
    pub string: Style,
    pub number: Style,
    pub ion_literal: Style,
    pub operator: Style,
    pub punctuation: Style,
    pub comment: Style,
    pub invalid: Style,
    pub meta_command: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            keyword: Color::Blue.bold(),
            non_reserved_keyword: Style::default(),
            constant: Color::Magenta.normal(),
            identifier: Style::default(),
            function: Color::Cyan.normal(),
            quoted_identifier: Color::Yellow.normal(),
            variable: Color::Cyan.normal(),
            string: Color::Green.normal(),
            number: Color::Magenta.normal(),
            ion_literal: Color::Yellow.normal(),
            operator: Style::default(),
            punctuation: Style::default(),
            comment: Color::DarkGray.normal(),
            invalid: Color::Red.normal(),
            meta_command: Color::Cyan.bold(),
        }
    }
}

impl Theme {
    fn style(&self, category: Category) -> Style {
        match category {
            Category::Keyword => self.keyword,
            Category::NonReservedKeyword => self.non_reserved_keyword,
            Category::Constant => self.constant,
            Category::Identifier => self.identifier,
            Category::QuotedIdentifier => self.quoted_identifier,
            Category::Variable => self.variable,
            Category::String => self.string,
            Category::Number => self.number,
            Category::IonLiteral => self.ion_literal,
            Category::Operator => self.operator,
            Category::Punctuation => self.punctuation,
            Category::Comment => self.comment,
            Category::Invalid => self.invalid,
        }
    }
}

/// reedline highlighter for PartiQL statements and `.`-prefixed meta-commands.
#[derive(Debug, Clone, Default)]
pub struct PqliteHighlighter {
    theme: Theme,
}

impl PqliteHighlighter {
    pub fn new(theme: Theme) -> Self {
        PqliteHighlighter { theme }
    }
}

impl Highlighter for PqliteHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut out = Segments::new(line);
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            let start = line.len() - trimmed.len();
            let end = trimmed
                .find(char::is_whitespace)
                .map_or(line.len(), |i| start + i);
            let style = if META_COMMANDS.contains(&&line[start..end]) {
                self.theme.meta_command
            } else {
                self.theme.invalid
            };
            out.push(start..end, style);
            return out.finish();
        }

        let tokens = classify(line);
        for (i, tok) in tokens.iter().enumerate() {
            let is_call = matches!(
                tok.category,
                Category::Identifier | Category::NonReservedKeyword
            ) && tokens
                .get(i + 1)
                .is_some_and(|next| &line[next.span.clone()] == "(");
            let style = if is_call {
                self.theme.function
            } else {
                self.theme.style(tok.category)
            };
            out.push(tok.span.clone(), style);
        }
        out.finish()
    }

    fn is_inside_string_literal(&self, line: &str, cursor: usize) -> bool {
        classify(line).iter().any(|t| {
            t.category == Category::String
                && t.span.start < cursor
                && (cursor < t.span.end || !t.terminated)
        })
    }
}

/// Builds a [`StyledText`] whose segments concatenate to exactly the input
/// (reedline indexes the result by byte offset); gaps get the default style
/// and adjacent equal styles are merged.
struct Segments<'a> {
    line: &'a str,
    pos: usize,
    out: StyledText,
}

impl<'a> Segments<'a> {
    fn new(line: &'a str) -> Self {
        Segments {
            line,
            pos: 0,
            out: StyledText::default(),
        }
    }

    fn push(&mut self, span: Range<usize>, style: Style) {
        if span.start > self.pos {
            self.emit(self.pos..span.start, Style::default());
        }
        let start = span.start.max(self.pos);
        if span.end > start {
            self.emit(start..span.end, style);
            self.pos = span.end;
        }
    }

    fn emit(&mut self, range: Range<usize>, style: Style) {
        let text = &self.line[range];
        match self.out.buffer.last_mut() {
            Some((last, buf)) if *last == style => buf.push_str(text),
            _ => self.out.push((style, text.to_owned())),
        }
    }

    fn finish(mut self) -> StyledText {
        if self.pos < self.line.len() {
            self.emit(self.pos..self.line.len(), Style::default());
        }
        self.out
    }
}

/// Whether the REPL should emit color: stdout is a terminal, `NO_COLOR` is
/// unset or empty (<https://no-color.org>), `TERM` isn't `dumb`, and
/// `--no-color` wasn't passed.
pub fn color_enabled(no_color_flag: bool) -> bool {
    should_color(
        std::io::stdout().is_terminal(),
        std::env::var_os("NO_COLOR").as_deref(),
        std::env::var_os("TERM").as_deref(),
        no_color_flag,
    )
}

fn should_color(
    is_tty: bool,
    no_color: Option<&OsStr>,
    term: Option<&OsStr>,
    no_color_flag: bool,
) -> bool {
    is_tty
        && !no_color_flag
        && no_color.is_none_or(OsStr::is_empty)
        && term != Some(OsStr::new("dumb"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<(&str, Category)> {
        classify(text)
            .into_iter()
            .map(|t| (&text[t.span], t.category))
            .collect()
    }

    fn unterminated(text: &str) -> Classified {
        let toks = classify(text);
        let last = toks.last().unwrap().clone();
        assert!(!last.terminated, "{text:?}: {toks:?}");
        assert_eq!(last.span.end, text.len(), "{text:?}");
        last
    }

    fn render(text: &str) -> Vec<(Style, String)> {
        PqliteHighlighter::default().highlight(text, 0).buffer
    }

    fn style_of(text: &str, needle: &str) -> Style {
        render(text)
            .into_iter()
            .find(|(_, s)| s.contains(needle))
            .unwrap_or_else(|| panic!("{needle:?} not in {text:?}"))
            .0
    }

    #[test]
    fn classifies_each_category() {
        use Category::*;
        assert_eq!(
            kinds("SELECT t.\"A\", @v, 'x', 1.5e3, `{a: 1}`, null FROM <<1>> t -- c"),
            vec![
                ("SELECT", Keyword),
                ("t", Identifier),
                (".", Punctuation),
                ("\"A\"", QuotedIdentifier),
                (",", Punctuation),
                ("@v", Variable),
                (",", Punctuation),
                ("'x'", String),
                (",", Punctuation),
                ("1.5e3", Number),
                (",", Punctuation),
                ("`{a: 1}`", IonLiteral),
                (",", Punctuation),
                ("null", Constant),
                ("FROM", Keyword),
                ("<<", Punctuation),
                ("1", Number),
                (">>", Punctuation),
                ("t", Identifier),
                ("-- c", Comment),
            ]
        );
        assert_eq!(
            kinds("a <> b || c -> label"),
            vec![
                ("a", Identifier),
                ("<>", Operator),
                ("b", Identifier),
                ("||", Operator),
                ("c", Identifier),
                ("->", Operator),
                ("label", NonReservedKeyword),
            ]
        );
    }

    #[test]
    fn keywords_are_case_insensitive() {
        for kw in ["select", "SELECT", "SeLeCt"] {
            assert_eq!(kinds(kw), vec![(kw, Category::Keyword)]);
        }
        assert_eq!(kinds("MiSsInG"), vec![("MiSsInG", Category::Constant)]);
    }

    #[test]
    fn delimited_literals_keep_their_delimiters() {
        assert_eq!(kinds("'it''s'"), vec![("'it''s'", Category::String)]);
        assert_eq!(
            kinds("```a`b```"),
            vec![("```a`b```", Category::IonLiteral)]
        );
        assert_eq!(
            kinds("/* a /* b */ c */ x"),
            vec![
                ("/* a /* b */ c */", Category::Comment),
                ("x", Category::Identifier)
            ]
        );
    }

    #[test]
    fn unterminated_literals_run_to_end_of_input() {
        for (text, category, start) in [
            ("SELECT 'abc", Category::String, 7),
            ("SELECT 'a;\nFROM t", Category::String, 7),
            ("SELECT \"ab", Category::QuotedIdentifier, 7),
            ("SELECT @\"ab", Category::Variable, 7),
            ("SELECT /* x", Category::Comment, 7),
            ("SELECT /* a /* b */ c", Category::Comment, 7),
            ("SELECT `{a:1", Category::IonLiteral, 7),
        ] {
            let last = unterminated(text);
            assert_eq!(
                (last.category, last.span.start),
                (category, start),
                "{text:?}"
            );
        }
    }

    #[test]
    fn invalid_input_is_local() {
        assert_eq!(
            kinds("SELECT # 1"),
            vec![
                ("SELECT", Category::Keyword),
                ("#", Category::Invalid),
                ("1", Category::Number)
            ]
        );
    }

    #[test]
    fn utf8_spans_land_on_char_boundaries() {
        assert_eq!(
            kinds("'é' \"名前\" ü"),
            vec![
                ("'é'", Category::String),
                ("\"名前\"", Category::QuotedIdentifier),
                ("ü", Category::Invalid)
            ]
        );
    }

    #[test]
    fn highlight_styles_tokens() {
        let theme = Theme::default();
        let q = "SELECT t.a FROM mem(3, 2) t WHERE t.b = 'x';";
        assert_eq!(style_of(q, "SELECT"), theme.keyword);
        assert_eq!(style_of(q, "mem"), theme.function);
        assert_eq!(style_of(q, "'x'"), theme.string);
        assert_eq!(style_of("SELECT 'abc;\nFROM t", "FROM t"), theme.string);
        assert_eq!(style_of(".help", ".help"), theme.meta_command);
        assert_eq!(style_of("  .nope arg", ".nope"), theme.invalid);
    }

    /// reedline slices the result by byte offset, so it must round-trip the
    /// input exactly — for complete fixtures and every partially-typed prefix.
    #[test]
    fn highlight_round_trips_any_input() {
        fn check(text: &str) {
            let styled = PqliteHighlighter::default().highlight(text, 0);
            assert_eq!(styled.raw_string(), text);
        }
        fn fixtures(dir: &std::path::Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    fixtures(&path, out);
                } else if path.to_string_lossy().ends_with(".test.ion") {
                    out.push(std::fs::read_to_string(path).unwrap());
                }
            }
        }
        let mut corpus = vec![
            "SELECT VALUE 'a;b' FROM << 1 >>;\nSELECT VALUE 2\n  FROM << 1 >>;".to_owned(),
            "SELECT \"é\" /* nested /* c */ */ ```a`b``` @x # 'it''s".to_owned(),
        ];
        let cases = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/pqlite/cases");
        fixtures(&cases, &mut corpus);
        assert!(corpus.len() > 2, "no fixtures under {}", cases.display());
        for text in &corpus {
            check(text);
            for (i, _) in text.char_indices().take(400) {
                check(&text[..i]);
            }
        }
    }

    #[test]
    fn inside_string_literal() {
        let h = PqliteHighlighter::default();
        assert!(h.is_inside_string_literal("SELECT 'ab'", 9));
        assert!(!h.is_inside_string_literal("SELECT 'ab'", 11));
        assert!(h.is_inside_string_literal("SELECT 'ab", 10));
        assert!(!h.is_inside_string_literal("SELECT ab", 8));
    }

    #[test]
    fn color_requires_tty_and_no_opt_out() {
        let s = |x| Some(OsStr::new(x));
        assert!(should_color(true, None, s("xterm"), false));
        assert!(should_color(true, s(""), None, false));
        assert!(!should_color(false, None, s("xterm"), false));
        assert!(!should_color(true, s("1"), s("xterm"), false));
        assert!(!should_color(true, None, s("dumb"), false));
        assert!(!should_color(true, None, s("xterm"), true));
    }
}
