package com.example.benchmark;

import org.springframework.batch.core.Job;
import org.springframework.batch.core.JobExecution;
import org.springframework.batch.core.JobParametersBuilder;
import org.springframework.batch.core.StepExecution;
import org.springframework.batch.core.job.builder.JobBuilder;
import org.springframework.batch.core.launch.JobLauncher;
import org.springframework.batch.core.repository.JobRepository;
import org.springframework.batch.core.Step;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.ApplicationRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;

import java.time.temporal.ChronoUnit;

/**
 * Entry point for the Spring Batch Java benchmark.
 *
 * <p>Runs a two-step ETL pipeline:
 * <ol>
 *   <li>Step 1 — reads 10M transactions from CSV, converts currencies to EUR,
 *       normalises statuses, and bulk-inserts into PostgreSQL (chunk = 1 000)</li>
 *   <li>Step 2 — reads PostgreSQL and exports to XML (chunk = 1 000)</li>
 * </ol>
 *
 * <p>Run with:
 * <pre>
 * mvn spring-boot:run \
 *   -Dspring-boot.run.jvmArguments="-Xms512m -Xmx4g -XX:+UseG1GC -Xlog:gc*:gc.log" \
 *   -Dspring-boot.run.arguments="--benchmark.csv.path=/tmp/transactions.csv"
 * </pre>
 */
@SpringBootApplication
public class BenchmarkApplication {

    private static final long TOTAL_RECORDS = 10_000_000L;

    @Value("${benchmark.csv.path:#{systemProperties['java.io.tmpdir']}/transactions.csv}")
    private String csvPath;

    public static void main(String[] args) {
        SpringApplication.run(BenchmarkApplication.class, args);
    }

    /**
     * Defines the benchmark job: Step 1 (CSV → PostgreSQL) then Step 2 (PostgreSQL → XML).
     */
    @Bean
    public Job benchmarkJob(JobRepository jobRepository, Step step1, Step step2) {
        return new JobBuilder("transactionBenchmarkJob", jobRepository)
            .start(step1)
            .next(step2)
            .build();
    }

    /**
     * Runs the benchmark on application startup:
     * generates CSV, executes both steps, and prints a metrics summary.
     */
    @Bean
    public ApplicationRunner benchmarkRunner(JobLauncher jobLauncher, Job benchmarkJob) {
        return args -> {
            System.err.println("╔══════════════════════════════════════════════════════════╗");
            System.err.println("║  Spring Batch Java — 10M Transaction Benchmark          ║");
            System.err.println("╚══════════════════════════════════════════════════════════╝");
            System.err.println();

            // Generate CSV data
            System.err.printf("[Generate] Writing %,d rows to %s …%n", TOTAL_RECORDS, csvPath);
            long genStart = System.currentTimeMillis();
            DataGenerator.generate(csvPath, TOTAL_RECORDS);
            System.err.printf("[Generate] Done in %.1fs%n%n",
                (System.currentTimeMillis() - genStart) / 1000.0);

            // Run batch job and measure wall time
            long jobStart = System.currentTimeMillis();
            JobExecution execution = jobLauncher.run(
                benchmarkJob,
                new JobParametersBuilder()
                    .addLong("run.id", System.currentTimeMillis())
                    .toJobParameters()
            );

            long totalMs = System.currentTimeMillis() - jobStart;

            // Print per-step metrics
            for (StepExecution step : execution.getStepExecutions()) {
                long stepMs = ChronoUnit.MILLIS.between(
                    step.getStartTime(), step.getEndTime());
                double throughput = stepMs > 0
                    ? step.getWriteCount() / (stepMs / 1000.0)
                    : 0;
                System.err.printf("[%s] read=%,d  write=%,d  skip=%d  duration=%.1fs  throughput=%.0f rec/s%n",
                    step.getStepName(),
                    step.getReadCount(),
                    step.getWriteCount(),
                    step.getSkipCount(),
                    stepMs / 1000.0,
                    throughput);
                if (step.getSkipCount() > 0) {
                    System.err.printf("[%s] WARNING: %d records skipped — throughput may be understated%n",
                        step.getStepName(), step.getSkipCount());
                }
            }

            System.err.println();
            System.err.println("╔══════════════════════════════════════════════════════════╗");
            System.err.println("║  BENCHMARK SUMMARY                                      ║");
            System.err.println("╠══════════════════════════════════════════════════════════╣");
            System.err.printf( "║  Job status              : %s%n", execution.getStatus());
            System.err.printf( "║  Total pipeline duration : %.1fs%n", totalMs / 1000.0);
            System.err.printf( "║  Records processed       : %,d%n", TOTAL_RECORDS);
            System.err.printf( "║  Average throughput      : %.0f rec/s%n",
                totalMs > 0 ? TOTAL_RECORDS / (totalMs / 1000.0) : 0);
            System.err.println("╚══════════════════════════════════════════════════════════╝");
            System.err.println();
            System.err.println("Hint: measure peak heap with:");
            System.err.println("  mvn spring-boot:run -Dspring-boot.run.jvmArguments=\"" +
                               "-Xms512m -Xmx4g -XX:+UseG1GC -Xlog:gc*:gc.log\"");
        };
    }
}
