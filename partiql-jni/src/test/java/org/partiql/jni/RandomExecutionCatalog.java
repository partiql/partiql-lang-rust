package org.partiql.jni;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Execution catalog for random data sources.
 * 
 * Creates actual DataSource instances at execution time.
 */
public class RandomExecutionCatalog implements ExecutionCatalog {
    private final Map<Long, TableMetadata> tables;

    private static class TableMetadata {
        final int numRows;
        final List<String> columnNames;

        TableMetadata(int numRows, List<String> columnNames) {
            this.numRows = numRows;
            this.columnNames = columnNames;
        }
    }

    public RandomExecutionCatalog(String tableName, int numRows, List<String> columnNames) {
        this.tables = new HashMap<>();
        this.tables.put(1L, new TableMetadata(numRows, columnNames));
    }

    @Override
    public DataSource create(long entryId, ScanLayout layout) {
        TableMetadata meta = tables.get(entryId);
        if (meta == null) {
            throw new IllegalStateException("Table with entryId " + entryId + " not found");
        }

        return new RandomDataSource(meta.numRows, meta.columnNames.size(), layout);
    }
}
