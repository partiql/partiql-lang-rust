package org.partiql.jni;

import java.nio.charset.StandardCharsets;

/**
 * Cursor-based writer for constructing complex PartiQL values (tuples, lists, bags)
 * directly into a buffer.
 * 
 * <p>Mirrors the Rust {@code ValueWriter} API symmetrically. Values are written
 * inline into the same buffer used by {@link BufferWriter}, using structural markers
 * that the Rust decoder reconstructs into nested {@code ValueRef} trees.</p>
 * 
 * <h2>Usage</h2>
 * <pre>{@code
 * // Write a tuple into slot 2
 * BufferValueWriter vw = writer.valueWriter(2);
 * vw.stepInTuple();
 * vw.putFieldName("name");
 * vw.putString("Alice");
 * vw.putFieldName("scores");
 * vw.stepInList();
 * vw.putI64(95);
 * vw.putI64(87);
 * vw.stepOut();  // closes list
 * vw.stepOut();  // closes tuple
 * }</pre>
 * 
 * <h2>Wire Format</h2>
 * <ul>
 *   <li>{@code TYPE_TUPLE} — followed by (MARKER_FIELD_NAME len bytes Value)* MARKER_CONTAINER_END</li>
 *   <li>{@code TYPE_LIST} — followed by Value* MARKER_CONTAINER_END</li>
 *   <li>{@code TYPE_BAG} — followed by Value* MARKER_CONTAINER_END</li>
 *   <li>Scalars use the same type tags as {@link BufferWriter}</li>
 * </ul>
 * 
 * <h2>Thread Safety</h2>
 * <p>Not thread-safe. Must be used from the same thread as the parent BufferWriter.</p>
 */
public final class BufferValueWriter {
    private final BufferWriter writer;
    private int depth;
    
    /**
     * Package-private constructor. Created by {@link BufferWriter#valueWriter(int)}.
     * 
     * @param writer The parent BufferWriter to write into
     */
    BufferValueWriter(BufferWriter writer) {
        this.writer = writer;
        this.depth = 0;
    }
    
    // =========================================================================
    // Container navigation
    // =========================================================================
    
    /**
     * Step into a new tuple container.
     * 
     * <p>Writes the {@code TYPE_TUPLE} tag. Subsequent calls should alternate
     * between {@link #putFieldName(String)} and a value method ({@code putI64},
     * {@code putString}, {@code stepInList}, etc.).</p>
     */
    public void stepInTuple() {
        writer.writeByte(BufferWriter.TYPE_TUPLE);
        depth++;
    }
    
    /**
     * Step into a new list container.
     * 
     * <p>Writes the {@code TYPE_LIST} tag. Subsequent value calls add elements
     * to the list in order.</p>
     */
    public void stepInList() {
        writer.writeByte(BufferWriter.TYPE_LIST);
        depth++;
    }
    
    /**
     * Step into a new bag container.
     * 
     * <p>Writes the {@code TYPE_BAG} tag. Subsequent value calls add elements
     * to the bag (unordered collection).</p>
     */
    public void stepInBag() {
        writer.writeByte(BufferWriter.TYPE_BAG);
        depth++;
    }
    
    /**
     * Step out of the current container.
     * 
     * <p>Writes the {@code MARKER_CONTAINER_END} tag, closing the most recently
     * opened container.</p>
     * 
     * @throws IllegalStateException if no container is open
     */
    public void stepOut() {
        if (depth <= 0) {
            throw new IllegalStateException("stepOut() called with no open container");
        }
        writer.writeByte(BufferWriter.MARKER_CONTAINER_END);
        depth--;
    }
    
    // =========================================================================
    // Tuple field names
    // =========================================================================
    
    /**
     * Write a field name for the next value in a tuple.
     * 
     * <p>Must be called before each value inside a tuple container.
     * Writes: {@code MARKER_FIELD_NAME len:u32 utf8_bytes}.</p>
     * 
     * @param name The field name
     */
    public void putFieldName(String name) {
        byte[] bytes = name.getBytes(StandardCharsets.UTF_8);
        writer.writeByte(BufferWriter.MARKER_FIELD_NAME);
        writer.writeInt(bytes.length);
        writer.writeBytes(bytes);
    }
    
    // =========================================================================
    // Scalar values
    // =========================================================================
    
    /**
     * Write a NULL value.
     */
    public void putNull() {
        writer.writeByte(BufferWriter.TYPE_NULL);
    }
    
    /**
     * Write a MISSING value.
     */
    public void putMissing() {
        writer.writeByte(BufferWriter.TYPE_MISSING);
    }
    
    /**
     * Write a boolean value.
     * 
     * @param value The boolean value
     */
    public void putBoolean(boolean value) {
        writer.writeByte(BufferWriter.TYPE_BOOL);
        writer.writeByte((byte) (value ? 1 : 0));
    }
    
    /**
     * Write a long (i64) value.
     * 
     * @param value The long value
     */
    public void putI64(long value) {
        writer.writeByte(BufferWriter.TYPE_I64);
        writer.writeLong(value);
    }
    
    /**
     * Write a double (f64) value.
     * 
     * @param value The double value
     */
    public void putF64(double value) {
        writer.writeByte(BufferWriter.TYPE_F64);
        writer.writeDouble(value);
    }
    
    /**
     * Write a string value.
     * 
     * <p>Writes: {@code TYPE_STRING len:u32 utf8_bytes}.</p>
     * 
     * @param value The string value
     */
    public void putString(String value) {
        byte[] bytes = value.getBytes(StandardCharsets.UTF_8);
        writer.writeByte(BufferWriter.TYPE_STRING);
        writer.writeInt(bytes.length);
        writer.writeBytes(bytes);
    }
    
    // =========================================================================
    // Validation
    // =========================================================================
    
    /**
     * Get the current nesting depth.
     * 
     * <p>Returns 0 when all containers have been closed. Useful for
     * debugging or assertions.</p>
     * 
     * @return Current nesting depth
     */
    public int getDepth() {
        return depth;
    }
}