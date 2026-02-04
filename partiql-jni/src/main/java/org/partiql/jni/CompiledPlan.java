package org.partiql.jni;



/**
 * Represents a compiled PartiQL query plan.
 * 
 * <p>A CompiledPlan is the result of compiling a PartiQL query and can be
 * executed multiple times by creating PartiQLVM instances from it.
 * 
 * <p>CompiledPlans are thread-safe and can be shared across multiple threads.
 * Each thread should create its own PartiQLVM instance from the shared plan.
 * 
 * <p>This class implements AutoCloseable and should be used with try-with-resources
 * to ensure proper cleanup of native resources.
 * 
 * <p>Example usage:
 * <pre>{@code
 * PlanCompiler compiler = new PlanCompiler();
 * try (CompiledPlan plan = compiler.compile("SELECT * FROM users")) {
 *     // Plan can be reused across multiple VM instances
 *     try (PartiQLVM vm = new PartiQLVM(plan)) {
 *         ExecutionResult result = vm.execute();
 *         // Process result...
 *     }
 * }
 * }</pre>
 */
public final class CompiledPlan implements AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native void nativeClose(long handle);
    
    /**
     * Package-private constructor. CompiledPlans are created by PlanCompiler.
     * 
     * @param nativeHandle The native handle to the compiled plan
     */
    CompiledPlan(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }
    
    /**
     * Gets the native handle for this compiled plan.
     * Package-private - used by PartiQLVM for construction.
     * 
     * @return The native handle
     * @throws IllegalStateException if the plan has been closed
     */
    long getNativeHandle() {
        checkNotClosed();
        return nativeHandle;
    }
    
    /**
     * Closes this compiled plan and releases native resources.
     * 
     * <p>After calling close(), any attempt to use this plan will throw
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
     * Checks if this plan has been closed.
     * 
     * @throws IllegalStateException if the plan has been closed
     */
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("CompiledPlan has been closed");
        }
    }
}
