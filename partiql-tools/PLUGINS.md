# pqlite table-function plugins

pqlite can load table functions from native shared libraries through a small,
versioned C ABI. The ABI is defined in
[`include/pqlite_plugin.h`](include/pqlite_plugin.h), and that header is the
source of truth for the contract.

> **Unstable.** Plugin support is in development; the ABI and CLI flags may
> change without notice. It is not a Cargo feature: it is compiled in only when
> rustc gets `--cfg pqlite_unstable_plugins`, so `--all-features` and
> dependent crates never enable it.

```bash
RUSTFLAGS="--cfg pqlite_unstable_plugins" cargo build --release -p partiql-tools
target/release/pqlite --load ./libmyplugin.so --plugin-opt my.key=value \
  exec "SELECT t.a FROM my_fn({'x': 1}) AS t"
```

* `--load PATH` is repeatable. Each plugin's functions are added to the
  built-ins. A name that is already taken causes a load error.
* `--plugin-opt KEY=VALUE` is repeatable. Every plugin receives all of the
  options, so by convention keys carry the plugin's name as a prefix.
* `PQLITE_PLUGIN_LOG=off|error|warn|info|debug|trace` controls plugin log
  output on stderr. The default is `warn`.
* `.help` in the REPL lists all table functions, including those from
  plugins.

## How a call flows

1. **Load.** `dlopen`, then `pqlite_plugin_init(host, &vtable)`. The host
   checks the ABI version and the `struct_size` fields, and validates every
   function definition before registering any of them. The library is never
   unloaded.
2. **Lower and compile.** Each `PqliteTableFnDef` becomes a `TableFnDef` in
   the session's `TableFnRegistry`. That gives one planner overload per arity
   in `min_args..=max_args`. The compile-time metadata is either dynamic
   (every field resolves) or based on the plugin's `static_schema`.
3. **Execute.** `TableFunction::create` copies the arguments. `DataSource::open`
   calls the plugin's `open` with a `PqliteOpenRequest`, which carries the
   arguments, the projection and a cancellation flag. The plugin fills in an
   Arrow C stream. `next_row` walks each record batch and copies every
   projected cell into its register slot.

## Design choices and trade-offs

The starting point was the v1 sketch in the research doc (§7.2). The changes
from that sketch, and the reasons for them:

| Choice | Instead of | Why | Cost |
|---|---|---|---|
| **Host context `PqliteHostV1` passed to `init`**: ABI version, host name and version, a log callback with a max level, config key/value pairs | `init(host_abi_version)` with config from env vars only | Plugins log through pqlite instead of writing to stderr themselves. Config comes from the command line, which matters once there are several plugins or the session is embedded. Init failures have somewhere to report. | One more struct to keep stable. The host must keep the struct and its callbacks alive for the process (it leaks one per plugin). |
| **Per-call `PqliteOpenRequest` struct passed to `open`** | Positional `open(fn_index, args, n_args, fields, n_fields, err)` | `open` will grow: a cancellation flag now, filter or limit hints later. A struct with `struct_size` can gain fields without an ABI bump. | Plugins must check `struct_size` before reading newer fields. |
| **Context goes to `open`, not only to `init`** | Only a process-wide context | Cancellation and anything else per query (deadline, query id) belong to one call, not to the process. | None worth noting. |
| **Cancellation is a `const uint32_t*` flag** that the host sets, and the plugin polls with atomic loads | A callback, or none | Polling a flag needs no call back into the host and works from any plugin thread, including tokio workers. The host sets it when the data source is closed or dropped (e.g. `LIMIT` stops early), so a plugin can stop prefetching. | pqlite has no Ctrl-C query cancellation yet, so today the flag only fires on early close. |
| **Rows come back as a standard Arrow C stream (`ArrowArrayStream`)** | A bespoke `schema` / `next_batch` / `last_error` / `close` set in the vtable | It is the same four operations, already specified by Arrow. arrow-rs implements both ends (`FFI_ArrowArrayStream::new(reader)` and `ArrowArrayStreamReader`), as do other Arrow libraries. Release semantics are standard. | Stream callbacks are written by the Arrow library and can't catch panics. A Rust plugin must wrap its reader (see `CatchUnwind` in PqliteCairnsPlugin). |
| **`struct_size` on every struct, fields only ever appended** | A new `...V2` struct for every addition | Compatible additions need no version bump. The major `abi_version` still guards breaking changes. | Readers must handle short structs. The host already does. |
| **`PqliteStr` (pointer and length) for all borrowed strings** | NUL-terminated `char*` | Field names and argument strings can contain anything, and no copies are needed to add NULs. | Only the strings returned in `err` are NUL-terminated, which is a mixed convention. |
| **Arguments: scalars direct; decimals, structs, lists and bags as Ion text; bytes raw** | Ion text for containers only | Decimals have no C equivalent. Bytes needed their own kind. | Plugins need an Ion parser to read container arguments. |
| **`whole_row` flag plus a `fields` list** | `n_fields == 0` meaning whole row | The VM compiler asks for a whole row whenever a query reads no specific fields (`SELECT *`, `COUNT(*)`). A separate flag keeps "no fields" distinct from "every field". | None worth noting. |
| **`static_schema` per function** (optional) | Per call | `CompilationCatalog::get_table_function` only sees the function name, never the arguments. With a static schema, known non-nullable primitive columns compile to typed slots. Everything else is dynamic. | A schema that depends on the arguments can't be typed (research §7.5). |

## Arrow to PartiQL mapping

See the module docs in `src/plugin/source.rs`. In summary:

* Integers map to int. `UInt64` values above `i64::MAX` become decimals.
* Floats map to float. `Decimal128` with scale ≤ 28 maps to decimal.
  Wider decimals become strings.
* Strings and binaries are borrowed with no copy.
* Dates, times and timestamps become ISO-8601 strings, because the VM has no
  datetime value.
* Structs become tuples. Lists become lists. Maps become lists of
  `{key, value}`. Dictionaries are decoded.
* A field the batch doesn't have reads as MISSING.

Borrowed strings are sound under `BufferStability::UntilNext`, because a batch
is kept for one extra batch after its last row.

## Known limits

* `COUNT(*)` and other queries that read no specific fields request the whole
  row, so the plugin decodes every column. Fixing this needs a VM compiler
  change: an empty layout for "rows only".
* `COUNT(*)` over an empty input returns no row. This is an existing VM
  bug, and it happens with the built-in sources too.
* A nested path (`t.a.b`) fails to compile with "unresolved var" when the
  same scan also reads another field, or when it appears in `WHERE`.
  `scan_ion` fails the same way, so this is an existing VM compiler issue.
  `SELECT t.a.b FROM f() t` on its own works, and so does reading the whole
  `t.a`.
* A runtime `.load` meta-command isn't supported. The function set is fixed
  when the session opens.
