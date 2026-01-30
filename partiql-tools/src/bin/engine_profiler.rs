use partiql_catalog::context::SystemContext;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_eval::{PlanCompiler, ReaderFactory, ScanProvider};
use partiql_logical::Scan;
use partiql_tools::common::{compile, create_catalog, lower, parse};
use partiql_value::{DateTime, Value};
use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

/// Available engines
#[derive(Debug, Clone, Copy)]
enum Engine {
    Legacy,
    Hybrid,
}

/// Available queries
const QUERIES: &[(&str, &str)] = &[
    ("proj", "SELECT a, b FROM data"),
    ("every_other", "SELECT a, b FROM data WHERE a % 2 = 0"),
    (
        "filter_complex",
        "SELECT a, b FROM data WHERE ((a - a + b - b + a - a + b - b) + a % 100000) = 0",
    ),
];

fn main() {
    // Read configuration from environment variables
    let engine = std::env::var("ENGINE")
        .unwrap_or_else(|_| "legacy".to_string())
        .to_lowercase();
    let query_name = std::env::var("QUERY")
        .unwrap_or_else(|_| "proj".to_string())
        .to_lowercase();
    let format = std::env::var("FORMAT")
        .unwrap_or_else(|_| "mem".to_string())
        .to_lowercase();
    let size: usize = std::env::var("SIZE")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);
    let iterations: usize = std::env::var("ITERATIONS")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);
    let data_path = std::env::var("DATA_PATH").ok();

    // Parse engine
    let engine = match engine.as_str() {
        "legacy" => Engine::Legacy,
        "hybrid" => Engine::Hybrid,
        _ => {
            eprintln!("Unknown engine: {}. Use: legacy, hybrid", engine);
            std::process::exit(1);
        }
    };

    // Find query
    let query = QUERIES
        .iter()
        .find(|(name, _)| *name == query_name)
        .map(|(_, q)| *q)
        .unwrap_or_else(|| {
            eprintln!(
                "Unknown query: {}. Available: {}",
                query_name,
                QUERIES
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            std::process::exit(1);
        });

    // Validate format
    if format != "mem" && format != "ion" {
        eprintln!("Unknown format: {}. Use: mem, ion", format);
        std::process::exit(1);
    }

    // Validate data_path for ion format
    if format == "ion" && data_path.is_none() {
        eprintln!("DATA_PATH environment variable required for ion format");
        std::process::exit(1);
    }

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                     ENGINE PROFILER CONFIGURATION                         ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Engine:     {:?}", engine);
    println!("Query:      {} ({})", query_name, query);
    println!("Format:     {}", format);
    println!("Size:       {} rows", size);
    println!("Iterations: {}", iterations);
    if let Some(ref path) = data_path {
        println!("Data Path:  {}", path);
    }
    println!();
    println!("Starting profiling workload...");
    println!();

    // Create plan once (NOT timed)
    let plan = match engine {
        Engine::Legacy => Plan::Legacy(LegacyPlan::new(query, &format, size, data_path.clone())),
        Engine::Hybrid => Plan::Hybrid(HybridPlan::new(query, &format, size, data_path.clone())),
    };

    // Execute many times (TIMED)
    let start = Instant::now();
    let mut total_rows = 0;
    for _ in 0..iterations {
        total_rows = black_box(plan.execute());
    }
    let elapsed = start.elapsed();

    println!();
    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                        PROFILING COMPLETE                                 ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total time:      {:.2}s", elapsed.as_secs_f64());
    println!("Iterations:      {}", iterations);
    println!("Rows/iteration:  {}", total_rows);
    println!(
        "Avg time/iter:   {:.2}ms",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );
    println!();
}

/// Compiled plan for Legacy engine
struct LegacyPlan {
    plan: partiql_eval::eval::EvalPlan,
}

impl LegacyPlan {
    fn new(query: &str, format: &str, size: usize, data_path: Option<String>) -> Self {
        // Adapt query for legacy (needs data() function call)
        let non_vec_query = query.replace("data", "data()");

        // Set environment variable for data generation
        std::env::set_var("TOTAL_ROWS", size.to_string());

        let catalog = create_catalog(format.to_string(), data_path);
        let parsed = parse(&non_vec_query).expect("Parse failed");
        let logical = lower(&*catalog, &parsed).expect("Lower failed");
        let plan = compile(EvaluationMode::Permissive, &*catalog, logical).expect("Compile failed");

        Self { plan }
    }

    fn execute(&self) -> usize {
        let bindings = MapBindings::default();
        let sys = SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);

        match self.plan.execute(&ctx) {
            Ok(evaluated) => match evaluated.result {
                Value::Bag(bag) => bag.len(),
                _ => 1,
            },
            Err(_) => 0,
        }
    }
}

/// Compiled plan for Hybrid engine
struct HybridPlan {
    compiled: std::sync::Arc<partiql_eval::CompiledPlan>,
    compiler: Rc<PlanCompiler<'static>>,
}

impl HybridPlan {
    fn new(query: &str, format: &str, size: usize, data_path: Option<String>) -> Self {
        let catalog = create_catalog("mem".to_string(), None);
        let parsed = parse(query).expect("Parse failed");
        let logical = lower(&*catalog, &parsed).expect("Lower failed");

        let provider = Box::leak(Box::new(HybridScanProvider {
            format: format.to_string(),
            data_path,
            num_rows: size,
        }));

        let compiler = PlanCompiler::new(provider as &'static dyn ScanProvider);
        let compiled = compiler.compile(&logical).expect("Compile failed");

        Self {
            compiled: std::sync::Arc::new(compiled),
            compiler: Rc::new(compiler),
        }
    }

    fn create_vm(&self) -> partiql_eval::PartiQLVM {
        self.compiler
            .instantiate((*self.compiled).clone(), None)
            .expect("Instantiate failed")
    }
}

/// Unified plan enum for all engine types
enum Plan {
    Legacy(LegacyPlan),
    Hybrid(HybridPlan),
}

impl Plan {
    fn execute(&self) -> usize {
        match self {
            Plan::Legacy(plan) => plan.execute(),
            Plan::Hybrid(plan) => {
                // Hybrid consumes the VM, so we need to recreate it
                // This is the same cost as criterion's iter_batched setup
                let mut vm = plan.create_vm();
                let mut row_count = 0;

                match vm.execute() {
                    Ok(partiql_eval::ExecutionResult::Query(iter)) => {
                        for row_result in iter {
                            match row_result {
                                Ok(_row) => row_count += 1,
                                Err(e) => {
                                    eprintln!("Execution error: {:?}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Execute error: {:?}", e);
                        std::process::exit(1);
                    }
                }

                row_count
            }
        }
    }
}

struct HybridScanProvider {
    format: String,
    data_path: Option<String>,
    num_rows: usize,
}

impl ScanProvider for HybridScanProvider {
    fn reader_factory(&self, _scan: &Scan) -> partiql_eval::Result<ReaderFactory> {
        match self.format.as_str() {
            "mem" => Ok(ReaderFactory::mem(
                self.num_rows,
                vec!["a".to_string(), "b".to_string()],
            )),
            "ion" => {
                let path = self.data_path.clone().ok_or_else(|| {
                    partiql_eval::EngineError::ReaderError("ion path required".to_string())
                })?;
                Ok(ReaderFactory::ion(path))
            }
            other => Err(partiql_eval::EngineError::ReaderError(format!(
                "unsupported format: {}",
                other
            ))),
        }
    }
}
