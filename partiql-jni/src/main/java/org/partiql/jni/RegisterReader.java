package org.partiql.jni;

import org.partiql.jni.exceptions.PartiQLException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;

/**
 * Java mirror of Rust's RegisterReader for accessing row data.
 * This class provides direct access to column values with zero-copy buffer reading.
 * 
 * Performance optimization: Caches field positions in an ArrayList for true O(1)
 * array-indexed access with minimal overhead.
 * 
 * Note: RegisterReader instances are only valid while their parent QueryIterator
 * is positioned on a row (between hasNext() returning true and next() being called).
 */
public final class RegisterReader {
    private final ByteBuffer buffer;
    private final int fieldSize;
    private final int[] fieldPositions;
    private final int[] fieldTypes;
    
    // Type tags (must match BufferWriter and Rust encoding)
    private static final byte TYPE_NULL = 0;
    private static final byte TYPE_MISSING = 1;
    private static final byte TYPE_BOOL = 2;
    private static final byte TYPE_I64 = 3;
    private static final byte TYPE_F64 = 4;
    private static final byte TYPE_STRING = 5;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    // Native methods for fallback to full Value conversion
    private static native Value nativeGetValue(long iteratorHandle, int col) throws PartiQLException;
    
    /**
     * Package-private constructor - RegisterReaders are created by QueryIterator
     */
    RegisterReader(ByteBuffer buffer) throws PartiQLException {
        this.buffer = buffer;
        // CRITICAL: Set to native byte order to match Rust's to_ne_bytes()
        this.buffer.order(java.nio.ByteOrder.nativeOrder());
        
        // Build field cache on construction - scan buffer once
        // Use ArrayList for true O(1) indexed access
        // TODO: In future, make fieldSize configurable via schema.
        this.fieldSize = 10;
        this.fieldPositions = new int[this.fieldSize];
        this.fieldTypes = new int[this.fieldSize];
        buildFieldCache();
    }
    
    /**
     * Scan buffer once and cache all field positions
     * Assumes slots are sequential (0, 1, 2...) for array-indexed access
     */
    private void buildFieldCache() throws PartiQLException {
        buffer.position(0);
        
        while (buffer.position() < buffer.limit()) {
            if (buffer.remaining() < 3) {
                // Not enough bytes for field header
                break;
            }
            
            // Read slot (2 bytes)
            int slot = buffer.getShort() & 0xFFFF;
            
            // Read type tag (1 byte)
            byte typeTag = buffer.get();
            
            // Cache the position where data starts (after type tag)
            int dataPosition = buffer.position();
            
            // Ideally grow the arrays if needed
            this.fieldPositions[slot] = dataPosition;
            this.fieldTypes[slot] = typeTag;
            
            // Skip past the data to continue scanning
            skipFieldData(typeTag);
        }
    }
    
    /**
     * Get an i64 value from the specified column.
     * Uses cached field position for true O(1) array-indexed access.
     * 
     * @param col Column index (0-based)
     * @return The i64 value, or null if the column doesn't contain an i64
     * @throws PartiQLException if an error occurs
     */
    public Long getI64(int col) throws PartiQLException {
        // Direct array access - fastest possible lookup
        if (col < 0 || col >= fieldSize) {
            return null;
        }
        
        int position = fieldPositions[col];
        int typeTag = fieldTypes[col];
        
        if (typeTag == TYPE_I64) {
            // Position buffer at cached data position
            buffer.position(position);
            if (buffer.remaining() < 8) {
                throw new PartiQLException("Buffer underflow reading i64");
            }
            return buffer.getLong();
        } else if (typeTag == TYPE_NULL || typeTag == TYPE_MISSING) {
            return null;
        } else {
            // Wrong type
            return null;
        }
    }
    
    /**
     * Get a string value from the specified column.
     * Uses cached field position for true O(1) array-indexed access.
     * 
     * @param col Column index (0-based)
     * @return The string value, or null if the column doesn't contain a string
     * @throws PartiQLException if an error occurs
     */
    public String getStr(int col) throws PartiQLException {
        // Direct array access - fastest possible lookup
        if (col < 0 || col >= fieldSize) {
            return null;
        }
        
        int position = fieldPositions[col];
        int typeTag = fieldTypes[col];
        
        if (typeTag == TYPE_STRING) {
            // Position buffer at cached data position
            buffer.position(position);
            if (buffer.remaining() < 4) {
                throw new PartiQLException("Buffer underflow reading string length");
            }
            int length = buffer.getInt();
            
            if (buffer.remaining() < length) {
                throw new PartiQLException("Buffer underflow reading string data");
            }
            
            byte[] bytes = new byte[length];
            buffer.get(bytes);
            return new String(bytes, StandardCharsets.UTF_8);
        } else if (typeTag == TYPE_NULL || typeTag == TYPE_MISSING) {
            return null;
        } else {
            // Wrong type
            return null;
        }
    }
    
    /**
     * Get a generic value from the specified column.
     * This returns a ValueOwned representation that can be any PartiQL type.
     * 
     * Note: This method is not yet implemented for buffer-based reading
     * and will throw UnsupportedOperationException.
     * 
     * @param col Column index (0-based)
     * @return The value (never null - returns Missing if column is out of bounds)
     * @throws PartiQLException if an error occurs
     */
    public Value getValue(int col) throws PartiQLException {
        // TODO: Implement full Value conversion from buffer
        throw new UnsupportedOperationException("getValue() not yet implemented for buffer-based RegisterReader");
    }
    
    /**
     * Skip past field data based on type tag
     */
    private void skipFieldData(byte typeTag) throws PartiQLException {
        switch (typeTag) {
            case TYPE_NULL:
            case TYPE_MISSING:
                // No data to skip
                break;
            case TYPE_BOOL:
                if (buffer.remaining() < 1) {
                    throw new PartiQLException("Buffer underflow skipping bool");
                }
                buffer.position(buffer.position() + 1);
                break;
            case TYPE_I64:
            case TYPE_F64:
                if (buffer.remaining() < 8) {
                    throw new PartiQLException("Buffer underflow skipping i64/f64");
                }
                buffer.position(buffer.position() + 8);
                break;
            case TYPE_STRING:
                if (buffer.remaining() < 4) {
                    throw new PartiQLException("Buffer underflow reading string length");
                }
                int length = buffer.getInt();
                if (buffer.remaining() < length) {
                    throw new PartiQLException("Buffer underflow skipping string data");
                }
                buffer.position(buffer.position() + length);
                break;
            default:
                throw new PartiQLException("Unknown type tag: " + typeTag);
        }
    }
}
