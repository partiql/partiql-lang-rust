package org.partiql.jni;

/**
 * Describes a single field projection in a scan operation.
 * 
 * Matches the Rust ScanProjection struct.
 */
public final class ScanProjection {
    private final ScanSource source;
    private final int targetSlot;
    private final TypeHint typeHint;
    
    public ScanProjection(ScanSource source, int targetSlot, TypeHint typeHint) {
        if (source == null) {
            throw new IllegalArgumentException("ScanSource cannot be null");
        }
        if (typeHint == null) {
            throw new IllegalArgumentException("TypeHint cannot be null");
        }
        if (targetSlot < 0) {
            throw new IllegalArgumentException("Target slot must be non-negative");
        }
        
        this.source = source;
        this.targetSlot = targetSlot;
        this.typeHint = typeHint;
    }
    
    /**
     * Returns the source of the field value.
     */
    public ScanSource getSource() { 
        return source; 
    }
    
    /**
     * Returns the target register slot for this projection.
     */
    public int getTargetSlot() { 
        return targetSlot; 
    }
    
    /**
     * Returns the type hint for this projection.
     */
    public TypeHint getTypeHint() { 
        return typeHint; 
    }
    
    @Override
    public String toString() {
        return String.format("ScanProjection{source=%s, targetSlot=%d, typeHint=%s}",
            source, targetSlot, typeHint);
    }
}
