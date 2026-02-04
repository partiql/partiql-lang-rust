package org.partiql.jni.benchmark;

import com.amazon.ion.IonStruct;
import com.amazon.ion.IonValue;
import com.amazon.ion.system.IonSystemBuilder;
import org.partiql.jni.Value;
import org.partiql.spi.value.Datum;
import org.partiql.spi.value.Field;

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
     * Generates a list of maps for use with partiql-jni.
     * Each row has: {a: Int, b: String}
     * 
     * @param rowCount Number of rows to generate
     * @return List of maps representing rows
     */
    public static List<Map<String, Object>> generateDataForJni(int rowCount) {
        List<Map<String, Object>> data = new ArrayList<>(rowCount);
        
        for (int i = 0; i < rowCount; i++) {
            Map<String, Object> row = new HashMap<>();
            row.put("a", i);
            row.put("b", i);
            data.add(row);
        }
        
        return data;
    }
    
    /**
     * Generates Ion-encoded data for use with partiql-eval.
     * Each row has: {a: Int, b: String}
     * 
     * @param rowCount Number of rows to generate
     * @return List of IonStruct representing rows
     */
    public static List<IonValue> generateDataForEval(int rowCount) {
        var ion = IonSystemBuilder.standard().build();
        List<IonValue> data = new ArrayList<>(rowCount);
        
        for (int i = 0; i < rowCount; i++) {
            IonStruct row = ion.newEmptyStruct();
            row.add("a", ion.newInt(i));
            row.add("b", ion.newString("value_" + i));
            data.add(row);
        }
        
        return data;
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
            Datum row = Datum.struct(
                Field.of("a", Datum.integer(i)),
                Field.of("b", Datum.string("value_" + i))
            );
            data.add(row);
        }
        
        return data;
    }
}
