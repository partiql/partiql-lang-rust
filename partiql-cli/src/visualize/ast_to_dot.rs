use partiql_ast::ast;

use dot_writer::{Attributes, DotWriter, Node, NodeId, Scope, Shape};

/*
subgraph cluster_legend {
    rank = same;
    variable[shape=Mdiamond]
    literal[shape=rect]
    "node"[shape=ellipse]
}
 */

trait ScopeExt<'d, 'w> {
    fn node_auto_labelled(&mut self, lbl: &str) -> Node<'_, 'w>;
    fn cluster_auto_labelled(&mut self, lbl: &str) -> Scope<'_, 'w>;
    fn with_cluster<F, R>(&mut self, lbl: &str, func: F) -> R
    where
        F: FnMut(Scope<'_, 'w>) -> R;
}

impl<'d, 'w> ScopeExt<'d, 'w> for Scope<'d, 'w> {
    #[inline]
    fn node_auto_labelled(&mut self, lbl: &str) -> Node<'_, 'w> {
        let mut node = self.node_auto();
        node.set_label(lbl);
        node
    }

    fn cluster_auto_labelled(&mut self, lbl: &str) -> Scope<'_, 'w> {
        let mut cluster = self.cluster();
        cluster.set("label", lbl, lbl.contains(" "));
        cluster
    }

    fn with_cluster<F, R>(&mut self, lbl: &str, mut func: F) -> R
    where
        F: FnMut(Scope<'_, 'w>) -> R,
    {
        let cluster = self.cluster_auto_labelled(lbl);
        func(cluster)
    }
}

trait ChildEdgeExt {
    fn edges(self, out: &mut Scope, from: &NodeId, lbl: &str) -> Targets;
}

impl ChildEdgeExt for Targets {
    fn edges(self, out: &mut Scope, from: &NodeId, lbl: &str) -> Targets {
        for target in &self {
            out.edge(&from, &target).attributes().set_label(lbl);
        }
        self
    }
}

type Targets = Vec<NodeId>;

pub trait ToDotGraph<T> {
    fn to_graph(self, ast: &T) -> String;
}

pub struct AstToDot {}

impl Default for AstToDot {
    fn default() -> Self {
        AstToDot {}
    }
}

const BG_COLOR: &'static str = "\"#002b3600\"";
const FG_COLOR: &'static str = "\"#839496\"";

impl<T> ToDotGraph<T> for AstToDot
where
    AstToDot: ToDot<T>,
{
    fn to_graph(mut self, ast: &T) -> String {
        let mut output_bytes = Vec::new();

        {
            let mut writer = DotWriter::from(&mut output_bytes);
            writer.set_pretty_print(true);
            let mut digraph = writer.digraph();
            digraph
                .graph_attributes()
                .set_rank_direction(dot_writer::RankDirection::TopBottom)
                .set("rankdir", "0.05", false)
                .set("bgcolor", BG_COLOR, false)
                .set("fontcolor", FG_COLOR, false)
                .set("pencolor", FG_COLOR, false);
            digraph.node_attributes().set("color", FG_COLOR, false).set(
                "fontcolor",
                FG_COLOR,
                false,
            );
            digraph.edge_attributes().set("color", FG_COLOR, false).set(
                "fontcolor",
                FG_COLOR,
                false,
            );

            self.to_dot(&mut digraph, ast);
        }

        return String::from_utf8(output_bytes).expect("invalid utf8");
    }
}

trait ToDot<T> {
    fn to_dot(&mut self, out: &mut Scope, ast: &T) -> Targets;
}

impl<T> ToDot<Box<T>> for AstToDot
where
    AstToDot: ToDot<T>,
{
    fn to_dot(&mut self, out: &mut Scope, ast: &Box<T>) -> Targets {
        self.to_dot(out, &**ast)
    }
}

impl<T> ToDot<Vec<T>> for AstToDot
where
    AstToDot: ToDot<T>,
{
    fn to_dot(&mut self, out: &mut Scope, asts: &Vec<T>) -> Targets {
        let mut res = Vec::with_capacity(asts.len());
        for ast in asts {
            res.extend(self.to_dot(out, &ast));
        }
        res
    }
}

impl<T> ToDot<Option<T>> for AstToDot
where
    AstToDot: ToDot<T>,
{
    fn to_dot(&mut self, out: &mut Scope, ast: &Option<T>) -> Targets {
        match ast {
            None => vec![],
            Some(ast) => self.to_dot(out, &ast),
        }
    }
}

impl<T> ToDot<ast::AstNode<T>> for AstToDot
where
    AstToDot: ToDot<T>,
{
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::AstNode<T>) -> Targets {
        self.to_dot(out, &ast.node)
    }
}

impl ToDot<ast::Expr> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Expr) -> Targets {
        let mut expr_subgraph = out.subgraph();

        use ast::Expr;
        match &ast {
            Expr::Lit(l) => self.to_dot(&mut expr_subgraph, l),
            Expr::VarRef(v) => self.to_dot(&mut expr_subgraph, v),
            Expr::BinOp(bop) => self.to_dot(&mut expr_subgraph, bop),
            Expr::UniOp(unop) => self.to_dot(&mut expr_subgraph, unop),
            Expr::Like(like) => self.to_dot(&mut expr_subgraph, like),
            Expr::Between(btwn) => self.to_dot(&mut expr_subgraph, btwn),
            Expr::In(in_expr) => self.to_dot(&mut expr_subgraph, in_expr),
            Expr::Case(_) => todo!(),
            Expr::Struct(_) => todo!(),
            Expr::Bag(_) => todo!(),
            Expr::List(_) => todo!(),
            Expr::Sexp(_) => todo!(),
            Expr::Path(p) => self.to_dot(&mut expr_subgraph, p),
            Expr::Call(c) => self.to_dot(&mut expr_subgraph, c),
            Expr::CallAgg(c) => self.to_dot(&mut expr_subgraph, c),
            Expr::Query(q) => self.to_dot(&mut expr_subgraph, q),
            Expr::Error => todo!(),
        }
    }
}

#[inline]
fn lit_to_str(ast: &ast::Lit) -> String {
    use ast::Lit;
    match ast {
        Lit::Null => "NULL".to_string(),
        Lit::Missing => "MISSING".to_string(),
        Lit::Int8Lit(l) => l.to_string(),
        Lit::Int16Lit(l) => l.to_string(),
        Lit::Int32Lit(l) => l.to_string(),
        Lit::Int64Lit(l) => l.to_string(),
        Lit::DecimalLit(l) => l.to_string(),
        Lit::NumericLit(l) => l.to_string(),
        Lit::RealLit(l) => l.to_string(),
        Lit::FloatLit(l) => l.to_string(),
        Lit::DoubleLit(l) => l.to_string(),
        Lit::BoolLit(l) => (if *l { "TRUE" } else { "FALSE" }).to_string(),
        Lit::IonStringLit(l) => format!("`{}`", l),
        Lit::CharStringLit(l) => format!("'{}'", l),
        Lit::NationalCharStringLit(l) => format!("'{}'", l),
        Lit::BitStringLit(l) => format!("b'{}'", l),
        Lit::HexStringLit(l) => format!("x'{}'", l),
        Lit::CollectionLit(l) => match l {
            ast::CollectionLit::ArrayLit(al) => format!("[{}]", al),
            ast::CollectionLit::BagLit(bl) => format!("<<{}>>", bl),
        },
        Lit::TypedLit(val_str, ty) => {
            format!("{} '{}'", type_to_str(ty), val_str)
        }
    }
}

#[inline]
fn custom_type_param_to_str(param: &ast::CustomTypeParam) -> String {
    use ast::CustomTypeParam;
    match param {
        CustomTypeParam::Lit(lit) => lit_to_str(lit),
        CustomTypeParam::Type(ty) => type_to_str(ty),
    }
}

#[inline]
fn custom_type_part_to_str(part: &ast::CustomTypePart) -> String {
    use ast::CustomTypePart;
    match part {
        CustomTypePart::Name(name) => symbol_primitive_to_label(name),
        CustomTypePart::Parameterized(name, args) => {
            let name = symbol_primitive_to_label(name);
            let args = args
                .iter()
                .map(custom_type_param_to_str)
                .collect::<Vec<_>>()
                .join(",");
            format!("{}({})", name, args)
        }
    }
}

#[inline]
fn type_to_str(ty: &ast::Type) -> String {
    use ast::Type;
    match ty {
        Type::CustomType(cty) => cty
            .parts
            .iter()
            .map(custom_type_part_to_str)
            .collect::<Vec<_>>()
            .join(" "),
        _ => format!("{:?}", ty),
    }
}

impl ToDot<ast::Lit> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Lit) -> Targets {
        let lbl = lit_to_str(ast);

        let mut node = out.node_auto();
        node.set_label(&lbl).set_shape(Shape::Rectangle);

        vec![node.id()]
    }
}

impl ToDot<ast::BinOp> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::BinOp) -> Targets {
        use ast::BinOpKind;
        let lbl = match ast.kind {
            BinOpKind::Add => "+",
            BinOpKind::Div => "/",
            BinOpKind::Exp => "^",
            BinOpKind::Mod => "%",
            BinOpKind::Mul => "*",
            BinOpKind::Neg => "-",
            BinOpKind::And => "AND",
            BinOpKind::Or => "OR",
            BinOpKind::Concat => "||",
            BinOpKind::Eq => "=",
            BinOpKind::Gt => ">",
            BinOpKind::Gte => ">=",
            BinOpKind::Lt => "<",
            BinOpKind::Lte => "<=",
            BinOpKind::Ne => "<>",
            BinOpKind::Is => "IS",
        };
        let id = out.node_auto_labelled(lbl).id();

        self.to_dot(out, &ast.lhs).edges(out, &id, "");
        self.to_dot(out, &ast.rhs).edges(out, &id, "");

        vec![id]
    }
}

impl ToDot<ast::UniOp> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::UniOp) -> Targets {
        use ast::UniOpKind;
        let lbl = match ast.kind {
            UniOpKind::Pos => "+",
            UniOpKind::Neg => "-",
            UniOpKind::Not => "NOT",
        };
        let id = out.node_auto_labelled(lbl).id();

        self.to_dot(out, &ast.expr).edges(out, &id, "");

        vec![id]
    }
}

impl ToDot<ast::Like> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Like) -> Targets {
        let id = out.node_auto_labelled("LIKE").id();

        self.to_dot(out, &ast.value).edges(out, &id, "value");
        self.to_dot(out, &ast.pattern).edges(out, &id, "pattern");
        self.to_dot(out, &ast.escape).edges(out, &id, "escape");

        vec![id]
    }
}

impl ToDot<ast::Between> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Between) -> Targets {
        let id = out.node_auto_labelled("BETWEEN").id();

        self.to_dot(out, &ast.value).edges(out, &id, "value");
        self.to_dot(out, &ast.from).edges(out, &id, "from");
        self.to_dot(out, &ast.to).edges(out, &id, "to");

        vec![id]
    }
}

impl ToDot<ast::In> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::In) -> Targets {
        let id = out.node_auto_labelled("IN").id();

        self.to_dot(out, &ast.lhs).edges(out, &id, "");
        self.to_dot(out, &ast.rhs).edges(out, &id, "");

        vec![id]
    }
}

impl ToDot<ast::Query> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Query) -> Targets {
        let id = out.node_auto_labelled("Query").id();

        self.to_dot(out, &ast.set).edges(out, &id, "");
        self.to_dot(out, &ast.order_by).edges(out, &id, "order_by");
        self.to_dot(out, &ast.limit).edges(out, &id, "limit");
        self.to_dot(out, &ast.offset).edges(out, &id, "offset");

        vec![id]
    }
}
impl ToDot<ast::QuerySet> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::QuerySet) -> Targets {
        use ast::QuerySet;
        match &ast {
            QuerySet::SetOp(_) => todo!(),
            QuerySet::Select(select) => self.to_dot(out, select),
            QuerySet::Expr(e) => self.to_dot(out, e),
            QuerySet::Values(_) => todo!(),
        }
    }
}

impl ToDot<ast::Select> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Select) -> Targets {
        let id = out.node_auto_labelled("Select").id();

        out.with_cluster("PROJECT", |mut cl| self.to_dot(&mut cl, &ast.project))
            .edges(out, &id, "");
        out.with_cluster("FROM", |mut cl| self.to_dot(&mut cl, &ast.from))
            .edges(out, &id, "");
        out.with_cluster("FROM LET", |mut cl| self.to_dot(&mut cl, &ast.from_let))
            .edges(out, &id, "");
        out.with_cluster("WHERE", |mut cl| self.to_dot(&mut cl, &ast.where_clause))
            .edges(out, &id, "");
        out.with_cluster("GROUP BY", |mut cl| self.to_dot(&mut cl, &ast.group_by))
            .edges(out, &id, "");
        out.with_cluster("HAVING", |mut cl| self.to_dot(&mut cl, &ast.having))
            .edges(out, &id, "");

        vec![id]
    }
}

impl ToDot<ast::Projection> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Projection) -> Targets {
        let lbl = match &ast.setq {
            Some(ast::SetQuantifier::Distinct) => "Projection | Distinct",
            _ => "Projection | All",
        };
        let id = out.node_auto_labelled(lbl).id();

        use ast::ProjectionKind;
        let children = {
            let mut expr_subgraph = out.subgraph();

            match &ast.kind {
                ProjectionKind::ProjectStar => vec![expr_subgraph.node_auto_labelled("*").id()],
                ProjectionKind::ProjectList(items) => {
                    let mut list = vec![];
                    for item in items {
                        list.extend(self.to_dot(&mut expr_subgraph, item));
                    }
                    list
                }
                ProjectionKind::ProjectPivot { .. } => todo!(),
                ProjectionKind::ProjectValue(_) => todo!(),
            }
        };

        children.edges(out, &id, "");

        vec![id]
    }
}

impl ToDot<ast::ProjectItem> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::ProjectItem) -> Targets {
        match ast {
            ast::ProjectItem::ProjectAll(all) => {
                let id = out.node_auto_labelled("ProjectAll").id();
                self.to_dot(out, &all.expr).edges(out, &id, "");
                vec![id]
            }
            ast::ProjectItem::ProjectExpr(expr) => {
                let id = out.node_auto_labelled("ProjectExpr").id();
                self.to_dot(out, &expr.expr).edges(out, &id, "");
                self.to_dot(out, &expr.as_alias).edges(out, &id, "as");
                vec![id]
            }
        }
    }
}

fn symbol_primitive_to_label(sym: &ast::SymbolPrimitive) -> String {
    use ast::CaseSensitivity;
    match &sym.case {
        CaseSensitivity::CaseSensitive => format!("'{}'", sym.value),
        CaseSensitivity::CaseInsensitive => format!("{}", sym.value),
    }
}

impl ToDot<ast::SymbolPrimitive> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::SymbolPrimitive) -> Targets {
        let lbl = symbol_primitive_to_label(ast);
        let id = out.node_auto_labelled(&lbl).id();
        vec![id]
    }
}

impl ToDot<ast::VarRef> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::VarRef) -> Targets {
        let lbl = symbol_primitive_to_label(&ast.name);
        let lbl = match &ast.qualifier {
            ast::ScopeQualifier::Unqualified => lbl,
            ast::ScopeQualifier::Qualified => format!("@{}", lbl),
        };
        let id = out.node_auto_labelled(&lbl).id();

        vec![id]
    }
}

impl ToDot<ast::OrderByExpr> for AstToDot {
    fn to_dot(&mut self, _out: &mut Scope, _ast: &ast::OrderByExpr) -> Targets {
        todo!("OrderByExpr");
    }
}

impl ToDot<ast::GroupByExpr> for AstToDot {
    fn to_dot(&mut self, _out: &mut Scope, _ast: &ast::GroupByExpr) -> Targets {
        todo!("GroupByExpr");
    }
}

impl ToDot<ast::FromClause> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::FromClause) -> Targets {
        match &ast {
            ast::FromClause::FromLet(fl) => self.to_dot(out, fl),
            ast::FromClause::Join(j) => self.to_dot(out, j),
        }
    }
}

impl ToDot<ast::FromLet> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::FromLet) -> Targets {
        let lbl = match &ast.kind {
            ast::FromLetKind::Scan => "Scan",
            ast::FromLetKind::Unpivot => "Unpivot",
        };
        let id = out.node_auto_labelled(lbl).id();

        self.to_dot(out, &ast.expr).edges(out, &id, "");
        self.to_dot(out, &ast.as_alias).edges(out, &id, "as");
        self.to_dot(out, &ast.at_alias).edges(out, &id, "at");
        self.to_dot(out, &ast.by_alias).edges(out, &id, "by");

        vec![id]
    }
}

impl ToDot<ast::Join> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Join) -> Targets {
        let lbl = match &ast.kind {
            ast::JoinKind::Inner => "Inner Join",
            ast::JoinKind::Left => "Left Join",
            ast::JoinKind::Right => "Right Join",
            ast::JoinKind::Full => "Full Join",
            ast::JoinKind::Cross => "Cross Join",
        };
        let id = out.node_auto_labelled(lbl).id();

        self.to_dot(out, &ast.left).edges(out, &id, "left");
        self.to_dot(out, &ast.right).edges(out, &id, "right");
        self.to_dot(out, &ast.predicate)
            .edges(out, &id, "predicate");

        vec![id]
    }
}

impl ToDot<ast::JoinSpec> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::JoinSpec) -> Targets {
        match &ast {
            ast::JoinSpec::On(fl) => {
                let id = out.node_auto_labelled("On").id();
                self.to_dot(out, fl).edges(out, &id, "");
                vec![id]
            }
            ast::JoinSpec::Using(j) => {
                let id = out.node_auto_labelled("Using").id();
                self.to_dot(out, j).edges(out, &id, "");
                vec![id]
            }
            ast::JoinSpec::Natural => vec![out.node_auto_labelled("Natural").id()],
        }
    }
}

impl ToDot<ast::Call> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Call) -> Targets {
        let id = out.node_auto_labelled("Call").id();

        self.to_dot(out, &ast.func_name).edges(out, &id, "name");
        self.to_dot(out, &ast.args).edges(out, &id, "args");

        vec![id]
    }
}

impl ToDot<ast::CallArg> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::CallArg) -> Targets {
        use ast::CallArg;
        match ast {
            ast::CallArg::Star() => vec![out.node_auto_labelled("*").id()],
            ast::CallArg::Positional(e) => self.to_dot(out, e),
            ast::CallArg::Named { name, value } => {
                let id = out.node_auto_labelled("Named").id();
                self.to_dot(out, name).edges(out, &id, "name");
                self.to_dot(out, value).edges(out, &id, "value");
                vec![id]
            }
            CallArg::PositionalType(ty) => {
                let mut node = out.node_auto_labelled(&type_to_str(ty));
                node.set("shape", "parallelogram", false);
                vec![node.id()]
            }
            CallArg::NamedType { name, ty } => {
                let id = out.node_auto_labelled("Named").id();
                self.to_dot(out, name).edges(out, &id, "name");

                let ty_target = {
                    let mut ty_node = out.node_auto_labelled(&type_to_str(ty));
                    ty_node.set("shape", "parallelogram", false);
                    vec![ty_node.id()]
                };
                ty_target.edges(out, &id, "type");

                vec![id]
            }
        }
    }
}

impl ToDot<ast::CallAgg> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::CallAgg) -> Targets {
        let lbl = match &ast.setq {
            Some(ast::SetQuantifier::Distinct) => "CallAgg | Distinct",
            _ => "CallAgg | All",
        };
        let id = out.node_auto_labelled(lbl).id();

        self.to_dot(out, &ast.func_name).edges(out, &id, "name");
        self.to_dot(out, &ast.args).edges(out, &id, "args");

        vec![id]
    }
}

impl ToDot<ast::Path> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::Path) -> Targets {
        let id = out.node_auto_labelled("Path").id();

        self.to_dot(out, &ast.root).edges(out, &id, "root");
        self.to_dot(out, &ast.steps).edges(out, &id, "steps");

        vec![id]
    }
}

impl ToDot<ast::PathStep> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::PathStep) -> Targets {
        match &ast {
            ast::PathStep::PathExpr(e) => self.to_dot(out, e),
            ast::PathStep::PathWildCard => vec![out.node_auto_labelled("*").id()],
            ast::PathStep::PathUnpivot => vec![out.node_auto_labelled("Unpivot").id()],
        }
    }
}

impl ToDot<ast::PathExpr> for AstToDot {
    fn to_dot(&mut self, out: &mut Scope, ast: &ast::PathExpr) -> Targets {
        let id = out.node_auto_labelled("PathExpr").id();

        self.to_dot(out, &ast.index).edges(out, &id, "index");

        vec![id]
    }
}

impl ToDot<ast::Let> for AstToDot {
    fn to_dot(&mut self, _out: &mut Scope, _ast: &ast::Let) -> Targets {
        todo!("Let");
    }
}
