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
        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L, // entryId matching the compilation catalog
                pool,
                writer -> {
                    writer.beginRow();
                    writer.putLong(0, 42);
                    writer.endRow();
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

            assertEquals(1, rowCount, "Expected 1 row");

            // Cleanup
            iterator.close();
            vm.close();
            plan.close();
            execContext.close();
            compContext.close();
        } // Buffer automatically returned to pool
    }

    @Test
    public void testMultipleRows() throws Exception {
        BufferedMemoryPool pool = new BufferedMemoryPool();

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
                            if (fieldName.equalsIgnoreCase("name")) {
                                return new ScanSource.FieldPath("name");
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

        PlanCompiler compiler = new PlanCompiler();
        CompiledPlan plan = compiler.compile("SELECT id, name FROM users", compContext);

        // Write 3 rows
        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    writer.beginRow();
                    writer.putLong(0, 1);
                    writer.putString(1, "Alice");
                    writer.endRow();

                    writer.beginRow();
                    writer.putLong(0, 2);
                    writer.putString(1, "Bob");
                    writer.endRow();

                    writer.beginRow();
                    writer.putLong(0, 3);
                    writer.putString(1, "Charlie");
                    writer.endRow();
                })) {
            ExecutionContext execContext = new ExecutionContext();
            execContext.addBufferedCatalog(catalogId, catalog);

            PartiQLVM vm = new PartiQLVM(plan, execContext);
            ExecutionResult result = vm.execute();

            assertTrue(result.isQuery(), "Expected query result");

            QueryIterator iterator = result.asQueryIterator();
            int rowCount = 0;

            while (iterator.hasNext()) {
                RegisterReader row = iterator.next();
                rowCount++;
                System.out.println("Row " + rowCount + ": id=" + row.getI64(0) + ", name=" + row.getStr(1));
            }

            assertEquals(3, rowCount, "Expected 3 rows");

            iterator.close();
            vm.close();
            plan.close();
            execContext.close();
            compContext.close();
        }
    }

    @Test
    public void testWithCustomBufferSize() throws Exception {
        // Test with custom buffer size and multiple rows
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                10 * 1024, // 10KB buffer
                writer -> {
                    for (int i = 0; i < 100; i++) {
                        writer.beginRow();
                        writer.putLong(0, i);
                        writer.putString(1, "User" + i);
                        writer.endRow();
                    }
                })) {
            assertTrue(catalog.getBufferSize() > 0);
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
                    writer.beginRow();
                    writer.putLong(0, 12345678);
                    writer.putString(1, "This will trigger buffer growth");
                    writer.endRow();
                })) {
            // Should succeed with auto-growth
            assertTrue(catalog.getBufferSize() > 10);
        }
    }

    // =========================================================================
    // Collection / complex value tests
    // =========================================================================

    @Test
    public void testBufferValueWriterList() {
        // Test writing a list value into a buffer slot
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    writer.beginRow();
                    writer.putLong(0, 1);

                    // Write a list into slot 1
                    BufferValueWriter vw = writer.valueWriter(1);
                    vw.stepInList();
                    vw.putI64(10);
                    vw.putI64(20);
                    vw.putI64(30);
                    vw.stepOut();
                    assertEquals(0, vw.getDepth(), "All containers should be closed");

                    writer.endRow();
                })) {
            assertTrue(catalog.getBufferSize() > 0, "Buffer should have data");
        }
    }

    @Test
    public void testBufferValueWriterTuple() {
        // Test writing a tuple value into a buffer slot
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    writer.beginRow();

                    // Write a tuple into slot 0
                    BufferValueWriter vw = writer.valueWriter(0);
                    vw.stepInTuple();
                    vw.putFieldName("city");
                    vw.putString("Seattle");
                    vw.putFieldName("zip");
                    vw.putI64(98101);
                    vw.stepOut();
                    assertEquals(0, vw.getDepth());

                    writer.endRow();
                })) {
            assertTrue(catalog.getBufferSize() > 0, "Buffer should have data");
        }
    }

    @Test
    public void testBufferValueWriterBag() {
        // Test writing a bag value into a buffer slot
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    writer.beginRow();

                    BufferValueWriter vw = writer.valueWriter(0);
                    vw.stepInBag();
                    vw.putString("a");
                    vw.putString("b");
                    vw.putString("c");
                    vw.stepOut();
                    assertEquals(0, vw.getDepth());

                    writer.endRow();
                })) {
            assertTrue(catalog.getBufferSize() > 0, "Buffer should have data");
        }
    }

    @Test
    public void testBufferValueWriterNestedStructure() {
        // Test writing a deeply nested structure: tuple with a list of tuples
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    writer.beginRow();

                    // slot 0: a tuple with nested list of tuples
                    BufferValueWriter vw = writer.valueWriter(0);
                    vw.stepInTuple();

                    vw.putFieldName("name");
                    vw.putString("Alice");

                    vw.putFieldName("scores");
                    vw.stepInList();

                    // First score tuple
                    vw.stepInTuple();
                    vw.putFieldName("subject");
                    vw.putString("math");
                    vw.putFieldName("grade");
                    vw.putI64(95);
                    vw.stepOut();

                    // Second score tuple
                    vw.stepInTuple();
                    vw.putFieldName("subject");
                    vw.putString("science");
                    vw.putFieldName("grade");
                    vw.putI64(87);
                    vw.stepOut();

                    vw.stepOut(); // close list

                    vw.putFieldName("active");
                    vw.putBoolean(true);

                    vw.stepOut(); // close outer tuple
                    assertEquals(0, vw.getDepth());

                    writer.endRow();
                })) {
            assertTrue(catalog.getBufferSize() > 0, "Buffer should have data");
        }
    }

    @Test
    public void testMultipleRowsWithCollections() {
        // Test writing multiple rows with a mix of scalars and collections
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    // Row 1: scalar id + list of tags
                    writer.beginRow();
                    writer.putLong(0, 1);

                    BufferValueWriter vw1 = writer.valueWriter(1);
                    vw1.stepInList();
                    vw1.putString("java");
                    vw1.putString("rust");
                    vw1.stepOut();

                    writer.endRow();

                    // Row 2: scalar id + empty list
                    writer.beginRow();
                    writer.putLong(0, 2);

                    BufferValueWriter vw2 = writer.valueWriter(1);
                    vw2.stepInList();
                    vw2.stepOut(); // empty list

                    writer.endRow();

                    // Row 3: scalar id + list with null
                    writer.beginRow();
                    writer.putLong(0, 3);

                    BufferValueWriter vw3 = writer.valueWriter(1);
                    vw3.stepInList();
                    vw3.putString("python");
                    vw3.putNull();
                    vw3.putString("go");
                    vw3.stepOut();

                    writer.endRow();
                })) {
            assertTrue(catalog.getBufferSize() > 0, "Buffer should have data");
        }
    }

    @Test
    public void testBufferValueWriterStepOutWithoutContainer() {
        BufferedMemoryPool pool = new BufferedMemoryPool();

        assertThrows(IllegalStateException.class, () -> {
            BufferedExecutionCatalog.create(
                    1L,
                    pool,
                    writer -> {
                        BufferValueWriter vw = writer.valueWriter(0);
                        vw.stepOut(); // no container open — should throw
                    });
        });
    }

    @Test
    public void testBufferValueWriterMixedTypes() {
        // Test all scalar types inside a list
        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    writer.beginRow();

                    BufferValueWriter vw = writer.valueWriter(0);
                    vw.stepInList();
                    vw.putNull();
                    vw.putMissing();
                    vw.putBoolean(true);
                    vw.putBoolean(false);
                    vw.putI64(42);
                    vw.putI64(-1);
                    vw.putF64(3.14);
                    vw.putString("hello");
                    vw.putString("");
                    vw.stepOut();

                    writer.endRow();
                })) {
            assertTrue(catalog.getBufferSize() > 0);
        }
    }

    @Disabled("Requires full end-to-end execution with collection projections")
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

        BufferedMemoryPool pool = new BufferedMemoryPool();

        try (BufferedExecutionCatalog catalog = BufferedExecutionCatalog.create(
                1L,
                pool,
                writer -> {
                    // Row with various types
                    writer.beginRow();
                    writer.putLong(0, 42);
                    writer.putString(1, "test");
                    writer.putNull(2);
                    writer.endRow();
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