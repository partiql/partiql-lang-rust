use jni::objects::{JClass, JString};
use jni::sys::jlong;
use jni::JNIEnv;

use partiql_catalog::catalog::PartiqlCatalog;
use partiql_eval::PlanCompiler;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::Parser;

use crate::{create_plan_handle, jni_guard};

/// JNI wrapper for PlanCompiler.compile()
///
/// Java signature:
/// ```java
/// public native long nativeCompile(String sql, long contextHandle) throws PartiQLException;
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PlanCompiler_nativeCompile(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    sql: JString<'_>,
    context_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        // Get SQL string from Java
        let sql_str: String = env.get_string(&sql)?.into();

        // 1. Parse SQL -> AST
        let parser = Parser::default();
        let parsed = match parser.parse(&sql_str) {
            Ok(parsed) => parsed,
            Err(parse_err) => {
                let err_msg = format!("Parse error: {:?}", parse_err);
                return Err(crate::error::JniError::EngineError(
                    partiql_eval::EngineError::InvalidPlan(err_msg),
                ));
            }
        };

        // 2. AST -> LogicalPlan
        let catalog = PartiqlCatalog::default();
        let shared_catalog = catalog.to_shared_catalog();
        let logical_planner = LogicalPlanner::new(&shared_catalog);
        let logical = match logical_planner.lower(&parsed) {
            Ok(plan) => plan,
            Err(lower_err) => {
                let err_msg = format!("Lowering error: {:?}", lower_err);
                return Err(crate::error::JniError::EngineError(
                    partiql_eval::EngineError::InvalidPlan(err_msg),
                ));
            }
        };

        // 3. LogicalPlan -> CompiledPlan
        // Get CompilationContext from handle (wrapped in Arc)
        let compilation_context = crate::context::get_compilation_context(context_handle as u64)?;

        let mut plan_compiler = PlanCompiler::new(&compilation_context);
        let compiled = plan_compiler.compile(&logical)?;

        Ok(create_plan_handle(compiled) as jlong)
    })
}
