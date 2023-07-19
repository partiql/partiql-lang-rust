use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use partiql_catalog::{Catalog, PartiqlCatalog};
use rand::{Rng, SeedableRng};

use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_logical::{BindingsOp, LogicalPlan};
use partiql_logical_planner::LogicalPlanner;

use partiql_parser::{Parser, ParserResult};
use partiql_value::{tuple, Bag, Value};

// Benchmarks:
//  - parsing,
//  - compiling
//  - planning
//  - evaluation
//
// of queries that  filter against 1, 15, or 30 `OR`ed `LIKE` expressions
// over 10201 rows of tuples containing an id and a string

fn employee_data() -> Vec<Value> {
    let name1 = vec![
        "Bob",
        "Madden",
        "Brycen",
        "Bryanna",
        "Zayne",
        "Jocelynn",
        "Breanna",
        "Margaret",
        "Jasmine",
        "Kenyon",
        "Aryanna",
        "Zackery",
        "Jorden",
        "Malia",
        "Raven",
        "Neveah",
        "Finley",
        "Austin",
        "Jaxson",
        "Tobias",
        "Dominique",
        "Devan",
        "Colby",
        "Tanner",
        "Mckenna",
        "Kristina",
        "Cristal",
        "River",
        "Taliyah",
        "Abagail",
        "Spencer",
        "Gage",
        "Ronnie",
        "Amari",
        "Jabari",
        "Alanna",
        "Anderson",
        "Saniya",
        "Baylee",
        "Elisa",
        "Savannah",
        "Jakobe",
        "Sandra",
        "Simone",
        "Frank",
        "Braedon",
        "Clark",
        "Francisco",
        "Roman",
        "Matias",
        "Messi",
        "Elisha",
        "Alexander",
        "Kadence",
        "Karsyn",
        "Adonis",
        "Ishaan",
        "Trevon",
        "Ryan",
        "Jaelynn",
        "Marilyn",
        "Emma",
        "Avah",
        "Jordan",
        "Riley",
        "Amelie",
        "Denisse",
        "Darion",
        "Lydia",
        "Marley",
        "Brogan",
        "Trace",
        "Maeve",
        "Elijah",
        "Kareem",
        "Erick",
        "Hope",
        "Elisabeth",
        "Antwan",
        "Francesca",
        "Layla",
        "Jase",
        "Angel",
        "Addyson",
        "Mckinley",
        "Julianna",
        "Winston",
        "Royce",
        "Paola",
        "Issac",
        "Zachary",
        "Niko",
        "Shania",
        "Colin",
        "Jesse",
        "Pedro",
        "Cheyenne",
        "Ashley",
        "Karli",
        "Bianca",
        "Mario",
    ];
    let name2 = vec![
        "Smith",
        "Oconnell",
        "Whitehead",
        "Carrillo",
        "Parrish",
        "Monroe",
        "Summers",
        "Hurst",
        "Durham",
        "Hardin",
        "Hunt",
        "Mitchell",
        "Pennington",
        "Woodward",
        "Franklin",
        "Martinez",
        "Shepard",
        "Khan",
        "Mcfarland",
        "Frey",
        "Mckenzie",
        "Blair",
        "Mercer",
        "Callahan",
        "Cameron",
        "Gilmore",
        "Bowers",
        "Donovan",
        "Meyers",
        "Horne",
        "Rice",
        "Castillo",
        "Cain",
        "Dickson",
        "Valenzuela",
        "Silva",
        "Prince",
        "Vance",
        "Berry",
        "Coffey",
        "Young",
        "Walker",
        "Burch",
        "Ross",
        "Mejia",
        "Zuniga",
        "Haney",
        "Jordan",
        "Love",
        "Larsen",
        "Bowman",
        "Werner",
        "Greer",
        "Krause",
        "Bishop",
        "Day",
        "Luna",
        "Patrick",
        "Adkins",
        "Benson",
        "Mcconnell",
        "Sanchez",
        "Villa",
        "Wu",
        "Duke",
        "Fisher",
        "Hess",
        "Lawrence",
        "Perry",
        "Hardy",
        "Wyatt",
        "Mcknight",
        "Thomas",
        "Trevino",
        "Flowers",
        "Cisneros",
        "Coleman",
        "Sanders",
        "Good",
        "Newton",
        "Carpenter",
        "Garza",
        "Barber",
        "Swanson",
        "Owen",
        "Anderson",
        "Bright",
        "Beck",
        "Lawson",
        "Jones",
        "Davila",
        "Porter",
        "Dougherty",
        "Stevenson",
        "Malone",
        "Garrison",
        "Bates",
        "Wheeler",
        "Petty",
        "Rojas",
        "Townsend",
    ];

    // cartesian product of name1 x name2 (e.g., "Bob Smith", ... "Mario Townsend")
    let combined = name1
        .iter()
        .cartesian_product(name2.iter())
        .map(|(n1, n2)| format!("{n1} {n2}"));

    // seed the rng with a known value to assure same data across runs
    let mut rng = rand::rngs::StdRng::from_seed([42; 32]);
    use rand::distributions::Distribution;
    let chars = rand::distributions::Alphanumeric;
    let random_size = rand::distributions::uniform::Uniform::from(5..=100);

    // add random string prefix and suffix to each combined name
    let employee_data: Vec<Value> = combined
        .enumerate()
        .map(|(id, person)| {
            let prefix_size = random_size.sample(&mut rng);
            let suffix_size = random_size.sample(&mut rng);
            let prefix: String = (0..prefix_size)
                .map(|_| rng.sample(chars) as char)
                .collect();
            let suffix: String = (0..suffix_size)
                .map(|_| rng.sample(chars) as char)
                .collect();
            let full_name = format!("{prefix} {person} {suffix}");
            tuple![("id", id), ("name", full_name)].into()
        })
        .collect_vec();

    employee_data
}

fn data() -> MapBindings<Value> {
    let data = tuple![("hr", tuple![("employees", Bag::from(employee_data()))])];

    data.into()
}

const QUERY_1: &str = "
            SELECT *
            FROM hr.employees as emp
            WHERE lower(emp.name) LIKE '%bob smith%'
            ";

const QUERY_15: &str = "
            SELECT *
            FROM hr.employees as emp
            WHERE lower(emp.name) LIKE '%bob smith%'
               OR lower(emp.name) LIKE '%gage swanson%'
               OR lower(emp.name) LIKE '%riley perry%'
               OR lower(emp.name) LIKE '%sandra woodward%'
               OR lower(emp.name) LIKE '%abagail oconnell%'
               OR lower(emp.name) LIKE '%amari duke%'
               OR lower(emp.name) LIKE '%elisha wyatt%'
               OR lower(emp.name) LIKE '%aryanna hess%'
               OR lower(emp.name) LIKE '%bryanna jones%'
               OR lower(emp.name) LIKE '%trace gilmore%'
               OR lower(emp.name) LIKE '%antwan stevenson%'
               OR lower(emp.name) LIKE '%julianna callahan%'
               OR lower(emp.name) LIKE '%jaelynn trevino%'
               OR lower(emp.name) LIKE '%kadence bates%'
               OR lower(emp.name) LIKE '%jakobe townsend%'
            ";

const QUERY_30: &str = "
            SELECT *
            FROM hr.employees as emp
            WHERE lower(emp.name) LIKE '%bob smith%'
               OR lower(emp.name) LIKE '%gage swanson%'
               OR lower(emp.name) LIKE '%riley perry%'
               OR lower(emp.name) LIKE '%sandra woodward%'
               OR lower(emp.name) LIKE '%abagail oconnell%'
               OR lower(emp.name) LIKE '%amari duke%'
               OR lower(emp.name) LIKE '%elisha wyatt%'
               OR lower(emp.name) LIKE '%aryanna hess%'
               OR lower(emp.name) LIKE '%bryanna jones%'
               OR lower(emp.name) LIKE '%trace gilmore%'
               OR lower(emp.name) LIKE '%antwan stevenson%'
               OR lower(emp.name) LIKE '%julianna callahan%'
               OR lower(emp.name) LIKE '%jaelynn trevino%'
               OR lower(emp.name) LIKE '%kadence bates%'
               OR lower(emp.name) LIKE '%jakobe townsend%'
               OR lower(emp.name) LIKE '%austin pennington%'
               OR lower(emp.name) LIKE '%colby woodward%'
               OR lower(emp.name) LIKE '%brycen blair%'
               OR lower(emp.name) LIKE '%cristal mercer%'
               OR lower(emp.name) LIKE '%river gilmore%'
               OR lower(emp.name) LIKE '%saniya bowers%'
               OR lower(emp.name) LIKE '%braedon ross%'
               OR lower(emp.name) LIKE '%clark mejia%'
               OR lower(emp.name) LIKE '%ryan day%'
               OR lower(emp.name) LIKE '%marilyn luna%'
               OR lower(emp.name) LIKE '%avah sanchez%'
               OR lower(emp.name) LIKE '%amelie wu%'
               OR lower(emp.name) LIKE '%paola duke%'
               OR lower(emp.name) LIKE '%jesse trevino%'
               OR lower(emp.name) LIKE '%bianca cisneros%'
            ";

#[inline]
fn parse(text: &str) -> ParserResult {
    Parser::default().parse(text)
}
#[inline]
fn compile(catalog: &dyn Catalog, parsed: &partiql_parser::Parsed) -> LogicalPlan<BindingsOp> {
    let planner = LogicalPlanner::new(catalog);
    planner.lower(parsed).expect("Expect no lower error")
}
#[inline]
fn plan(catalog: &dyn Catalog, logical: &LogicalPlan<BindingsOp>) -> EvalPlan {
    EvaluatorPlanner::new(EvaluationMode::Permissive, catalog)
        .compile(logical)
        .expect("Expect no plan error")
}
#[inline]
pub(crate) fn evaluate(mut eval: EvalPlan, bindings: MapBindings<Value>) -> Value {
    if let Ok(out) = eval.execute_mut(bindings) {
        out.result
    } else {
        Value::Missing
    }
}

/// benchmark parsing of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_parse(c: &mut Criterion) {
    let parsed_1 = parse(QUERY_1);
    assert!(parsed_1.is_ok());
    let parsed_15 = parse(QUERY_15);
    assert!(parsed_15.is_ok());
    let parsed_30 = parse(QUERY_30);
    assert!(parsed_30.is_ok());

    c.bench_function("parse-1", |b| b.iter(|| parse(black_box(QUERY_1))));
    c.bench_function("parse-15", |b| b.iter(|| parse(black_box(QUERY_15))));
    c.bench_function("parse-30", |b| b.iter(|| parse(black_box(QUERY_30))));
}

/// benchmark compiling of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_compile(c: &mut Criterion) {
    let catalog = PartiqlCatalog::default();

    let parsed_1 = parse(QUERY_1).unwrap();
    let parsed_15 = parse(QUERY_15).unwrap();
    let parsed_30 = parse(QUERY_30).unwrap();

    let compiled_1 = compile(&catalog, &parsed_1);
    assert_eq!(compiled_1.operator_count(), 4);
    let compiled_15 = compile(&catalog, &parsed_15);
    assert_eq!(compiled_15.operator_count(), 4);
    let compiled_30 = compile(&catalog, &parsed_30);
    assert_eq!(compiled_30.operator_count(), 4);

    c.bench_function("compile-1", |b| {
        b.iter(|| compile(&catalog, black_box(&parsed_1)))
    });
    c.bench_function("compile-15", |b| {
        b.iter(|| compile(&catalog, black_box(&parsed_15)))
    });
    c.bench_function("compile-30", |b| {
        b.iter(|| compile(&catalog, black_box(&parsed_30)))
    });
}

/// benchmark planning of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_plan(c: &mut Criterion) {
    let catalog = PartiqlCatalog::default();

    let compiled_1 = compile(&catalog, &parse(QUERY_1).unwrap());
    let compiled_15 = compile(&catalog, &parse(QUERY_15).unwrap());
    let compiled_30 = compile(&catalog, &parse(QUERY_30).unwrap());

    let _planned_1 = plan(&catalog, &compiled_1);
    let _planned_15 = plan(&catalog, &compiled_15);
    let _planned_30 = plan(&catalog, &compiled_30);

    c.bench_function("plan-1", |b| {
        b.iter(|| plan(&catalog, black_box(&compiled_1)))
    });
    c.bench_function("plan-15", |b| {
        b.iter(|| plan(&catalog, black_box(&compiled_15)))
    });
    c.bench_function("plan-30", |b| {
        b.iter(|| plan(&catalog, black_box(&compiled_30)))
    });
}

/// benchmark evaluation of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_eval(c: &mut Criterion) {
    let catalog = PartiqlCatalog::default();

    let compiled_1 = compile(&catalog, &parse(QUERY_1).unwrap());
    let compiled_15 = compile(&catalog, &parse(QUERY_15).unwrap());
    let compiled_30 = compile(&catalog, &parse(QUERY_30).unwrap());

    let bindings = data();

    c.bench_function("eval-1", |b| {
        b.iter(|| {
            let plan = plan(&catalog, &compiled_1);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
    c.bench_function("eval-15", |b| {
        b.iter(|| {
            let plan = plan(&catalog, &compiled_15);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
    c.bench_function("eval-30", |b| {
        b.iter(|| {
            let plan = plan(&catalog, &compiled_30);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
}

criterion_group! {
    name = eval;
    config = Criterion::default().measurement_time(Duration::new(5, 0));
    targets = bench_parse, bench_compile, bench_plan, bench_eval
}

criterion_main!(eval);
