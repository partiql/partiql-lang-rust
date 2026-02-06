package org.partiql.jni;

/**
 * Provides type hints for scan projections.
 * 
 * Matches the Rust TypeHint enum.
 */
public enum TypeHint {
    /**
     * No specific type hint - any type is acceptable.
     */
    ANY
    // More specific hints may be added in the future
}
