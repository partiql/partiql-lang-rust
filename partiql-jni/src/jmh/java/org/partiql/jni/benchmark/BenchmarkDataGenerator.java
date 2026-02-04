package org.partiql.jni.benchmark;

import org.partiql.spi.types.PType;
import org.partiql.spi.value.Datum;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Utility class for generating consistent test data for benchmarks.
 * Creates data with schema: {a: Int, b: String}
 */
public class BenchmarkDataGenerator {
    
    /**
     * Generates optimized columnar data for use with partiql-jni.
     * Uses primitive arrays to avoid boxing overhead.
     * 
     * @param rowCount Number of rows to generate
     * @return Columnar data structure with primitive arrays
     */
    public static BenchmarkData generateDataForJni(int rowCount) {
        long[] columnA = new long[rowCount];
        long[] columnB = new long[rowCount];
        
        for (int i = 0; i < rowCount; i++) {
            columnA[i] = i;
            columnB[i] = i;
        }
        
        return new BenchmarkData(columnA, columnB);
    }
    
    /**
     * Column-oriented data structure for benchmark.
     * Uses primitive arrays to avoid boxing overhead.
     */
    public static class BenchmarkData {
        public final long[] columnA;
        public final long[] columnB;
        
        public BenchmarkData(long[] columnA, long[] columnB) {
            this.columnA = columnA;
            this.columnB = columnB;
        }
        
        public int size() {
            return columnA.length;
        }
    }
    
    /**
     * Generates Datum rows for use with partiql-eval.
     * Each row has: {a: Int, b: String}
     * 
     * @param rowCount Number of rows to generate
     * @return List of Datum representing rows
     */
    public static List<Datum> generateDatumRows(int rowCount) {
        List<Datum> data = new ArrayList<>(rowCount);
        for (int i = 0; i < rowCount; i++) {
            Map<String, Integer> delegate = new HashMap<>();
            delegate.put("a", i);
            delegate.put("b", i);
            data.add(new DatumWrapper(delegate));
        }
        return data;
    }

    static class DatumWrapper implements Datum {
        private final Map<String, Integer> _map;

        DatumWrapper(Map<String, Integer> input) {
            _map = input;
        }

        @Override
        public PType getType() {
            return PType.struct();
        }

        @Override
        public Datum get(String name) {
            Integer result = _map.get(name);
            return Datum.integer(result);
        }

        @Override
        public Datum getInsensitive(String name) {
            Integer result = _map.get(name);
            return Datum.integer(result);
        }
    }
}
