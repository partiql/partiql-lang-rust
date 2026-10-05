/*
 * pqlite_plugin.h — C ABI for pqlite table-function plugins (ABI version 1).
 *
 * A plugin is a shared library that exports one symbol, `pqlite_plugin_init`.
 * The host (pqlite, built with RUSTFLAGS="--cfg pqlite_unstable_plugins")
 * loads it with `pqlite --load <path>`, passes it a host context, and receives
 * a vtable of table functions. Rows cross the boundary as Arrow C Data
 * Interface record batches inside an Arrow C Stream (`struct
 * ArrowArrayStream`). Plugins never see PartiQL types; the host never sees
 * plugin types.
 *
 * UNSTABLE: this interface is in development and may change without notice.
 *
 * ---------------------------------------------------------------------------
 * Contract
 * ---------------------------------------------------------------------------
 * Versioning.  `PQLITE_PLUGIN_ABI_VERSION` is a major version. The host passes
 *   its version to `pqlite_plugin_init`; the plugin returns a vtable carrying
 *   its version, and the host refuses to load on mismatch. Every struct that
 *   crosses the boundary starts with `struct_size` (sizeof as compiled by the
 *   producer). Within one major version, fields may only be APPENDED; a reader
 *   must treat fields past the producer's `struct_size` as absent/zero. Any
 *   other change bumps the major version.
 *
 * Threading.  `pqlite_plugin_init` is called once, from one thread. `open`
 *   may be called concurrently from several threads. A given ArrowArrayStream
 *   is driven by one thread at a time but may move between threads. Plugin
 *   global state must be thread-safe. Host callbacks are thread-safe.
 *
 * Panics / exceptions.  Must not unwind across any function in this header.
 *   Rust plugins wrap every export (including stream callbacks) in
 *   `catch_unwind` and turn a panic into an error return.
 *
 * Memory.  Each side frees only what it allocated. Arrow structs are released
 *   through their own `release` callback. Strings returned through `char**
 *   err` are freed by the host calling the plugin's `free_string`. Everything
 *   the host passes in (`PqliteStr`, `PqliteArg`, `PqliteOpenRequest`,
 *   config) is BORROWED for the duration of the call that receives it; copy
 *   anything retained. The `PqliteHostV1` struct itself and its callbacks stay
 *   valid for the life of the process.
 *
 * Lifetime.  The host never unloads a plugin (`dlclose`): Arrow release
 *   callbacks point into the plugin's code. Plugins live for the process.
 *
 * Trust.  A plugin is arbitrary native code running in-process. Loading is
 *   explicit (`--load` only, no auto-load directory) and the host feature is
 *   off by default.
 *
 * Strings.  All strings are UTF-8 `PqliteStr` (pointer + length, not
 *   necessarily NUL-terminated) unless noted. `char*` error strings are
 *   NUL-terminated.
 */
#ifndef PQLITE_PLUGIN_H
#define PQLITE_PLUGIN_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define PQLITE_PLUGIN_ABI_VERSION 1u
#define PQLITE_PLUGIN_INIT_SYMBOL "pqlite_plugin_init"

/* ---- Arrow C Data / Stream Interface, verbatim from the Arrow spec ------- */
#ifndef ARROW_C_DATA_INTERFACE
#define ARROW_C_DATA_INTERFACE

#define ARROW_FLAG_DICTIONARY_ORDERED 1
#define ARROW_FLAG_NULLABLE 2
#define ARROW_FLAG_MAP_KEYS_SORTED 4

struct ArrowSchema {
  const char* format;
  const char* name;
  const char* metadata;
  int64_t flags;
  int64_t n_children;
  struct ArrowSchema** children;
  struct ArrowSchema* dictionary;
  void (*release)(struct ArrowSchema*);
  void* private_data;
};

struct ArrowArray {
  int64_t length;
  int64_t null_count;
  int64_t offset;
  int64_t n_buffers;
  int64_t n_children;
  const void** buffers;
  struct ArrowArray** children;
  struct ArrowArray* dictionary;
  void (*release)(struct ArrowArray*);
  void* private_data;
};

#endif /* ARROW_C_DATA_INTERFACE */

#ifndef ARROW_C_STREAM_INTERFACE
#define ARROW_C_STREAM_INTERFACE

struct ArrowArrayStream {
  int (*get_schema)(struct ArrowArrayStream*, struct ArrowSchema* out);
  int (*get_next)(struct ArrowArrayStream*, struct ArrowArray* out);
  const char* (*get_last_error)(struct ArrowArrayStream*);
  void (*release)(struct ArrowArrayStream*);
  void* private_data;
};

#endif /* ARROW_C_STREAM_INTERFACE */

/* ---- Common types --------------------------------------------------------- */

typedef struct PqliteStr {
  const char* ptr; /* UTF-8, may be NULL iff len == 0 */
  size_t len;
} PqliteStr;

typedef struct PqliteKeyValue {
  PqliteStr key;
  PqliteStr value;
} PqliteKeyValue;

/* Log levels, most to least severe. */
typedef enum PqliteLogLevel {
  PQLITE_LOG_ERROR = 1,
  PQLITE_LOG_WARN = 2,
  PQLITE_LOG_INFO = 3,
  PQLITE_LOG_DEBUG = 4,
  PQLITE_LOG_TRACE = 5
} PqliteLogLevel;

/* ---- Host context: passed once to pqlite_plugin_init ---------------------- */

typedef struct PqliteHostV1 {
  uint32_t struct_size; /* sizeof(PqliteHostV1) as compiled by the host */
  uint32_t abi_version; /* PQLITE_PLUGIN_ABI_VERSION of the host */
  PqliteStr host_name;    /* e.g. "pqlite" */
  PqliteStr host_version; /* e.g. "0.14.0@abc1234" */

  /* Opaque host pointer, passed back to every callback below. */
  void* host_data;

  /* Most verbose level the host will print. Plugins should skip formatting
   * messages above it. `log` may still be called with any level. */
  int32_t max_log_level;
  /* Emit a log line attributed to this plugin. `target` is a free-form
   * sub-component name (may be empty). Thread-safe. */
  void (*log)(void* host_data, int32_t level, PqliteStr target, PqliteStr message);

  /* Key/value options the user passed for plugins (`--plugin-opt k=v`).
   * Borrowed: valid only during pqlite_plugin_init. Keys are conventionally
   * prefixed with the plugin's name, e.g. "cairns.cache_dir". */
  const PqliteKeyValue* config;
  size_t n_config;
} PqliteHostV1;

/* ---- Table functions ------------------------------------------------------ */

/* Argument values. Scalars come through directly; everything else (structs,
 * lists, bags, decimals) is rendered as Ion text, so plugins never need
 * PartiQL types. MISSING is distinct from NULL. */
typedef enum PqliteArgKind {
  PQLITE_ARG_NULL = 0,
  PQLITE_ARG_MISSING = 1,
  PQLITE_ARG_BOOL = 2,
  PQLITE_ARG_INT = 3,
  PQLITE_ARG_FLOAT = 4,
  PQLITE_ARG_STRING = 5,
  PQLITE_ARG_BYTES = 6,
  PQLITE_ARG_ION_TEXT = 7
} PqliteArgKind;

typedef struct PqliteArg {
  int32_t kind; /* PqliteArgKind */
  union {
    int32_t b;   /* BOOL: 0 or 1 */
    int64_t i;   /* INT */
    double f;    /* FLOAT */
    PqliteStr s; /* STRING, BYTES (raw bytes), ION_TEXT */
  } v;
} PqliteArg;

typedef struct PqliteTableFnDef {
  uint32_t struct_size;
  PqliteStr name;  /* SQL name, e.g. "scan_cairns_partition"; unique per host */
  PqliteStr usage; /* one line for `.help`, e.g. "scan_x(spec) — ..." */
  uint32_t min_args;
  uint32_t max_args;
  /* Optional fixed output schema (a struct-typed ArrowSchema, one child per
   * column). NULL means schema-on-read: every field resolves and is typed per
   * row. When set, the host resolves field names against it at compile time.
   * Returns 0 on success. */
  int32_t (*static_schema)(void* plugin_data, uint32_t fn_index, struct ArrowSchema* out);
} PqliteTableFnDef;

/* Per-call request. Borrowed for the duration of `open` only. */
typedef struct PqliteOpenRequest {
  uint32_t struct_size;
  uint32_t fn_index; /* index into PqlitePluginV1.functions */
  const PqliteArg* args;
  size_t n_args;
  /* Projection pushdown. If `whole_row` is non-zero the query needs every
   * column (e.g. SELECT *, COUNT(*)) and `fields` is empty. Otherwise
   * `fields` lists the field names the query reads; the stream may contain
   * only those columns (extra columns are ignored, absent ones read as
   * MISSING). */
  int32_t whole_row;
  const PqliteStr* fields;
  size_t n_fields;
  /* Cooperative cancellation. Points at a 32-bit flag the host sets to
   * non-zero (with an atomic store) when the query is abandoned. Plugins
   * should poll it with an atomic load during long work and fail the stream.
   * Valid until the stream is released. Never NULL. */
  const uint32_t* cancelled;
} PqliteOpenRequest;

typedef struct PqlitePluginV1 {
  uint32_t struct_size;
  uint32_t abi_version; /* must equal the host's PQLITE_PLUGIN_ABI_VERSION */
  PqliteStr plugin_name;
  PqliteStr plugin_version;
  void* plugin_data; /* opaque, passed back to open/static_schema */

  size_t n_functions;
  const PqliteTableFnDef* functions;

  /* Start a scan. On success returns 0 and fills `*out` (host-allocated) with
   * a stream whose schema is a struct, one child per column; each `get_next`
   * yields one record batch (a struct array). On failure returns non-zero and
   * may set `*err` to a message the host frees with `free_string`. */
  int32_t (*open)(void* plugin_data, const PqliteOpenRequest* req,
                  struct ArrowArrayStream* out, char** err);

  void (*free_string)(char* s);
} PqlitePluginV1;

/* The only exported symbol. Returns 0 and sets `*out` to a vtable that stays
 * valid for the life of the process. On failure returns non-zero; report the
 * reason through `host->log` at PQLITE_LOG_ERROR first. */
typedef int32_t (*PqlitePluginInitFn)(const PqliteHostV1* host, const PqlitePluginV1** out);
int32_t pqlite_plugin_init(const PqliteHostV1* host, const PqlitePluginV1** out);

#ifdef __cplusplus
}
#endif

#endif /* PQLITE_PLUGIN_H */
