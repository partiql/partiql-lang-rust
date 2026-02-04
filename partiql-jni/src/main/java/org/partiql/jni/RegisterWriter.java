package org.partiql.jni;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;

/**
 * Provides write access to VM registers during data source iteration.
 * 
 * This class enables custom DataSource implementations to write values
 * directly to the VM's register file with optimal performance via buffering.
 * 
 * Values are buffered internally and flushed to Rust in a single JNI call
 * when flush() is called, eliminating per-field JNI overhead.
 * 
 * RegisterWriter instances are provided by the VM during data source iteration
 * and should not be created directly.
 */
public final class RegisterWriter {
    private final long nativeHandle;
    private final ByteBuffer buffer;
    
    // Type tags for encoding (package-private for testing)
    static final byte TYPE_NULL = 0;
    static final byte TYPE_MISSING = 1;
    static final byte TYPE_BOOL = 2;
    static final byte TYPE_I64 = 3;
    static final byte TYPE_F64 = 4;
    static final byte TYPE_STRING = 5;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    // Legacy native methods (deprecated, kept for compatibility)
    private static native void nativeWriteNull(long writerHandle, int slot);
    private static native void nativeWriteBoolean(long writerHandle, int slot, boolean value);
    private static native void nativeWriteLong(long writerHandle, int slot, long value);
    private static native void nativeWriteDouble(long writerHandle, int slot, double value);
    private static native void nativeWriteString(long writerHandle, int slot, String value);
    
    // New native method for buffered writes
    private static native void nativeFlush(long writerHandle, ByteBuffer buffer, int limit);
    
    /**
     * Package-private constructor. RegisterWriter instances are created by the VM.
     * 
     * @param nativeHandle Handle to the native RegisterWriter
     */
    RegisterWriter(long nativeHandle) {
        this(nativeHandle, 1024); // Default 1KB buffer
    }
    
    /**
     * Package-private constructor with buffer size.
     * 
     * @param nativeHandle Handle to the native RegisterWriter
     * @param bufferSize Initial buffer size in bytes
     */
    RegisterWriter(long nativeHandle, int bufferSize) {
        this.nativeHandle = nativeHandle;
        // Allocate direct buffer for zero-copy transfer to Rust
        this.buffer = ByteBuffer.allocateDirect(bufferSize)
                                .order(ByteOrder.nativeOrder());
    }
    
    /**
     * Write a NULL value to the specified register slot.
     * Value is buffered and will be sent to Rust on flush().
     * 
     * @param slot Register slot index (0-based)
     */
    public void putNull(int slot) {
        ensureCapacity(3); // slot(2) + type(1)
        buffer.putShort((short) slot);
        buffer.put(TYPE_NULL);
    }
    
    /**
     * Write a boolean value to the specified register slot.
     * Value is buffered and will be sent to Rust on flush().
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
     * Value is buffered and will be sent to Rust on flush().
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
     * Value is buffered and will be sent to Rust on flush().
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
     * Value is buffered and will be sent to Rust on flush().
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
     * Flush all buffered writes to Rust registers.
     * 
     * This method must be called after writing all fields for a row to actually
     * transfer the data to the VM. This performs a single JNI call regardless of
     * the number of fields written, providing optimal performance.
     * 
     * After flush(), the buffer is cleared and ready for the next row.
     */
    public void flush() {
        int position = buffer.position();
        buffer.flip();
        nativeFlush(nativeHandle, buffer, position);
        buffer.clear();
    }
    
    /**
     * Ensure the buffer has at least the specified number of bytes remaining.
     * 
     * @param bytes Number of bytes needed
     * @throws IllegalStateException if buffer is too small
     */
    private void ensureCapacity(int bytes) {
        if (buffer.remaining() < bytes) {
            // TODO: Implement buffer growth
            // For now, just fail with a clear error message
            throw new IllegalStateException(
                String.format("Buffer overflow: need %d bytes, only %d remaining. " +
                             "Buffer size: %d, position: %d. " +
                             "TODO: Implement automatic buffer growth.",
                             bytes, buffer.remaining(), buffer.capacity(), buffer.position()));
        }
    }
}
