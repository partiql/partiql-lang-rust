package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;
import java.util.Iterator;
import java.util.NoSuchElementException;

/**
 * Iterator over query result rows, providing access via RegisterReader.
 * 
 * <p>QueryIterator implements the standard Java {@link Iterator} pattern,
 * returning a {@link RegisterReader} for each row that provides direct
 * access to column values.
 * 
 * <p>This class implements AutoCloseable and should be used with try-with-resources
 * to ensure proper cleanup of native resources.
 * 
 * <p>Example usage:
 * <pre>{@code
 * try (ExecutionResult result = vm.execute()) {
 *     if (result.isQuery()) {
 *         try (QueryIterator iter = result.asQueryIterator()) {
 *             while (iter.hasNext()) {
 *                 RegisterReader row = iter.next();
 *                 Long id = row.getI64(0);
 *                 String name = row.getStr(1);
 *                 Integer age = row.getI64(2).intValue();
 *                 System.out.println("User: " + name + " (ID: " + id + ", Age: " + age + ")");
 *             }
 *         }
 *     }
 * }
 * }</pre>
 * 
 * <p><b>Important</b>: Each RegisterReader returned by {@link #next()} is only
 * valid until the next call to {@link #next()} or {@link #close()}. If you need
 * to retain values beyond that point, copy them to Java objects.
 */
public final class QueryIterator implements Iterator<RegisterReader>, AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    private Boolean hasNextCache = null;  // Cache for hasNext result
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native int nativeNext(long handle) throws PartiQLException;
    private static native void nativeClose(long handle);
    
    /**
     * Package-private constructor. QueryIterators are created by ExecutionResult.
     * 
     * @param nativeHandle The native handle to the query iterator
     */
    QueryIterator(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }
    
    /**
     * Returns {@code true} if the iteration has more rows.
     * 
     * <p>This method can be called multiple times without advancing the iterator.
     * The result is cached until {@link #next()} is called.
     * 
     * @return {@code true} if there are more rows, {@code false} otherwise
     * @throws RuntimeException if an error occurs checking for more rows
     * @throws IllegalStateException if the iterator has been closed
     */
    @Override
    public boolean hasNext() {
        checkNotClosed();
        
        // If we haven't checked yet, advance to the next row
        if (hasNextCache == null) {
            try {
                int status = nativeNext(nativeHandle);
                hasNextCache = (status != 0);
            } catch (PartiQLException e) {
                // Wrap checked exception as unchecked for Iterator interface
                throw new RuntimeException("Error checking for next row", e);
            }
        }
        
        return hasNextCache;
    }
    
    /**
     * Returns the next row as a RegisterReader.
     * 
     * <p>The returned RegisterReader provides direct access to the current row's
     * column values. It remains valid only until the next call to {@link #next()}
     * or {@link #close()}.
     * 
     * <p>If you need to retain values beyond that point, extract and copy them:
     * <pre>{@code
     * RegisterReader row = iter.next();
     * Long id = row.getI64(0);        // Copy the value
     * String name = row.getStr(1);    // Copy the value
     * // Now id and name are safe to use after next() is called
     * }</pre>
     * 
     * @return A RegisterReader for accessing the current row's columns
     * @throws NoSuchElementException if there are no more rows
     * @throws IllegalStateException if the iterator has been closed
     */
    @Override
    public RegisterReader next() {
        if (!hasNext()) {
            throw new NoSuchElementException("No more rows in query result");
        }
        
        // Clear the cache so the next hasNext() call will advance
        hasNextCache = null;
        
        // Return a RegisterReader that accesses the current row via the iterator handle
        return new RegisterReader(nativeHandle);
    }
    
    /**
     * Closes this iterator and releases native resources.
     * 
     * <p>After calling close(), any attempt to use this iterator will throw
     * an IllegalStateException.
     * 
     * <p>It is safe to call close() multiple times.
     */
    @Override
    public void close() {
        if (!closed) {
            nativeClose(nativeHandle);
            closed = true;
            nativeHandle = 0;
            hasNextCache = null;
        }
    }
    
    /**
     * Checks if this iterator has been closed.
     * 
     * @throws IllegalStateException if the iterator has been closed
     */
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("QueryIterator has been closed");
        }
    }
}
