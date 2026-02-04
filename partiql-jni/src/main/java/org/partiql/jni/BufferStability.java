package org.partiql.jni;

/**
 * Indicates how long a DataSource's buffer remains valid.
 * 
 * Matches the Rust BufferStability enum.
 */
public enum BufferStability {
    /**
     * Buffer is valid only until the next next() call.
     * Data must be copied if needed beyond that point.
     */
    UNTIL_NEXT,
    
    /**
     * Buffer is valid until the iterator is closed.
     * Data can be referenced without copying.
     */
    UNTIL_CLOSE
}
