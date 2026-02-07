package org.partiql.jni.benchmark;

import org.openjdk.jmh.annotations.*;
import org.openjdk.jmh.infra.Blackhole;
import org.partiql.jni.*;

import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

/**
 * JMH benchmark for partiql-jni with BufferedExecutionCatalog and BufferedMemoryPool.
 * 
 * This benchmark measures the full data injection and execution pipeline:
 * - Data generation (HashMap per row)
 * - Buffer population from HashMap data (with buffer reuse via pool)
 * - ExecutionContext creation and registration
 * - Query execution
 * 
 * Measures execution time for: SELECT a, b FROM data WHERE a % 2 = 0
 * 
 * This tests both data injection overhead and query execution performance,
 * with the added benefit of buffer pooling to amortize DirectByteBuffer allocation costs.
 */
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Warmup(iterations = 5, time = 1)
@Measurement(iterations = 10, time = 1)
@Fork(2)
@State(Scope.Benchmark)
public class PartiQLJniBufferedBenchmark {
    
    @Param({"1"})
    private int rowCount;
    
    private String[] fieldNames = {"a", "b"};
    private CompiledPlan compiledPlan;
    private long catalogId;
    private PartiQLVM vm;  // VM created once in Trial setup and reused

    private List<Map<String, Integer>> backingData;
    private ExecutionContext execContext;  // Execution context created per iteration
    private BufferedExecutionCatalog bufferedCatalog;  // Catalog for current iteration
    private BufferedMemoryPool pool;  // Pool for buffer reuse across iterations
    private int nextValue = 0;  // Counter for generating new data values
    
    @Setup(Level.Trial)
    public void setupTrial() {
        try {
            // Create memory pool for buffer reuse across iterations
            // Using default settings: 1MB initial size, 4 buffer max pool size
            pool = new BufferedMemoryPool();
            
            // Compile the query once (reused across all iterations)
            PlanCompiler compiler = new PlanCompiler();
            CompilationContext compilationContext = new CompilationContext();
            
            // Register the "data" table in compilation catalog using FieldPath
            BenchmarkCompilationCatalog compilationCatalog = new BenchmarkCompilationCatalog();
            catalogId = compilationContext.addCatalog("default", compilationCatalog);
            
            String query = "SELECT a, b FROM data WHERE a % 2 = 0";
            compiledPlan = compiler.compile(query, compilationContext);
            
            // Create VM without context initially - context will be set per iteration
            // We'll use setContext() in the Invocation setup
            this.backingData = BenchmarkDataGenerator.generateHashMapRows(fieldNames, nextValue++, rowCount);
            this.bufferedCatalog = createBufferedExecutionCatalog();
            this.execContext = new ExecutionContext();
            this.execContext.addBufferedCatalog(catalogId, bufferedCatalog);
            vm = new PartiQLVM(compiledPlan, execContext);
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }
    
    @Setup(Level.Invocation)
    public void setupInvocation() {
        try {
            this.backingData = BenchmarkDataGenerator.generateHashMapRows(fieldNames, nextValue++, rowCount);
        } catch (Exception e) {
            throw new RuntimeException("Invocation setup failed", e);
        }
    }
    
    public BufferedExecutionCatalog createBufferedExecutionCatalog() {
        // Use the pool for efficient buffer reuse across iterations
        return BufferedExecutionCatalog.create(
                1L, // entryId
                pool, // Use the shared pool for buffer reuse
                writer -> {
                    // Loop through each row and write HashMap entries to buffer
                    // Buffer will auto-grow if needed
                    // TODO: We eventually need to incorporate the ScanLayout!
                    for (Map<String, Integer> row : this.backingData) {
                        int registerIndex = 0;
                        // Write each field value to sequential registers
                        for (Map.Entry<String, Integer> entry : row.entrySet()) {
                            writer.putLong(registerIndex++, (long) entry.getValue());
                        }
                    }
                }
            );
    }

    @TearDown(Level.Invocation)
    public void teardownInvocation() {
        bufferedCatalog.close();
    }
    
    @TearDown(Level.Trial)
    public void teardownTrial() {
        if (bufferedCatalog != null) {
            bufferedCatalog.close();
            bufferedCatalog = null;
        }
        if (vm != null) {
            vm.close();
        }

        // Close VM and compiled plan
        if (compiledPlan != null) {
            try {
                compiledPlan.close();
            } catch (Exception e) {
                // Ignore cleanup errors
            }
        }
        
        // Clear the pool (optional cleanup)
        if (pool != null) {
            pool.clear();
        }
    }
    
    @Benchmark
    public int executeQuery(Blackhole blackhole) {
        int count = 0;
        
        try {
            // Execute query with the context that was set up in setupInvocation
            // This measures execution time with data already injected
            this.bufferedCatalog = createBufferedExecutionCatalog();
            execContext.addBufferedCatalog(catalogId, bufferedCatalog);
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
    
    // Helper catalog implementation
    private static class BenchmarkCompilationCatalog implements CompilationCatalog {
        @Override
        public DataSourceHandle getTable(List<BindingsName> path) {
            if (path.size() == 1 && path.get(0).getName().equals("data")) {
                DataSourceConfig config = new DataSourceConfig() {
                    @Override
                    public ScanCapabilities getCaps() {
                        // Use UNTIL_NEXT for consistency with buffered approach
                        return new ScanCapabilities(BufferStability.UNTIL_NEXT, true, false);
                    }
                    
                    @Override
                    public ScanSource resolve(String fieldName) {
                        // Always return FieldPath for field-based access
                        // This allows runtime resolution of field positions
                        return new ScanSource.FieldPath(fieldName);
                    }
                };
                return new DataSourceHandle(1L, config);
            }
            return null;
        }
    }
}
