package org.partiql.jni;


/**
 * Registry that maps CatalogIds to BufferedExecutionCatalog instances.
 * 
 * Used during query execution to resolve CatalogIds (from the compiled plan)
 * to BufferedExecutionCatalog instances that provide access to actual data.
 * 
 * ExecutionContext can be created per-thread to provide different datasets
 * for the same compiled plan. Each thread maintains its own catalog mappings.
 * 
 * Matches the Rust ExecutionContext struct.
 */
public final class ExecutionContext implements AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    public ExecutionContext() {
        this.nativeHandle = nativeNew();
    }
    
    /**
     * Add a buffered execution catalog with the given CatalogId.
     * 
     * BufferedExecutionCatalogs provide optimal performance by pre-populating
     * all data in a DirectByteBuffer at registration time, eliminating
     * Rust-to-Java callback overhead during query execution.
     * 
     * The CatalogId should match the one returned by CompilationContext.addCatalog()
     * during compilation.
     * 
     * If a catalog with this ID already exists, it will be replaced.
     * 
     * @param catalogId CatalogId from CompilationContext.addCatalog()
     * @param catalog BufferedExecutionCatalog with pre-populated data
     */
    public void addBufferedCatalog(long catalogId, BufferedExecutionCatalog catalog) {
        checkNotClosed();
        if (catalog == null) {
            throw new IllegalArgumentException("BufferedExecutionCatalog cannot be null");
        }
        
        // Get buffer and its ID from the pool for caching in Rust
        java.nio.ByteBuffer buffer = catalog.getBuffer();
        BufferedMemoryPool pool = catalog.getPool();
        int bufferId = pool.getBufferId(buffer);
        
        // Pass catalog data to Rust for registration
        // Buffer ID enables Rust-side caching of address/capacity
        nativeAddBufferedCatalog(nativeHandle, catalogId, catalog.getEntryId(), buffer, buffer.limit(), bufferId);
    }
    
    @Override
    public void close() {
        if (!closed) {
            nativeClose(nativeHandle);
            closed = true;
        }
    }
    
    long getNativeHandle() { 
        return nativeHandle; 
    }
    
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("ExecutionContext already closed");
        }
    }
    
    // Native methods
    private static native long nativeNew();
    private static native void nativeAddBufferedCatalog(long handle, long catalogId, long entryId, java.nio.ByteBuffer buffer, int bufferSize, int bufferId);
    private static native void nativeClose(long handle);
}
