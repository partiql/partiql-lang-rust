use crate::error::{LowerError, LoweringError};
use crate::lower::AstToLogical;
use crate::name_resolver::NameResolver;
use partiql_ast::ast;
use partiql_logical as logical;
use partiql_parser::Parsed;

mod call_defs;
pub mod error;
mod lower;
mod name_resolver;

// TODO better encapsulate.
pub fn lower(parsed: &Parsed) -> Result<logical::LogicalPlan<logical::BindingsOp>, LoweringError> {
    if let ast::Expr::Query(q) = parsed.ast.as_ref() {
        let mut resolver = NameResolver::default();
        let registry = resolver.resolve(q)?;
        let planner = AstToLogical::new(registry);
        planner.lower_query(q)
    } else {
        Err(LoweringError {
            errors: vec![LowerError::NotYetImplemented(
                "Expr type not supported yet".to_string(),
            )],
        })
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use partiql_eval::env::basic::MapBindings;

    use partiql_eval::plan;

    use crate::error::LoweringError;
    use partiql_logical as logical;
    use partiql_logical::{BindingsOp, LogicalPlan};
    use partiql_parser::{Parsed, Parser};
    use partiql_value::{partiql_bag, partiql_tuple, Value};

    #[track_caller]
    fn parse(text: &str) -> Parsed {
        Parser::default().parse(text).unwrap()
    }

    #[track_caller]
    fn lower(parsed: &Parsed) -> Result<logical::LogicalPlan<logical::BindingsOp>, LoweringError> {
        super::lower(parsed)
    }

    #[track_caller]
    fn evaluate(logical: LogicalPlan<BindingsOp>, bindings: MapBindings<Value>) -> Value {
        let mut planner = plan::EvaluatorPlanner::new();
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
            partiql_tuple![("id", id), ("firstName", first_name), ("balance", balance),].into()
        }

        let customer_val = partiql_bag![
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
            let expected = partiql_bag![
                partiql_tuple![("firstName", "jason"), ("doubleName", "jasonjason")],
                partiql_tuple![("firstName", "miriam"), ("doubleName", "miriammiriam")],
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
            let expected = partiql_bag![partiql_tuple![("_1", 5)]];
            assert_eq!(*bag, expected);
        });
    }
}
