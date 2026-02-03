package org.partiql.jni;

/**
 * Provides compile-time metadata for a data source.
 * 
 * Used by the query compiler to:
 * - Understand data source capabilities (projection pushdown, etc.)
 * - Resolve field names to scan sources
 * - Make optimization decisions
 * 
 * Matches the Rust DataSourceConfig trait.
 */
public interface DataSourceConfig {
    /**
     * Get the data source capabilities for optimization.
     */
    ScanCapabilities getCapabilities();
    
    /**
     * Resolve a field name to ScanSource at compile time.
     * 
     * @param fieldName The field name to resolve
     * @return ScanSource if the field exists, null otherwise
     */
    ScanSource resolveField(String fieldName);
}
