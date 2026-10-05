/*
 * A pqlite plugin in plain C99, written against include/pqlite_plugin.h only
 * (no Arrow library). tests/pqlite_plugin_c.rs compiles it into a shared
 * library and loads it into the pqlite binary with `--load`.
 *
 *   c_seq(n [, batch])  n rows {id: int64, name: 'row<id>'} in batches of
 *                       `batch` rows (default 4); has a static schema
 *   c_args(...)         one row {kind, text} per argument, as received
 *   c_config()          one row {key, value} per --plugin-opt seen at init
 *   c_fail()            open fails with an error message
 *   c_fail_next()       open succeeds, the first get_next fails
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pqlite_plugin.h"

#define MAX_COLS 2
#define MAX_CONFIG 16

enum { FN_SEQ, FN_ARGS, FN_CONFIG, FN_FAIL, FN_FAIL_NEXT, N_FNS };

static char* dup_str(PqliteStr s) {
  char* out = malloc(s.len + 1);
  if (out) {
    if (s.len) memcpy(out, s.ptr, s.len);
    out[s.len] = '\0';
  }
  return out;
}

static PqliteStr cstr(const char* s) {
  PqliteStr out = {s, strlen(s)};
  return out;
}

/* ---- Columns and record batches ------------------------------------------ */

/* An owned column: either int64 values or NUL-terminated strings. */
typedef struct Column {
  const char* name;
  int is_str;
  int64_t* ints;
  char** strs;
} Column;

typedef struct Table {
  Column cols[MAX_COLS];
  int n_cols;
  int64_t n_rows;
} Table;

static void table_free(Table* t) {
  for (int c = 0; c < t->n_cols; c++) {
    if (t->cols[c].strs)
      for (int64_t r = 0; r < t->n_rows; r++) free(t->cols[c].strs[r]);
    free(t->cols[c].strs);
    free(t->cols[c].ints);
  }
  memset(t, 0, sizeof *t);
}

/* Schema: a struct with one child per column. Format and name strings are
 * static or owned by the stream, so only the child array needs freeing. */
typedef struct SchemaPriv {
  struct ArrowSchema children[MAX_COLS];
  struct ArrowSchema* child_ptrs[MAX_COLS];
} SchemaPriv;

static void release_child_schema(struct ArrowSchema* s) { s->release = NULL; }

static void release_schema(struct ArrowSchema* s) {
  SchemaPriv* p = s->private_data;
  for (int64_t i = 0; i < s->n_children; i++)
    if (p->children[i].release) p->children[i].release(&p->children[i]);
  free(p);
  s->release = NULL;
}

static int make_schema(const Table* t, struct ArrowSchema* out) {
  SchemaPriv* p = calloc(1, sizeof *p);
  if (!p) return ENOMEM;
  for (int c = 0; c < t->n_cols; c++) {
    struct ArrowSchema* ch = &p->children[c];
    ch->format = t->cols[c].is_str ? "u" : "l";
    ch->name = t->cols[c].name;
    ch->flags = 0; /* not nullable */
    ch->release = release_child_schema;
    p->child_ptrs[c] = ch;
  }
  memset(out, 0, sizeof *out);
  out->format = "+s";
  out->name = "";
  out->n_children = t->n_cols;
  out->children = p->child_ptrs;
  out->release = release_schema;
  out->private_data = p;
  return 0;
}

/* A child array owns its buffers; the parent owns its children's storage. */
typedef struct ChildPriv {
  const void* buffers[3];
  void* owned[3];
} ChildPriv;

typedef struct BatchPriv {
  struct ArrowArray children[MAX_COLS];
  struct ArrowArray* child_ptrs[MAX_COLS];
  const void* buffers[1];
} BatchPriv;

static void release_child_array(struct ArrowArray* a) {
  ChildPriv* p = a->private_data;
  for (int i = 0; i < 3; i++) free(p->owned[i]);
  free(p);
  a->release = NULL;
}

static void release_batch(struct ArrowArray* a) {
  BatchPriv* p = a->private_data;
  for (int64_t i = 0; i < a->n_children; i++)
    if (p->children[i].release) p->children[i].release(&p->children[i]);
  free(p);
  a->release = NULL;
}

/* Copy rows [start, start + len) of `t` into a struct array. */
static int make_batch(const Table* t, int64_t start, int64_t len, struct ArrowArray* out) {
  BatchPriv* p = calloc(1, sizeof *p);
  if (!p) return ENOMEM;
  for (int c = 0; c < t->n_cols; c++) {
    const Column* col = &t->cols[c];
    struct ArrowArray* ch = &p->children[c];
    ChildPriv* cp = calloc(1, sizeof *cp);
    if (!cp) goto oom;
    ch->private_data = cp;
    ch->release = release_child_array;
    ch->length = len;
    ch->buffers = cp->buffers;
    cp->buffers[0] = NULL; /* no validity bitmap: no nulls */
    if (col->is_str) {
      int32_t* offsets = malloc((size_t)(len + 1) * sizeof *offsets);
      size_t total = 0;
      for (int64_t r = 0; r < len; r++) total += strlen(col->strs[start + r]);
      char* data = malloc(total ? total : 1);
      cp->owned[1] = offsets;
      cp->owned[2] = data;
      if (!offsets || !data) goto oom;
      offsets[0] = 0;
      for (int64_t r = 0; r < len; r++) {
        size_t n = strlen(col->strs[start + r]);
        memcpy(data + offsets[r], col->strs[start + r], n);
        offsets[r + 1] = offsets[r] + (int32_t)n;
      }
      ch->n_buffers = 3;
      cp->buffers[1] = offsets;
      cp->buffers[2] = data;
    } else {
      int64_t* values = malloc((size_t)(len ? len : 1) * sizeof *values);
      cp->owned[1] = values;
      if (!values) goto oom;
      memcpy(values, col->ints + start, (size_t)len * sizeof *values);
      ch->n_buffers = 2;
      cp->buffers[1] = values;
    }
    p->child_ptrs[c] = ch;
  }
  memset(out, 0, sizeof *out);
  out->length = len;
  out->n_buffers = 1;
  out->buffers = p->buffers;
  out->n_children = t->n_cols;
  out->children = p->child_ptrs;
  out->release = release_batch;
  out->private_data = p;
  return 0;
oom:
  for (int c = 0; c < t->n_cols; c++)
    if (p->children[c].release) p->children[c].release(&p->children[c]);
  free(p);
  return ENOMEM;
}

/* ---- Streams -------------------------------------------------------------- */

typedef struct StreamPriv {
  Table table;
  int64_t next_row;
  int64_t batch_rows;
  int fail_next;
  const uint32_t* cancelled;
  char error[128];
} StreamPriv;

static int stream_get_schema(struct ArrowArrayStream* s, struct ArrowSchema* out) {
  StreamPriv* p = s->private_data;
  return make_schema(&p->table, out);
}

static int stream_get_next(struct ArrowArrayStream* s, struct ArrowArray* out) {
  StreamPriv* p = s->private_data;
  if (__atomic_load_n(p->cancelled, __ATOMIC_ACQUIRE)) {
    snprintf(p->error, sizeof p->error, "cancelled");
    return ECANCELED;
  }
  if (p->fail_next) {
    snprintf(p->error, sizeof p->error, "c_fail_next: batch failed on purpose");
    return EIO;
  }
  if (p->next_row >= p->table.n_rows) {
    out->release = NULL; /* end of stream */
    return 0;
  }
  int64_t len = p->table.n_rows - p->next_row;
  if (len > p->batch_rows) len = p->batch_rows;
  int rc = make_batch(&p->table, p->next_row, len, out);
  if (rc == 0) p->next_row += len;
  return rc;
}

static const char* stream_last_error(struct ArrowArrayStream* s) {
  StreamPriv* p = s->private_data;
  return p->error[0] ? p->error : NULL;
}

static void stream_release(struct ArrowArrayStream* s) {
  StreamPriv* p = s->private_data;
  table_free(&p->table);
  free(p);
  s->release = NULL;
}

/* ---- Plugin state and table functions ------------------------------------- */

static char* config_keys[MAX_CONFIG];
static char* config_values[MAX_CONFIG];
static size_t n_config;

static int alloc_cols(Table* t, int64_t n_rows, const char* a, int a_str, const char* b, int b_str) {
  const char* names[MAX_COLS] = {a, b};
  int strs[MAX_COLS] = {a_str, b_str};
  t->n_cols = MAX_COLS;
  t->n_rows = n_rows;
  for (int c = 0; c < MAX_COLS; c++) {
    size_t n = (size_t)(n_rows ? n_rows : 1);
    t->cols[c].name = names[c];
    t->cols[c].is_str = strs[c];
    if (strs[c]) t->cols[c].strs = calloc(n, sizeof(char*));
    else t->cols[c].ints = calloc(n, sizeof(int64_t));
    if (!t->cols[c].strs && !t->cols[c].ints) return ENOMEM;
  }
  return 0;
}

static const char* kind_name(int32_t kind) {
  switch (kind) {
    case PQLITE_ARG_NULL: return "null";
    case PQLITE_ARG_MISSING: return "missing";
    case PQLITE_ARG_BOOL: return "bool";
    case PQLITE_ARG_INT: return "int";
    case PQLITE_ARG_FLOAT: return "float";
    case PQLITE_ARG_STRING: return "string";
    case PQLITE_ARG_BYTES: return "bytes";
    case PQLITE_ARG_ION_TEXT: return "ion_text";
    default: return "unknown";
  }
}

static char* arg_text(const PqliteArg* a) {
  char buf[64];
  switch (a->kind) {
    case PQLITE_ARG_BOOL: return dup_str(cstr(a->v.b ? "true" : "false"));
    case PQLITE_ARG_INT: snprintf(buf, sizeof buf, "%lld", (long long)a->v.i); return dup_str(cstr(buf));
    case PQLITE_ARG_FLOAT: snprintf(buf, sizeof buf, "%g", a->v.f); return dup_str(cstr(buf));
    case PQLITE_ARG_STRING:
    case PQLITE_ARG_ION_TEXT: return dup_str(a->v.s);
    case PQLITE_ARG_BYTES: snprintf(buf, sizeof buf, "%zu bytes", a->v.s.len); return dup_str(cstr(buf));
    default: return dup_str(cstr(""));
  }
}

static char* error_string(const char* msg) { return dup_str(cstr(msg)); }

static int fill_table(const PqliteOpenRequest* req, StreamPriv* p, char** err) {
  Table* t = &p->table;
  p->batch_rows = 4;
  switch (req->fn_index) {
    case FN_SEQ: {
      if (req->args[0].kind != PQLITE_ARG_INT || req->args[0].v.i < 0) {
        *err = error_string("c_seq: n must be a non-negative integer");
        return EINVAL;
      }
      if (req->n_args > 1) {
        if (req->args[1].kind != PQLITE_ARG_INT || req->args[1].v.i < 1) {
          *err = error_string("c_seq: batch must be a positive integer");
          return EINVAL;
        }
        p->batch_rows = req->args[1].v.i;
      }
      int64_t n = req->args[0].v.i;
      if (alloc_cols(t, n, "id", 0, "name", 1)) return ENOMEM;
      for (int64_t i = 0; i < n; i++) {
        char buf[32];
        snprintf(buf, sizeof buf, "row%lld", (long long)i);
        t->cols[0].ints[i] = i;
        if (!(t->cols[1].strs[i] = dup_str(cstr(buf)))) return ENOMEM;
      }
      return 0;
    }
    case FN_ARGS:
      if (alloc_cols(t, (int64_t)req->n_args, "kind", 1, "text", 1)) return ENOMEM;
      for (size_t i = 0; i < req->n_args; i++) {
        t->cols[0].strs[i] = dup_str(cstr(kind_name(req->args[i].kind)));
        t->cols[1].strs[i] = arg_text(&req->args[i]);
        if (!t->cols[0].strs[i] || !t->cols[1].strs[i]) return ENOMEM;
      }
      return 0;
    case FN_CONFIG:
      if (alloc_cols(t, (int64_t)n_config, "key", 1, "value", 1)) return ENOMEM;
      for (size_t i = 0; i < n_config; i++) {
        t->cols[0].strs[i] = dup_str(cstr(config_keys[i]));
        t->cols[1].strs[i] = dup_str(cstr(config_values[i]));
        if (!t->cols[0].strs[i] || !t->cols[1].strs[i]) return ENOMEM;
      }
      return 0;
    case FN_FAIL:
      *err = error_string("c_fail: open failed on purpose");
      return EIO;
    case FN_FAIL_NEXT:
      p->fail_next = 1;
      return alloc_cols(t, 0, "id", 0, "name", 1);
    default:
      *err = error_string("unknown function index");
      return EINVAL;
  }
}

static int32_t plugin_open(void* plugin_data, const PqliteOpenRequest* req,
                           struct ArrowArrayStream* out, char** err) {
  (void)plugin_data;
  StreamPriv* p = calloc(1, sizeof *p);
  if (!p) return ENOMEM;
  p->cancelled = req->cancelled;
  int rc = fill_table(req, p, err);
  if (rc) {
    table_free(&p->table);
    free(p);
    return rc;
  }
  out->get_schema = stream_get_schema;
  out->get_next = stream_get_next;
  out->get_last_error = stream_last_error;
  out->release = stream_release;
  out->private_data = p;
  return 0;
}

static int32_t seq_static_schema(void* plugin_data, uint32_t fn_index, struct ArrowSchema* out) {
  (void)plugin_data;
  (void)fn_index;
  Table t = {0};
  t.n_cols = 2;
  t.cols[0].name = "id";
  t.cols[1].name = "name";
  t.cols[1].is_str = 1;
  return make_schema(&t, out);
}

static void plugin_free_string(char* s) { free(s); }

#define FN_DEF(n, u, lo, hi, schema) \
  {sizeof(PqliteTableFnDef), {n, sizeof(n) - 1}, {u, sizeof(u) - 1}, lo, hi, schema}

static const PqliteTableFnDef functions[N_FNS] = {
    FN_DEF("c_seq", "c_seq(n [, batch]) - n sequential rows from a C plugin", 1, 2, seq_static_schema),
    FN_DEF("c_args", "c_args(...) - echo each argument's kind and text", 0, 4, NULL),
    FN_DEF("c_config", "c_config() - the --plugin-opt pairs seen at init", 0, 0, NULL),
    FN_DEF("c_fail", "c_fail() - fails in open", 0, 0, NULL),
    FN_DEF("c_fail_next", "c_fail_next() - fails in get_next", 0, 0, NULL),
};

static PqlitePluginV1 vtable = {
    sizeof(PqlitePluginV1),
    PQLITE_PLUGIN_ABI_VERSION,
    {"c_test_plugin", sizeof("c_test_plugin") - 1},
    {"1.0", sizeof("1.0") - 1},
    NULL,
    N_FNS,
    functions,
    plugin_open,
    plugin_free_string,
};

int32_t pqlite_plugin_init(const PqliteHostV1* host, const PqlitePluginV1** out) {
  if (host->abi_version != PQLITE_PLUGIN_ABI_VERSION) return 1;
  for (size_t i = 0; i < host->n_config && n_config < MAX_CONFIG; i++) {
    config_keys[n_config] = dup_str(host->config[i].key);
    config_values[n_config] = dup_str(host->config[i].value);
    n_config++;
  }
  if (host->max_log_level >= PQLITE_LOG_INFO) {
    char msg[64];
    snprintf(msg, sizeof msg, "loaded with %zu option(s)", n_config);
    host->log(host->host_data, PQLITE_LOG_INFO, cstr("init"), cstr(msg));
  }
  *out = &vtable;
  return 0;
}
