package org.partiql.jni.benchmark;

import org.openjdk.jmh.annotations.*;
import org.openjdk.jmh.infra.Blackhole;
import org.partiql.eval.Mode;
import org.partiql.eval.compiler.PartiQLCompiler;
import org.partiql.parser.PartiQLParser;
import org.partiql.plan.Plan;
import org.partiql.planner.PartiQLPlanner;
import org.partiql.spi.catalog.Catalog;
import org.partiql.spi.catalog.Identifier;
import org.partiql.spi.catalog.Name;
import org.partiql.spi.catalog.Session;
import org.partiql.spi.catalog.Table;
import org.partiql.spi.function.AggOverload;
import org.partiql.spi.function.FnOverload;
import org.partiql.spi.value.Datum;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

/**
 * JMH benchmark for partiql-eval query execution.
 * Measures execution time for: SELECT a, b FROM data WHERE a % 2 = 0
 */
@BenchmarkMode(org.openjdk.jmh.annotations.Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Warmup(iterations = 5, time = 1)
@Measurement(iterations = 10, time = 1)
@Fork(2)
@State(Scope.Benchmark)
public class PartiQLEvalBenchmark {
    
    // @Param({"100", "1000", "10000"})
    @Param({"1"})
    private int rowCount;
    
    private String[] fieldNames = {"a", "b"};
    private List<Map<String, Integer>> backingData;
    private Datum backingDatum;
    private PartiQLCompiler compiler;
    private Plan plan;
    private Session session;
    
    @Setup(Level.Trial)
    public void setup() {
        try {
            // Create a bag of the generated data
            BenchmarkDataGenerator.generateHashMapRow(fieldNames, rowCount);
            backingData = BenchmarkDataGenerator.generateHashMapRows(fieldNames, 0, rowCount);
            backingDatum = createBackingDatum();
            
            // Create catalog
            Catalog catalog = new MutableCatalog("memory", backingDatum);
            
            // Create session
            session = Session.builder()
                .catalog("memory")
                .catalogs(catalog)
                .build();
            
            // Parse and plan the query
            String query = "SELECT a, b FROM data WHERE a % 2 = 0";
            PartiQLParser parser = PartiQLParser.standard();
            var parseResult = parser.parse(query);
            var statement = parseResult.statements.get(0);
            
            PartiQLPlanner planner = PartiQLPlanner.standard();
            plan = planner.plan(statement, session).getPlan();
            
            // Create compiler
            compiler = PartiQLCompiler.standard();
        } catch (Exception e) {
            throw new RuntimeException("Setup failed", e);
        }
    }

    @Setup(Level.Invocation)
    public void setupInvocation() {
        backingData = BenchmarkDataGenerator.generateHashMapRows(fieldNames, 0, rowCount);
    }

    @Benchmark
    public int executeQuery(Blackhole blackhole) {
        int count = 0;
        
        try {
            // Execute query
            backingDatum = createBackingDatum();
            Datum result = compiler.prepare(plan, Mode.STRICT()).execute();
            
            // Iterate through results
            for (Datum row : result) {
                // Consume the fields to prevent JIT optimization
                blackhole.consume(row.get("a"));
                blackhole.consume(row.get("b"));
                count++;
            }
        } catch (Exception e) {
            throw new RuntimeException("Execution failed", e);
        }
        
        return count;
    }

    Datum createBackingDatum() {
        List<Datum> elements = new ArrayList<>();
        for (Map<String, Integer> entry : backingData) {
            elements.add(new BenchmarkDataGenerator.DatumWrapper(entry));
        }
        return Datum.bag(elements);
    }

    static class MutableCatalog implements Catalog {
        private Datum delegate;
        private final String catalogName;

        MutableCatalog(String catalogName, Datum delegate) {
            this.delegate = delegate;
            this.catalogName = catalogName;
        }

        @Override
        public String getName() {
            return this.catalogName;
        }

        @Override
        public Table getTable(Session arg0, Name arg1) {
            return Table.standard(Name.of("data"), delegate);
        }

        @Override
        public Name resolveTable(Session arg0, Identifier arg1) {
            if (arg1.matches("data", false)) {
                return Name.of("data");
            }
            return null;
        }

        @Override
        public Collection<FnOverload> getFunctions(Session arg0, String arg1) {
            return new ArrayList<>();
        }

        @Override
        public Collection<AggOverload> getAggregations(Session arg0, String arg1) {
            return new ArrayList<>();
        }
    }
}
