package org.partiql.jni;


/**
 * Registry that maps catalog names to CompilationCatalog instances.
 * 
 * Used during compilation to resolve catalog names in queries to actual
 * CompilationCatalog instances. Returns CatalogIds that can be used to set up
 * ExecutionContext for execution-time catalog resolution.
 * 
 * Matches the Rust CompilationContext struct.
 */
public final class CompilationContext implements AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    public CompilationContext() {
        this.nativeHandle = nativeNew();
    }
    
    /**
     * Add a catalog with the given name.
     * 
     * The returned CatalogId should be used when setting up ExecutionContext
     * to map the same catalog ID to an ExecutionCatalog instance.
     * 
     * If a catalog with this name already exists, it will be replaced and
     * a new CatalogId will be generated.
     * 
     * @param name Catalog name for query resolution (e.g., "main", "external")
     * @param catalog CompilationCatalog implementation
     * @return CatalogId for use in ExecutionContext
     */
    public long addCatalog(String name, CompilationCatalog catalog) {
        checkNotClosed();
        if (name == null) {
            throw new IllegalArgumentException("Catalog name cannot be null");
        }
        if (catalog == null) {
            throw new IllegalArgumentException("CompilationCatalog cannot be null");
        }
        
        // Register catalog in Rust and get back the catalog ID
        // The catalog object is stored in Rust via GlobalRef
        return nativeAddCatalog(nativeHandle, name, catalog);
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
            throw new IllegalStateException("CompilationContext already closed");
        }
    }
    
    // Native methods
    private static native long nativeNew();
    private static native long nativeAddCatalog(long handle, String name, CompilationCatalog catalog);
    private static native void nativeClose(long handle);
}
