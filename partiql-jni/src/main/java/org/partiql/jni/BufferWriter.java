package org.partiql.jni;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;

/**
 * Provides write access to a ByteBuffer for populating BufferedExecutionCatalog data.
 * 
 * This class enables users to write rows of values directly to a buffer using the same
 * encoding format as the Rust RegisterWriter, but without any JNI overhead.
 * 
 * <h2>Row-Oriented Format</h2>
 * <p>Data is organized into rows, each delimited by a row-end marker. Within each row,
 * slot entries map a register slot index to a value (scalar or collection).</p>
 * 
 * <pre>{@code
 * writer.beginRow();
 * writer.putLong(0, 42);
 * writer.putString(1, "Alice");
 * writer.endRow();
 * 
 * writer.beginRow();
 * writer.putLong(0, 43);
 * writer.putString(1, "Bob");
 * writer.endRow();
 * }</pre>
 * 
 * <h2>Collection Support</h2>
 * <p>Use {@link #valueWriter(int)} to create a {@link BufferValueWriter} for writing
 * structured values (tuples, lists, bags) into a slot:</p>
 * 
 * <pre>{@code
 * writer.beginRow();
 * writer.putLong(0, 1);
 * 
 * BufferValueWriter vw = writer.valueWriter(1);
 * vw.stepInList();
 * vw.putI64(95);
 * vw.putI64(87);
 * vw.stepOut();
 * 
 * writer.endRow();
 * }</pre>
 * 
 * <p>BufferWriter instances are provided by BufferedExecutionCatalog during data
 * population and should not be created directly.</p>
 */
public final class BufferWriter {
    ByteBuffer buffer;
    final BufferedMemoryPool pool;
    
    // =========================================================================
    // Type tags for scalar encoding (must match Rust decoder)
    // =========================================================================
    static final byte TYPE_NULL = 0;
    static final byte TYPE_MISSING = 1;
    static final byte TYPE_BOOL = 2;
    static final byte TYPE_I64 = 3;
    static final byte TYPE_F64 = 4;
    static final byte TYPE_STRING = 5;
    
    // =========================================================================
    // Container type tags (must match Rust decoder)
    // =========================================================================
    static final byte TYPE_TUPLE = 6;
    static final byte TYPE_LIST = 7;
    static final byte TYPE_BAG = 8;
    
    // =========================================================================
    // Structural markers (must match Rust decoder)
    // =========================================================================
    static final byte MARKER_FIELD_NAME = 0x10;
    static final byte MARKER_CONTAINER_END = 0x11;
    static final byte MARKER_ROW_END = 0x12;
    
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
    
    // =========================================================================
    // Row boundaries
    // =========================================================================
    
    /**
     * Begin a new row.
     * 
     * <p>This is a logical marker. All {@code put*} calls between {@code beginRow()}
     * and {@code endRow()} belong to the same row. The Rust decoder will deliver
     * these as one {@code next_row()} call.</p>
     */
    public void beginRow() {
        // No wire bytes needed — the row starts implicitly.
        // This method exists for API clarity and future validation.
    }
    
    /**
     * End the current row by writing a row-end marker.
     * 
     * <p>The Rust decoder reads fields until it encounters this marker,
     * then returns the row.</p>
     */
    public void endRow() {
        ensureCapacity(1);
        buffer.put(MARKER_ROW_END);
    }
    
    // =========================================================================
    // Scalar puts — slot-targeted
    // =========================================================================
    
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
    
    // =========================================================================
    // Complex value construction
    // =========================================================================
    
    /**
     * Create a {@link BufferValueWriter} for constructing a complex value
     * (tuple, list, bag) in the specified register slot.
     * 
     * <p>The returned writer writes the slot header and then delegates
     * container/scalar content directly into this buffer. Call
     * {@code stepInTuple()}, {@code stepInList()}, or {@code stepInBag()}
     * on the returned writer to begin building the value.</p>
     * 
     * <p><b>Important:</b> You must complete the value writer (close all
     * containers via {@code stepOut()}) before writing additional slots
     * or calling {@code endRow()}.</p>
     * 
     * @param slot Register slot index (0-based)
     * @return A BufferValueWriter for constructing the value
     */
    public BufferValueWriter valueWriter(int slot) {
        // Write the slot header now; the type tag will be written by stepIn*
        ensureCapacity(2); // slot(2)
        buffer.putShort((short) slot);
        return new BufferValueWriter(this);
    }
    
    // =========================================================================
    // Buffer introspection
    // =========================================================================
    
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
    
    // =========================================================================
    // Internal — raw write helpers used by BufferValueWriter
    // =========================================================================
    
    /** Write a raw byte to the buffer. */
    void writeByte(byte b) {
        ensureCapacity(1);
        buffer.put(b);
    }
    
    /** Write a raw int (4 bytes) to the buffer. */
    void writeInt(int v) {
        ensureCapacity(4);
        buffer.putInt(v);
    }
    
    /** Write a raw long (8 bytes) to the buffer. */
    void writeLong(long v) {
        ensureCapacity(8);
        buffer.putLong(v);
    }
    
    /** Write a raw double (8 bytes) to the buffer. */
    void writeDouble(double v) {
        ensureCapacity(8);
        buffer.putDouble(v);
    }
    
    /** Write raw bytes to the buffer. */
    void writeBytes(byte[] bytes) {
        ensureCapacity(bytes.length);
        buffer.put(bytes);
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
    void ensureCapacity(int bytes) {
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