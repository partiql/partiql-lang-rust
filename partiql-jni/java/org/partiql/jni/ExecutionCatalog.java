package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;

/**
 * Execution-time catalog that creates DataSource instances.
 * 
 * Used during query execution to:
 * - Resolve EntryIds to actual data sources
 * - Provide access to the underlying data
 * - Enable data swapping without recompilation
 * 
 * Each ExecutionCatalog instance represents a specific dataset.
 * Different ExecutionCatalogs can provide different data for the same EntryIds.
 * 
 * Matches the Rust ExecutionCatalog trait.
 */
public interface ExecutionCatalog {
    /**
     * Create a DataSource for the given entry and layout.
     * 
     * @param entryId Entry ID assigned during compilation
     * @param layout Scan layout specifying projection and hints
     * @return DataSource instance ready to read data
     * @throws PartiQLException if entry not found or creation fails
     */
    DataSource create(long entryId, ScanLayout layout) throws PartiQLException;
}
