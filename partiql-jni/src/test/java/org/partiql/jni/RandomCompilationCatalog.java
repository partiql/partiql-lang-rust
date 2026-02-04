package org.partiql.jni;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Compilation catalog for random data sources.
 * 
 * Provides compile-time metadata and assigns EntryIds for execution-time resolution.
 */
public class RandomCompilationCatalog implements CompilationCatalog {
    private final Map<String, TableMetadata> tables;

    private static class TableMetadata {
        final long entryId;
        final int numRows;
        final List<String> columnNames;

        TableMetadata(long entryId, int numRows, List<String> columnNames) {
            this.entryId = entryId;
            this.numRows = numRows;
            this.columnNames = columnNames;
        }
    }

    public RandomCompilationCatalog(String tableName, int numRows, List<String> columnNames) {
        this.tables = new HashMap<>();
        this.tables.put(tableName.toLowerCase(), new TableMetadata(1L, numRows, columnNames));
    }

    @Override
    public DataSourceHandle getTable(List<BindingsName> path) {
        if (path == null || path.size() != 1) {
            return null;
        }

        String tableName = path.get(0).getName().toLowerCase();
        TableMetadata meta = tables.get(tableName);
        
        if (meta == null) {
            return null;
        }

        RandomDataSourceConfig config = new RandomDataSourceConfig(meta.columnNames);
        return new DataSourceHandle(meta.entryId, config);
    }
}
