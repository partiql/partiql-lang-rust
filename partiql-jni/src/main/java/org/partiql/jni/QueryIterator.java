package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;
import java.nio.ByteBuffer;
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
    
    // Internal memory pool for buffer reuse (passed from PartiQLVM)
    private final CrossLanguageMemoryPool memoryPool;
    
    // Reusable buffer for zero-copy row access (obtained from pool)
    private ByteBuffer rowBuffer;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native int nativeNextToBuffer(long handle, ByteBuffer buffer, int bufferId) throws PartiQLException;
    private static native void nativeClose(long handle);
    
    /**
     * Package-private constructor. QueryIterators are created by ExecutionResult.
     * 
     * @param nativeHandle The native handle to the query iterator
     * @param memoryPool The internal memory pool for buffer reuse
     */
    QueryIterator(long nativeHandle, CrossLanguageMemoryPool memoryPool) {
        this.nativeHandle = nativeHandle;
        this.memoryPool = memoryPool;
        
        // Checkout a buffer from the pool instead of allocating
        this.rowBuffer = memoryPool.checkout();
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
        
        // If we haven't checked yet, advance to the next row and write to buffer
        if (hasNextCache == null) {
            try {
                // Clear buffer for new row
                rowBuffer.clear();
                
                // Get buffer ID for caching
                int bufferId = memoryPool.getBufferId(rowBuffer);
                
                // Rust writes row to buffer and returns status
                // Returns: bytes written (>0) if has next, 0 if no more rows, -1 if buffer too small
                int bytesWritten = nativeNextToBuffer(nativeHandle, rowBuffer, bufferId);
                
                if (bytesWritten < 0) {
                    // Buffer was too small, need to grow it
                    // Double the buffer size and retry
                    int requiredSize = rowBuffer.capacity() * 2;
                    rowBuffer = memoryPool.ensureCapacity(rowBuffer, requiredSize);
                    
                    // Retry with larger buffer (get new buffer ID)
                    rowBuffer.clear();
                    bufferId = memoryPool.getBufferId(rowBuffer);
                    bytesWritten = nativeNextToBuffer(nativeHandle, rowBuffer, bufferId);
                    
                    if (bytesWritten < 0) {
                        // Still too small - this shouldn't happen with doubling
                        throw new RuntimeException("Row buffer growth failed after resize");
                    }
                }
                
                hasNextCache = (bytesWritten > 0);
                
                if (hasNextCache) {
                    // Set limit to the bytes written for RegisterReader to read
                    rowBuffer.limit(bytesWritten);
                    rowBuffer.position(0);
                }
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
        
        // Return a RegisterReader that reads from the buffer (zero-copy!)
        // Note: rowBuffer position is at 0, limit is set to bytes written
        try {
            return new RegisterReader(rowBuffer);
        } catch (PartiQLException e) {
            // Wrap checked exception as unchecked for Iterator interface
            throw new RuntimeException("Error creating RegisterReader", e);
        }
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
            
            // Return buffer to pool for reuse
            if (rowBuffer != null) {
                memoryPool.returnBuffer(rowBuffer);
                rowBuffer = null;
            }
            
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
