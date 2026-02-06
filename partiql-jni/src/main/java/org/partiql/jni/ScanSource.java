package org.partiql.jni;

/**
 * Represents how a field value is sourced during a scan operation.
 * 
 * Matches the Rust ScanSource enum with three variants:
 * - ColumnIndex: Value at a specific column index
 * - FieldPath: Value accessed by field path (e.g., nested field)
 * - BaseRow: The entire row without projection
 */
public abstract class ScanSource {
    
    /**
     * Source is a column at the specified index.
     */
    public static final class ColumnIndex extends ScanSource {
        private final int index;
        
        public ColumnIndex(int index) {
            if (index < 0) {
                throw new IllegalArgumentException("Column index must be non-negative");
            }
            this.index = index;
        }
        
        public int getIndex() { 
            return index; 
        }
        
        @Override
        public String toString() {
            return "ColumnIndex(" + index + ")";
        }
    }
    
    /**
     * Source is a field accessed by path (e.g., nested field).
     */
    public static final class FieldPath extends ScanSource {
        private final String path;
        
        public FieldPath(String path) {
            if (path == null) {
                throw new IllegalArgumentException("Field path cannot be null");
            }
            this.path = path;
        }
        
        public String getPath() { 
            return path; 
        }
        
        @Override
        public String toString() {
            return "FieldPath(\"" + path + "\")";
        }
    }
    
    /**
     * Source is the entire base row (no projection).
     */
    public static final class BaseRow extends ScanSource {
        public static final BaseRow INSTANCE = new BaseRow();
        
        private BaseRow() {}
        
        @Override
        public String toString() {
            return "BaseRow";
        }
    }
}
