# Design: syntax highlighting in the `pqlite` REPL

Status: proposal (no code yet). Branch: `feat/pqlite-repl-highlighting`.

## Problem

`pqlite open` echoes input uncolored. We want per-keystroke highlighting of
PartiQL in the REPL: keywords, identifiers, literals, Ion literals, comments,
operators, meta-commands. It must stay in sync with the grammar without someone
keeping a second keyword list up to date. It must also handle partial input
(the buffer is mid-edit nearly every time it's painted) and respect `NO_COLOR`.

## How the REPL crate does it

`pqlite` uses **reedline 0.48.0** (`partiql-tools/Cargo.toml`). Relevant API:

```rust
pub trait Highlighter: Send {
    fn highlight(&self, line: &str, cursor: usize) -> StyledText;           // required
    fn is_inside_string_literal(&self, line: &str, cursor: usize) -> bool;  // default false
}
pub struct StyledText { pub buffer: Vec<(nu_ansi_term::Style, String)> }
Reedline::with_highlighter(Box<dyn Highlighter>) / with_ansi_colors(bool)
```

- `highlight` is called from `buffer_paint` on **every repaint**, i.e. every
  keystroke. `line` is the **whole multi-line buffer** (including `\n` and
  every `;`-separated statement so far), not just the current line.
- After highlighting, reedline calls `StyledText::style_range(from, to, …)` for
  the visual selection and `render_around_insertion_point(cursor)`. Both index
  by byte offsets into the concatenated segments. **Invariant: the segments
  must concatenate to exactly `line`.** That means whitespace and newlines are
  emitted too, and no characters are dropped or rewritten.
- If no highlighter is set, reedline installs `ExampleHighlighter::new(vec![])`,
  which renders everything in one style. That's what `pqlite` does today.
- `with_ansi_colors(false)` makes the painter emit `raw_string()` with no
  escape codes. reedline does **not** check `NO_COLOR` or whether output is a tty.
- `nu_ansi_term` is not re-exported by reedline, so we add `nu-ansi-term = "0.50"`
  to match reedline's version and avoid a second copy in the dependency tree.

Current wiring (`partiql-tools/src/bin/pqlite.rs`, from #685):
`Reedline::create().with_history(..).with_validator(Box::new(PqliteValidator))`.
`PqliteValidator` marks the entry complete on a trailing `;` or a leading `.`.
`handle_entry(session, buffer, stdout, stderr)` is the no-tty test seam. The
validator is unit-tested directly (`PqliteValidator.validate(..)`). The
highlighter gets tested the same way.

## Tokenization source

`partiql-parser` has a **logos** lexer (`src/lexer/partiql.rs`, `Token<'input>`,
about 150 variants, `#[non_exhaustive]`), plus sub-lexers for nested `/* */`
comments (`comment.rs`) and backtick Ion literals (`embedded_doc.rs`). LALRPOP
uses it as an external lexer. `Token` is `pub`, but `mod lexer` is private, so
it **isn't reachable from outside the crate**. That's a good thing. Keywords
are case-insensitive regexes (`(?i:Select)`). Non-reserved keywords (`ANY`,
`SIMPLE`, graph words like `LABEL` and `NODE`) carry their text and can be
identifiers.

Empirical lexer behavior on broken input (probed with a scratch test, now reverted):

| Input | Lexer output |
|---|---|
| `SELECT 'abc` | `Select`, then `Err(InvalidInput("'abc"))` spanning **to EOF** |
| `SELECT "ab` | same as above, `InvalidInput` to EOF |
| ``SELECT `{a:1`` | `Err(UnterminatedDocLiteral)` 7..EOF, **then resumes inside**: `{`, `a`, `:`, `1` |
| `SELECT /* x` | `Err(UnterminatedComment)` 7..EOF, then resumes: `x` |
| `SELECT # 1` | `Err(InvalidInput("#"))` 7..8, then continues normally |
| `'it''s'`, ```` ```a`b``` ````, `@x`, `-- c` | `String` / `EmbeddedDoc` / `UnquotedAtIdentifier` / `CommentLine`; spans include the delimiters |

So the classifier has to handle two error cases. A doc literal or comment that
never closes gets styled as that kind through EOF, and lexing stops there.
Without the stop, the resumed tokens inside it would overlap the error span. An
`InvalidInput` that starts with `'` or `"` means an unterminated string or
quoted identifier. Any other `InvalidInput` is a local error token.

## Proposed architecture

### 1. `partiql-parser`: a small, stable classification API (additive, minor bump)

New file `partiql-parser/src/lexer/classify.rs`, re-exported as
`partiql_parser::highlight`:

```rust
/// Coarse lexical category of a span of PartiQL text, for editors/highlighters.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenCategory {
    Keyword,            // SELECT, FROM, CASE, DATE, ... (reserved)
    NonReservedKeyword, // ANY, SIMPLE, LABEL, NODE, ... (also valid identifiers)
    Constant,           // TRUE FALSE NULL MISSING
    Identifier,         // foo, $x
    QuotedIdentifier,   // "Foo"
    Variable,           // @x, @"x"
    String,             // 'abc'
    Number,             // 1, 1.5, 1e3
    IonLiteral,         // `{a: 1}`
    Operator,           // = <> || + - * / % < <= ~> ...
    Punctuation,        // ( ) [ ] { } << >> , ; : .
    Comment,            // -- ..., /* ... */
    Invalid,            // unrecognized input
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedToken {
    pub span: std::ops::Range<usize>, // byte offsets into the input
    pub category: TokenCategory,
    /// false for an unterminated string / quoted ident / comment / Ion literal
    /// (span then runs to end of input and classification stops).
    pub terminated: bool,
}

/// Lex `text` for highlighting. Never fails; whitespace/newlines are not emitted.
pub fn classify(text: &str) -> impl Iterator<Item = ClassifiedToken> + '_;
```

How it works: wrap `PartiqlLexer` (the raw lexer, not the preprocessor, which
rewrites special-form function syntax for the parser) and map each token
through one **exhaustive `match` on `Token`**:

```rust
fn category(tok: &Token<'_>) -> TokenCategory {
    use Token::*;
    match tok {
        Select | From | Where | /* ... every reserved keyword ... */ => TokenCategory::Keyword,
        t if t.is_var_non_reserved() => TokenCategory::NonReservedKeyword,
        True | False | Null | Missing => TokenCategory::Constant,
        UnquotedIdent(_) => TokenCategory::Identifier,
        QuotedIdent(_) => TokenCategory::QuotedIdentifier,
        UnquotedAtIdentifier(_) | QuotedAtIdentifier(_) => TokenCategory::Variable,
        String(_) => TokenCategory::String,
        Int(_) | Real(_) | ExpReal(_) => TokenCategory::Number,
        EmbeddedDoc(_) => TokenCategory::IonLiteral,
        CommentLine(_) | CommentBlock(_) => TokenCategory::Comment,
        OpenParen | CloseParen | Comma | Semicolon | /* ... */ => TokenCategory::Punctuation,
        Equal | LessGreater | DblPipe | /* ... incl. graph arrows */ => TokenCategory::Operator,
        // never surfaced by PartiqlLexer
        Newline | CommentBlockStart | EmbeddedDocQuote | EmptyEmbeddedDocQuote => TokenCategory::Invalid,
    }
}
```

**Staying in sync with the grammar:** `#[non_exhaustive]` doesn't apply inside
the defining crate, so the match has no `_` arm. Adding a `Token` variant
**fails to compile** until someone classifies it, which keeps highlighting in
step with the lexer the grammar uses. `Token` and LALRPOP stay private. The
public surface is one function, one struct, and one `#[non_exhaustive]` enum,
so adding a category later isn't a breaking change. No new dependencies and no
feature flag are needed.

### 2. `partiql-tools`: `PqliteHighlighter`

New module `partiql-tools/src/session/highlight.rs`, public so the binary and
tests can use it:

```rust
pub struct Theme { pub keyword: Style, pub constant: Style, /* one per category */
                   pub function: Style, pub meta: Style, pub invalid: Style }
impl Default for Theme { /* ANSI-16 table below */ }

pub struct PqliteHighlighter { theme: Theme }

impl reedline::Highlighter for PqliteHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut out = Segments::new(line);           // fills gaps with default style,
        if let Some(meta) = meta_command_span(line) { // coalesces equal adjacent styles
            out.push(meta, self.meta_style(&line[meta.clone()]));
            return out.finish();
        }
        let mut toks = classify(line).peekable();
        while let Some(t) = toks.next() {
            let style = match t.category {
                // identifier immediately followed by `(` => function call (mem, scan_ion, upper, ...)
                TokenCategory::Identifier | TokenCategory::NonReservedKeyword
                    if toks.peek().is_some_and(|n| &line[n.span.clone()] == "(") => self.theme.function,
                c => self.theme.style_for(c),
            };
            out.push(t.span, style);
        }
        out.finish()                                  // trailing gap; asserts concat == line in debug
    }
    fn is_inside_string_literal(&self, line: &str, cursor: usize) -> bool {
        classify(line).any(|t| t.category == TokenCategory::String
            && t.span.start < cursor && (cursor < t.span.end || !t.terminated))
    }
}
```

- **Meta-commands:** if the trimmed buffer starts with `.`, the first word gets
  the meta style when it's a known command (`.help`, `.quit`, `.exit`) and the
  invalid style otherwise. The rest is left plain. The known-command list moves
  into a `const META_COMMANDS: &[&str]` that `handle_meta_command` and
  `print_help` also use, so there's one source of truth.
- **Function heuristic:** this lives in pqlite, not in the parser API. It's a
  presentation decision, and peeking at the next token is cheap.
- **Wiring** in `run_repl`:
  ```rust
  let color = use_color(no_color_flag);
  let mut editor = Reedline::create().with_history(..).with_validator(..).with_ansi_colors(color);
  if color { editor = editor.with_highlighter(Box::new(PqliteHighlighter::default())); }
  ```
  `use_color` returns true only when stdout is a terminal, `NO_COLOR` is unset
  or empty (per no-color.org), `TERM != "dumb"`, and `--no-color` wasn't
  passed. `--no-color` is a new flag on the `open` subcommand. `exec` doesn't
  print colored output, so it doesn't need the flag.

### Default theme (ANSI 16 colors, readable on light and dark backgrounds)

| Category | Style | Example |
|---|---|---|
| Keyword | bold blue | `SELECT`, `from` |
| NonReservedKeyword | default (plain). The lexer can't tell keyword use from identifier use. | `label`, `any` |
| Constant | magenta | `TRUE`, `null`, `MISSING` |
| Identifier | default | `t`, `name` |
| Function (pqlite heuristic) | cyan | `mem(`, `scan_ion(` |
| QuotedIdentifier | yellow | `"Name"` |
| Variable | cyan | `@x` |
| String | green | `'abc'` (unterminated: green to end of buffer) |
| Number | magenta | `42`, `1.5e3` |
| IonLiteral | yellow (no italic: many terminals don't support it) | `` `{a: 1}` `` |
| Operator | default | `=`, `||`, `->` |
| Punctuation | default; `<<` `>>` bold | `( ) , ;` |
| Comment | dark gray (`Color::DarkGray`) | `-- hi`, `/* … */` |
| Invalid | red | `#` |
| Meta command (known / unknown) | bold cyan / red | `.help` / `.foo` |

Customization isn't in v1. `Theme` is a plain struct, so a later
`PQLITE_COLORS` env var or config file can be added without changing the
architecture.

## Edge cases

- **Partial or incomplete input:** the lexer never fails hard, and everything
  that isn't classified is emitted as a default-styled gap, so any buffer
  renders.
- **Unterminated `'…`, `"…`, `/*…`, `` `… ``:** styled as their kind through
  the end of the buffer, across lines. This matches editor convention and
  shows the user what's still open.
- **Nested `/* /* */ */`** and **```` ```a`b``` ````** (odd backtick fences)
  are handled by the existing sub-lexers.
- **`''` escapes** inside strings are part of the `String` token.
- **Multi-byte UTF-8** (`'é'`, `"名前"`): logos spans are byte offsets on char
  boundaries, which is what reedline expects. Gaps are filled by slicing
  `line`, never by counting chars.
- **Multi-statement / multi-line:** the whole buffer is lexed as one text, so
  a string that spans a `;` or a newline is styled correctly.
- **Keywords used as identifiers** (`SELECT t.value`): the lexer emits `Value`
  here, so it shows as a keyword. That's lexically correct and the same thing
  the parser sees. Acceptable.
- **Validator interplay (out of scope, noted):** `PqliteValidator` treats
  `SELECT 'a;` as complete. `classify` could fix this by requiring the final
  token to be a terminated `;`. That's a separate FOLLOWUP, and it costs one
  line once the API exists.

## Performance

`classify` is a single linear logos pass with no allocation beyond the
iterator. `StyledText` allocates one `String` per segment, and coalescing
adjacent same-style segments reduces that. REPL buffers are rarely over a few
KB, which lexes in microseconds, so per-keystroke cost is negligible. Even a
100 KB paste is well under a millisecond. No caching is needed. A full LALRPOP
parse per keystroke is explicitly avoided.

## Alternatives considered

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **A. `classify` API in partiql-parser (proposed)** | Same lexer as the grammar; compile error on new tokens; tiny stable API; handles nesting, escapes, and backtick fences correctly | Small public API addition to a published crate | **Recommended** |
| B. Make `lexer`/`Token` public (or behind a feature) | Zero new code | Exposes ~150 variants plus logos details; every token rename becomes a semver break; consumers re-implement classification | Reject |
| C. Regex or keyword list in pqlite | No parser changes | Drifts from the grammar; wrong on nested comments, `''`, backtick fences, `@"x"` | Reject |
| D. Extract keywords at build time from `partiql.rs` / `.lalrpop` | Auto-synced keyword list | Brittle source scraping; still needs a hand-written lexer for everything else | Reject |
| E. syntect / tree-sitter grammar | Rich ecosystem; reusable in editors | Heavy dependencies; a second grammar to maintain; drifts | Reject for the REPL (a future editor-plugin project could still use it) |
| F. Full parse per keystroke (semantic highlighting) | Could color parse errors or distinguish functions exactly | Partial input almost never parses; costlier; noisy | Defer (maybe "red on error at Enter") |

## Testing plan

- **partiql-parser** (`classify.rs` unit tests):
  - one case per category
  - keyword case-insensitivity (`select`, `SeLeCt`, `SELECT` → `Keyword`)
  - `TRUE`/`null` → `Constant`
  - `label` → `NonReservedKeyword`
  - unterminated `'abc`, `"ab`, `/* x`, `` `{a:1 `` → that category, `terminated: false`, span to EOF, nothing after
  - nested comment
  - ```` ```a`b``` ````
  - `'it''s'`
  - `#` → `Invalid`, with lexing continuing after it
  - UTF-8 spans
  - spans are sorted and non-overlapping (property-style loop over a corpus)
- **partiql-tools** (`highlight.rs` unit tests, no tty, same approach as the validator tests):
  - **concatenation invariant** `highlight(s).raw_string() == s` over a
    corpus. The corpus is every `sql` string under
    `tests/pqlite/cases/**/*.test.ion`, plus every prefix of a few queries to
    simulate typing.
  - expected style per segment for representative inputs: keyword, string, Ion, comment, `mem(` → function
  - meta commands (known and unknown)
  - `is_inside_string_literal`
  - `use_color` logic, factored as a pure fn taking `(is_tty, NO_COLOR, TERM, flag)`
- **pqlite_cli:** `pqlite open --no-color` parses (the REPL itself still needs
  a tty, so it isn't exercised end to end).

## Rollout

One PR, `feat(pqlite): syntax highlighting in the REPL`, with two commits:

1. `feat(parser): add token classification API for highlighting`
2. `feat(pqlite): syntax highlighting in the REPL`. Includes the highlighter,
   theme, `--no-color`/`NO_COLOR`, and a `PQLITE.md` note.

Colors are on by default when stdout is a tty. There is no config surface
beyond `--no-color` and `NO_COLOR`.

## Open questions for review

1. Is a new public `partiql_parser::highlight` module OK, or should it be
   `#[doc(hidden)]` or feature-gated (e.g. `highlight`) until the API settles?
2. Should non-reserved keywords (`label`, `node`, `any`, …) render plain (proposed) or as keywords?
3. Should the function-call heuristic be included in v1 (proposed: yes)?
