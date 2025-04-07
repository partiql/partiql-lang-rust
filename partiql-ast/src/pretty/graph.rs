use crate::ast::*;
use crate::pretty::pretty_parenthesized_expr;
use partiql_common::pretty::{
    pretty_doc_list, pretty_list, pretty_parenthesized_doc, pretty_surrounded,
    pretty_surrounded_doc, PrettyDoc, PRETTY_INDENT_MINOR_NEST,
};
use pretty::{DocAllocator, DocBuilder};
use std::borrow::Cow;

impl PrettyDoc for GraphMatch {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let GraphMatch {
            expr,
            pattern,
            shape,
        } = self;
        let head = arena.intersperse([expr.pretty_doc(arena), arena.text("MATCH")], arena.space());
        let patterns = pattern.pretty_doc(arena).group();
        let mut match_expr = arena.intersperse([head, patterns], arena.space());
        let shapes = [
            shape.rows.as_ref().map(|d| d.pretty_doc(arena)),
            shape.cols.as_ref().map(|d| d.pretty_doc(arena)),
            shape.export.as_ref().map(|d| d.pretty_doc(arena)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if !shapes.is_empty() {
            let docs = std::iter::once(match_expr).chain(shapes);
            match_expr = arena.intersperse(docs, arena.hardline());
        }

        pretty_parenthesized_doc(match_expr, arena)
    }
}

impl PrettyDoc for GraphTableRows {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphTableRows::OneRowPerMatch => arena.text("ONE ROW PER MATCH"),
            GraphTableRows::OneRowPerVertex { v, in_paths } => {
                let prefix = arena.text("ONE ROW PER NODE");
                let spec = pretty_parenthesized_doc(arena.text(&v.value), arena);
                let in_paths = in_paths.as_ref().map(|paths| {
                    let paths = pretty_parenthesized_doc(
                        pretty_doc_list(paths.iter().map(|p| arena.text(&p.value)), 0, arena),
                        arena,
                    );
                    [arena.text("IN"), paths]
                });
                arena.intersperse(
                    [Some([prefix, spec]), in_paths]
                        .into_iter()
                        .flatten()
                        .flatten(),
                    arena.softline(),
                )
            }
            GraphTableRows::OneRowPerStep {
                v1,
                e,
                v2,
                in_paths,
            } => {
                let prefix = arena.text("ONE ROW PER STEP");
                let step = pretty_doc_list(
                    [v1, e, v2].into_iter().map(|n| arena.text(&n.value)),
                    0,
                    arena,
                );
                let spec = pretty_parenthesized_doc(step, arena);
                let in_paths = in_paths.as_ref().map(|paths| {
                    let paths =
                        pretty_doc_list(paths.iter().map(|p| arena.text(&p.value)), 0, arena);
                    [arena.text("IN"), pretty_parenthesized_doc(paths, arena)]
                });
                arena.intersperse(
                    [Some([prefix, spec]), in_paths]
                        .into_iter()
                        .flatten()
                        .flatten(),
                    arena.softline(),
                )
            }
        }
    }
}

impl PrettyDoc for GraphTableColumns {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let col_defs = pretty_list(&self.columns, 0, arena);
        arena.intersperse(
            [
                arena.text("COLUMNS"),
                pretty_parenthesized_doc(col_defs, arena),
            ],
            arena.space(),
        )
    }
}

impl PrettyDoc for GraphTableColumnDef {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphTableColumnDef::Expr(expr, as_ident) => {
                let parts = if let Some(as_ident) = as_ident {
                    vec![
                        expr.pretty_doc(arena),
                        arena.text("AS"),
                        arena.text(&as_ident.value),
                    ]
                } else {
                    vec![expr.pretty_doc(arena)]
                };
                arena.intersperse(parts, arena.space())
            }
            GraphTableColumnDef::AllProperties(_) => {
                unreachable!()
            }
        }
    }
}

impl PrettyDoc for GraphTableExport {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphTableExport::AllSingletons { except } => {
                let prefix = arena.text("EXPORT ALL SINGLETONS");
                if let Some(except) = except {
                    let except =
                        pretty_doc_list(except.iter().map(|s| arena.text(&s.value)), 0, arena);
                    let parts = [
                        prefix,
                        arena.text("EXCEPT"),
                        pretty_parenthesized_doc(except, arena),
                    ];
                    arena.intersperse(parts, arena.space())
                } else {
                    prefix
                }
            }
            GraphTableExport::Singletons { exports } => {
                let prefix = arena.text("EXPORT SINGLETONS");
                let exports =
                    pretty_doc_list(exports.iter().map(|e| arena.text(&e.value)), 0, arena);
                let parts = [prefix, pretty_parenthesized_doc(exports, arena)];
                arena.intersperse(parts, arena.space())
            }
            GraphTableExport::NoSingletons => arena.text("EXPORT NO SINGLETONS"),
        }
    }
}

impl PrettyDoc for GraphPattern {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let GraphPattern {
            mode,
            patterns,
            keep,
            where_clause,
        } = &self;
        let patterns = pretty_list(patterns, PRETTY_INDENT_MINOR_NEST, arena).group();
        let parts = [
            mode.as_ref().map(|inner| inner.pretty_doc(arena)),
            Some(patterns),
            keep.as_ref().map(|keep| {
                arena.intersperse([arena.text("KEEP"), keep.pretty_doc(arena)], arena.space())
            }),
            where_clause.as_ref().map(|clause| {
                arena.intersperse(
                    [arena.text("WHERE"), clause.pretty_doc(arena)],
                    arena.space(),
                )
            }),
        ]
        .into_iter()
        .flatten();

        arena.intersperse(parts, arena.space())
    }
}

impl PrettyDoc for GraphMatchMode {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphMatchMode::DifferentEdges => arena.text("DIFFERENT EDGES"),
            GraphMatchMode::RepeatableElements => arena.text("REPEATABLE ELEMENTS"),
        }
    }
}

impl PrettyDoc for GraphPathPrefix {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphPathPrefix::Mode(mode) => mode.pretty_doc(arena),
            GraphPathPrefix::Search(search, mode) => {
                let mode = mode.as_ref().map(|mode| mode.pretty_doc(arena));
                let parts = match search {
                    GraphPathSearchPrefix::All => vec![Some(arena.text("ALL")), mode],
                    GraphPathSearchPrefix::Any => vec![Some(arena.text("ANY")), mode],
                    GraphPathSearchPrefix::AnyK(k) => vec![
                        Some(arena.text("ANY")),
                        Some(arena.text(k.to_string())),
                        mode,
                    ],
                    GraphPathSearchPrefix::AllShortest => {
                        vec![Some(arena.text("ALL")), Some(arena.text("SHORTEST")), mode]
                    }
                    GraphPathSearchPrefix::AnyShortest => {
                        vec![Some(arena.text("ANY")), Some(arena.text("SHORTEST")), mode]
                    }
                    GraphPathSearchPrefix::ShortestK(k) => vec![
                        Some(arena.text("SHORTEST")),
                        Some(arena.text(k.to_string())),
                        mode,
                    ],
                    GraphPathSearchPrefix::ShortestKGroup(k) => vec![
                        Some(arena.text("SHORTEST")),
                        k.as_ref().map(|k| arena.text(k.to_string())),
                        mode,
                        Some(arena.text("GROUPS")),
                    ],
                }
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
                arena.intersperse(parts, arena.space())
            }
        }
    }
}

impl PrettyDoc for GraphPathMode {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let txt = match self {
            GraphPathMode::Walk => "WALK",
            GraphPathMode::Trail => "TRAIL",
            GraphPathMode::Acyclic => "ACYCLIC",
            GraphPathMode::Simple => "SIMPLE",
        };
        arena.text(txt)
    }
}

impl PrettyDoc for GraphPathPattern {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let var = self
            .variable
            .as_ref()
            .map(|var| arena.intersperse([arena.text(&var.value), arena.text("=")], arena.space()));

        let prefix = self.prefix.as_ref().map(|prefix| prefix.pretty_doc(arena));

        let parts = [var, prefix, Some(self.path.pretty_doc(arena))]
            .into_iter()
            .flatten();
        arena.intersperse(parts, arena.space())
    }
}

impl PrettyDoc for GraphPathSubPattern {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let var = self
            .variable
            .as_ref()
            .map(|var| arena.intersperse([arena.text(&var.value), arena.text("=")], arena.space()));

        let mode = self.mode.as_ref().map(|prefix| prefix.pretty_doc(arena));
        let where_clause = self.where_clause.as_ref().map(|clause| {
            arena.intersperse(
                [arena.text("WHERE"), clause.pretty_doc(arena)],
                arena.space(),
            )
        });

        let parts = [var, mode, Some(self.path.pretty_doc(arena)), where_clause]
            .into_iter()
            .flatten();
        arena.intersperse(parts, arena.space())
    }
}

impl PrettyDoc for GraphMatchPathPattern {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphMatchPathPattern::Path(path) => {
                arena.intersperse(path.iter().map(|e| e.pretty_doc(arena)), arena.space())
            }
            GraphMatchPathPattern::Quantified(GraphMatchPathPatternQuantified { path, quant }) => {
                arena.concat([path.pretty_doc(arena), quant.pretty_doc(arena)])
            }
            GraphMatchPathPattern::Questioned(p) => {
                arena.concat([p.pretty_doc(arena), arena.text("?")])
            }
            GraphMatchPathPattern::Sub(path) => pretty_surrounded(path, "(", ")", arena),
            GraphMatchPathPattern::Node(node) => node.pretty_doc(arena),
            GraphMatchPathPattern::Edge(edge) => edge.pretty_doc(arena),
            GraphMatchPathPattern::Union(u) => {
                arena.intersperse(u.iter().map(|l| l.pretty_doc(arena)), arena.text(" | "))
            }
            GraphMatchPathPattern::Multiset(s) => {
                arena.intersperse(s.iter().map(|l| l.pretty_doc(arena)), arena.text(" |+| "))
            }
            GraphMatchPathPattern::Simplified(simplified) => simplified.pretty_doc(arena),
        }
    }
}

impl PrettyDoc for GraphMatchSimplified {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let ends = match self.dir {
            GraphMatchDirection::Left => ["<-/", "/-"],
            GraphMatchDirection::Undirected => ["~/", "/~"],
            GraphMatchDirection::Right => ["-/", "/->"],
            GraphMatchDirection::LeftOrUndirected => ["<~/", "/~"],
            GraphMatchDirection::UndirectedOrRight => ["~/", "/~>"],
            GraphMatchDirection::LeftOrRight => ["<-/", "/->"],
            GraphMatchDirection::LeftOrUndirectedOrRight => ["-/", "/-"],
        };

        let parts = [
            arena.text(ends[0]),
            self.pattern.pretty_doc(arena),
            arena.text(ends[1]),
        ];
        arena.intersperse(parts, arena.space()).group()
    }
}

impl PrettyDoc for GraphMatchSimplifiedPattern {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphMatchSimplifiedPattern::Union(u) => {
                arena.intersperse(u.iter().map(|l| l.pretty_doc(arena)), arena.text(" | "))
            }
            GraphMatchSimplifiedPattern::Multiset(s) => {
                arena.intersperse(s.iter().map(|l| l.pretty_doc(arena)), arena.text(" |+| "))
            }
            GraphMatchSimplifiedPattern::Path(path) => {
                arena.intersperse(path.iter().map(|e| e.pretty_doc(arena)), arena.space())
            }
            GraphMatchSimplifiedPattern::Sub(path) => pretty_surrounded(path, "(", ")", arena),
            GraphMatchSimplifiedPattern::Conjunction(c) => {
                arena.intersperse(c.iter().map(|l| l.pretty_doc(arena)), arena.text("&"))
            }
            GraphMatchSimplifiedPattern::Questioned(p) => {
                arena.concat([p.pretty_doc(arena), arena.text("?")])
            }
            GraphMatchSimplifiedPattern::Quantified(GraphMatchSimplifiedPatternQuantified {
                path,
                quant,
            }) => arena.concat([path.pretty_doc(arena), quant.pretty_doc(arena)]),
            GraphMatchSimplifiedPattern::Direction(GraphMatchSimplifiedPatternDirected {
                dir,
                path,
            }) => {
                let path = path.pretty_doc(arena);
                let parts = match dir {
                    GraphMatchDirection::Left => vec![arena.text("<"), path],
                    GraphMatchDirection::Undirected => vec![arena.text("~"), path],
                    GraphMatchDirection::Right => vec![path, arena.text(">")],
                    GraphMatchDirection::LeftOrUndirected => vec![arena.text("<~"), path],
                    GraphMatchDirection::UndirectedOrRight => {
                        vec![arena.text("~"), path, arena.text(">")]
                    }
                    GraphMatchDirection::LeftOrRight => {
                        vec![arena.text("<"), path, arena.text(">")]
                    }
                    GraphMatchDirection::LeftOrUndirectedOrRight => vec![arena.text("-"), path],
                };
                arena.concat(parts).group()
            }
            GraphMatchSimplifiedPattern::Negated(l) => {
                arena.concat([arena.text("!"), l.pretty_doc(arena)])
            }
            GraphMatchSimplifiedPattern::Label(l) => arena.text(&l.value),
        }
    }
}

impl PrettyDoc for GraphMatchNode {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let name: Vec<_> = [
            self.variable.as_ref().map(|var| arena.text(&var.value)),
            self.label
                .as_ref()
                .map(|label| arena.concat([arena.text(":"), label.pretty_doc(arena)])),
        ]
        .into_iter()
        .flatten()
        .collect();
        let name = if name.is_empty() {
            None
        } else {
            Some(arena.concat(name))
        };
        let where_clause = self.where_clause.as_ref().map(|clause| {
            arena.intersperse(
                [arena.text("WHERE"), clause.pretty_doc(arena)],
                arena.space(),
            )
        });
        let parts = [name, where_clause].into_iter().flatten();

        let spec = arena.intersperse(parts, arena.space());
        pretty_surrounded_doc(spec, "(", ")", arena).group()
    }
}

impl PrettyDoc for GraphMatchEdge {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let name: Vec<_> = [
            self.variable.as_ref().map(|var| arena.text(&var.value)),
            self.label
                .as_ref()
                .map(|label| arena.concat([arena.text(":"), label.pretty_doc(arena)])),
        ]
        .into_iter()
        .flatten()
        .collect();
        let name = if name.is_empty() {
            None
        } else {
            Some(arena.concat(name))
        };
        let where_clause = self.where_clause.as_ref().map(|clause| {
            arena.intersperse(
                [arena.text("WHERE"), clause.pretty_doc(arena)],
                arena.space(),
            )
        });
        let parts = [name, where_clause]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let edge = if !parts.is_empty() {
            let (prefix, suffix) = match self.direction {
                GraphMatchDirection::Right => ("-[", "]->"),
                GraphMatchDirection::Left => ("<-[", "]-"),
                GraphMatchDirection::Undirected => ("~[", "]~"),
                GraphMatchDirection::UndirectedOrRight => ("~[", "]~>"),
                GraphMatchDirection::LeftOrUndirected => ("<~[", "]~"),
                GraphMatchDirection::LeftOrRight => ("<-[", "]->"),
                GraphMatchDirection::LeftOrUndirectedOrRight => ("-[", "]-"),
            };
            let spec = arena.intersperse(parts, arena.space());
            pretty_surrounded_doc(spec, prefix, suffix, arena)
        } else {
            let edge = match self.direction {
                GraphMatchDirection::Right => "->",
                GraphMatchDirection::Left => "<-",
                GraphMatchDirection::Undirected => "~",
                GraphMatchDirection::UndirectedOrRight => "~>",
                GraphMatchDirection::LeftOrUndirected => "<~",
                GraphMatchDirection::LeftOrRight => "<->",
                GraphMatchDirection::LeftOrUndirectedOrRight => "-",
            };
            arena.text(edge)
        };
        edge.group()
    }
}

impl PrettyDoc for GraphMatchLabel {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            GraphMatchLabel::Name(name) => arena.text(&name.value),
            GraphMatchLabel::Wildcard => arena.text("%"),
            GraphMatchLabel::Negated(l) => {
                arena.concat([arena.text("!"), pretty_parenthesized_expr(l, 0, arena)])
            }
            GraphMatchLabel::Conjunction(c) => pretty_parenthesized_doc(
                arena.intersperse(c.iter().map(|l| l.pretty_doc(arena)), arena.text("&")),
                arena,
            ),
            GraphMatchLabel::Disjunction(d) => pretty_parenthesized_doc(
                arena.intersperse(d.iter().map(|l| l.pretty_doc(arena)), arena.text("|")),
                arena,
            ),
        }
    }
}

impl PrettyDoc for GraphMatchQuantifier {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let GraphMatchQuantifier { lower, upper } = &self;
        match (lower, upper) {
            (0, None) => arena.text("*"),
            (1, None) => arena.text("+"),
            (l, u) => {
                let l = Cow::Owned(l.to_string());
                let u = u.map(|u| Cow::Owned(u.to_string())).unwrap_or("".into());
                pretty_surrounded_doc(arena.concat([l, ",".into(), u]), "{", "}", arena)
            }
        }
    }
}
