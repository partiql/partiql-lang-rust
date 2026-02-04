package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;

/**
 * Java mirror of Rust's RegisterReader for accessing row data.
 * This class provides direct access to column values without conversion overhead.
 * 
 * Note: RegisterReader instances are only valid while their parent QueryIterator
 * is positioned on a row (between hasNext() returning true and next() being called).
 */
public final class RegisterReader {
    private final long iteratorHandle;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    // Native methods
    private static native Long nativeGetI64(long iteratorHandle, int col) throws PartiQLException;
    private static native String nativeGetStr(long iteratorHandle, int col) throws PartiQLException;
    private static native Value nativeGetValue(long iteratorHandle, int col) throws PartiQLException;
    
    /**
     * Package-private constructor - RegisterReaders are created by QueryIterator
     */
    RegisterReader(long iteratorHandle) {
        this.iteratorHandle = iteratorHandle;
    }
    
    /**
     * Get an i64 value from the specified column.
     * 
     * @param col Column index (0-based)
     * @return The i64 value, or null if the column doesn't contain an i64
     * @throws PartiQLException if an error occurs
     */
    public Long getI64(int col) throws PartiQLException {
        return nativeGetI64(iteratorHandle, col);
    }
    
    /**
     * Get a string value from the specified column.
     * 
     * @param col Column index (0-based)
     * @return The string value, or null if the column doesn't contain a string
     * @throws PartiQLException if an error occurs
     */
    public String getStr(int col) throws PartiQLException {
        return nativeGetStr(iteratorHandle, col);
    }
    
    /**
     * Get a generic value from the specified column.
     * This returns a ValueOwned representation that can be any PartiQL type.
     * 
     * @param col Column index (0-based)
     * @return The value (never null - returns Missing if column is out of bounds)
     * @throws PartiQLException if an error occurs
     */
    public Value getValue(int col) throws PartiQLException {
        return nativeGetValue(iteratorHandle, col);
    }
}
