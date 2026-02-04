package org.partiql.jni.benchmark;

import com.amazon.ion.IonValue;
import org.openjdk.jmh.annotations.*;
import org.openjdk.jmh.infra.Blackhole;
import org.partiql.eval.Mode;
import org.partiql.eval.compiler.PartiQLCompiler;
import org.partiql.parser.PartiQLParser;
import org.partiql.plan.Plan;
import org.partiql.planner.PartiQLPlanner;
import org.partiql.spi.catalog.Catalog;
import org.partiql.spi.catalog.Name;
import org.partiql.spi.catalog.Session;
import org.partiql.spi.catalog.Table;
import org.partiql.spi.value.Datum;
import org.partiql.spi.types.PType;

import java.util.ArrayList;
import java.util.List;
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
    @Param({"100"})
    private int rowCount;
    
    private PartiQLCompiler compiler;
    private Plan plan;
    private Session session;
    
    @Setup(Level.Trial)
    public void setup() {
        try {
            // Generate test data as Datum rows
            List<Datum> datumList = BenchmarkDataGenerator.generateDatumRows(rowCount);
            
            // Create a bag of the data
            Datum dataBag = Datum.bag(datumList);
            
            // Create table
            Table dataTable = Table.standard(
                Name.of("data"),
                PType.dynamic(),
                dataBag
            );
            
            // Create catalog
            Catalog catalog = Catalog.builder()
                .name("memory")
                .define(dataTable)
                .build();
            
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
    
    @Benchmark
    public int executeQuery(Blackhole blackhole) {
        int count = 0;
        
        try {
            // Execute query
            Datum result = compiler.prepare(plan, Mode.PERMISSIVE()).execute();
            
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
}
