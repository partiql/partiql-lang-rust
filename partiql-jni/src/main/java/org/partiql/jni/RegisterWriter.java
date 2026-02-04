package org.partiql.jni;

/**
 * Provides write access to VM registers during data source iteration.
 * 
 * This class enables custom DataSource implementations to write values
 * directly to the VM's register file, avoiding intermediate object allocation.
 * 
 * RegisterWriter instances are provided by the VM during data source iteration
 * and should not be created directly.
 */
public final class RegisterWriter {
    private final long nativeHandle;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    // Native methods (package-private)
    private static native void nativeWriteNull(long writerHandle, int slot);
    private static native void nativeWriteBoolean(long writerHandle, int slot, boolean value);
    private static native void nativeWriteLong(long writerHandle, int slot, long value);
    private static native void nativeWriteDouble(long writerHandle, int slot, double value);
    private static native void nativeWriteString(long writerHandle, int slot, String value);
    
    /**
     * Package-private constructor. RegisterWriter instances are created by the VM.
     */
    RegisterWriter(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }
    
    /**
     * Write a NULL value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     */
    public void putNull(int slot) {
        nativeWriteNull(nativeHandle, slot);
    }
    
    /**
     * Write a boolean value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The boolean value to write
     */
    public void putBoolean(int slot, boolean value) {
        nativeWriteBoolean(nativeHandle, slot, value);
    }
    
    /**
     * Write a long (i64) value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The long value to write
     */
    public void putLong(int slot, long value) {
        nativeWriteLong(nativeHandle, slot, value);
    }
    
    /**
     * Write a double (f64) value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The double value to write
     */
    public void putDouble(int slot, double value) {
        nativeWriteDouble(nativeHandle, slot, value);
    }
    
    /**
     * Write a string value to the specified register slot.
     * 
     * @param slot Register slot index (0-based)
     * @param value The string value to write
     */
    public void putString(int slot, String value) {
        nativeWriteString(nativeHandle, slot, value);
    }
}
