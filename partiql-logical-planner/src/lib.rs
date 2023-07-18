use crate::lower::AstToLogical;

use partiql_ast_passes::error::AstTransformationError;
use partiql_ast_passes::name_resolver::NameResolver;
use partiql_logical as logical;
use partiql_parser::Parsed;

use partiql_catalog::Catalog;

mod builtins;
mod lower;

pub struct LogicalPlanner<'c> {
    catalog: &'c dyn Catalog,
}

impl<'c> LogicalPlanner<'c> {
    pub fn new(catalog: &'c dyn Catalog) -> Self {
        LogicalPlanner { catalog }
    }

    #[inline]
    pub fn lower(
        &self,
        parsed: &Parsed,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let q = &parsed.ast;
        let mut resolver = NameResolver::default();
        let registry = resolver.resolve(q)?;
        let planner = AstToLogical::new(self.catalog, registry);
        planner.lower_query(q)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use partiql_ast_passes::error::AstTransformationError;
    use partiql_catalog::PartiqlCatalog;

    use partiql_eval::env::basic::MapBindings;

    use partiql_eval::plan;
    use partiql_eval::plan::EvaluationMode;

    use crate::LogicalPlanner;
    use partiql_logical as logical;
    use partiql_logical::{BindingsOp, LogicalPlan};
    use partiql_parser::{Parsed, Parser};
    use partiql_value::{bag, tuple, Value};

    #[track_caller]
    fn parse(text: &str) -> Parsed {
        Parser::default().parse(text).unwrap()
    }

    #[track_caller]
    fn lower(
        parsed: &Parsed,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let catalog = PartiqlCatalog::default();
        let planner = LogicalPlanner::new(&catalog);
        planner.lower(parsed)
    }

    #[track_caller]
    fn evaluate(logical: LogicalPlan<BindingsOp>, bindings: MapBindings<Value>) -> Value {
        let catalog = PartiqlCatalog::default();
        let mut planner = plan::EvaluatorPlanner::new(EvaluationMode::Permissive, &catalog);
        let mut plan = planner.compile(&logical).expect("Expect no plan error");
        println!("{}", plan.to_dot_graph());

        if let Ok(out) = plan.execute_mut(bindings) {
            out.result
        } else {
            Value::Missing
        }
    }

    #[track_caller]
    fn evaluate_query(query: &str) -> Value {
        let parsed = parse(query);
        let lowered = lower(&parsed).expect("Expect no lower error");
        evaluate(lowered, Default::default())
    }

    fn data_customer() -> MapBindings<Value> {
        fn customer_tuple(id: i64, first_name: &str, balance: i64) -> Value {
            tuple![("id", id), ("firstName", first_name), ("balance", balance),].into()
        }

        let customer_val = bag![
            customer_tuple(5, "jason", 100),
            customer_tuple(4, "sisko", 0),
            customer_tuple(3, "jason", -30),
            customer_tuple(2, "miriam", 20),
            customer_tuple(1, "miriam", 10),
        ];

        let mut bindings = MapBindings::default();
        bindings.insert("customer", customer_val.into());
        bindings
    }

    #[test]
    pub fn test() {
        // Plan for `SELECT DISTINCT firstName, (firstName || firstName) AS doubleName FROM customer WHERE balance > 0`
        let query = "\
        SELECT DISTINCT firstName, (firstName || firstName) AS doubleName \
        FROM customer \
        WHERE balance > 0";
        let parsed = parse(query);
        let lowered = lower(&parsed).expect("Expect no lower error");
        let out = evaluate(lowered, data_customer());

        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = bag![
                tuple![("firstName", "jason"), ("doubleName", "jasonjason")],
                tuple![("firstName", "miriam"), ("doubleName", "miriammiriam")],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    pub fn test_5() {
        let out = evaluate_query("5");
        println!("{:?}", &out);
        assert_matches!(out, Value::Integer(5));
    }

    #[test]
    pub fn test_from_5() {
        let out = evaluate_query("SELECT * FROM 5");
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = bag![tuple![("_1", 5)]];
            assert_eq!(*bag, expected);
        });
    }
}
