package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;


/**
 * Represents the result of executing a PartiQL statement.
 * 
 * <p>An ExecutionResult can contain one of three types of results:
 * <ul>
 *   <li><b>Query Result</b>: Rows of data from a SELECT statement</li>
 *   <li><b>Mutation Result</b>: Summary of INSERT/UPDATE/DELETE operations</li>
 *   <li><b>Definition Result</b>: Summary of CREATE/DROP operations</li>
 * </ul>
 * 
 * <p>Use {@link #isQuery()} to determine the result type, then call
 * {@link #asQueryIterator()} to access the rows.
 * 
 * <p>This class implements AutoCloseable and should be used with try-with-resources
 * to ensure proper cleanup of native resources.
 * 
 * <p>Example usage:
 * <pre>{@code
 * try (PartiQLVM vm = new PartiQLVM(plan)) {
 *     try (ExecutionResult result = vm.execute()) {
 *         if (result.isQuery()) {
 *             try (QueryIterator iter = result.asQueryIterator()) {
 *                 while (iter.hasNext()) {
 *                     RegisterReader row = iter.next();
 *                     Long id = row.getI64(0);
 *                     String name = row.getStr(1);
 *                     // Process row...
 *                 }
 *             }
 *         }
 *     }
 * }
 * }</pre>
 */
public final class ExecutionResult implements AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    // Internal memory pool passed from PartiQLVM for buffer reuse
    private final CrossLanguageMemoryPool memoryPool;
    
    // VM handle for buffer caching
    private final long vmHandle;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native boolean nativeIsQuery(long handle) throws PartiQLException;
    private static native long nativeAsQueryIterator(long handle, long vmHandle) throws PartiQLException;
    private static native void nativeClose(long handle);
    
    /**
     * Package-private constructor. ExecutionResults are created by PartiQLVM.
     * 
     * @param nativeHandle The native handle to the execution result
     * @param memoryPool The internal memory pool for buffer reuse
     * @param vmHandle The VM handle for buffer caching
     */
    ExecutionResult(long nativeHandle, CrossLanguageMemoryPool memoryPool, long vmHandle) {
        this.nativeHandle = nativeHandle;
        this.memoryPool = memoryPool;
        this.vmHandle = vmHandle;
    }
    
    /**
     * Checks if this result is a query result (contains rows of data).
     * 
     * <p>If true, you can call {@link #asQueryIterator()} to access the rows.
     * If false, this is either a mutation result or a definition result.
     * 
     * @return true if this is a query result, false otherwise
     * @throws PartiQLException if an error occurs checking the result type
     * @throws IllegalStateException if the result has been closed
     */
    public boolean isQuery() throws PartiQLException {
        checkNotClosed();
        return nativeIsQuery(nativeHandle);
    }
    
    /**
     * Converts this result into a QueryIterator for accessing rows.
     * 
     * <p>This method can only be called if {@link #isQuery()} returns true.
     * 
     * <p>The returned QueryIterator takes ownership of the iteration state
     * and must be closed when done. After calling this method, this
     * ExecutionResult should not be used further (except to close it).
     * 
     * @return A QueryIterator for accessing the query results
     * @throws PartiQLException if this is not a query result or iterator creation fails
     * @throws IllegalStateException if the result has been closed
     */
    public QueryIterator asQueryIterator() throws PartiQLException {
        checkNotClosed();
        long iterHandle = nativeAsQueryIterator(nativeHandle, vmHandle);
        // Pass internal memory pool to iterator for buffer reuse
        return new QueryIterator(iterHandle, memoryPool);
    }
    
    /**
     * Closes this result and releases native resources.
     * 
     * <p>After calling close(), any attempt to use this result will throw
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
     * Checks if this result has been closed.
     * 
     * @throws IllegalStateException if the result has been closed
     */
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("ExecutionResult has been closed");
        }
    }
}
