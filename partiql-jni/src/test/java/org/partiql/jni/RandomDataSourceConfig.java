package org.partiql.jni;

import java.util.List;

/**
 * Compile-time configuration for random data tables.
 * 
 * Provides metadata about the table without needing access to actual data.
 */
public class RandomDataSourceConfig implements DataSourceConfig {
    private final List<String> columnNames;

    public RandomDataSourceConfig(List<String> columnNames) {
        this.columnNames = columnNames;
    }

    @Override
    public ScanCapabilities getCaps() {
        return new ScanCapabilities(
            BufferStability.UNTIL_NEXT,
            true,  // can_project
            false  // can_return_opaque
        );
    }

    @Override
    public ScanSource resolve(String fieldName) {
        for (int i = 0; i < columnNames.size(); i++) {
            if (columnNames.get(i).equalsIgnoreCase(fieldName)) {
                return new ScanSource.ColumnIndex(i);
            }
        }
        return null;
    }
}
