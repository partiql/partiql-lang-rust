package org.partiql.jni;

/**
 * Describes the capabilities of a data source.
 * 
 * Matches the Rust ScanCapabilities struct.
 */
public final class ScanCapabilities {
    private final BufferStability stability;
    private final boolean canProject;
    private final boolean canReturnOpaque;
    
    public ScanCapabilities(BufferStability stability, boolean canProject, boolean canReturnOpaque) {
        if (stability == null) {
            throw new IllegalArgumentException("BufferStability cannot be null");
        }
        this.stability = stability;
        this.canProject = canProject;
        this.canReturnOpaque = canReturnOpaque;
    }
    
    /**
     * Returns the buffer stability guarantee of this data source.
     */
    public BufferStability getStability() { 
        return stability; 
    }
    
    /**
     * Returns true if the data source supports projection pushdown.
     */
    public boolean canProject() { 
        return canProject; 
    }
    
    /**
     * Returns true if the data source can return opaque values.
     */
    public boolean canReturnOpaque() { 
        return canReturnOpaque; 
    }
    
    @Override
    public String toString() {
        return String.format("ScanCapabilities{stability=%s, canProject=%s, canReturnOpaque=%s}",
            stability, canProject, canReturnOpaque);
    }
}
