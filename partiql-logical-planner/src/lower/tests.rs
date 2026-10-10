use super::*;
use crate::LogicalPlanner;
use assert_matches::assert_matches;
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, TypeEnvEntry};
use partiql_logical::BindingsOp::Project;
use partiql_logical::ValueExpr;
use partiql_types::PartiqlShape;

#[test]
fn test_plan_non_existent_fns() {
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let statement = "foo(1, 2) + bar(3)";
    let parsed = partiql_parser::Parser::default()
        .parse(statement)
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let logical = planner.lower(&parsed);
    assert!(logical.is_err());
    let lowering_errs = logical.expect_err("Expect errs").errors;
    assert_eq!(lowering_errs.len(), 2);
    assert_matches!(
        lowering_errs.first(),
        Some(AstTransformError::UnsupportedFunction(fnc)) if fnc == "foo"
    );
    assert_matches!(
        lowering_errs.get(1),
        Some(AstTransformError::UnsupportedFunction(fnc)) if fnc == "bar"
    );
}

#[test]
fn test_plan_bad_num_arguments() {
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let statement = "abs(1, 2) + mod(3)";
    let parsed = partiql_parser::Parser::default()
        .parse(statement)
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let logical = planner.lower(&parsed);
    assert!(logical.is_err());
    let lowering_errs = logical.expect_err("Expect errs").errors;
    assert_eq!(lowering_errs.len(), 2);
    assert_matches!(
        lowering_errs.first(),
        Some(AstTransformError::InvalidNumberOfArguments(fnc)) if fnc == "abs"
    );
    assert_matches!(
        lowering_errs.get(1),
        Some(AstTransformError::InvalidNumberOfArguments(fnc)) if fnc == "mod"
    );
}

#[test]
fn test_plan_type_entry_in_catalog() {
    // Expected Logical Plan
    let mut expected_logical = LogicalPlan::new();
    let my_id = ValueExpr::Path(
        Box::new(ValueExpr::DynamicLookup(Box::new(vec![
            ValueExpr::VarRef(
                BindingsName::CaseInsensitive("c".to_string().into()),
                VarRefType::Local,
            ),
            ValueExpr::VarRef(
                BindingsName::CaseInsensitive("c".to_string().into()),
                VarRefType::Global,
            ),
        ]))),
        vec![PathComponent::Key(BindingsName::CaseInsensitive(
            "id".to_string().into(),
        ))],
    );

    let my_name = ValueExpr::Path(
        Box::new(ValueExpr::DynamicLookup(Box::new(vec![ValueExpr::VarRef(
            BindingsName::CaseInsensitive("customers".to_string().into()),
            VarRefType::Global,
        )]))),
        vec![PathComponent::Key(BindingsName::CaseInsensitive(
            "name".to_string().into(),
        ))],
    );

    let scan = expected_logical.add_operator(BindingsOp::Scan(logical::Scan {
        expr: ValueExpr::DynamicLookup(Box::new(vec![ValueExpr::VarRef(
            BindingsName::CaseInsensitive("customers".to_string().into()),
            VarRefType::Global,
        )])),
        as_key: "c".to_string(),
        at_key: None,
    }));

    let project = expected_logical.add_operator(Project(logical::Project {
        exprs: Vec::from([
            ("my_id".to_string(), my_id),
            ("my_name".to_string(), my_name),
        ]),
    }));
    let sink = expected_logical.add_operator(BindingsOp::Sink);
    expected_logical.add_flow_with_branch_num(scan, project, 0);
    expected_logical.add_flow_with_branch_num(project, sink, 0);

    let mut catalog = PartiqlCatalog::default();
    let _oid = catalog.add_type_entry(TypeEnvEntry::new("customers", &[], PartiqlShape::Dynamic));
    let catalog = catalog.to_shared_catalog();
    let statement = "SELECT c.id AS my_id, customers.name AS my_name FROM customers AS c";
    let parsed = partiql_parser::Parser::default()
        .parse(statement)
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let logical = planner.lower(&parsed).expect("Expect successful lowering");
    assert_eq!(expected_logical, logical);

    println!("logical: {:?}", &logical);
}

#[test]
fn test_plan_type_entry_in_catalog_static() {
    // Expected Logical Plan
    let mut expected_logical = LogicalPlan::new();

    // c.id resolves to VarRef(Local) since c is a local alias
    let my_id = ValueExpr::Path(
        Box::new(ValueExpr::VarRef(
            BindingsName::CaseInsensitive("c".to_string().into()),
            VarRefType::Local,
        )),
        vec![PathComponent::Key(BindingsName::CaseInsensitive(
            "id".to_string().into(),
        ))],
    );

    // customers.name resolves to DBRef since customers is in the catalog
    let my_name = ValueExpr::Path(
        Box::new(ValueExpr::DBRef(logical::DBRef {
            catalog: "default".to_string(),
            path: vec![BindingsName::CaseInsensitive(
                "customers".to_string().into(),
            )],
        })),
        vec![PathComponent::Key(BindingsName::CaseInsensitive(
            "name".to_string().into(),
        ))],
    );

    // Scan expr resolves to DBRef since customers is in the catalog
    let scan = expected_logical.add_operator(BindingsOp::Scan(logical::Scan {
        expr: ValueExpr::DBRef(logical::DBRef {
            catalog: "default".to_string(),
            path: vec![BindingsName::CaseInsensitive(
                "customers".to_string().into(),
            )],
        }),
        as_key: "c".to_string(),
        at_key: None,
    }));

    let project = expected_logical.add_operator(Project(logical::Project {
        exprs: Vec::from([
            ("my_id".to_string(), my_id),
            ("my_name".to_string(), my_name),
        ]),
    }));
    let sink = expected_logical.add_operator(BindingsOp::Sink);
    expected_logical.add_flow_with_branch_num(scan, project, 0);
    expected_logical.add_flow_with_branch_num(project, sink, 0);

    let mut catalog = PartiqlCatalog::default();
    let _oid = catalog.add_type_entry(TypeEnvEntry::new("customers", &[], PartiqlShape::Dynamic));
    let catalog = catalog.to_shared_catalog();
    let statement = "SELECT c.id AS my_id, customers.name AS my_name FROM customers AS c";
    let parsed = partiql_parser::Parser::default()
        .parse(statement)
        .expect("Expect successful parse");
    let planner = LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static);
    let logical = planner.lower(&parsed).expect("Expect successful lowering");
    assert_eq!(expected_logical, logical);

    println!("logical: {:?}", logical);
}

#[test]
fn test_ctas_lowers_to_create_table_as() {
    use partiql_logical::{BindingsOp, LogicalStatement};
    use partiql_value::BindingsName;

    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let statement = "CREATE TABLE t AS (SELECT a FROM foo WHERE a > 1)";
    let parsed = partiql_parser::Parser::default()
        .parse(statement)
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let (table_name, query) = assert_matches!(
        stmt,
        LogicalStatement::CreateTableAs { table_name, query } => (table_name, query)
    );

    // Target carried verbatim, case-insensitive (bare identifier).
    assert_eq!(table_name, BindingsName::CaseInsensitive("t".into()));

    // Inner plan is the ordinary relational DAG, terminating in Sink.
    assert!(
        query.operator_count() >= 2,
        "expected a non-trivial inner plan"
    );
    assert_matches!(
        query.operators().last(),
        Some(BindingsOp::Sink),
        "inner plan must terminate in Sink"
    );
}

#[test]
fn test_plain_create_table_lowers_to_create_table() {
    use partiql_logical::LogicalStatement;
    use partiql_value::BindingsName;

    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("CREATE TABLE t")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let table_name = assert_matches!(
        stmt,
        LogicalStatement::CreateTable { table_name } => table_name
    );
    assert_eq!(table_name, BindingsName::CaseInsensitive("t".into()));
}

#[test]
fn test_ctas_preserves_quoted_target_casing() {
    use partiql_logical::LogicalStatement;
    use partiql_value::BindingsName;

    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("CREATE TABLE \"T\" AS (SELECT a FROM foo)")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let table_name = assert_matches!(
        stmt,
        LogicalStatement::CreateTableAs { table_name, .. } => table_name
    );
    // Quoted identifier -> CaseSensitive, value preserved verbatim.
    assert_eq!(table_name, BindingsName::CaseSensitive("T".into()));
}

#[test]
fn test_ctas_inner_source_resolves_through_existing_path() {
    use partiql_logical::{self as logical, LogicalStatement};
    use partiql_value::BindingsName;

    let mut catalog = PartiqlCatalog::default();
    let _oid = catalog.add_type_entry(TypeEnvEntry::new("customers", &[], PartiqlShape::Dynamic));
    let catalog = catalog.to_shared_catalog();

    let parsed = partiql_parser::Parser::default()
        .parse("CREATE TABLE t AS (SELECT customers.name FROM customers AS c)")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let query = assert_matches!(
        stmt,
        LogicalStatement::CreateTableAs { query, .. } => query
    );

    // The catalog-registered source lowered to a Scan over a DBRef (not a fallback
    // Global VarRef), proving the inner query used the normal resolution path.
    let has_dbref_scan = query.operators().iter().any(|op| {
        matches!(
            op,
            BindingsOp::Scan(logical::Scan { expr: ValueExpr::DBRef(db), .. })
                if db.catalog == "default"
                    && db.path == vec![BindingsName::CaseInsensitive("customers".into())]
        )
    });
    assert!(
        has_dbref_scan,
        "inner source `customers` must resolve to a DBRef Scan"
    );
}

#[test]
fn test_legacy_lower_rejects_ctas() {
    // Back-compat contract: the legacy `lower` entry point (used by ~16 callers
    // that only handle relational plans) must still reject DDL rather than
    // silently changing its return type. CTAS is surfaced via `lower_statement`.
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("CREATE TABLE t AS (SELECT a FROM foo)")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let errs = planner
        .lower(&parsed)
        .expect_err("legacy lower() must reject CTAS")
        .errors;
    // Assert the specific rejection error, not merely that *some* error occurred,
    // so a future change that swapped this for a different error (a parse error,
    // the inner query's error, etc.) would fail rather than silently pass.
    assert_matches!(
        errs.as_slice(),
        [AstTransformError::NotYetImplemented(msg)] if msg == "DDL statement lowering"
    );
}

#[test]
fn test_legacy_lower_short_circuits_ddl_before_inner_query() {
    // The legacy `lower` shim must reject DDL at the front door, BEFORE lowering
    // the inner query. Here the CTAS source references an undefined function, which
    // would itself produce a lowering error — but `lower()` must still return the
    // uniform DDL rejection, never leak the inner-query error.
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("CREATE TABLE t AS (SELECT undefined_fn(a) FROM foo)")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let errs = planner
        .lower(&parsed)
        .expect_err("legacy lower() must reject CTAS")
        .errors;
    // Uniform DDL error — NOT the inner UnsupportedFunction("undefined_fn") error.
    assert_matches!(
        errs.as_slice(),
        [AstTransformError::NotYetImplemented(msg)] if msg == "DDL statement lowering"
    );
}

#[test]
fn test_legacy_lower_rejects_plain_create_table() {
    // The no-AS form (`CreateTable`, as_query: None) must also be rejected by the
    // legacy `lower` shim with the same uniform DDL error — covering the
    // CreateTable branch of the shim, not just CTAS.
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("CREATE TABLE t")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let errs = planner
        .lower(&parsed)
        .expect_err("legacy lower() must reject plain CREATE TABLE")
        .errors;
    assert_matches!(
        errs.as_slice(),
        [AstTransformError::NotYetImplemented(msg)] if msg == "DDL statement lowering"
    );
}

#[test]
fn test_insert_lowers_to_insert_into() {
    use partiql_logical::{BindingsOp, LogicalStatement};
    use partiql_value::BindingsName;

    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("INSERT INTO t SELECT a FROM foo WHERE a > 1")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let (table_name, query) = assert_matches!(
        stmt,
        LogicalStatement::InsertInto { table_name, query } => (table_name, query)
    );
    // Target carried verbatim, case-insensitive (bare identifier).
    assert_eq!(table_name, BindingsName::CaseInsensitive("t".into()));
    // Inner plan is the ordinary relational DAG, terminating in Sink.
    assert!(
        query.operator_count() >= 2,
        "expected a non-trivial inner plan"
    );
    assert_matches!(
        query.operators().last(),
        Some(BindingsOp::Sink),
        "inner plan must terminate in Sink"
    );
}

#[test]
fn test_insert_preserves_quoted_target_casing() {
    use partiql_logical::LogicalStatement;
    use partiql_value::BindingsName;

    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("INSERT INTO \"T\" SELECT a FROM foo")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let table_name = assert_matches!(
        stmt,
        LogicalStatement::InsertInto { table_name, .. } => table_name
    );
    // Quoted identifier -> CaseSensitive, value preserved verbatim.
    assert_eq!(table_name, BindingsName::CaseSensitive("T".into()));
}

#[test]
fn test_insert_inner_source_resolves_through_existing_path() {
    use partiql_logical::{self as logical, LogicalStatement};
    use partiql_value::BindingsName;

    let mut catalog = PartiqlCatalog::default();
    let _oid = catalog.add_type_entry(TypeEnvEntry::new("customers", &[], PartiqlShape::Dynamic));
    let catalog = catalog.to_shared_catalog();

    let parsed = partiql_parser::Parser::default()
        .parse("INSERT INTO t SELECT customers.name FROM customers AS c")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static);
    let stmt = planner
        .lower_statement(&parsed.statements[0])
        .expect("Expect successful lowering");

    let query = assert_matches!(
        stmt,
        LogicalStatement::InsertInto { query, .. } => query
    );
    // The catalog-registered source lowers to a Scan over a DBRef (not a
    // fallback Global VarRef), proving the wrapped inner query used the normal
    // resolution path — the same assertion the CTAS twin makes.
    let has_dbref_scan = query.operators().iter().any(|op| {
        matches!(
            op,
            BindingsOp::Scan(logical::Scan { expr: ValueExpr::DBRef(db), .. })
                if db.catalog == "default"
                    && db.path == vec![BindingsName::CaseInsensitive("customers".into())]
        )
    });
    assert!(
        has_dbref_scan,
        "inner source `customers` must resolve to a DBRef Scan"
    );
}

#[test]
fn test_legacy_lower_rejects_insert() {
    // Same back-compat contract as the DDL arms: the legacy `lower` shim must
    // reject DML with the uniform error, not silently change its return type.
    // INSERT is surfaced via `lower_statement`.
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("INSERT INTO t SELECT a FROM foo")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let errs = planner
        .lower(&parsed)
        .expect_err("legacy lower() must reject INSERT")
        .errors;
    assert_matches!(
        errs.as_slice(),
        [AstTransformError::NotYetImplemented(msg)] if msg == "DML statement lowering"
    );
}

#[test]
fn test_legacy_lower_short_circuits_dml_before_inner_query() {
    // The shim must reject DML at the front door, BEFORE lowering the inner
    // query. The INSERT source references an undefined function that would
    // itself error, but `lower()` must still return the uniform DML rejection.
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("INSERT INTO t SELECT undefined_fn(a) FROM foo")
        .expect("Expect successful parse");
    let planner = LogicalPlanner::new(&catalog);
    let errs = planner
        .lower(&parsed)
        .expect_err("legacy lower() must reject INSERT")
        .errors;
    assert_matches!(
        errs.as_slice(),
        [AstTransformError::NotYetImplemented(msg)] if msg == "DML statement lowering"
    );
}

// --- Static resolution of FROM aliases, lateral items and unqualified names ---

fn lower_static(
    statement: &str,
) -> std::result::Result<LogicalPlan<BindingsOp>, AstTransformationError> {
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse(statement)
        .expect("Expect successful parse");
    LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static).lower(&parsed)
}

fn scan_expr<'p>(plan: &'p LogicalPlan<BindingsOp>, as_key: &str) -> &'p ValueExpr {
    plan.operators()
        .iter()
        .find_map(|op| match op {
            BindingsOp::Scan(logical::Scan {
                expr, as_key: k, ..
            }) if k == as_key => Some(expr),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no scan `{as_key}` in {plan:?}"))
}

fn project_exprs(plan: &LogicalPlan<BindingsOp>) -> &[(String, ValueExpr)] {
    plan.operators()
        .iter()
        .find_map(|op| match op {
            Project(logical::Project { exprs }) => Some(exprs.as_slice()),
            _ => None,
        })
        .expect("project")
}

fn global(name: &str) -> ValueExpr {
    ValueExpr::VarRef(
        BindingsName::CaseInsensitive(name.to_string().into()),
        VarRefType::Global,
    )
}

fn attr(var: &str, key: &str) -> ValueExpr {
    ValueExpr::Path(
        Box::new(ValueExpr::VarRef(
            BindingsName::CaseInsensitive(var.to_string().into()),
            VarRefType::Local,
        )),
        vec![PathComponent::Key(BindingsName::CaseInsensitive(
            key.to_string().into(),
        ))],
    )
}

#[test]
fn test_static_from_alias_does_not_capture_its_own_source() {
    // `example` used to lower to `e.example`.
    let plan = lower_static("SELECT e.a FROM example e").expect("lower");
    assert_eq!(scan_expr(&plan, "e"), &global("example"));
    assert_eq!(project_exprs(&plan), &[("a".to_string(), attr("e", "a"))]);
}

#[test]
fn test_static_unqualified_name_is_attribute_of_single_binding() {
    let plan = lower_static("SELECT a FROM example AS e WHERE b > 1").expect("lower");
    assert_eq!(project_exprs(&plan), &[("a".to_string(), attr("e", "a"))]);
}

#[test]
fn test_static_from_source_ignores_group_by_keys() {
    // `example` used to lower to `c.example` (GROUP BY key registered on all ancestors).
    let plan = lower_static("SELECT c, COUNT(*) AS n FROM example e GROUP BY e.c").expect("lower");
    assert_eq!(scan_expr(&plan, "e"), &global("example"));
}

#[test]
fn test_static_lateral_from_item_sees_preceding_binding() {
    let plan = lower_static("SELECT n.x FROM table1 t1, t1.nodes n").expect("lower");
    assert_eq!(scan_expr(&plan, "t1"), &global("table1"));
    assert_eq!(scan_expr(&plan, "n"), &attr("t1", "nodes"));
}

#[test]
fn test_static_unqualified_from_source_is_global_not_lateral_attribute() {
    // `FROM t1, t2` is a join of two tables, not `t1.t2`.
    let plan = lower_static("SELECT u.x FROM table1 t, table2 u").expect("lower");
    assert_eq!(scan_expr(&plan, "u"), &global("table2"));
}

#[test]
fn test_static_unqualified_name_with_two_bindings_is_ambiguous() {
    let err = lower_static("SELECT x FROM table1 t1, t1.nodes n").expect_err("ambiguous");
    assert_matches!(
        err.errors.as_slice(),
        [AstTransformError::AmbiguousReference { name, candidates }]
            if name == "x" && candidates == &["t1".to_string(), "n".to_string()]
    );
}

#[test]
fn test_static_subquery_from_items_do_not_leak_into_outer_query() {
    let plan = lower_static("SELECT a FROM example e WHERE EXISTS (SELECT 1 FROM table1 t)")
        .expect("lower");
    assert_eq!(project_exprs(&plan), &[("a".to_string(), attr("e", "a"))]);
}

fn subquery_scan_project_exprs<'p>(
    plan: &'p LogicalPlan<BindingsOp>,
    as_key: &str,
) -> &'p [(String, ValueExpr)] {
    match scan_expr(plan, as_key) {
        ValueExpr::SubQueryExpr(sq) => project_exprs(&sq.plan),
        other => panic!("scan `{as_key}` is not a subquery: {other:?}"),
    }
}

#[test]
fn test_static_subquery_in_from_source_does_not_see_later_from_items() {
    // `t` inside `s` is not the later outer item `t` (unavailable when `s` is evaluated)
    // but an attribute of the subquery's own binding `q`.
    let plan =
        lower_static("SELECT s.x FROM (SELECT t.x AS x FROM <<{'x':1}>> q) s, <<{'x':2}>> t")
            .expect("lower");
    let t_x = ValueExpr::Path(
        Box::new(attr("q", "t")),
        vec![PathComponent::Key(BindingsName::CaseInsensitive(
            "x".into(),
        ))],
    );
    assert_eq!(
        subquery_scan_project_exprs(&plan, "s"),
        &[("x".to_string(), t_x)]
    );
}

#[test]
fn test_static_subquery_in_from_source_sees_preceding_from_items() {
    let plan =
        lower_static("SELECT s.y FROM table1 a, (SELECT a.x AS y FROM <<1>> q) s").expect("lower");
    assert_eq!(
        subquery_scan_project_exprs(&plan, "s"),
        &[("y".to_string(), attr("a", "x"))]
    );
}

#[test]
fn test_static_from_less_lateral_subquery_sees_implicit_attribute() {
    // The FROM-source rule (unmatched name is a global) applies to the FROM source
    // itself, not to a query nested in it: `x` is `a.x`.
    let plan = lower_static("SELECT s.y FROM <<{'x':1}>> a, (SELECT x AS y) s").expect("lower");
    assert_eq!(
        subquery_scan_project_exprs(&plan, "s"),
        &[("y".to_string(), attr("a", "x"))]
    );
}

#[test]
fn test_static_graph_pattern_predicate_sees_pattern_binding() {
    let plan = lower_static("SELECT x.n FROM (g MATCH (n WHERE n.a = 1)) AS x").expect("lower");
    let plan = format!("{plan:?}");
    assert!(
        plan.contains(r#"Path(VarRef(CaseInsensitive("n"), Local), [Key(CaseInsensitive("a"))])"#),
        "{plan}"
    );
}

#[test]
fn test_static_nested_group_by_key_is_attribute_of_its_own_query() {
    // The GROUP BY key sees the outer `o` too, but `x` belongs to the inner query's
    // single FROM binding `i` -- not ambiguous.
    let plan = lower_static(
        "SELECT * FROM outer_table o WHERE EXISTS (SELECT COUNT(*) FROM inner_table i GROUP BY x)",
    )
    .expect("lower");
    let group_by = format!("{plan:?}");
    assert!(
        group_by.contains(r#"exprs: {"x": Path(VarRef(CaseInsensitive("i"), Local), [Key(CaseInsensitive("x"))])}"#),
        "{group_by}"
    );
}

#[test]
fn test_static_local_binding_shadows_global_outside_from() {
    // `t` is both a catalog table and the alias of `orders`: in FROM the table is used,
    // elsewhere the local binding wins.
    let mut catalog = PartiqlCatalog::default();
    for name in ["t", "orders"] {
        let _oid = catalog.add_type_entry(TypeEnvEntry::new(name, &[], PartiqlShape::Dynamic));
    }
    let catalog = catalog.to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("SELECT t.a FROM orders AS t")
        .expect("Expect successful parse");
    let plan = LogicalPlanner::with_var_resolution(&catalog, VarRefResolution::Static)
        .lower(&parsed)
        .expect("lower");
    assert_matches!(scan_expr(&plan, "t"), ValueExpr::DBRef(db)
        if db.path == vec![BindingsName::CaseInsensitive("orders".into())]);
    assert_eq!(project_exprs(&plan), &[("a".to_string(), attr("t", "a"))]);
}

#[test]
fn test_static_from_infers_alias_from_table_name_or_last_path_step() {
    let mut catalog = PartiqlCatalog::default();
    let _oid = catalog.add_type_entry(TypeEnvEntry::new("onek", &[], PartiqlShape::Dynamic));
    let with_catalog = catalog.to_shared_catalog();
    let without_catalog = PartiqlCatalog::default().to_shared_catalog();
    for catalog in [&with_catalog, &without_catalog] {
        let lower = |statement: &str| {
            let parsed = partiql_parser::Parser::default()
                .parse(statement)
                .expect("Expect successful parse");
            LogicalPlanner::with_var_resolution(catalog, VarRefResolution::Static)
                .lower(&parsed)
                .expect("lower")
        };

        // `FROM onek` binds `onek` (not a generated `_1`), whether or not the catalog
        // knows the table.
        let plan = lower("SELECT onek.x FROM onek WHERE onek.x > 1");
        scan_expr(&plan, "onek");
        assert_eq!(
            project_exprs(&plan),
            &[("x".to_string(), attr("onek", "x"))]
        );

        // A path source binds its last step.
        let plan = lower("SELECT unique1.y FROM onek.unique1");
        scan_expr(&plan, "unique1");
        assert_eq!(
            project_exprs(&plan),
            &[("y".to_string(), attr("unique1", "y"))]
        );
    }
}

fn join_right_scan(plan: &LogicalPlan<BindingsOp>) -> &ValueExpr {
    let join = plan
        .operators()
        .iter()
        .find_map(|op| match op {
            BindingsOp::Join(join) => Some(join),
            _ => None,
        })
        .expect("join");
    match join.right.as_ref() {
        BindingsOp::Scan(scan) => &scan.expr,
        other => panic!("right side is not a scan: {other:?}"),
    }
}

#[test]
fn test_static_inner_left_and_cross_joins_are_lateral() {
    for join in ["JOIN", "INNER JOIN", "LEFT JOIN", "CROSS JOIN"] {
        let on = if join == "CROSS JOIN" { "" } else { " ON true" };
        let plan = lower_static(&format!("SELECT n.x FROM t1 AS a {join} a.nodes AS n{on}"))
            .expect("lower");
        assert_eq!(join_right_scan(&plan), &attr("a", "nodes"), "{join}");
    }
}

#[test]
fn test_static_right_and_full_joins_are_not_lateral() {
    // The right side does not see `a`, so `a.nodes` is a path on the global `a`.
    for join in ["RIGHT JOIN", "FULL JOIN"] {
        let plan = lower_static(&format!(
            "SELECT n.x FROM t1 AS a {join} a.nodes AS n ON true"
        ))
        .expect("lower");
        let expected = ValueExpr::Path(
            Box::new(global("a")),
            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                "nodes".into(),
            ))],
        );
        assert_eq!(join_right_scan(&plan), &expected, "{join}");
    }
}

#[test]
fn test_select_item_equal_to_group_key_reads_the_key() {
    // `e.c` is the GROUP BY key `c`, so the projection reads `c`, whatever its own alias.
    let plan = lower_static("SELECT e.c AS k FROM example AS e GROUP BY e.c").expect("lower");
    assert_eq!(
        project_exprs(&plan),
        &[(
            "k".to_string(),
            ValueExpr::VarRef(BindingsName::CaseSensitive("c".into()), VarRefType::Local)
        )]
    );
}

#[test]
fn test_static_order_by_sees_select_aliases() {
    let plan = lower_static("SELECT e.a AS x FROM example AS e ORDER BY x").expect("lower");
    let order_by = plan
        .operators()
        .iter()
        .find_map(|op| match op {
            BindingsOp::OrderBy(order_by) => Some(order_by),
            _ => None,
        })
        .expect("order by");
    assert_eq!(order_by.specs[0].expr, attr("e", "a"));
}

#[test]
fn test_dynamic_outer_query_bindings_are_globals_in_a_subquery() {
    // The evaluator binds an enclosing query's variables as globals of the subquery.
    let catalog = PartiqlCatalog::default().to_shared_catalog();
    let parsed = partiql_parser::Parser::default()
        .parse("SELECT o.a FROM outer_t AS o WHERE EXISTS (SELECT i.b FROM o.items AS i)")
        .expect("Expect successful parse");
    let plan = LogicalPlanner::new(&catalog).lower(&parsed).expect("lower");
    let plan = format!("{plan:?}");
    assert!(
        plan.contains(r#"Scan(Scan { expr: Path(DynamicLookup([VarRef(CaseInsensitive("o"), Global)]), [Key(CaseInsensitive("items"))]), as_key: "i""#),
        "{plan}"
    );
}

#[test]
fn test_unnamed_select_items_are_named_by_position() {
    let plan = lower_static("SELECT e.a, e.b + 1, 2 FROM example AS e").expect("lower");
    let names: Vec<&str> = project_exprs(&plan)
        .iter()
        .map(|(n, _)| n.as_str())
        .collect();
    assert_eq!(names, ["a", "_2", "_3"]);
}

#[test]
fn test_static_order_by_binding_shadows_select_alias() {
    let plan = lower_static("SELECT a.x AS a FROM t AS a ORDER BY a.y").expect("lower");
    let order_by = plan
        .operators()
        .iter()
        .find_map(|op| match op {
            BindingsOp::OrderBy(order_by) => Some(order_by),
            _ => None,
        })
        .expect("order by");
    assert_eq!(order_by.specs[0].expr, attr("a", "y"));
}
