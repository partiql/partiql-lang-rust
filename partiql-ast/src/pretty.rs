use crate::ast::*;
use partiql_common::pretty::{
    pretty_list, pretty_parenthesized_doc, pretty_prefixed_doc, pretty_seperated,
    pretty_seperated_doc, pretty_seq, pretty_seq_doc, PrettyDoc, PRETTY_INDENT_MINOR_NEST,
    PRETTY_INDENT_SUBORDINATE_CLAUSE_NEST,
};
use pretty::{DocAllocator, DocBuilder};
impl<T> PrettyDoc for AstNode<T>
where
    T: PrettyDoc,
{
    #[inline]
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.node.pretty_doc(arena)
    }
}

impl PrettyDoc for TopLevelQuery {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        if self.with.is_some() {
            todo!("WITH Clause")
        }
        self.query.pretty_doc(arena)
    }
}

impl PrettyDoc for Query {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let Query {
            set,
            order_by,
            limit_offset,
        } = self;

        let clauses = [
            Some(set.pretty_doc(arena)),
            order_by.as_ref().map(|inner| inner.pretty_doc(arena)),
            limit_offset.as_ref().map(|inner| inner.pretty_doc(arena)),
        ]
        .into_iter()
        .flatten();

        arena.intersperse(clauses, arena.softline()).group()
    }
}

impl PrettyDoc for QuerySet {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            QuerySet::BagOp(op) => op.pretty_doc(arena),
            QuerySet::Select(sel) => sel.pretty_doc(arena),
            QuerySet::Expr(e) => e.pretty_doc(arena),
            QuerySet::Values(v) => pretty_prefixed_doc("VALUES", pretty_list(v, 0, arena), arena),
            QuerySet::Table(t) => pretty_prefixed_expr("TABLE", t, 0, arena),
        }
    }
}

impl PrettyDoc for BagOpExpr {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let op = match self.bag_op {
            BagOperator::Union => "UNION",
            BagOperator::Except => "EXCEPT",
            BagOperator::Intersect => "INTERSECT",
            BagOperator::OuterUnion => "OUTER UNION",
            BagOperator::OuterExcept => "OUTER EXCEPT",
            BagOperator::OuterIntersect => "OUTER INTERSECT",
        };
        let op = arena.text(op);
        let op = match self.setq {
            None => op,
            Some(SetQuantifier::All) => op.append(" ALL"),
            Some(SetQuantifier::Distinct) => op.append(" DISTINCT"),
        };

        let lhs = pretty_parenthesized_expr(&self.lhs, PRETTY_INDENT_MINOR_NEST, arena);
        let rhs = pretty_parenthesized_expr(&self.rhs, PRETTY_INDENT_MINOR_NEST, arena);

        arena.intersperse([lhs, op, rhs], arena.hardline()).group()
    }
}

impl PrettyDoc for QueryTable {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.table_name.pretty_doc(arena)
    }
}

impl PrettyDoc for Select {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        fn format<'b, C, D, A>(child: &'b C, arena: &'b D) -> DocBuilder<'b, D, A>
        where
            D: DocAllocator<'b, A>,
            D::Doc: Clone,
            A: Clone,
            C: PrettyDoc,
        {
            child.pretty_doc(arena).group()
        }

        fn delegate<'b, C, D, A>(child: &'b Option<C>, arena: &'b D) -> Option<DocBuilder<'b, D, A>>
        where
            D: DocAllocator<'b, A>,
            D::Doc: Clone,
            A: Clone,
            C: PrettyDoc,
        {
            child.as_ref().map(|inner| format(inner, arena))
        }

        let Select {
            project,
            exclude,
            from,
            from_let,
            where_clause,
            group_by,
            having,
        } = self;
        let mut clauses = [
            Some(format(project, arena)),
            delegate(exclude, arena),
            delegate(from, arena),
            delegate(from_let, arena),
            delegate(where_clause, arena),
            delegate(group_by, arena),
            delegate(having, arena),
        ]
        .into_iter()
        .flatten();

        let mut result = arena.nil();
        let separator = arena.line();
        if let Some(first) = clauses.next() {
            let mut curr = first;

            for clause in clauses {
                result = result.append(curr.append(separator.clone()).group());
                curr = clause;
            }

            result = result.append(curr);
        }

        result
    }
}

impl PrettyDoc for Projection {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        if self.setq.is_some() {
            todo!("project SetQuantifier")
        }
        self.kind.pretty_doc(arena)
    }
}

impl PrettyDoc for ProjectionKind {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            ProjectionKind::ProjectStar => arena.text("SELECT *"),
            ProjectionKind::ProjectList(l) => pretty_prefixed_doc(
                "SELECT",
                pretty_list(l, PRETTY_INDENT_MINOR_NEST, arena),
                arena,
            ),
            ProjectionKind::ProjectPivot(ProjectPivot { key, value }) => {
                let parts = [
                    value.pretty_doc(arena),
                    arena.text("AT"),
                    key.pretty_doc(arena),
                ];
                let decl = arena.intersperse(parts, arena.space()).group();
                pretty_prefixed_doc("PIVOT", decl, arena)
            }
            ProjectionKind::ProjectValue(ctor) => {
                pretty_prefixed_expr("SELECT VALUE", ctor, PRETTY_INDENT_MINOR_NEST, arena)
            }
        }
        .group()
    }
}

impl PrettyDoc for ProjectItem {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            ProjectItem::ProjectAll(_) => {
                todo!("ProjectItem::ProjectAll; remove this?")
            }
            ProjectItem::ProjectExpr(e) => e.pretty_doc(arena),
        }
    }
}

impl PrettyDoc for ProjectExpr {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_source_as_alias(&self.expr, self.as_alias.as_ref(), arena)
            .unwrap_or_else(|| self.expr.pretty_doc(arena))
    }
}

impl PrettyDoc for Exclusion {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_prefixed_doc(
            "EXCLUDE",
            pretty_list(&self.items, PRETTY_INDENT_MINOR_NEST, arena),
            arena,
        )
    }
}

impl PrettyDoc for ExcludePath {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let ExcludePath { root, steps } = self;
        let mut path = root.pretty_doc(arena);
        for step in steps {
            path = path.append(match step {
                ExcludePathStep::PathProject(e) => arena.text(".").append(e.pretty_doc(arena)),
                ExcludePathStep::PathIndex(e) => arena
                    .text("[")
                    .append(e.pretty_doc(arena))
                    .append(arena.text("]")),
                ExcludePathStep::PathForEach => arena.text("[*]"),
                ExcludePathStep::PathUnpivot => arena.text(".*"),
            });
        }

        path
    }
}

impl PrettyDoc for Expr {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Expr::Lit(inner) => inner.pretty_doc(arena),
            Expr::VarRef(inner) => inner.pretty_doc(arena),
            Expr::BinOp(inner) => inner.pretty_doc(arena),
            Expr::UniOp(inner) => inner.pretty_doc(arena),
            Expr::Like(inner) => inner.pretty_doc(arena),
            Expr::Between(inner) => inner.pretty_doc(arena),
            Expr::In(inner) => inner.pretty_doc(arena),
            Expr::Case(inner) => inner.pretty_doc(arena),
            Expr::Struct(inner) => inner.pretty_doc(arena),
            Expr::Bag(inner) => inner.pretty_doc(arena),
            Expr::List(inner) => inner.pretty_doc(arena),
            Expr::Sexp(inner) => inner.pretty_doc(arena),
            Expr::Path(inner) => inner.pretty_doc(arena),
            Expr::Call(inner) => inner.pretty_doc(arena),

            Expr::CallAgg(inner) => inner.pretty_doc(arena),

            Expr::Query(inner) => {
                let inner = inner.pretty_doc(arena).group();
                arena
                    .text("(")
                    .append(inner.nest(PRETTY_INDENT_SUBORDINATE_CLAUSE_NEST))
                    .append(arena.text(")"))
            }
            Expr::Error => {
                unreachable!();
            }
            Expr::GraphMatch(_inner) => {
                todo!("inner.pretty_doc(arena)")
            }
        }
        .group()
    }
}

impl PrettyDoc for Path {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let Path { root, steps } = self;
        let mut path = root.pretty_doc(arena);
        for step in steps {
            path = path.append(match step {
                PathStep::PathProject(e) => arena.text(".").append(e.index.pretty_doc(arena)),
                PathStep::PathIndex(e) => arena
                    .text("[")
                    .append(e.index.pretty_doc(arena))
                    .append(arena.text("]")),
                PathStep::PathForEach => arena.text("[*]"),
                PathStep::PathUnpivot => arena.text(".*"),
            });
        }

        path
    }
}

impl PrettyDoc for VarRef {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let name = self.name.pretty_doc(arena);
        match self.qualifier {
            ScopeQualifier::Unqualified => name,
            ScopeQualifier::Qualified => arena.text("@").append(name).group(),
        }
    }
}

impl PrettyDoc for Lit {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Lit::Null => arena.text("NULL"),
            Lit::Missing => arena.text("MISSING"),
            Lit::Int8Lit(inner) => arena.text(inner.to_string()),
            Lit::Int16Lit(inner) => arena.text(inner.to_string()),
            Lit::Int32Lit(inner) => arena.text(inner.to_string()),
            Lit::Int64Lit(inner) => arena.text(inner.to_string()),
            Lit::DecimalLit(inner) => inner.pretty_doc(arena),
            Lit::NumericLit(inner) => inner.pretty_doc(arena),
            Lit::RealLit(inner) => arena.text(inner.to_string()),
            Lit::FloatLit(inner) => arena.text(inner.to_string()),
            Lit::DoubleLit(inner) => arena.text(inner.to_string()),
            Lit::BoolLit(inner) => arena.text(inner.to_string()),
            Lit::EmbeddedDocLit(inner, _typ) => inner.pretty_doc(arena), // TODO better pretty for embedded doc: https://github.com/partiql/partiql-lang-rust/issues/508
            Lit::CharStringLit(inner) => inner.pretty_doc(arena),
            Lit::NationalCharStringLit(inner) => inner.pretty_doc(arena),
            Lit::BitStringLit(inner) => inner.pretty_doc(arena),
            Lit::HexStringLit(inner) => inner.pretty_doc(arena),
            Lit::StructLit(inner) => inner.pretty_doc(arena),
            Lit::BagLit(inner) => inner.pretty_doc(arena),
            Lit::ListLit(inner) => inner.pretty_doc(arena),
            Lit::TypedLit(s, ty) => {
                let ty = ty.pretty_doc(arena);
                let s = s.pretty_doc(arena);
                pretty_seperated_doc(arena.space(), [ty, s], 0, arena)
            }
        }
    }
}

impl PrettyDoc for Type {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Type::CustomType(cty) => cty.pretty_doc(arena),
            Type::NullType => arena.text("NULL"),
            Type::BooleanType => arena.text("BOOL"),
            Type::Integer2Type => arena.text("INT2"),
            Type::Integer4Type => arena.text("INT4"),
            Type::Integer8Type => arena.text("INT8"),
            Type::DecimalType => arena.text("DECIMAL"),
            Type::NumericType => arena.text("NUMERIC"),
            Type::RealType => arena.text("REAL"),
            Type::DoublePrecisionType => arena.text("DOUBLE PRECISION"),
            Type::TimestampType => arena.text("TIMESTAMP"),
            Type::CharacterType => arena.text("CHAR"),
            Type::CharacterVaryingType => arena.text("VARCHAR"),
            Type::MissingType => arena.text("MISSING"),
            Type::StringType => arena.text("STRING"),
            Type::SymbolType => arena.text("SYMBOL"),
            Type::BlobType => arena.text("BLOB"),
            Type::ClobType => arena.text("CLOB"),
            Type::DateType => arena.text("DATE"),
            Type::TimeType => arena.text("TIME"),
            Type::ZonedTimestampType => arena.text("TIMESTAMPTZ"),
            Type::StructType => arena.text("STRUCT"),
            Type::TupleType => arena.text("TUPLE"),
            Type::ListType => arena.text("LIST"),
            Type::SexpType => arena.text("SEXP"),
            Type::BagType => arena.text("BAG"),
            Type::AnyType => arena.text("ANY"),
        }
    }
}

impl PrettyDoc for CustomType {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seperated(arena.space(), &self.parts, 0, arena)
    }
}

impl PrettyDoc for CustomTypePart {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            CustomTypePart::Name(sym) => sym.pretty_doc(arena),
            CustomTypePart::Parameterized(sym, param) => {
                let sym = sym.pretty_doc(arena);
                let list = pretty_list(param, 0, arena);
                let list = pretty_parenthesized_doc(list, arena);
                sym.append(list)
            }
        }
    }
}

impl PrettyDoc for CustomTypeParam {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            CustomTypeParam::Lit(l) => l.pretty_doc(arena),
            CustomTypeParam::Type(ty) => ty.pretty_doc(arena),
        }
    }
}

impl PrettyDoc for BinOp {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let BinOp { kind, lhs, rhs } = self;
        let (nest, sym) = match kind {
            BinOpKind::Add => (0, "+"),
            BinOpKind::Div => (0, "/"),
            BinOpKind::Exp => (0, "^"),
            BinOpKind::Mod => (0, "%"),
            BinOpKind::Mul => (0, "*"),
            BinOpKind::Sub => (0, "-"),
            BinOpKind::And => (PRETTY_INDENT_MINOR_NEST, "AND"),
            BinOpKind::Or => (PRETTY_INDENT_MINOR_NEST, "OR"),
            BinOpKind::Concat => (0, "||"),
            BinOpKind::Eq => (0, "="),
            BinOpKind::Gt => (0, ">"),
            BinOpKind::Gte => (0, ">="),
            BinOpKind::Lt => (0, "<"),
            BinOpKind::Lte => (0, "<="),
            BinOpKind::Ne => (0, "<>"),
            BinOpKind::Is => (0, "IS"),
        };
        let op = arena.text(sym);
        let lhs = lhs.pretty_doc(arena).nest(nest);
        let rhs = rhs.pretty_doc(arena).nest(nest);
        let sep = if nest == 0 {
            arena.space()
        } else {
            arena.softline()
        };
        let expr = arena.intersperse([lhs, op, rhs], sep).group();
        pretty_parenthesized_doc(expr, arena).group()
    }
}

impl PrettyDoc for UniOp {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        // TODO NOT LIKE, NOT IN, NOT BETWEEN?
        let UniOp { kind, expr } = self;
        let (sym, paren) = match kind {
            UniOpKind::Pos => ("+", false),
            UniOpKind::Neg => ("-", false),
            UniOpKind::Not => ("NOT ", true),
        };
        let op = arena.text(sym);
        let expr = expr.pretty_doc(arena);
        if paren {
            let open = arena.text("(");
            let close = arena.text(")");
            arena.concat([op, open, expr, close]).group()
        } else {
            arena.concat([op, expr]).group()
        }
    }
}

impl PrettyDoc for Like {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let Like {
            value,
            pattern,
            escape,
        } = self;

        let sep = arena.space();
        let value = value.pretty_doc(arena);
        let kw_like = arena.text("LIKE");
        let pattern = pattern.pretty_doc(arena);
        if let Some(escape) = escape {
            let kw_esc = arena.text("ESCAPE");
            let escape = escape.pretty_doc(arena);
            arena.intersperse([value, kw_like, pattern, kw_esc, escape], sep)
        } else {
            arena.intersperse([value, kw_like, pattern], sep)
        }
        .group()
    }
}

impl PrettyDoc for Between {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let Between { value, from, to } = self;

        let value = value.pretty_doc(arena);
        let kw_b = arena.text("BETWEEN");
        let kw_a = arena.text("AND");
        let from = from.pretty_doc(arena);
        let to = to.pretty_doc(arena);
        let sep = arena.space();
        let expr = arena
            .intersperse([value, kw_b, from, kw_a, to], sep)
            .group();
        expr.group()
    }
}

impl PrettyDoc for In {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let In { lhs, rhs } = self;

        let kw_in = arena.text("IN");
        let lhs = lhs.pretty_doc(arena);
        let rhs = rhs.pretty_doc(arena);
        let sep = arena.space();
        let expr = arena.intersperse([lhs, kw_in, rhs], sep).group();
        expr.group()
    }
}

impl PrettyDoc for Case {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Case::SimpleCase(inner) => inner.pretty_doc(arena),
            Case::SearchedCase(inner) => inner.pretty_doc(arena),
        }
    }
}

impl PrettyDoc for SimpleCase {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let SimpleCase {
            expr,
            cases,
            default,
        } = self;

        let search = expr.pretty_doc(arena);
        let branches = case_branches(arena, cases, default);
        pretty_seq_doc(
            branches,
            "CASE",
            Some(search),
            "END",
            " ",
            PRETTY_INDENT_MINOR_NEST,
            arena,
        )
    }
}

impl PrettyDoc for SearchedCase {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let SearchedCase { cases, default } = self;

        let branches = case_branches(arena, cases, default);
        pretty_seq_doc(
            branches,
            "CASE",
            None,
            "END",
            " ",
            PRETTY_INDENT_MINOR_NEST,
            arena,
        )
    }
}

impl PrettyDoc for Struct {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let fields = self.fields.iter().map(|expr_pair| {
            let k = expr_pair.first.pretty_doc(arena);
            let v = expr_pair.second.pretty_doc(arena);
            let sep = arena.text(": ");

            k.append(sep).group().append(v).group()
        });
        pretty_seq_doc(fields, "{", None, "}", ",", PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for StructLit {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let fields = self.fields.iter().map(|expr_pair| {
            let k = expr_pair.first.pretty_doc(arena);
            let v = expr_pair.second.pretty_doc(arena);
            let sep = arena.text(": ");

            k.append(sep).group().append(v).group()
        });
        pretty_seq_doc(fields, "{", None, "}", ",", PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for Bag {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seq(
            &self.values,
            "<<",
            ">>",
            ",",
            PRETTY_INDENT_MINOR_NEST,
            arena,
        )
    }
}

impl PrettyDoc for List {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seq(&self.values, "[", "]", ",", PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for BagLit {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seq(
            &self.values,
            "<<",
            ">>",
            ",",
            PRETTY_INDENT_MINOR_NEST,
            arena,
        )
    }
}

impl PrettyDoc for ListLit {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_seq(&self.values, "[", "]", ",", PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for Sexp {
    fn pretty_doc<'b, D, A>(&'b self, _arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        todo!("remove s-expr from ast?");
    }
}

impl PrettyDoc for Call {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let name = self.func_name.pretty_doc(arena);
        let list = pretty_list(&self.args, 0, arena);
        name.append(arena.text("("))
            .append(list.nest(PRETTY_INDENT_MINOR_NEST))
            .append(arena.text(")"))
    }
}

impl PrettyDoc for CallAgg {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let name = self.func_name.pretty_doc(arena);
        let list = pretty_list(&self.args, 0, arena);
        name.append(arena.text("("))
            .append(list.nest(PRETTY_INDENT_MINOR_NEST))
            .append(arena.text(")"))
    }
}

impl PrettyDoc for CallArg {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            CallArg::Star() => arena.text("*"),
            CallArg::Positional(arg) => arg.pretty_doc(arena),
            CallArg::PositionalType(_) => {
                todo!("CallArg::PositionalType")
            }
            CallArg::Named(arg) => arg.pretty_doc(arena),
            CallArg::NamedType(_) => {
                todo!("CallArg::NamedType")
            }
        }
    }
}

impl PrettyDoc for CallArgNamed {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let CallArgNamed { name, value } = self;
        let name = name.pretty_doc(arena);
        let value = value.pretty_doc(arena);
        pretty_seperated_doc(":", [name, value], 0, arena)
    }
}

impl PrettyDoc for SymbolPrimitive {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let sym = arena.text(self.value.as_str());
        match self.case {
            CaseSensitivity::CaseSensitive => arena.text("\"").append(sym).append(arena.text("\"")),
            CaseSensitivity::CaseInsensitive => sym,
        }
    }
}

impl PrettyDoc for FromClause {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_prefixed_expr("FROM", &self.source, PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for FromSource {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            FromSource::FromLet(fl) => fl.pretty_doc(arena),
            FromSource::Join(join) => join.pretty_doc(arena),
        }
    }
}

impl PrettyDoc for FromLet {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let FromLet {
            expr,
            kind,
            as_alias,
            at_alias,
            by_alias,
        } = self;

        let expr = expr.pretty_doc(arena);
        let as_alias = pretty_as_alias(as_alias.as_ref(), arena);
        let at_alias = pretty_at_alias(at_alias.as_ref(), arena);
        let by_alias = pretty_by_alias(by_alias.as_ref(), arena);
        let aliases: Vec<_> = [as_alias, at_alias, by_alias]
            .into_iter()
            .flatten()
            .collect();

        let clause = match kind {
            FromLetKind::Scan => expr,
            FromLetKind::Unpivot => pretty_prefixed_doc("UNPIVOT", expr, arena),
        };

        if aliases.is_empty() {
            clause
        } else {
            clause.append(arena.concat(aliases).group())
        }
        .group()
    }
}

impl PrettyDoc for Join {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let Join {
            kind,
            left,
            right,
            predicate,
        } = self;

        let arms = [left.as_ref(), right.as_ref()];
        let kw_join = match kind {
            JoinKind::Cross => " CROSS JOIN ",
            JoinKind::Inner => " INNER JOIN ",
            JoinKind::Left => " LEFT JOIN ",
            JoinKind::Right => " RIGHT JOIN ",
            JoinKind::Full => " FULL JOIN ",
        };

        match (kind, predicate) {
            (JoinKind::Cross, Some(_)) => {
                todo!("CROSS JOIN with predicate")
            }
            (JoinKind::Cross, None) => pretty_list(arms, 0, arena),
            (_, None) => pretty_seperated(kw_join, arms, 0, arena),
            (_, Some(pred)) => match &pred.node {
                JoinSpec::Natural => {
                    let kw = arena.text(" NATURAL").append(kw_join);
                    pretty_seperated(kw, arms, 0, arena)
                }
                JoinSpec::On(on) => {
                    let join = pretty_seperated(kw_join, arms, 0, arena);
                    let pred = arena
                        .softline()
                        .append(arena.text("ON"))
                        .append(arena.softline())
                        .append(on.pretty_doc(arena).nest(PRETTY_INDENT_MINOR_NEST));
                    join.append(pred)
                }
                JoinSpec::Using(using) => {
                    let join = pretty_seperated(kw_join, arms, 0, arena);
                    let using = pretty_list(using, PRETTY_INDENT_MINOR_NEST, arena);
                    let pred = arena
                        .softline()
                        .append(arena.text("USING"))
                        .append(arena.softline())
                        .append(using);
                    join.append(pred)
                }
            },
        }
        .group()
    }
}

impl PrettyDoc for Let {
    fn pretty_doc<'b, D, A>(&'b self, _arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        todo!("LET")
    }
}

impl PrettyDoc for WhereClause {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_prefixed_expr("WHERE", &self.expr, PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for GroupByExpr {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let GroupByExpr {
            strategy,
            keys,
            group_as_alias,
        } = self;

        let mut doc = match strategy {
            None => arena.text("GROUP"),
            Some(GroupingStrategy::GroupFull) => arena.text("GROUP ALL"),
            Some(GroupingStrategy::GroupPartial) => arena.text("GROUP PARTIAL"),
        };

        if !keys.is_empty() {
            doc = doc.append(arena.space()).append(arena.text("BY")).group();
            doc = doc.append(arena.softline()).append(pretty_list(
                keys,
                PRETTY_INDENT_MINOR_NEST,
                arena,
            ));
        }

        match group_as_alias {
            None => doc,
            Some(gas) => {
                let gas = pretty_source_as_alias("GROUP", Some(gas), arena);
                doc.append(gas)
            }
        }
        .group()
    }
}

impl PrettyDoc for GroupKey {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_source_as_alias(&self.expr, self.as_alias.as_ref(), arena)
            .unwrap_or_else(|| self.expr.pretty_doc(arena))
    }
}

impl PrettyDoc for HavingClause {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        pretty_prefixed_expr("HAVING", &self.expr, PRETTY_INDENT_MINOR_NEST, arena)
    }
}

impl PrettyDoc for OrderByExpr {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        if self.sort_specs.is_empty() {
            arena.text("ORDER BY PRESERVE")
        } else {
            pretty_prefixed_doc(
                "ORDER BY",
                pretty_list(&self.sort_specs, PRETTY_INDENT_MINOR_NEST, arena),
                arena,
            )
        }
        .group()
    }
}

impl PrettyDoc for SortSpec {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let SortSpec {
            expr,
            ordering_spec,
            null_ordering_spec,
        } = self;
        let mut doc = expr.pretty_doc(arena);
        if let Some(os) = ordering_spec {
            let os = arena.space().append(os.pretty_doc(arena)).group();
            doc = doc.append(os)
        };
        if let Some(nos) = null_ordering_spec {
            let nos = arena.space().append(nos.pretty_doc(arena)).group();
            doc = doc.append(nos)
        };

        doc.group()
    }
}

impl PrettyDoc for OrderingSpec {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        arena.text(match self {
            OrderingSpec::Asc => "ASC",
            OrderingSpec::Desc => "DESC",
        })
    }
}

impl PrettyDoc for NullOrderingSpec {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        arena.text(match self {
            NullOrderingSpec::First => "NULLS FIRST",
            NullOrderingSpec::Last => "NULLS LAST",
        })
    }
}

impl PrettyDoc for LimitOffsetClause {
    fn pretty_doc<'b, D, A>(&'b self, arena: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let limit = self
            .limit
            .as_ref()
            .map(|l| pretty_prefixed_expr("LIMIT", l, PRETTY_INDENT_MINOR_NEST, arena));

        let offset = self
            .offset
            .as_ref()
            .map(|o| pretty_prefixed_expr("OFFSET", o, PRETTY_INDENT_MINOR_NEST, arena));

        match (limit, offset) {
            (None, None) => unreachable!(),
            (Some(limit), None) => limit,
            (None, Some(offset)) => offset,
            (Some(limit), Some(offset)) => limit.append(arena.softline()).append(offset),
        }
    }
}

fn case_branches<'b, D, A>(
    arena: &'b D,
    cases: &'b [ExprPair],
    default: &'b Option<Box<Expr>>,
) -> impl Iterator<Item = DocBuilder<'b, D, A>>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone + 'b,
{
    cases
        .iter()
        .map(|ExprPair { first, second }| {
            let kw_when = arena.text("WHEN");
            let test = first.pretty_doc(arena);
            let kw_then = arena.text("THEN");
            let then = second.pretty_doc(arena);
            arena
                .intersperse([kw_when, test, kw_then, then], arena.space())
                .group()
        })
        .chain(
            default
                .iter()
                .map(|d| arena.text("ELSE ").append(d.pretty_doc(arena)).group()),
        )
}

fn pretty_prefixed_expr<'b, P, D, A>(
    annot: &'static str,
    expr: &'b P,
    nest: isize,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    P: PrettyDoc,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_prefixed_doc(annot, expr.pretty_doc(arena).nest(nest), arena)
}

fn pretty_parenthesized_expr<'b, P, D, A>(
    expr: &'b P,
    nest: isize,
    arena: &'b D,
) -> DocBuilder<'b, D, A>
where
    P: PrettyDoc,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_parenthesized_doc(expr.pretty_doc(arena).nest(nest), arena)
}

fn pretty_alias_helper<'b, D, A>(
    kw: &'static str,
    sym: Option<&'b SymbolPrimitive>,
    arena: &'b D,
) -> Option<DocBuilder<'b, D, A>>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    sym.map(|sym| {
        arena
            .space()
            .append(arena.text(kw))
            .append(arena.space())
            .append(sym.pretty_doc(arena))
            .group()
    })
}

fn pretty_source_as_alias<'b, S, D, A>(
    source: &'b S,
    as_alias: Option<&'b SymbolPrimitive>,
    arena: &'b D,
) -> Option<DocBuilder<'b, D, A>>
where
    S: PrettyDoc + ?Sized,
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_as_alias(as_alias, arena).map(|alias| {
        let expr = source.pretty_doc(arena);
        arena.concat([expr, alias]).group()
    })
}

fn pretty_as_alias<'b, D, A>(
    sym: Option<&'b SymbolPrimitive>,
    arena: &'b D,
) -> Option<DocBuilder<'b, D, A>>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_alias_helper("AS", sym, arena)
}

fn pretty_at_alias<'b, D, A>(
    sym: Option<&'b SymbolPrimitive>,
    arena: &'b D,
) -> Option<DocBuilder<'b, D, A>>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_alias_helper("AT", sym, arena)
}

fn pretty_by_alias<'b, D, A>(
    sym: Option<&'b SymbolPrimitive>,
    arena: &'b D,
) -> Option<DocBuilder<'b, D, A>>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    pretty_alias_helper("BY", sym, arena)
}
