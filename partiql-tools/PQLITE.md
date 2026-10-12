# pqlite Project Handoff

Interactive PartiQL embedded database and REPL.

Project Repo: [https://github.com/partiql/partiql-lang-rust/tree/dev](https://github.com/partiql/partiql-lang-rust/tree/dev) (until merged)

Maintainers: hashPirate, johnedquinn

## Background

partiql-lang-rust currently uses a tree-walker structure, and is transitioning to a fast bytecode VM on the `dev` branch. This experimental VM gives us the correct shape to stream rows out of a real storage backend.

### Conformance status on the VM

pqlite does not sit on the conformance-test call graph. Any regression is due to ongoing experimental VM changes on `dev` branch. 

## Installation

### Install via Homebrew

Not available yet.

<!-- TODO: when the Homebrew formula is added, generate shell completions at install time by
adding this to the formula's `install` block (runs `pqlite completions bash|zsh|fish` and
installs the results into Homebrew's completion directories):

    generate_completions_from_executable(bin/"pqlite", "completions")
-->

### Install from source

From a checkout of this repository:

```
cargo install --path partiql-tools --bin pqlite
```

This puts `pqlite` in `~/.cargo/bin` (make sure it's on your `PATH`). Re-run it to pick up
local changes.

**Shell completions**

`pqlite completions <bash|zsh|fish|powershell|elvish>` prints a
[`clap_complete`](https://crates.io/crates/clap_complete) completion script to stdout.
Completions cover subcommands, flags, `--format`/`--debug` values, shell names, and file
paths for `open <db>` and `exec --db`. Add the line for your shell to its startup file:

```
# bash (~/.bashrc)
source <(pqlite completions bash)

# zsh (~/.zshrc, after `compinit`)
source <(pqlite completions zsh)

# fish (~/.config/fish/config.fish)
pqlite completions fish | source

# PowerShell ($PROFILE)
pqlite completions powershell | Out-String | Invoke-Expression

# elvish (~/.config/elvish/rc.elv)
eval (pqlite completions elvish | slurp)
```

The script is regenerated every time a shell starts, so it always matches the installed
`pqlite`: after `cargo install` picks up new flags, open a new shell. Completions apply to
the `pqlite` command, not to `cargo run --bin pqlite --`.

To install a file once instead, write it to the shell's completion directory, e.g.
`pqlite completions fish > ~/.config/fish/completions/pqlite.fish` or
`pqlite completions zsh > ~/.zfunc/_pqlite` (with `~/.zfunc` on `$fpath`), and regenerate it
after upgrading pqlite.

## Usage

As of August 18th 2026, Ingestion Functions (`curl()`,`read()`,`stdin()`,`exec()`) are present on `feat/pqlite-ingestion-functions` branch. There is an open PR for this ([#661](https://github.com/partiql/partiql-lang-rust/pull/661)), and switching to that branch will allow for usage of these functions.

**Build**

```
cargo build --release --bin pqlite
```

**Usage**

```
pqlite open [--no-color] <db>                 # REPL against <db>
pqlite exec [--db <path>] [--format text|ion|none] "<stmt>[; <stmt>...]"
pqlite --version
pqlite --debug ast,plan,program ...           # or --debug all (alias '*')
pqlite completions <shell>                    # print a shell completion script (see Installation)
```

**Multiple statements**

`exec` and the REPL both accept several `;`-separated statements in one input
(`"CREATE TABLE t; INSERT INTO t SELECT * FROM << {'a': 1} >>; SELECT * FROM t"`).
In the REPL an entry runs once it ends with `;`, so a script can span several lines.

- Splitting is done by the PartiQL parser, so a `;` inside a string, quoted
  identifier, or comment does not split. A trailing `;` is optional; an empty
  statement (`;;`) is a parse error.
- A syntax error anywhere rejects the whole input before any statement runs.
- Otherwise statements run in order. The first failing statement stops the run
  and its error is labeled `Statement N: ...`. Earlier statements' output and
  writes are kept. `exec` then exits 1; the REPL skips the rest of that entry and
  keeps the session open.

**Output formats** (`exec --format`)

```
text   (default) rows on stdout; per-statement summary and timing on stderr
ion    one Ion envelope per statement on stdout; stderr silent on success
none   runs every statement to completion (all rows drained) but writes nothing
       to stdout; timing and --debug output still go to stderr. For benchmarking.
```

In `text` and `none` mode, timing is printed to stderr, one line per statement. The
input is parsed once up front, so parse time appears only on the total line. Rows are
rows returned (queries) or written (CTAS / INSERT), and the total sums them:

```
Statement 1: (3 rows in 0.5ms, lower: 0.3ms, compile: 0.1ms, exec: 0.1ms)
Statement 2: (2 rows in 0.3ms, lower: 0.1ms, compile: 0.1ms, exec: 0.1ms)
Total Timing: (5 rows in 0.9ms, parse: 0.1ms, lower: 0.4ms, compile: 0.2ms, exec: 0.2ms)
```

A single statement prints only the `Total Timing:` line.

**REPL meta commands**

```
.help
.quit    (alias .exit)
```

**REPL syntax highlighting**

The REPL colors keywords, literals, Ion literals, comments, and function calls as you
type, using the parser's own lexer. An unclosed string or comment stays colored to the
end of the entry. It's off when stdout isn't a terminal, `NO_COLOR` is set, `TERM=dumb`,
or with `pqlite open --no-color <db>`.

**Table functions**

```
mem(rows, cols)      generated integer data (cols named 'a', 'b'; use cols<=2)
read(path)           JSON or Ion from a file
stdin()              JSON or Ion from stdin
exec(command)        sh -c <command>, read stdout
curl(url)            HTTPS GET, read body
```

**Examples**

```
pqlite exec "SELECT * FROM mem(5, 2)"

pqlite exec "SELECT id, author.login FROM read('issues.json')"

curl -s https://api.github.com/repos/partiql/partiql-lang-rust/issues \
  | pqlite exec "SELECT number, state, title FROM stdin()"

pqlite exec "SELECT * FROM exec('gh issue list --json number,state,title')"

pqlite exec --db repos.pqlite \
  "CREATE TABLE issues AS (SELECT number, state FROM curl('https://api.github.com/repos/partiql/partiql-lang-rust/issues'))"

pqlite exec --db repos.pqlite "SELECT COUNT(*) FROM issues WHERE state = 'open'"

pqlite exec --db repos.pqlite "INSERT INTO issues SELECT * FROM read('more.json')"

pqlite exec --db repos.pqlite "SELECT * FROM _tables"

pqlite open repos.pqlite -> opens the REPL for commands like CREATE TABLE/INSERT directly 
```

**Files**

```
<name>.pqlite         LMDB env
<name>.pqlite-lock    LMDB lock file
~/.pqlite_history     REPL history
```

## File Structure

```
partiql-tools/
├── src/
│   ├── bin/
│   │   └── pqlite.rs           # CLI entry point (open / exec subcommands)
│   ├── session.rs              # re-export hub: PqliteSession, Commands, OutputFormat, DebugFlags
│   ├── session/
│   │   ├── bootstrap.rs        # schema versioning, _tables self-registration
│   │   ├── exec.rs             # dispatch, exec_ctas, exec_create_table
│   │   ├── planner.rs          # compilation catalog, drain_source_rows
│   │   ├── render.rs           # text / ion output formatting (both query rows and outcomes)
│   │   ├── outcome.rs          # StatementOutcome, StatementTiming, DebugCapture
│   │   ├── ion_output.rs       # Ion envelope for non-query outcomes + shared escape helper
│   │   ├── value.rs            # VM register rows -> partiql_value::Value; Ion encoder adapter
│   │   ├── debug.rs            # --debug ast|plan|program capture
│   │   ├── script.rs           # parse `;`-separated input once into a Script (via the parser)
│   │   └── naming.rs           # table-name canonicalization
│   ├── bootstrap/
│   │   └── v1.pql              # v1 DDL script, embedded via include_str!
│   ├── storage.rs              # HeedDB: LMDB env, tables, writers
│   ├── catalog.rs              # HeedTableSource (chunked reader), metadata
│   ├── row_codec.rs            # tagged binary row format
│   ├── common.rs               # parse/lower helpers; mem, rand, scan_ion table functions
│   └── lib.rs
└── tests/
    ├── pqlite_cli.rs           # CLI integration tests via subprocess
    ├── pqlite_e2e.rs           # harness entry point (#[rstest] + #[files(...)])
    ├── pqlite_e2e/             # harness code (loader.rs, runner.rs)
    ├── pqlite/cases/           # .test.ion case files under catalog/, errors/, query/
    └── pqlite_ingestion.rs     # ingestion-function integration tests

partiql-vm/src/                # bytecode VM
├── compiler.rs                 # logical plan → Inst stream
├── expr.rs                     # Inst enum
├── plan.rs                     # CompiledPlan, agg_step, sorter
├── arena.rs                    # register arena 
└── value/shape.rs              # RowShape { Struct, Register }

```


## What was built

Full stack from REPL down to disk:

• Frontend: Interactive terminal REPL with pqlite open / pqlite exec subcommands, file-backed history, multiline statements terminated by ;, and five streaming table functions (mem, read, stdin, exec, curl) that pull rows from files, subprocesses, and live HTTP endpoints.

• Compiler: Extended the LALRPOP grammar to parse `CREATE TABLE`, `CREATE TABLE AS (query)`, and `INSERT INTO … SELECT`, added multi-statement parsing to run the bootstrap DDL script, and threaded each statement through the AST → logical-planner pipeline as a new LogicalStatement variant.

• Execution: Wired the read-only bytecode VM into pqlite's write path via a session-owned dispatcher that drains query rows into the writer.

• Engine: LMDB-backed read/write pipelines. Created a bounded 64-row streaming reader that uses a keyset resume cursor across the read path.

• Storage: Custom zero-allocation self-describing tagged binary row codec covering the full PartiQL value space.

• Testing: Designed an E2E test harness for devs building the new VM. Primary regression suite for new VM, and storage engine changes.

## Overview

Currently supports INSERT INTO ... SELECT, and CREATE TABLE AS (SELECT)

The CLI orchestrates writes from the Session. `exec.rs::dispatch_write` reads a `LogicalStatement`, and routes CTAS/INSERT/CREATE TABLE statements to their handler functions. After that, each handler will drain rows through the engine and open an LMDB write and commit.

LMDB (Lightning Memory-Mapped Database) was used for storage via the heed crate. It was chosen because of its ability to have multiple concurrent readers at the same time. It gives us atomic writes, and a single file memory-mapped database. A bonus is its quick compile time (relative to RocksDB).

`row_codec.rs` contains on-disk format. It is a self-describing tagged binary with 8 scalar tags (0x00 to 0x07) and 3 container tags (0x08 to 0x0A). `tag>=TAG_TUPLE` is used to distinguish container rows from scalar rows, so do not add scalar tags at 0x08 or above.

pqlite starts with a `CREATE TABLE _tables` bootstrap command in v1.pql. This is the system catalog containing names of all tables on disk.

## How is code organized?

- The row codec functions are meant for serialization/deserialization of row bytes only
- Storage methods like `HeedDB::create_table` are LMDB primitives. They accept byte slices. \_tables registration happens here because it is an operation done by LMDB but the decision to register a table happens in the session.
- Session functions are for orchestrating. They own the engine, write transaction lifetime and ingestion functions. Anything that the user faces (parsing, planning, errors, output rendering) is within session.

## Instructions for Local Development

Prerequisites:

- Rust toolchain (MSRV 1.86.0 per AGENTS.md)
- git submodules initialized: `git submodule update --init --recursive` (for conformance tests)

```bash
Build and run:
    cargo build --release --bin pqlite      # builds ./target/release/pqlite
    ./target/release/pqlite open demo.pqlite    # REPL
    ./target/release/pqlite exec --db demo.pqlite "SELECT * FROM _tables"

Iterate:
    cargo build --release --bin pqlite && ./target/release/pqlite open demo.pqlite

CI checks (run before every push):
    cargo test --workspace                          # all tests except conformance
    cargo clippy --all-features --workspace -- -D warnings
    cargo fmt --all -- --check
    make ci-check                                   # runs build + test + fmt + clippy + cargo-deny

Running a single test:
    cargo test -p partiql-tools --test pqlite_cli test_name
    cargo test -p partiql-tools --test pqlite_e2e   # file-driven e2e harness
```

## How to write new tests
Every test is a  `.test.ion`  file in  `partiql-tools/tests/pqlite/cases`  and can be represented as shown below:

```
{ sql: "CREATE TABLE orders", expect: { created_table: { name: ["orders"], rows: 0 } } }

{ sql: "INSERT INTO orders SELECT * FROM << {'a': 1}, {'a': 2} >>", expect: { affected_rows: 2 } }

{ sql: "SELECT name FROM _tables", expect: { rows: $bag::[ { name: ["_tables"] }, { name: ["orders"] } ] } }

{ sql: "SELECT * FROM orders", expect: { rows: $bag::[ { a: 1 }, { a: 2 } ] } }
```
`expect:` can hold either `error: "<substring>"`, or one of the success keys `rows` / `affected_rows` / `created_table`. A step can be prefixed with the `skip::` annotation to load but not execute it.

## Common Issues


- `Error: storage engine error: MDB_MAP_FULL: Environment mapsize limit reached` happens because there is a 1GiB map size cap ( `storage.rs:6` , `DEFAULT_MAP_SIZE`). Increase and rebuild
 - `Table alias fails: SELECT t.x FROM foo t returns MISSING` - Known bug in stored-table aliasing ([#657](https://github.com/partiql/partiql-lang-rust/issues/657))
- Wrapped-array API response returns one row because currently curl() unwraps a top-level JSON array but not a wrapper struct. `SELECT * FROM curl('.../search/issues')` returns one row containing `{total_count, items: [...]}`. Workaround: remove wrapper with jq till fixed.

## Next Steps

- Application-owned transactions - The write path currently buffers all source rows in RAM before opening the LMDB write transaction. Problem is `Box<dyn DataSource>` requires `'static` but `heed::RoTxn<'env>` borrows, so a live read txn can't be held inside a trait. Fix is a LazyHeedSource that owns its own RoTxn for reads, and the pqlite session owns a separate RwTxn so CTAS and row writes commit atomically as one write txn.
- Storage trait - HeedDB is concrete so a Storage trait that would abstract Env / RwTxn / RoTxn / Database / Cursor, would allow for other non-LMDB backends. Deferred until a second backend is needed.
- DROP TABLE - Parser grammar contains `DdlOp::DropTable` but it is rejected by the lowerer. Wiring it end-to-end would need extending LogicalStatement with a DropTable variant, executing it in `session/exec.rs`.

