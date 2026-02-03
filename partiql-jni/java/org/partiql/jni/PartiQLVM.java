package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;

/**
 * The PartiQL Virtual Machine for executing compiled query plans.
 * 
 * <p>A PartiQLVM executes CompiledPlans and maintains execution state including:
 * <ul>
 *   <li>Memory arena for value storage</li>
 *   <li>Register array for intermediate results</li>
 *   <li>Execution context</li>
 * </ul>
 * 
 * <p>Each VM instance is single-threaded and should not be shared across threads.
 * For concurrent execution, create multiple VM instances from the same CompiledPlan.
 * 
 * <p>This class implements AutoCloseable and should be used with try-with-resources
 * to ensure proper cleanup of native resources.
 * 
 * <p>Example usage:
 * <pre>{@code
 * PlanCompiler compiler = new PlanCompiler();
 * try (CompiledPlan plan = compiler.compile("SELECT * FROM users WHERE age > 21")) {
 *     try (PartiQLVM vm = new PartiQLVM(plan)) {
 *         ExecutionResult result = vm.execute();
 *         
 *         if (result.isQuery()) {
 *             try (QueryIterator iter = result.asQueryIterator()) {
 *                 while (iter.hasNext()) {
 *                     RegisterReader row = iter.next();
 *                     // Process row...
 *                 }
 *             }
 *         }
 *     }
 * }
 * }</pre>
 */
public final class PartiQLVM implements AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native long nativeNew(long planHandle, long contextHandle) throws PartiQLException;
    private static native long nativeExecute(long handle) throws PartiQLException;
    private static native void nativeLoadPlan(long handle, long planHandle) throws PartiQLException;
    private static native void nativeClose(long handle);
    
    /**
     * Creates a new PartiQLVM from a compiled plan with an execution context.
     * 
     * @param plan The compiled plan to execute
     * @param context ExecutionContext with catalog mappings for data access
     * @throws PartiQLException if VM creation fails
     * @throws NullPointerException if plan or context is null
     */
    public PartiQLVM(CompiledPlan plan, ExecutionContext context) throws PartiQLException {
        if (plan == null) {
            throw new NullPointerException("CompiledPlan cannot be null");
        }
        if (context == null) {
            throw new NullPointerException("ExecutionContext cannot be null");
        }
        
        this.nativeHandle = nativeNew(plan.getNativeHandle(), context.getNativeHandle());
    }
    
    /**
     * Executes the currently loaded plan.
     * 
     * <p>The execution result can be:
     * <ul>
     *   <li>A query result (rows of data)</li>
     *   <li>A mutation result (INSERT/UPDATE/DELETE summary)</li>
     *   <li>A definition result (CREATE/DROP summary)</li>
     * </ul>
     * 
     * @return ExecutionResult containing the query results or operation summary
     * @throws PartiQLException if execution fails
     * @throws IllegalStateException if the VM has been closed or has an active iterator
     */
    public ExecutionResult execute() throws PartiQLException {
        checkNotClosed();
        long resultHandle = nativeExecute(nativeHandle);
        return new ExecutionResult(resultHandle);
    }
    
    /**
     * Loads a new plan for execution.
     * 
     * <p>This allows reusing the same VM instance with different plans,
     * avoiding the overhead of creating new VM instances.
     * 
     * <p>The VM must not have an active iterator when loading a new plan.
     * 
     * @param plan The new compiled plan to load
     * @throws PartiQLException if plan loading fails
     * @throws IllegalStateException if the VM has been closed or has an active iterator
     * @throws NullPointerException if plan is null
     */
    public void loadPlan(CompiledPlan plan) throws PartiQLException {
        checkNotClosed();
        if (plan == null) {
            throw new NullPointerException("CompiledPlan cannot be null");
        }
        
        nativeLoadPlan(nativeHandle, plan.getNativeHandle());
    }
    
    /**
     * Closes this VM and releases native resources.
     * 
     * <p>After calling close(), any attempt to use this VM will throw
     * an IllegalStateException.
     * 
     * <p>It is safe to call close() multiple times.
     */
    @Override
    public void close() {
        if (!closed) {
            nativeClose(nativeHandle);
            closed = true;
            nativeHandle = 0;
        }
    }
    
    /**
     * Checks if this VM has been closed.
     * 
     * @throws IllegalStateException if the VM has been closed
     */
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("PartiQLVM has been closed");
        }
    }
}
