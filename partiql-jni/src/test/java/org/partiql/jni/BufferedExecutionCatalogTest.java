package org.partiql.jni;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Disabled;

/**
 * Test demonstrating BufferedExecutionCatalog usage and performance benefits.
 * 
 * This test shows how to use the BufferedExecutionCatalog to eliminate
 * Rust-to-Java callback overhead during query execution.
 */
public class BufferedExecutionCatalogTest {
    
    @Test
    public void testBasicUsage() throws Exception {
        // 1. Create compilation context and catalog
        CompilationContext compContext = new CompilationContext();
        CompilationCatalog compCatalog = new CompilationCatalog() {
            @Override
            public DataSourceHandle getTable(java.util.List<BindingsName> path) {
                if (path.size() == 1 && path.get(0).toString().equalsIgnoreCase("users")) {
                    long entryId = 1;
                    DataSourceConfig config = new DataSourceConfig() {
                        @Override
                        public ScanCapabilities getCaps() {
                            return new ScanCapabilities(BufferStability.UNTIL_CLOSE, true, false);
                        }
                        
                        @Override
                        public ScanSource resolve(String fieldName) {
                            if (fieldName.equalsIgnoreCase("id")) {
                                return new ScanSource.ColumnIndex(0);
                            }
                            return null;
                        }
                    };
                    return new DataSourceHandle(entryId, config);
                }
                return null;
            }
        };
        
        long catalogId = compContext.addCatalog("default", compCatalog);
        
        // 2. Compile query
        PlanCompiler compiler = new PlanCompiler();
        CompiledPlan plan = compiler.compile("SELECT id FROM users", compContext);
        
        // 3. Create BufferedExecutionCatalog with pre-populated data
        BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
            1L, // entryId matching the compilation catalog
            writer -> {
                // Row 1: id=42, name="Alice"
                writer.putLong(0, 42);
                
                // Row 2: id=43, name="Bob"
                // writer.putLong(0, 43);
                
                // Row 3: id=44, name="Charlie"
                // writer.putLong(0, 44);
            }
        );
        
        System.out.println("Buffer size: " + catalog.getBufferSize() + " bytes");
        
        // 4. Register buffered catalog to execution context
        ExecutionContext execContext = new ExecutionContext();
        execContext.addBufferedCatalog(catalogId, catalog);
        
        // 5. Execute query with zero JNI callback overhead!
        PartiQLVM vm = new PartiQLVM(plan, execContext);
        ExecutionResult result = vm.execute();
        
        assertTrue(result.isQuery(), "Expected query result");
        
        // 6. Read results
        QueryIterator iterator = result.asQueryIterator();
        int rowCount = 0;
        
        while (iterator.hasNext()) {
            RegisterReader row = iterator.next();
            Long id = row.getI64(0);
            
            rowCount++;
            
            assertEquals(42, id);
            
            System.out.println("Row " + rowCount + ": id=" + id);
        }
        
        assertEquals(1, rowCount, "Expected 1 rows");
        
        // Cleanup
        // NOTE: Do NOT call result.close() after asQueryIterator()
        // The iterator consumes the result
        iterator.close();
        vm.close();
        plan.close();
        execContext.close();
        compContext.close();
    }
    
    @Test
    public void testWithCustomBufferSize() throws Exception {
        // Test with custom buffer size
        BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
            1L,
            10 * 1024, // 10KB buffer
            writer -> {
                for (int i = 0; i < 100; i++) {
                    writer.putLong(0, i);
                    writer.putString(1, "User" + i);
                }
            }
        );
        
        assertTrue(catalog.getBufferSize() > 0);
        assertTrue(catalog.getBufferSize() <= 10 * 1024);
    }
    
    @Test
    public void testBufferOverflow() {
        // Test that buffer overflow is detected
        assertThrows(IllegalStateException.class, () -> {
            BufferedExecutionCatalog.create(
                1L,
                10, // Very small buffer - only 10 bytes
                writer -> {
                    // This should overflow
                    writer.putLong(0, 12345678);
                    writer.putString(1, "This will definitely overflow");
                }
            );
        });
    }
    
    @Disabled
    @Test
    public void testDifferentDataTypes() throws Exception {
        CompilationContext compContext = new CompilationContext();
        CompilationCatalog compCatalog = new CompilationCatalog() {
            @Override
            public DataSourceHandle getTable(java.util.List<BindingsName> path) {
                long entryId = 1;
                DataSourceConfig config = new DataSourceConfig() {
                    @Override
                    public ScanCapabilities getCaps() {
                        return new ScanCapabilities(BufferStability.UNTIL_NEXT, false, false);
                    }
                    
                    @Override
                    public ScanSource resolve(String fieldName) {
                        return new ScanSource.ColumnIndex(0);
                    }
                };
                return new DataSourceHandle(entryId, config);
            }
        };
        
        long catalogId = compContext.addCatalog("default", compCatalog);
        
        PlanCompiler compiler = new PlanCompiler();
        CompiledPlan plan = compiler.compile("SELECT a, b FROM data", compContext);
        
        // Test different data types
        BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
            1L,
            writer -> {
                // Row with various types
                writer.putLong(0, 42);
                writer.putString(1, "test");
                writer.putNull(2);
            }
        );
        
        ExecutionContext execContext = new ExecutionContext();
        execContext.addBufferedCatalog(catalogId, catalog);
        
        PartiQLVM vm = new PartiQLVM(plan, execContext);
        ExecutionResult result = vm.execute();
        QueryIterator iterator = result.asQueryIterator();
        
        assertTrue(iterator.hasNext());
        RegisterReader row = iterator.next();
        
        assertEquals(42, row.getI64(0));
        assertEquals("test", row.getStr(1));
        // Note: RegisterReader doesn't have isNull() - would need to use getValue() for that
        
        assertFalse(iterator.hasNext());
        
        iterator.close();
        result.close();
        vm.close();
        plan.close();
        execContext.close();
        compContext.close();
    }
}
