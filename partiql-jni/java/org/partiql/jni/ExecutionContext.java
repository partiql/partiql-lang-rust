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
        
        // Register catalog ID in Rust
        nativeAddCatalog(nativeHandle, catalogId);
        
        // Store Java catalog reference for callbacks
        registerCatalogCallback(catalogId, catalog);
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
    private static native void nativeAddCatalog(long handle, long catalogId);
    private static native void nativeClose(long handle);
    private native void registerCatalogCallback(long catalogId, ExecutionCatalog catalog);
}
