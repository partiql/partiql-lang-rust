package org.partiql.jni;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Describes the layout of a scan operation, including projections.
 * 
 * Matches the Rust ScanLayout struct.
 */
public final class ScanLayout {
    private final List<ScanProjection> projections;
    
    public ScanLayout(List<ScanProjection> projections) {
        if (projections == null) {
            throw new IllegalArgumentException("Projections list cannot be null");
        }
        this.projections = new ArrayList<>(projections);
    }
    
    /**
     * Returns the list of projections for this scan.
     */
    public List<ScanProjection> getProjections() { 
        return Collections.unmodifiableList(projections); 
    }
    
    /**
     * Creates a base row layout (no projection, return entire row).
     * 
     * Matches the Rust ScanLayout::base_row() method.
     */
    public static ScanLayout baseRow() {
        return new ScanLayout(List.of(
            new ScanProjection(ScanSource.BaseRow.INSTANCE, 0, TypeHint.ANY)
        ));
    }
    
    /**
     * Checks if this layout is a base row only layout.
     */
    public boolean isBaseRowOnly() {
        return projections.size() == 1 &&
               projections.get(0).getSource() instanceof ScanSource.BaseRow &&
               projections.get(0).getTargetSlot() == 0;
    }
    
    @Override
    public String toString() {
        return "ScanLayout{projections=" + projections + "}";
    }
}
