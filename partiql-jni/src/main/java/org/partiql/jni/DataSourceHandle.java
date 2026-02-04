package org.partiql.jni;

/**
 * Handle for a data source with compile-time metadata.
 * 
 * Contains:
 * - EntryId for execution-time resolution
 * - DataSourceConfig for compile-time metadata
 * 
 * Matches the Rust DataSourceHandle struct (catalog variant).
 */
public final class DataSourceHandle {
    private final long entryId;
    private final DataSourceConfig config;
    
    public DataSourceHandle(long entryId, DataSourceConfig config) {
        if (config == null) {
            throw new IllegalArgumentException("DataSourceConfig cannot be null");
        }
        this.entryId = entryId;
        this.config = config;
    }
    
    /**
     * Returns the entry ID for execution-time resolution.
     */
    public long getEntryId() { 
        return entryId; 
    }
    
    /**
     * Returns the compile-time configuration.
     */
    public DataSourceConfig getConfig() { 
        return config; 
    }
    
    @Override
    public String toString() {
        return String.format("DataSourceHandle{entryId=%d, config=%s}", 
            entryId, config);
    }
}
