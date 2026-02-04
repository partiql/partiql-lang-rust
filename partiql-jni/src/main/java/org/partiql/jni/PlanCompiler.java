package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;
import org.partiql.jni.exceptions.PlanningException;

/**
 * Compiles PartiQL queries into executable plans.
 * 
 * <p>The PlanCompiler translates SQL text through the complete compilation pipeline:
 * <ol>
 *   <li>SQL text → Abstract Syntax Tree (AST)</li>
 *   <li>AST → Logical Plan</li>
 *   <li>Logical Plan → Compiled Plan</li>
 * </ol>
 * 
 * <p>The resulting CompiledPlan can be executed multiple times and is thread-safe.
 * 
 * <p>Example usage:
 * <pre>{@code
 * PlanCompiler compiler = new PlanCompiler();
 * try (CompiledPlan plan = compiler.compile("SELECT id, name FROM users WHERE age > 21")) {
 *     try (PartiQLVM vm = new PartiQLVM(plan)) {
 *         ExecutionResult result = vm.execute();
 *         // Process result...
 *     }
 * }
 * }</pre>
 */
public final class PlanCompiler {
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native long nativeCompile(String sql, long contextHandle) throws PartiQLException;
    
    /**
     * Creates a new PlanCompiler instance.
     */
    public PlanCompiler() {
        // No-op constructor - stateless for now
    }
    
    /**
     * Compiles a PartiQL query string into an executable plan.
     * 
     * <p>The compilation process includes:
     * <ul>
     *   <li>Parsing the SQL text into an AST</li>
     *   <li>Lowering the AST into a logical plan</li>
     *   <li>Compiling the logical plan into an executable plan</li>
     * </ul>
     * 
     * @param sql The PartiQL query string to compile
     * @param context CompilationContext with registered catalogs for table resolution
     * @return A CompiledPlan that can be executed
     * @throws PartiQLException if compilation fails due to syntax errors,
     *         semantic errors, or other compilation issues
     * @throws PlanningException if the query cannot be planned
     * @throws NullPointerException if sql or context is null
     */
    public CompiledPlan compile(String sql, CompilationContext context) throws PartiQLException {
        if (sql == null) {
            throw new NullPointerException("SQL string cannot be null");
        }
        if (context == null) {
            throw new NullPointerException("CompilationContext cannot be null");
        }
        
        long planHandle = nativeCompile(sql, context.getNativeHandle());
        return new CompiledPlan(planHandle);
    }
}
