package org.partiql.jni;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;

/**
 * Provides write access to a ByteBuffer for populating BufferedExecutionCatalog data.
 * 
 * This class enables users to write values directly to a buffer using the same
 * encoding format as RegisterWriter, but without any JNI overhead.
 * 
 * BufferWriter instances are provided by BufferedExecutionCatalog during data
 * population and should not be created directly.
 */
public final class BufferWriter {
    private ByteBuffer buffer;
    private final BufferedMemoryPool pool;
    
    // Type tags for encoding (must match RegisterWriter and Rust decoder)
    static final byte TYPE_NULL = 0;
    static final byte TYPE_MISSING = 1;
    static final byte TYPE_BOOL = 2;
    static final byte TYPE_I64 = 3;
    static final byte TYPE_F64 = 4;
    static final byte TYPE_STRING = 5;
    
    /**
     * Package-private constructor for non-pooled usage.
     * BufferWriter instances are created by BufferedExecutionCatalog.
     * 
     * @param buffer The ByteBuffer to write to
     */
    BufferWriter(ByteBuffer buffer) {
        this.buffer = buffer;
        this.pool = null;
    }
    
    /**
     * Package-private constructor for pooled usage with auto-growth support.
     * 
     * @param buffer The ByteBuffer to write to
     * @param pool The memory pool for buffer growth
     */
    BufferWriter(ByteBuffer buffer, BufferedMemoryPool pool) {
        this.buffer = buffer;
        this.pool = pool;
    }
    
    /**
     * Write a NULL value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     */
    public void putNull(int slot) {
        ensureCapacity(3); // slot(2) + type(1)
        buffer.putShort((short) slot);
        buffer.put(TYPE_NULL);
    }
    
    /**
     * Write a MISSING value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     */
    public void putMissing(int slot) {
        ensureCapacity(3); // slot(2) + type(1)
        buffer.putShort((short) slot);
        buffer.put(TYPE_MISSING);
    }
    
    /**
     * Write a boolean value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The boolean value to write
     */
    public void putBoolean(int slot, boolean value) {
        ensureCapacity(4); // slot(2) + type(1) + value(1)
        buffer.putShort((short) slot);
        buffer.put(TYPE_BOOL);
        buffer.put((byte) (value ? 1 : 0));
    }
    
    /**
     * Write a long (i64) value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The long value to write
     */
    public void putLong(int slot, long value) {
        ensureCapacity(11); // slot(2) + type(1) + value(8)
        buffer.putShort((short) slot);
        buffer.put(TYPE_I64);
        buffer.putLong(value);
    }
    
    /**
     * Write a double (f64) value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The double value to write
     */
    public void putDouble(int slot, double value) {
        ensureCapacity(11); // slot(2) + type(1) + value(8)
        buffer.putShort((short) slot);
        buffer.put(TYPE_F64);
        buffer.putDouble(value);
    }
    
    /**
     * Write a string value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The string value to write
     */
    public void putString(int slot, String value) {
        byte[] bytes = value.getBytes(StandardCharsets.UTF_8);
        ensureCapacity(7 + bytes.length); // slot(2) + type(1) + length(4) + bytes
        buffer.putShort((short) slot);
        buffer.put(TYPE_STRING);
        buffer.putInt(bytes.length);
        buffer.put(bytes);
    }
    
    /**
     * Get the current position in the buffer.
     * Useful for debugging or monitoring buffer usage.
     * 
     * @return Current buffer position
     */
    public int position() {
        return buffer.position();
    }
    
    /**
     * Get the remaining capacity in the buffer.
     * 
     * @return Number of bytes remaining
     */
    public int remaining() {
        return buffer.remaining();
    }
    
    /**
     * Get the current buffer. May be different from the initial buffer if grown.
     * Package-private for use by BufferedExecutionCatalog.
     * 
     * @return The current ByteBuffer
     */
    ByteBuffer getBuffer() {
        return buffer;
    }
    
    /**
     * Ensure the buffer has at least the specified number of bytes remaining.
     * 
     * <p>If a memory pool is available, the buffer will automatically grow to
     * accommodate the requested size. Otherwise, an exception is thrown.
     * 
     * @param bytes Number of bytes needed
     * @throws IllegalStateException if buffer is too small and no pool is available
     */
    private void ensureCapacity(int bytes) {
        if (buffer.remaining() < bytes) {
            if (pool != null) {
                // Use pool to grow buffer
                int requiredSize = buffer.position() + bytes;
                ByteBuffer newBuffer = pool.ensureCapacity(buffer, requiredSize);
                
                if (newBuffer != buffer) {
                    // Buffer was grown - update reference
                    buffer = newBuffer;
                }
            } else {
                // No pool available - throw exception
                throw new IllegalStateException(
                    String.format("Buffer overflow: need %d bytes, only %d remaining. " +
                                 "Buffer size: %d, position: %d. " +
                                 "Consider allocating a larger buffer or using a BufferedMemoryPool for automatic growth.",
                                 bytes, buffer.remaining(), buffer.capacity(), buffer.position()));
            }
        }
    }
}
