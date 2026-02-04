package org.partiql.jni;

import java.util.List;

/**
 * Compilation-time catalog that provides table metadata.
 * 
 * Used during query compilation to:
 * - Validate table existence
 * - Provide schema information
 * - Assign EntryIds for execution-time resolution
 * - Enable optimization decisions
 * 
 * The CompilationCatalog is data-independent - it only provides metadata.
 * Multiple datasets with the same schema can share the same CompilationCatalog.
 * 
 * Matches the Rust CompilationCatalog trait.
 */
public interface CompilationCatalog {
    /**
     * Get table metadata by path.
     * 
     * Path examples:
     * - Single table: [BindingsName.undelimited("users")]
     * - Schema.table: [BindingsName.undelimited("public"), 
     *                  BindingsName.undelimited("users")]
     * - Multi-level: [BindingsName.undelimited("db"), 
     *                 BindingsName.undelimited("public"),
     *                 BindingsName.undelimited("users")]
     * 
     * @param path List of BindingsName for path resolution
     * @return DataSourceHandle if table exists, null otherwise
     */
    DataSourceHandle getTable(List<BindingsName> path);
}
