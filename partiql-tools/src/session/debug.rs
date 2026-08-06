//! Debug flag parsing from CLI args. Bin-side accessor into what statements
//! do (ast/plan/program) — used by both the CLI and the test harness.

#[derive(Debug, Default, Clone)]
pub struct DebugFlags {
    pub ast: bool,
    pub plan: bool,
    pub program: bool,
}

impl DebugFlags {
    pub fn from_args(args: &[String]) -> Self {
        let all = args.iter().any(|s| s == "*");
        DebugFlags {
            ast: all || args.iter().any(|s| s == "ast"),
            plan: all || args.iter().any(|s| s == "plan"),
            program: all || args.iter().any(|s| s == "program"),
        }
    }
}
