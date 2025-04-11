use dot_writer::{Attributes, DotWriter, NodeId, Scope, Shape};
use itertools::Itertools;
use partiql_logical::{
    AggregateExpression, BinaryOp, BindingsOp, JoinKind, LogicalPlan, ValueExpr,
};

use std::collections::HashMap;

use crate::common::{ToDotGraph, FG_COLOR};

#[derive(Default)]
pub struct PlanToDot {}

impl PlanToDot {
    pub(crate) fn to_dot(&self, scope: &mut Scope<'_, '_>, plan: &LogicalPlan<BindingsOp>) {
        let mut graph_nodes = HashMap::new();
        for (opid, op) in plan.operators_by_id() {
            graph_nodes.insert(opid, self.op_to_dot(scope, op));
        }

        for (src, dst, branch) in plan.flows() {
            let src = graph_nodes.get(src).expect("src op");
            let dst = graph_nodes.get(dst).expect("dst op");

            scope
                .edge(src, dst)
                .attributes()
                .set_label(&branch.to_string());
        }
    }

    fn op_to_dot(&self, scope: &mut Scope<'_, '_>, op: &BindingsOp) -> NodeId {
        let mut node = scope.node_auto();
        let label = match op {
            BindingsOp::Scan(s) => {
                format!("{{scan | {} | as {} }}", expr_to_str(&s.expr), s.as_key)
            }
            BindingsOp::Pivot(p) => format!(
                "{{pivot | {} | at {} }}",
                expr_to_str(&p.value),
                expr_to_str(&p.key)
            ),
            BindingsOp::Unpivot(u) => format!(
                "{{unpivot | {}  | as {} | at {} }}",
                expr_to_str(&u.expr),
                &u.as_key,
                &u.at_key.as_deref().unwrap_or("")
            ),
            BindingsOp::Filter(f) => format!("{{filter | {} }}", expr_to_str(&f.expr)),
            BindingsOp::OrderBy(o) => {
                let specs = o
                    .specs
                    .iter()
                    .map(|s| {
                        format!(
                            "{} {:?} NULLS {:?}",
                            expr_to_str(&s.expr),
                            s.order,
                            s.null_order
                        )
                    })
                    .join(" | ");
                format!("{{order by | {} }}", specs)
            }
            BindingsOp::LimitOffset(lo) => {
                let clauses = [
                    lo.limit
                        .as_ref()
                        .map(|e| format!("limit {}", expr_to_str(e))),
                    lo.offset
                        .as_ref()
                        .map(|e| format!("offset {}", expr_to_str(e))),
                ]
                .iter()
                .filter_map(|o| o.as_ref())
                .join(" | ");
                format!("{{ {clauses} }}")
            }
            BindingsOp::Join(join) => {
                let kind = match join.kind {
                    JoinKind::Inner => "inner",
                    JoinKind::Left => "left",
                    JoinKind::Right => "right",
                    JoinKind::Full => "full",
                    JoinKind::Cross => "cross",
                };
                format!(
                    "{{ {} join | {} }}",
                    kind,
                    join.on.as_ref().map(expr_to_str).unwrap_or("".to_string())
                )
            }
            BindingsOp::BagOp(_) => "bag op (TODO)".to_string(),
            BindingsOp::Project(p) => {
                format!(
                    "{{project  | {} }}",
                    p.exprs
                        .iter()
                        .map(|(k, e)| format!("{}:{}", k, expr_to_str(e)))
                        .join(" | "),
                )
            }
            BindingsOp::ProjectAll(_) => "{project  * }".to_string(),
            BindingsOp::ProjectValue(pv) => {
                format!("{{project value | {} }}", expr_to_str(&pv.expr))
            }
            BindingsOp::ExprQuery(eq) => {
                format!("{{ {} }}", expr_to_str(&eq.expr))
            }
            BindingsOp::Distinct => "distinct".to_string(),
            BindingsOp::GroupBy(g) => {
                format!(
                    "{{group by | {:?} | {{ keys | {{ {} }} }} | {{ aggs | {{ {} }} }} | as {} }}",
                    g.strategy,
                    g.exprs
                        .iter()
                        .map(|(k, e)| format!("{}:{}", k, expr_to_str(e)))
                        .join(" | "),
                    g.aggregate_exprs.iter().map(agg_expr_to_str).join(" | "),
                    g.group_as_alias.as_deref().unwrap_or(""),
                )
            }
            BindingsOp::Having(h) => {
                format!("{{ having | {} }}", expr_to_str(&h.expr))
            }
            BindingsOp::Sink => "sink".to_string(),
        };
        node.set_shape(Shape::Mrecord).set_label(&label.to_string());

        node.id()
    }
}

fn expr_to_str(expr: &ValueExpr) -> String {
    match expr {
        ValueExpr::BinaryExpr(BinaryOp::And, lhs, rhs) => {
            format!(
                "{{  AND | {{ {} | {} }}  }}",
                expr_to_str(lhs),
                expr_to_str(rhs)
            )
        }
        ValueExpr::BinaryExpr(BinaryOp::Or, lhs, rhs) => {
            format!(
                "{{  OR | {{ {} | {} }}  }}",
                expr_to_str(lhs),
                expr_to_str(rhs)
            )
        }
        expr => {
            let expr: String = format!("{:?}", expr).escape_default().collect();
            let expr = expr.replace('{', "\\{");
            let expr = expr.replace('}', "\\}");
            let expr = expr.replace('<', "\\<");

            expr.replace('>', "\\>")
        }
    }
}

fn agg_expr_to_str(agg_expr: &AggregateExpression) -> String {
    let expr: String = format!("{:?}", agg_expr.expr).escape_default().collect();
    let expr = expr.replace('{', "\\{");
    let expr = expr.replace('}', "\\}");
    format!(
        "{}:{}:{:?}:{:?}",
        agg_expr.name, expr, agg_expr.func, agg_expr.setq
    )
}

impl ToDotGraph<LogicalPlan<BindingsOp>> for PlanToDot {
    fn to_graph(self, plan: &LogicalPlan<BindingsOp>) -> String {
        let mut output_bytes = Vec::new();

        {
            let mut writer = DotWriter::from(&mut output_bytes);
            writer.set_pretty_print(true);
            let mut digraph = writer.digraph();
            digraph
                .graph_attributes()
                .set_rank_direction(dot_writer::RankDirection::TopBottom)
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

            self.to_dot(&mut digraph, plan);
        }

        String::from_utf8(output_bytes).expect("invalid utf8")
    }
}
