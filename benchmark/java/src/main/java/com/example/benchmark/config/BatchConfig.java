package com.example.benchmark.config;

import com.example.benchmark.Transaction;
import com.example.benchmark.TransactionProcessor;
import org.springframework.batch.core.Step;
import org.springframework.batch.core.repository.JobRepository;
import org.springframework.batch.core.step.builder.StepBuilder;
import org.springframework.batch.item.database.JdbcBatchItemWriter;
import org.springframework.batch.item.database.builder.JdbcBatchItemWriterBuilder;
import org.springframework.batch.item.file.FlatFileItemReader;
import org.springframework.batch.item.file.builder.FlatFileItemReaderBuilder;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.io.FileSystemResource;
import org.springframework.transaction.PlatformTransactionManager;

import javax.sql.DataSource;

/**
 * Step 1 configuration: reads 10M transactions from CSV and bulk-inserts
 * into PostgreSQL using chunk-oriented processing (chunk size = 1 000).
 *
 * <p>This is the Java equivalent of the Rust {@code run_step1} function.
 */
@Configuration
public class BatchConfig {

    @Value("${benchmark.csv.path:#{systemProperties['java.io.tmpdir']}/transactions.csv}")
    private String csvPath;

    /**
     * Reads financial transactions from a CSV file.
     *
     * <p>Skips the first header row and maps each subsequent row to a
     * {@link Transaction} using Spring Batch's built-in bean-wrapper field mapper.
     */
    @Bean
    public FlatFileItemReader<Transaction> csvReader() {
        return new FlatFileItemReaderBuilder<Transaction>()
            .name("transactionCsvReader")
            .resource(new FileSystemResource(csvPath))
            .linesToSkip(1)  // skip header row
            .delimited()
            .delimiter(",")
            .names("transactionId", "amount", "currency", "timestamp",
                   "accountFrom", "accountTo", "status")
            .targetType(Transaction.class)
            .build();
    }

    /**
     * Writes transactions to PostgreSQL using batch INSERT statements.
     *
     * <p>Uses named-parameter SQL mapped from JavaBean properties via
     * {@code beanMapped()}, leveraging HikariCP with pool size = 10.
     */
    @Bean
    public JdbcBatchItemWriter<Transaction> postgresWriter(DataSource dataSource) {
        return new JdbcBatchItemWriterBuilder<Transaction>()
            .dataSource(dataSource)
            .sql("INSERT INTO transactions " +
                 "(transaction_id, amount, currency, timestamp, " +
                 " account_from, account_to, status, amount_eur) " +
                 "VALUES " +
                 "(:transactionId, :amount, :currency, :timestamp, " +
                 " :accountFrom, :accountTo, :status, :amountEur)")
            .beanMapped()
            .build();
    }

    /**
     * Step 1: CSV → PostgreSQL.
     *
     * <p>Chunk size 1 000 — same as the Rust benchmark for fair comparison.
     */
    @Bean
    public Step step1(JobRepository jobRepository,
                      PlatformTransactionManager transactionManager,
                      FlatFileItemReader<Transaction> csvReader,
                      TransactionProcessor processor,
                      JdbcBatchItemWriter<Transaction> postgresWriter) {
        return new StepBuilder("csvToPostgresStep", jobRepository)
            .<Transaction, Transaction>chunk(1_000, transactionManager)
            .reader(csvReader)
            .processor(processor)
            .writer(postgresWriter)
            .build();
    }
}
