package org.partiql.jni.benchmark;

import org.openjdk.jmh.annotations.*;
import org.openjdk.jmh.infra.Blackhole;
import org.partiql.jni.*;

import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

/**
 * JMH benchmark for partiql-jni query execution.
 * Measures execution time for: SELECT a, b FROM data WHERE a % 2 = 0
 */
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Warmup(iterations = 1, time = 1)
@Measurement(iterations = 3, time = 1)
@Fork(1)
@State(Scope.Benchmark)
public class PartiQLJniBenchmark {
    
    @Param({"100"})
    private int rowCount;
    
    private CompiledPlan compiledPlan;
    private long catalogId;
    private List<Map<String, Object>> data;
    private PartiQLVM vm;  // VM created once in setup and reused
    private ExecutionContext execContext;  // Execution context created once
    
    @Setup(Level.Trial)
    public void setup() {
        try {
            // Generate test data (shared across iterations)
            data = BenchmarkDataGenerator.generateDataForJni(rowCount);
            
            // Compile the query once
            PlanCompiler compiler = new PlanCompiler();
            CompilationContext compilationContext = new CompilationContext();
            
            // Register the "data" table in compilation catalog
            BenchmarkCompilationCatalog compilationCatalog = new BenchmarkCompilationCatalog();
            catalogId = compilationContext.addCatalog("default", compilationCatalog);
            
            String query = "SELECT a, b FROM data WHERE a % 2 = 0";
            compiledPlan = compiler.compile(query, compilationContext);
            
            // Create execution context once
            execContext = new ExecutionContext();
            BenchmarkExecutionCatalog executionCatalog = new BenchmarkExecutionCatalog(data);
            execContext.addCatalog(catalogId, executionCatalog);
            
            // Create VM once with the context
            vm = new PartiQLVM(compiledPlan, execContext);
            
            System.out.println("=== BENCHMARK SETUP COMPLETE ===");
            System.out.println("Row count: " + rowCount);
            System.out.println("Data size: " + data.size());
        } catch (Exception e) {
            throw new RuntimeException("Setup failed", e);
        }
    }
    
    @TearDown(Level.Trial)
    public void teardown() {
        // Close VM first, then plan
        if (vm != null) {
            vm.close();
        }
        if (compiledPlan != null) {
            try {
                compiledPlan.close();
            } catch (Exception e) {
                // Ignore cleanup errors
            }
        }
    }
    
    @Benchmark
    public int executeQuery(Blackhole blackhole) {
        int count = 0;
        
        try {
            // Execute query (VM and context already set up)
            ExecutionResult result = vm.execute();
            
            if (result.isQuery()) {
                try (QueryIterator iterator = result.asQueryIterator()) {
                    while (iterator.hasNext()) {
                        RegisterReader row = iterator.next();
                        // Consume the fields to prevent JIT optimization
                        blackhole.consume(row.getI64(0));  // field 'a'
                        blackhole.consume(row.getI64(1));  // field 'b'
                        count++;
                    }
                }
                // NOTE: Do NOT call result.close() here!
                // asQueryIterator() consumes the ExecutionResult,
                // so calling close() would cause a double-free
            }
        } catch (Exception e) {
            throw new RuntimeException("Query execution failed", e);
        }
        
        return count;
    }
    
    // Helper catalog implementations
    private static class BenchmarkCompilationCatalog implements CompilationCatalog {
        @Override
        public DataSourceHandle getTable(List<BindingsName> path) {
            if (path.size() == 1 && path.get(0).getName().equals("data")) {
                DataSourceConfig config = new DataSourceConfig() {
                    @Override
                    public ScanCapabilities getCaps() {
                        return new ScanCapabilities(BufferStability.UNTIL_CLOSE, true, false);
                    }
                    
                    @Override
                    public ScanSource resolve(String fieldName) {
                        if ("a".equals(fieldName)) {
                            return new ScanSource.ColumnIndex(0);
                        } else if ("b".equals(fieldName)) {
                            return new ScanSource.ColumnIndex(1);
                        }
                        return null;
                    }
                };
                return new DataSourceHandle(1L, config);
            }
            return null;
        }
    }
    
    private static class BenchmarkExecutionCatalog implements ExecutionCatalog {
        private final List<Map<String, Object>> data;
        
        BenchmarkExecutionCatalog(List<Map<String, Object>> data) {
            this.data = data;
        }
        
        @Override
        public DataSource create(long entryId, ScanLayout layout) {
            return new BenchmarkDataSource(data, layout);
        }
    }
    
    private static class BenchmarkDataSource implements DataSource {
        private final List<Map<String, Object>> data;
        private final ScanLayout layout;
        private int currentRow = 0;
        
        BenchmarkDataSource(List<Map<String, Object>> data, ScanLayout layout) {
            this.data = data;
            this.layout = layout;
        }
        
        @Override
        public void open() {
            currentRow = 0;
        }
        
        @Override
        public boolean nextRow(RegisterWriter writer) {
            if (currentRow >= data.size()) {
                return false;
            }
            
            Map<String, Object> row = data.get(currentRow);
            
            // Write projected columns to registers
            for (ScanProjection proj : layout.getProjections()) {
                int targetSlot = proj.getTargetSlot();
                ScanSource source = proj.getSource();
                
                if (source instanceof ScanSource.ColumnIndex) {
                    int colIndex = ((ScanSource.ColumnIndex) source).getIndex();
                    
                    // Column 0 = 'a' (integer), Column 1 = 'b' (string)
                    if (colIndex == 0) {
                        Integer value = (Integer) row.get("a");
                        writer.putLong(targetSlot, value.longValue());
                    } else if (colIndex == 1) {
                        Integer value = (Integer) row.get("b");
                        writer.putLong(targetSlot, value.longValue());
                    }
                }
            }
            
            currentRow++;
            return true;
        }
        
        @Override
        public void close() {
            // No resources to clean up
        }
    }
}
