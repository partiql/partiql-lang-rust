package org.partiql.jni;

import org.partiql.jni.RegisterReader;
import org.partiql.jni.exceptions.PartiQLException;

/**
 * Interface for reading data during query execution.
 * 
 * Provides row-by-row data access with projection support.
 * Matches the Rust DataSource trait.
 */
public interface DataSource extends AutoCloseable {
    /**
     * Get the next row.
     * 
     * @return RegisterReader for the next row, or null when no more rows
     * @throws PartiQLException if an error occurs during reading
     */
    RegisterReader next() throws PartiQLException;
    
    /**
     * Check if more rows are available.
     * 
     * @return true if next() will return a row, false otherwise
     */
    boolean hasNext();
    
    /**
     * Close the data source and release resources.
     */
    @Override
    void close();
}
