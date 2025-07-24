#![deny(clippy::all)]
#![warn(clippy::pedantic)]

#[cfg(test)]
mod tests {
    use partiql_catalog::catalog::{PartiqlCatalog, SharedCatalog};
    use partiql_catalog::context::SystemContext;
    use partiql_eval::env::basic::MapBindings;
    use partiql_eval::error::EvalErr;
    use partiql_eval::eval::BasicContext;
    use partiql_eval::plan::EvaluationMode;
    use partiql_logical::{LogicalPlan, ProjectValue, VarRefType};
    use partiql_value::{bag, tuple, Bag, BindingsName, DateTime, Value};
    use std::any::Any;

    #[track_caller]
    #[inline]
    pub(crate) fn evaluate(
        catalog: &dyn SharedCatalog,
        logical: &LogicalPlan<partiql_logical::BindingsOp>,
        bindings: MapBindings<Value>,
        ctx_vals: &[(String, &(dyn Any))],
    ) -> Result<Value, EvalErr> {
        let mut planner =
            partiql_eval::plan::EvaluatorPlanner::new(EvaluationMode::Strict, catalog);

        let plan = planner.compile(logical).expect("Expect no plan error");

        let sys = SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let mut ctx = BasicContext::new(bindings, sys);
        for (k, v) in ctx_vals {
            ctx.user.insert(k.as_str().into(), *v);
        }
        plan.execute(&ctx).map(|out| out.result)
    }

    #[test]
    fn locals_in_subqueries() {
        //  `SELECT VALUE _1 from (SELECT VALUE foo from <<{'a': 'b'}>> AS foo) AS _1;`
        let mut sub_query = LogicalPlan::new();

        let data = Box::new(partiql_logical::Lit::Bag(vec![
            partiql_logical::Lit::Struct(vec![(
                "a".to_string(),
                partiql_logical::Lit::String("b".to_string()),
            )]),
        ]));
        let scan_op_id =
            sub_query.add_operator(partiql_logical::BindingsOp::Scan(partiql_logical::Scan {
                expr: partiql_logical::ValueExpr::Lit(data),
                as_key: "foo".into(),
                at_key: None,
            }));
        let project_value_op_id = sub_query.add_operator(
            partiql_logical::BindingsOp::ProjectValue(partiql_logical::ProjectValue {
                expr: partiql_logical::ValueExpr::VarRef(
                    BindingsName::CaseSensitive("foo".into()),
                    VarRefType::Local,
                ),
            }),
        );
        sub_query.add_flow(scan_op_id, project_value_op_id);

        let sink_op_id = sub_query.add_operator(partiql_logical::BindingsOp::Sink);
        sub_query.add_flow(project_value_op_id, sink_op_id);

        let mut plan = LogicalPlan::new();
        let scan_op_id =
            plan.add_operator(partiql_logical::BindingsOp::Scan(partiql_logical::Scan {
                expr: partiql_logical::ValueExpr::SubQueryExpr(partiql_logical::SubQueryExpr {
                    plan: sub_query,
                }),
                as_key: "_1".into(),
                at_key: None,
            }));

        let project_value_op_id =
            plan.add_operator(partiql_logical::BindingsOp::ProjectValue(ProjectValue {
                expr: partiql_logical::ValueExpr::VarRef(
                    BindingsName::CaseSensitive("_1".into()),
                    VarRefType::Local,
                ),
            }));
        plan.add_flow(scan_op_id, project_value_op_id);

        let sink_op_id = plan.add_operator(partiql_logical::BindingsOp::Sink);
        plan.add_flow(project_value_op_id, sink_op_id);

        let catalog = PartiqlCatalog::default().to_shared_catalog();
        let bindings = MapBindings::default();
        let res = evaluate(&catalog, &plan, bindings, &[]).expect("should eval correctly");
        dbg!(&res);
        assert!(res != Value::Missing);
        assert_eq!(res, Value::from(bag![tuple![("a", "b")]]));
    }

    #[test]
    fn globals_in_subqueries() {
        //  `foo` is defined in global environment as `<<{'a': 'b'}>>`
        //  `SELECT VALUE _1 FROM (SELECT VALUE foo FROM foo AS foo) AS _1;`
        let mut sub_query = partiql_logical::LogicalPlan::new();
        let scan_op_id =
            sub_query.add_operator(partiql_logical::BindingsOp::Scan(partiql_logical::Scan {
                expr: partiql_logical::ValueExpr::VarRef(
                    BindingsName::CaseSensitive("foo".into()),
                    VarRefType::Global,
                ),
                as_key: "foo".into(),
                at_key: None,
            }));
        let project_value_op_id = sub_query.add_operator(
            partiql_logical::BindingsOp::ProjectValue(partiql_logical::ProjectValue {
                expr: partiql_logical::ValueExpr::VarRef(
                    BindingsName::CaseSensitive("foo".into()),
                    VarRefType::Local,
                ),
            }),
        );
        sub_query.add_flow(scan_op_id, project_value_op_id);

        let sink_op_id = sub_query.add_operator(partiql_logical::BindingsOp::Sink);
        sub_query.add_flow(project_value_op_id, sink_op_id);

        let mut plan = LogicalPlan::new();
        let scan_op_id =
            plan.add_operator(partiql_logical::BindingsOp::Scan(partiql_logical::Scan {
                expr: partiql_logical::ValueExpr::SubQueryExpr(partiql_logical::SubQueryExpr {
                    plan: sub_query,
                }),
                as_key: "_1".into(),
                at_key: None,
            }));

        let project_value_op_id =
            plan.add_operator(partiql_logical::BindingsOp::ProjectValue(ProjectValue {
                expr: partiql_logical::ValueExpr::VarRef(
                    BindingsName::CaseSensitive("_1".into()),
                    VarRefType::Local,
                ),
            }));
        plan.add_flow(scan_op_id, project_value_op_id);

        let sink_op_id = plan.add_operator(partiql_logical::BindingsOp::Sink);
        plan.add_flow(project_value_op_id, sink_op_id);

        let catalog = PartiqlCatalog::default().to_shared_catalog();
        let mut bindings = MapBindings::default();
        bindings.insert(
            "foo",
            Value::Bag(Box::new(Bag::from(vec![tuple![("a", "b")].into()]))),
        );
        let res = evaluate(&catalog, &plan, bindings, &[]).expect("should eval correctly");
        dbg!(&res);
        assert!(res != Value::Missing);
        assert_eq!(res, Value::from(bag![tuple![("a", "b")]]));
    }
}
