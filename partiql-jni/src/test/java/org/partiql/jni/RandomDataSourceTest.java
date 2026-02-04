package org.partiql.jni;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

/**
 * Test for custom random data source implementation.
 * 
 * Demonstrates end-to-end custom catalog integration with PartiQL JNI.
 */
class RandomDataSourceTest {

    @Test
    void testRandomDataSourceWithFilter() {
        // Configure random data source: 1000 rows, columns [a, b]
        RandomCompilationCatalog compCatalog = 
            new RandomCompilationCatalog("data", 1000, List.of("a", "b"));
        RandomExecutionCatalog execCatalog = 
            new RandomExecutionCatalog("data", 1000, List.of("a", "b"));
        
        // Query: SELECT a, b FROM data WHERE a % 2 = 0
        String query = "SELECT a, b FROM data WHERE a % 2 = 0";
        
        int rowCount = 0;
        
        // Compile and execute - exceptions will cause test failure
        try (CompilationContext compContext = new CompilationContext()) {
            long catalogId = compContext.addCatalog("default", compCatalog);
            
            PlanCompiler compiler = new PlanCompiler();
            try (CompiledPlan plan = compiler.compile(query, compContext)) {
                
                // Execute
                try (ExecutionContext execContext = new ExecutionContext()) {
                    execContext.addCatalog(catalogId, execCatalog);
                    
                    try (PartiQLVM vm = new PartiQLVM(plan, execContext)) {
                        ExecutionResult result = vm.execute();
                        
                        assertTrue(result.isQuery(), "Result should be a query");
                        
                        // Verify: all returned 'a' values should be even
                        try (QueryIterator iter = result.asQueryIterator()) {
                            while (iter.hasNext()) {
                                RegisterReader row = iter.next();
                                
                                // Get value from column 0 (column 'a')
                                Long aValue = row.getI64(0);
                                
                                assertNotNull(aValue, "Column 'a' should not be null");
                                assertEquals(0, aValue % 2, 
                                    "Value 'a' should be even, but got: " + aValue);
                                
                                rowCount++;
                            }
                        }
                    }
                }
            }
        } catch (Exception e) {
            // Fail the test with the exception message
            fail("Test failed with exception: " + e.getMessage(), e);
        }
        
        // Verify we got results - expect approximately 50% of 1000 rows to be even
        assertTrue(rowCount > 400, 
            "Expected at least 400 even values from 1000 random numbers, but got: " + rowCount);
        assertTrue(rowCount < 600,
            "Expected at most 600 even values from 1000 random numbers, but got: " + rowCount);
    }
}
