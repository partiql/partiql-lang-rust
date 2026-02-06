package org.partiql.jni;


/**
 * Registry that maps CatalogIds to ExecutionCatalog instances.
 * 
 * Used during query execution to resolve CatalogIds (from the compiled plan)
 * to ExecutionCatalog instances that provide access to actual data.
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
     * Add an execution catalog with the given CatalogId.
     * 
     * The CatalogId should match the one returned by CompilationContext.addCatalog()
     * during compilation. This allows the execution context to resolve the same
     * logical catalog to a different physical dataset.
     * 
     * If a catalog with this ID already exists, it will be replaced.
     * 
     * @param catalogId CatalogId from CompilationContext.addCatalog()
     * @param catalog ExecutionCatalog implementation
     */
    public void addCatalog(long catalogId, ExecutionCatalog catalog) {
        checkNotClosed();
        if (catalog == null) {
            throw new IllegalArgumentException("ExecutionCatalog cannot be null");
        }
        
        // Pass catalog object to Rust for registration
        nativeAddCatalog(nativeHandle, catalogId, catalog);
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
        
        // Pass catalog data to Rust for registration
        // Pass buffer size directly to avoid extra JNI call
        java.nio.ByteBuffer buffer = catalog.getBuffer();
        nativeAddBufferedCatalog(nativeHandle, catalogId, catalog.getEntryId(), buffer, buffer.limit());
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
    private static native void nativeAddCatalog(long handle, long catalogId, ExecutionCatalog catalog);
    private static native void nativeAddBufferedCatalog(long handle, long catalogId, long entryId, java.nio.ByteBuffer buffer, int bufferSize);
    private static native void nativeClose(long handle);
}
