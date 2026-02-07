package org.partiql.jni;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.HashMap;
import java.util.Map;

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
        // 0. Create memory pool for buffer management
        BufferedMemoryPool pool = new BufferedMemoryPool();

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
                                return new ScanSource.FieldPath("id");
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

        // 3. Create BufferedExecutionCatalog with pre-populated data using pool
        Map<String, Integer> delegate = new HashMap<>();
        delegate.put("id", 42);
        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L, // entryId matching the compilation catalog
                pool,
                writer -> {
                    int registerIndex = 0;
                    for (Map.Entry<String, Integer> entry : delegate.entrySet()) {
                        writer.putLong(registerIndex++, (long) entry.getValue());
                    }
                })) {
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
        } // Buffer automatically returned to pool
    }

    @Test
    public void testWithCustomBufferSize() throws Exception {
        // Test with custom buffer size
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                10 * 1024, // 10KB buffer
                writer -> {
                    for (int i = 0; i < 100; i++) {
                        writer.putLong(0, i);
                        writer.putString(1, "User" + i);
                    }
                })) {
            assertTrue(catalog.getBufferSize() > 0);
            assertTrue(catalog.getBufferSize() <= 10 * 1024);
        }
    }

    @Test
    public void testBufferOverflow() {
        // Test that buffer with auto-growth doesn't overflow
        BufferedMemoryPool pool = new BufferedMemoryPool();

        // With pooled buffers, auto-growth prevents overflow
        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                10, // Very small initial buffer - only 10 bytes
                writer -> {
                    // This will trigger auto-growth instead of overflow
                    writer.putLong(0, 12345678);
                    writer.putString(1, "This will trigger buffer growth");
                })) {
            // Should succeed with auto-growth
            assertTrue(catalog.getBufferSize() > 10);
        }
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
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    // Row with various types
                    writer.putLong(0, 42);
                    writer.putString(1, "test");
                    writer.putNull(2);
                })) {
            ExecutionContext execContext = new ExecutionContext();
            execContext.addBufferedCatalog(catalogId, catalog);

            PartiQLVM vm = new PartiQLVM(plan, execContext);
            ExecutionResult result = vm.execute();
            QueryIterator iterator = result.asQueryIterator();

            assertTrue(iterator.hasNext());
            RegisterReader row = iterator.next();

            assertEquals(42, row.getI64(0));
            assertEquals("test", row.getStr(1));
            // Note: RegisterReader doesn't have isNull() - would need to use getValue() for
            // that

            assertFalse(iterator.hasNext());

            iterator.close();
            result.close();
            vm.close();
            plan.close();
            execContext.close();
            compContext.close();
        }
    }
}
