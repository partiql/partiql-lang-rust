//! A tree view of a plan, and a text printer over it.
//!
//! [`tree`] turns a plan into a uniform [`TreeNode`] tree: a node kind name (e.g.
//! `RexSelect`), its type, its attributes, and its children by role. The text
//! printer ([`Plan::dump`]) and [`crate::validate`] are written over this tree, and a plan
//! comparator can be too. Shapes are kept as shapes; how they are spelled is the caller's choice
//! ([`Plan::dump_with`]); [`render_shape`] is the default.
//!
//! EXPERIMENTAL: the tree's shape and the text format may change; they are a debugging and
//! comparison aid, not a stable serialization. The dump is order-preserving, not canonical: data
//! whose equality ignores order (bag and tuple literals, `AnyOf` members, struct constraint sets)
//! prints in insertion order, and a graph shared by several literals prints once and then as
//! `Graph #n` while an identical but separate graph prints in full again, so two `==` plans can
//! dump differently. Compare plans with `==` (plus
//! `Debug` for unspecified-parameter flags); compare dumps only between plans built the same way,
//! as snapshots do. Names, string literals and variant encodings are escaped, so unequal plans do
//! not dump alike, with two exceptions: a variant literal prints only its type, and every plan too
//! deep to print prints the same notice (as would graph literals of a graph kind that partiql-value
//! adds later). Note that `==` on a plan holding a graph literal panics, because partiql-value does
//! not implement graph equality yet; there, the dump is the comparison that works. Like
//! `validate`'s walk, the printer recurses once per nesting level. [`Plan::dump`] checks the depth
//! bound itself and prints a notice for a plan that is too deep (the same notice for every such
//! plan); [`tree`], [`TreeNode::write_text`], the visitor, the rewriter and the derived `Clone` /
//! `==` / `Debug` / `Drop` do not, so run [`crate::validate`] first on untrusted plans.

use crate::plan::{Action, Plan, Version};
use crate::refs::RoutineRef;
use crate::rel::{Collation, ExclusionItem, JoinType, Nulls, Order, Rel, RelOp, RelType};
use crate::rex::{Comparison, MapEntry, Quantifier, Rex, RexOp, SubqueryTest};
use partiql_types::{PartiqlShape, Static, StructType};
use partiql_value::{Graph, SimpleGraph, Value};
use std::collections::HashMap;

/// Graphs already printed in one dump, by address, with the number they were printed under;
/// graph values are reference-counted and may be shared, and a repeat prints as a reference.
type SeenGraphs = HashMap<*const SimpleGraph, usize>;
use std::fmt::Write;

/// One node of a plan's tree view.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TreeNode<'a> {
    /// The node kind name (`RexVar`, `RelScan`, `RexCase.Branch`, ..).
    pub kind: &'static str,
    /// The node's type, for a `Rex` or `Rel`.
    pub ty: Option<NodeType<'a>>,
    /// The node's non-child attributes, in field order.
    pub attrs: Vec<(String, Attr<'a>)>,
    /// The node's children, in field order, each with its role (field name, `[i]` for a list).
    pub children: Vec<(String, TreeNode<'a>)>,
    /// The `Rex` or `Rel` this node shows, for checks that need the typed node.
    pub source: Option<Source<'a>>,
}

/// The plan node a [`TreeNode`] shows.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Source<'a> {
    Rex(&'a Rex),
    Rel(&'a Rel),
}

/// The type of a [`TreeNode`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum NodeType<'a> {
    Rex(&'a PartiqlShape),
    Rel(&'a RelType),
}

/// An attribute value of a [`TreeNode`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Attr<'a> {
    Text(String),
    Shape(&'a PartiqlShape),
    Value(&'a Value),
}

impl<'a> TreeNode<'a> {
    fn new(kind: &'static str, ty: Option<NodeType<'a>>) -> Self {
        TreeNode {
            kind,
            ty,
            attrs: vec![],
            children: vec![],
            source: None,
        }
    }

    fn text(mut self, name: &str, value: impl ToString) -> Self {
        self.attrs
            .push((name.to_string(), Attr::Text(value.to_string())));
        self
    }

    fn shape(mut self, name: impl Into<String>, shape: &'a PartiqlShape) -> Self {
        self.attrs.push((name.into(), Attr::Shape(shape)));
        self
    }

    fn child(mut self, role: impl Into<String>, node: TreeNode<'a>) -> Self {
        self.children.push((role.into(), node));
        self
    }

    fn rex(self, role: &str, rex: &'a Rex) -> Self {
        self.child(role, rex_tree(rex))
    }

    fn rel(self, role: &str, rel: &'a Rel) -> Self {
        self.child(role, rel_tree(rel))
    }

    fn rexes(self, role: &str, rexes: &'a [Rex]) -> Self {
        rexes.iter().enumerate().fold(self, |n, (i, r)| {
            n.child(format!("{role}[{i}]"), rex_tree(r))
        })
    }

    fn list<T: 'a>(self, role: &str, xs: &'a [T], f: impl Fn(&'a T) -> TreeNode<'a>) -> Self {
        xs.iter()
            .enumerate()
            .fold(self, |n, (i, x)| n.child(format!("{role}[{i}]"), f(x)))
    }
}

/// The tree view of `plan`. Its root is a `Plan` node whose child is the action.
pub fn tree(plan: &Plan) -> TreeNode<'_> {
    let version = match plan.version() {
        Version::V1 => "V1",
    };
    let root = TreeNode::new("Plan", None).text("version", version);
    match plan.action() {
        Action::Query { rex } => root.child("action", TreeNode::new("Query", None).rex("rex", rex)),
    }
}

fn routine_tree(r: &RoutineRef) -> TreeNode<'_> {
    let n = TreeNode::new("RoutineRef", None)
        .text("name", qualified_text(r.name()))
        .text("specific", name_text(r.specific()));
    let n = r
        .params()
        .iter()
        .enumerate()
        .fold(n, |n, (i, p)| n.shape(format!("params[{i}]"), p));
    n.shape("returns", r.returns())
}

fn collation_tree(c: &Collation) -> TreeNode<'_> {
    let order = match c.order() {
        Order::Asc => "ASC",
        Order::Desc => "DESC",
    };
    let nulls = match c.nulls() {
        Nulls::First => "FIRST",
        Nulls::Last => "LAST",
    };
    TreeNode::new("Collation", None)
        .text("order", order)
        .text("nulls", nulls)
        .rex("column", c.column())
}

fn exclusion_item_tree(item: &ExclusionItem) -> TreeNode<'_> {
    let n = match item {
        ExclusionItem::CollIndex { index, .. } => {
            TreeNode::new("Exclusion.CollIndex", None).text("index", index)
        }
        ExclusionItem::CollWildcard { .. } => TreeNode::new("Exclusion.CollWildcard", None),
        ExclusionItem::StructKey { key, .. } => {
            TreeNode::new("Exclusion.StructKey", None).text("key", name_text(key))
        }
        ExclusionItem::StructSymbol { symbol, .. } => {
            TreeNode::new("Exclusion.StructSymbol", None).text("symbol", name_text(symbol))
        }
        ExclusionItem::StructWildcard { .. } => TreeNode::new("Exclusion.StructWildcard", None),
    };
    n.list("items", item.items(), exclusion_item_tree)
}

fn map_entry_tree(e: &MapEntry) -> TreeNode<'_> {
    TreeNode::new("RexMap.Entry", None)
        .rex("key", e.key())
        .rex("value", e.value())
}

fn join_type(j: JoinType) -> &'static str {
    match j {
        JoinType::Inner => "INNER",
        JoinType::Left => "LEFT",
        JoinType::Right => "RIGHT",
        JoinType::Full => "FULL",
    }
}

/// The tree view of one expression.
pub fn rex_tree(rex: &Rex) -> TreeNode<'_> {
    let ty = Some(NodeType::Rex(rex.ty()));
    let n = |kind| TreeNode {
        source: Some(Source::Rex(rex)),
        ..TreeNode::new(kind, ty)
    };
    match rex.op() {
        RexOp::Array { values } => n("RexArray").rexes("values", values),
        RexOp::Bag { values } => n("RexBag").rexes("values", values),
        RexOp::Call { function, args } => n("RexCall")
            .child("function", routine_tree(function))
            .rexes("args", args),
        RexOp::Case {
            match_,
            branches,
            default,
        } => {
            let mut c = n("RexCase");
            if let Some(m) = match_ {
                c = c.rex("match", m);
            }
            c = c.list("branches", branches, |b| {
                TreeNode::new("RexCase.Branch", None)
                    .rex("condition", b.condition())
                    .rex("result", b.result())
            });
            if let Some(d) = default {
                c = c.rex("default", d);
            }
            c
        }
        RexOp::Cast { operand, target } => {
            n("RexCast").shape("target", target).rex("operand", operand)
        }
        RexOp::Coalesce { args } => n("RexCoalesce").rexes("args", args),
        RexOp::Dispatch {
            name,
            functions,
            args,
        } => n("RexDispatch")
            .text("name", name_text(name))
            .list("functions", functions, routine_tree)
            .rexes("args", args),
        RexOp::Error {} => n("RexError"),
        RexOp::Lit { value } => {
            let mut l = n("RexLit");
            l.attrs.push(("value".to_string(), Attr::Value(value)));
            l
        }
        RexOp::Map {
            key_type,
            value_type,
            entries,
        } => n("RexMap")
            .shape("key_type", key_type)
            .shape("value_type", value_type)
            .list("entries", entries, map_entry_tree),
        RexOp::MapDynamic { entries } => {
            n("RexMapDynamic").list("entries", entries, map_entry_tree)
        }
        RexOp::NullIf { v1, v2 } => n("RexNullIf").rex("v1", v1).rex("v2", v2),
        RexOp::PathIndex { operand, index } => n("RexPathIndex")
            .rex("operand", operand)
            .rex("index", index),
        RexOp::PathKey { operand, key } => n("RexPathKey").rex("operand", operand).rex("key", key),
        RexOp::PathSymbol { operand, symbol } => n("RexPathSymbol")
            .text("symbol", name_text(symbol))
            .rex("operand", operand),
        RexOp::Pivot { input, key, value } => n("RexPivot")
            .rel("input", input)
            .rex("key", key)
            .rex("value", value),
        RexOp::Select { input, constructor } => n("RexSelect")
            .rel("input", input)
            .rex("constructor", constructor),
        RexOp::Spread { args } => n("RexSpread").rexes("args", args),
        RexOp::Struct { fields } => n("RexStruct").list("fields", fields, |f| {
            TreeNode::new("RexStruct.Field", None)
                .rex("key", f.key())
                .rex("value", f.value())
        }),
        RexOp::Subquery {
            input,
            constructor,
            scalar,
        } => n("RexSubquery")
            .text("scalar", scalar)
            .rel("input", input)
            .rex("constructor", constructor),
        RexOp::SubqueryComp {
            input,
            args,
            comparison,
            quantifier,
        } => {
            let comparison = match comparison {
                Comparison::Eq => "EQ",
                Comparison::Ne => "NE",
                Comparison::Lt => "LT",
                Comparison::Le => "LE",
                Comparison::Gt => "GT",
                Comparison::Ge => "GE",
            };
            let quantifier = match quantifier {
                Quantifier::Any => "ANY",
                Quantifier::All => "ALL",
                Quantifier::Some => "SOME",
            };
            n("RexSubqueryComp")
                .text("comparison", comparison)
                .text("quantifier", quantifier)
                .rel("input", input)
                .rexes("args", args)
        }
        RexOp::SubqueryIn { input, args } => {
            n("RexSubqueryIn").rel("input", input).rexes("args", args)
        }
        RexOp::SubqueryTest { input, test } => {
            let test = match test {
                SubqueryTest::Exists => "EXISTS",
                SubqueryTest::Unique => "UNIQUE",
            };
            n("RexSubqueryTest").text("test", test).rel("input", input)
        }
        RexOp::Table { table } => n("RexTable").text("table", qualified_text(table)),
        RexOp::TableRef {
            catalog_id,
            table_id,
        } => n("RexTableRef")
            .text("catalog_id", catalog_id)
            .text("table_id", table_id),
        RexOp::Var { scope, offset } => n("RexVar").text("scope", scope).text("offset", offset),
    }
}

/// The tree view of one relational operator.
pub fn rel_tree(rel: &Rel) -> TreeNode<'_> {
    let ty = Some(NodeType::Rel(rel.ty()));
    let n = |kind| TreeNode {
        source: Some(Source::Rel(rel)),
        ..TreeNode::new(kind, ty)
    };
    match rel.op() {
        RelOp::Aggregate {
            input,
            measures,
            groups,
        } => n("RelAggregate")
            .rel("input", input)
            .list("measures", measures, |m| {
                TreeNode::new("RelAggregate.Measure", None)
                    .text("distinct", m.is_distinct())
                    .child("agg", routine_tree(m.agg()))
                    .rexes("args", m.args())
            })
            .rexes("groups", groups),
        RelOp::Correlate {
            left,
            right,
            join_type: j,
        } => n("RelCorrelate")
            .text("join_type", join_type(*j))
            .rel("left", left)
            .rel("right", right),
        RelOp::Distinct { input } => n("RelDistinct").rel("input", input),
        RelOp::Except { left, right, all } => n("RelExcept")
            .text("all", all)
            .rel("left", left)
            .rel("right", right),
        RelOp::Exclude { input, exclusions } => {
            n("RelExclude")
                .rel("input", input)
                .list("exclusions", exclusions, |e| {
                    TreeNode::new("Exclusion", None).rex("var", e.var()).list(
                        "items",
                        e.items(),
                        exclusion_item_tree,
                    )
                })
        }
        RelOp::Filter { input, predicate } => n("RelFilter")
            .rel("input", input)
            .rex("predicate", predicate),
        RelOp::Intersect { left, right, all } => n("RelIntersect")
            .text("all", all)
            .rel("left", left)
            .rel("right", right),
        RelOp::Iterate { rex } => n("RelIterate").rex("rex", rex),
        RelOp::Join {
            left,
            right,
            condition,
            join_type: j,
        } => n("RelJoin")
            .text("join_type", join_type(*j))
            .rel("left", left)
            .rel("right", right)
            .rex("condition", condition),
        RelOp::Limit { input, limit } => n("RelLimit").rel("input", input).rex("limit", limit),
        RelOp::Offset { input, offset } => n("RelOffset").rel("input", input).rex("offset", offset),
        RelOp::Project { input, projections } => n("RelProject")
            .rel("input", input)
            .rexes("projections", projections),
        RelOp::Scan { rex } => n("RelScan").rex("rex", rex),
        RelOp::Sort { input, collations } => {
            n("RelSort")
                .rel("input", input)
                .list("collations", collations, collation_tree)
        }
        RelOp::Union { left, right, all } => n("RelUnion")
            .text("all", all)
            .rel("left", left)
            .rel("right", right),
        RelOp::Unpivot { rex } => n("RelUnpivot").rex("rex", rex),
        RelOp::Window {
            input,
            name,
            bindings,
            functions,
            collations,
            partitions,
        } => {
            let mut w = n("RelWindow");
            if let Some(name) = name {
                w = w.text("name", name_text(name));
            }
            let w = bindings.iter().enumerate().fold(w, |w, (i, b)| {
                w.text(&format!("bindings[{i}]"), name_text(b))
            });
            w.rel("input", input)
                .list("functions", functions, |f| {
                    let sig = f.signature();
                    let s = TreeNode::new("WindowFunctionNode", None)
                        .text("name", name_text(sig.name()))
                        .text("ignore_nulls", sig.is_ignore_nulls());
                    let s = sig
                        .params()
                        .iter()
                        .enumerate()
                        .fold(s, |s, (i, p)| s.shape(format!("params[{i}]"), p));
                    s.shape("returns", sig.returns()).rexes("args", f.args())
                })
                .list("collations", collations, collation_tree)
                .rexes("partitions", partitions)
        }
        RelOp::With { input, elements } => {
            n("RelWith")
                .rel("input", input)
                .list("elements", elements, |e| {
                    TreeNode::new("WithListElement", None)
                        .text("name", name_text(e.name()))
                        .rex("representation", e.representation())
                })
        }
    }
}

/// The default spelling of a shape in a dump: `PartiqlShape`'s `Display`, recursively, plus
/// what `Display` leaves out: unspecified parameters (`{unspecified: precision}`) and a non-zero
/// node id (`#id`).
pub fn render_shape(shape: &PartiqlShape) -> String {
    let mut out = String::new();
    write_shape(&mut out, shape);
    out
}

fn write_shape(out: &mut String, shape: &PartiqlShape) {
    match shape {
        PartiqlShape::Static(st) => {
            if st.is_not_nullable() {
                out.push_str("NOT NULL ");
            }
            match st.ty() {
                // Only rows and `new_any()` (`{...}`) are written compactly; any other struct
                // shows its constraints, so that unequal structs print differently.
                Static::Struct(s) if !is_canonical_struct(s) => {
                    let _ = write!(out, "{s:?}");
                }
                Static::Struct(s) => {
                    out.push_str(if s.is_row() { "Row{" } else { "{" });
                    for (i, f) in s.fields().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        let opt = if f.is_optional() { "?" } else { "" };
                        let _ = write!(out, "{}{opt}: ", name_text(f.name()));
                        write_shape(out, f.ty());
                    }
                    if s.is_partial() {
                        out.push_str(if s.fields().next().is_some() {
                            ", ...}"
                        } else {
                            "...}"
                        });
                    } else {
                        out.push('}');
                    }
                }
                Static::Bag(b) => {
                    out.push_str("<<");
                    write_shape(out, b.element_type());
                    out.push_str(">>");
                }
                Static::Array(a) => {
                    out.push('[');
                    write_shape(out, a.element_type());
                    out.push(']');
                }
                Static::Map(m) => {
                    out.push_str("Map<");
                    write_shape(out, m.key_type());
                    out.push_str(", ");
                    write_shape(out, m.value_type());
                    out.push('>');
                }
                // The encoding is a free-form string; escape it as a name.
                Static::Variant(v) => {
                    let _ = write!(out, "Variant({})", name_text(v.encoding()));
                }
                other => {
                    let _ = write!(out, "{other}");
                }
            }
            let p = st.params();
            if !p.is_default() {
                let flags: Vec<&str> = [
                    (p.unspecified_length, "length"),
                    (p.unspecified_precision, "precision"),
                    (p.unspecified_scale, "scale"),
                    (p.unspecified_fractional_precision, "fractional_precision"),
                ]
                .into_iter()
                .filter_map(|(set, name)| set.then_some(name))
                .collect();
                // `TypeParams` is non-exhaustive: name a flag added later generically.
                let flags = if flags.is_empty() {
                    vec!["other"]
                } else {
                    flags
                };
                let _ = write!(out, " {{unspecified: {}}}", flags.join(", "));
            }
            let id = st.ty_id().0;
            if id != 0 {
                let _ = write!(out, " #{id}");
            }
        }
        PartiqlShape::AnyOf(a) => {
            out.push_str("AnyOf(");
            for (i, t) in a.types().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_shape(out, t);
            }
            out.push(')');
        }
        other => {
            let _ = write!(out, "{other}");
        }
    }
}

/// Whether `s` is one of the typed planner's two struct kinds: a row exactly as built by
/// `StructType::new_row`, or `StructType::new_any()` (`STRUCT`).
// Rebuilding and comparing clones the fields, so this costs O(size of the subtree) per struct
// level, superlinear in row nesting depth (bounded by `MAX_DEPTH`). partiql-types exposes no
// getter for a struct's constraints that would make it O(1).
pub(crate) fn is_canonical_struct(s: &StructType) -> bool {
    *s == StructType::new_row(s.fields().cloned()) || *s == StructType::new_any()
}

/// A name as written in a dump: bare if it is a non-empty run of ASCII letters, digits, `_` and
/// `$`, otherwise a double-quoted, escaped string literal (Rust escaping: `"`, `\` and control
/// characters such as a newline are escaped), so that a name cannot be mistaken for dump syntax
/// or start a new line.
pub fn name_text(name: &str) -> String {
    let simple = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if simple {
        name.to_string()
    } else {
        format!("{name:?}")
    }
}

/// A qualified name as written in a dump: every part a double-quoted, escaped string literal
/// (Rust escaping), joined by `.`, so parts cannot hold dump syntax or line breaks.
fn qualified_text(q: &crate::QualifiedName) -> String {
    std::iter::once(q.catalog())
        .chain(q.namespace().iter().map(String::as_str))
        .chain(std::iter::once(q.name()))
        .map(|p| format!("{p:?}"))
        .collect::<Vec<_>>()
        .join(".")
}

/// A literal as written in a dump: its kind, then its value. Strings, tuple keys and any other
/// text are escaped as Rust string literals; collections, tuples and graph element values are
/// printed element by element, so the printer controls (and [`crate::validate`] bounds) the
/// recursion. A variant (an embedded document, e.g. Ion) prints only its type, not its
/// content, which this crate cannot bound: `Variant "ion"`. E.g. `Integer 1`, `Real 1.0`,
/// `String "a"`, `List [Integer 1, Integer 2]`.
pub fn value_text(v: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, v, &mut SeenGraphs::new());
    out
}

fn write_value(out: &mut String, v: &Value, seen: &mut SeenGraphs) {
    let items = |out: &mut String,
                 seen: &mut SeenGraphs,
                 open: &str,
                 vs: &mut dyn Iterator<Item = &Value>,
                 close| {
        out.push_str(open);
        for (i, x) in vs.enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_value(out, x, seen);
        }
        out.push_str(close);
    };
    match v {
        Value::Null => out.push_str("NULL"),
        Value::Missing => out.push_str("MISSING"),
        Value::Boolean(b) => {
            let _ = write!(out, "Boolean {b}");
        }
        Value::Integer(i) => {
            let _ = write!(out, "Integer {i}");
        }
        Value::Real(r) => {
            let _ = write!(out, "Real {:?}", r.0);
        }
        Value::Decimal(d) => {
            let _ = write!(out, "Decimal {d}");
        }
        Value::String(s) => {
            let _ = write!(out, "String {:?}", s.as_str());
        }
        Value::List(l) => items(out, seen, "List [", &mut l.iter(), "]"),
        Value::Bag(b) => items(out, seen, "Bag <<", &mut b.iter(), ">>"),
        Value::Tuple(t) => {
            out.push_str("Tuple {");
            for (i, (k, x)) in t.pairs().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                let _ = write!(out, "{k:?}: ");
                write_value(out, x, seen);
            }
            out.push('}');
        }
        Value::Graph(g) => match g.as_ref() {
            Graph::Simple(g) => {
                let key = std::rc::Rc::as_ptr(g);
                if let Some(n) = seen.get(&key) {
                    let _ = write!(out, "Graph #{n}");
                } else {
                    let n = seen.len();
                    seen.insert(key, n);
                    let _ = write!(out, "Graph #{n} ");
                    write_graph(out, g, seen);
                }
            }
            // `Graph` is non-exhaustive; a kind added later prints opaquely (documented).
            _ => out.push_str("Graph <unknown kind>"),
        },
        Value::Variant(x) => {
            let _ = write!(out, "Variant {:?}", x.type_tag().name());
        }
        // Scalars with no nesting (datetime, blob, and kinds added later): their Debug text,
        // escaped.
        other => {
            let _ = write!(out, "Value {:?}", format!("{other:?}"));
        }
    }
}

/// A simple graph: each node and edge by index with its labels (sorted, escaped) and value,
/// then every directed and undirected connection as `(from -[edge]-> to)` / `(a -[edge]- b)`,
/// so graphs that differ in labels or connectivity print differently. A label the graph's
/// interner does not know prints as `<label #key>` (the interner's
/// 1-based key) instead of panicking; a key from another interner that happens to be in range
/// prints as that label, which cannot be detected here.
fn write_graph(out: &mut String, g: &SimpleGraph, seen: &mut SeenGraphs) {
    let elem = |out: &mut String, seen: &mut SeenGraphs, i: usize, e: &partiql_value::GElem| {
        let mut labels: Vec<String> = e
            .labels
            .0
            .iter()
            .map(|l| match g.labels.try_resolve(&l.0) {
                Some(name) => format!("{name:?}"),
                None => format!("<label #{}>", l.0.into_inner()),
            })
            .collect();
        labels.sort_unstable();
        let _ = write!(out, "{i} [{}] ", labels.join(", "));
        match &e.value {
            Some(x) => write_value(out, x, seen),
            None => out.push('_'),
        }
    };
    let list =
        |out: &mut String, seen: &mut SeenGraphs, name: &str, es: &[partiql_value::GElem]| {
            let _ = write!(out, "{name}: [");
            for (i, e) in es.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                elem(out, seen, i, e);
            }
            out.push(']');
        };
    let links = |out: &mut String,
                 name: &str,
                 ts: &[(
        partiql_value::GNodeId,
        partiql_value::GEdgeId,
        partiql_value::GNodeId,
    )],
                 arrow: &str| {
        let _ = write!(out, "{name}: [");
        for (i, (a, e, b)) in ts.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "({} -[{}]{arrow} {})", a.0, e.0, b.0);
        }
        out.push(']');
    };
    out.push('{');
    list(out, seen, "nodes", &g.nodes);
    out.push_str(", ");
    list(out, seen, "edges", &g.edges);
    out.push_str(", ");
    links(out, "directed", &g.g_dir, "->");
    out.push_str(", ");
    links(out, "undirected", &g.g_undir, "-");
    out.push('}');
}

fn render_rel_type(ty: &RelType, shape: &dyn Fn(&PartiqlShape) -> String) -> String {
    let fields: Vec<String> = ty
        .fields()
        .iter()
        .map(|f| format!("{}: {}", name_text(f.name()), shape(f.ty())))
        .collect();
    let ordered = if ty.is_ordered() { " ordered" } else { "" };
    format!("Rel[{}]{ordered}", fields.join(", "))
}

impl TreeNode<'_> {
    /// Writes the tree as indented text, one node per line:
    /// `role: Kind :: TYPE {attr: value, ..}`, children indented by two spaces.
    ///
    /// A graph value shared by several literals (graphs are reference-counted) is printed in
    /// full once, as `Graph #n {..}`, and as `Graph #n` wherever it appears again.
    pub fn write_text(&self, out: &mut String, shape: &dyn Fn(&PartiqlShape) -> String) {
        self.write_at(out, None, 0, shape, &mut SeenGraphs::new())
    }

    fn write_at(
        &self,
        out: &mut String,
        role: Option<&str>,
        depth: usize,
        shape: &dyn Fn(&PartiqlShape) -> String,
        seen: &mut SeenGraphs,
    ) {
        for _ in 0..depth {
            out.push_str("  ");
        }
        if let Some(role) = role {
            let _ = write!(out, "{role}: ");
        }
        out.push_str(self.kind);
        match self.ty {
            Some(NodeType::Rex(t)) => {
                let _ = write!(out, " :: {}", shape(t));
            }
            Some(NodeType::Rel(t)) => {
                let _ = write!(out, " :: {}", render_rel_type(t, shape));
            }
            None => {}
        }
        if !self.attrs.is_empty() {
            let attrs: Vec<String> = self
                .attrs
                .iter()
                .map(|(k, v)| match v {
                    Attr::Text(s) => format!("{k}: {s}"),
                    Attr::Shape(s) => format!("{k}: {}", shape(s)),
                    Attr::Value(v) => {
                        let mut text = String::new();
                        write_value(&mut text, v, seen);
                        format!("{k}: {text}")
                    }
                })
                .collect();
            let _ = write!(out, " {{{}}}", attrs.join(", "));
        }
        out.push('\n');
        for (role, child) in &self.children {
            child.write_at(out, Some(role), depth + 1, shape, seen);
        }
    }
}

impl Plan {
    /// The plan as indented text (see [`TreeNode::write_text`]), shapes spelled by
    /// [`render_shape`].
    pub fn dump(&self) -> String {
        self.dump_with(&render_shape)
    }

    /// The plan as indented text, shapes spelled by `shape`.
    ///
    /// A plan nested deeper than [`crate::MAX_DEPTH`] is not printed (that would risk a stack
    /// overflow); the result is a one-line notice instead, the same for every such plan. Use
    /// [`Plan::dump_with_max_depth`] with the limit given to [`crate::validate_with_max_depth`].
    pub fn dump_with(&self, shape: &dyn Fn(&PartiqlShape) -> String) -> String {
        self.dump_with_max_depth(crate::MAX_DEPTH, shape)
    }

    /// [`Plan::dump_with`] with a different depth limit, for callers whose stack allows deeper
    /// plans (see [`crate::MAX_DEPTH`]).
    pub fn dump_with_max_depth(
        &self,
        max_depth: usize,
        shape: &dyn Fn(&PartiqlShape) -> String,
    ) -> String {
        if crate::validate::exceeds_depth(self, max_depth) {
            return format!("<plan nested deeper than {max_depth}; not printed>\n");
        }
        let mut out = String::new();
        tree(self).write_text(&mut out, shape);
        out
    }
}
