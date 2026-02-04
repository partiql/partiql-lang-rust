package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;

/**
 * Interface for reading data during query execution.
 * 
 * Provides row-by-row data access with projection support.
 * Matches the Rust DataSource trait.
 * 
 * Custom DataSource implementations write values directly to VM registers
 * using the RegisterWriter for optimal performance.
 */
public interface DataSource extends AutoCloseable {
    /**
     * Initialize the data source for reading.
     * Called once before the first row is requested.
     * 
     * @throws PartiQLException if an error occurs during initialization
     */
    void open() throws PartiQLException;
    
    /**
     * Fetch the next row and write values to registers.
     * 
     * The implementation should:
     * 1. Check if more rows are available
     * 2. For each projection in the ScanLayout, write the appropriate value
     *    to the target register slot using RegisterWriter methods
     * 3. Return true if a row was written, false if no more rows
     * 
     * @param writer RegisterWriter for writing values to VM registers
     * @return true if a row was written, false if no more rows
     * @throws PartiQLException if an error occurs during reading
     */
    boolean nextRow(RegisterWriter writer) throws PartiQLException;
    
    /**
     * Close the data source and release resources.
     */
    @Override
    void close();
}
