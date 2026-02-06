package org.partiql.jni;

import java.util.Random;

/**
 * Custom DataSource that generates random integer data.
 * 
 * Demonstrates how to implement DataSource for custom readers that write
 * directly to VM registers for optimal performance.
 */
public class RandomDataSource implements DataSource {
    private int currentRow;
    private final int totalRows;
    private final ScanLayout layout;
    private final int numColumns;
    private final Random rng;

    public RandomDataSource(int totalRows, int numColumns, ScanLayout layout) {
        this.currentRow = 0;
        this.totalRows = totalRows;
        this.layout = layout;
        this.numColumns = numColumns;
        this.rng = new Random();
    }

    @Override
    public void open() {
        this.currentRow = 0;
    }

    @Override
    public boolean nextRow(RegisterWriter writer) {
        if (currentRow >= totalRows) {
            return false;
        }

        // Generate random values for each projected column
        for (ScanProjection proj : layout.getProjections()) {
            int targetSlot = proj.getTargetSlot();
            ScanSource source = proj.getSource();

            if (source instanceof ScanSource.ColumnIndex) {
                int colIndex = ((ScanSource.ColumnIndex) source).getIndex();
                if (colIndex < numColumns) {
                    long randomValue = rng.nextLong();
                    // Write directly to VM register
                    writer.putLong(targetSlot, randomValue);
                } else {
                    throw new IllegalStateException(
                        "Column index " + colIndex + " out of bounds (max: " + (numColumns - 1) + ")"
                    );
                }
            } else {
                throw new UnsupportedOperationException(
                    "Random reader only supports ColumnIndex projections"
                );
            }
        }

        writer.flush();
        currentRow++;
        return true;
    }

    @Override
    public void close() {
        // No resources to clean up
    }
}
