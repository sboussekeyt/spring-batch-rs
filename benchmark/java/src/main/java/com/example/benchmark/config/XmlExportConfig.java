package com.example.benchmark.config;

import com.example.benchmark.Transaction;
import org.springframework.batch.core.step.Step;
import org.springframework.batch.core.repository.JobRepository;
import org.springframework.batch.core.step.builder.StepBuilder;
import org.springframework.batch.infrastructure.item.database.JdbcPagingItemReader;
import org.springframework.batch.infrastructure.item.database.Order;
import org.springframework.batch.infrastructure.item.database.builder.JdbcPagingItemReaderBuilder;
import org.springframework.batch.infrastructure.item.xml.StaxEventItemWriter;
import org.springframework.batch.infrastructure.item.xml.builder.StaxEventItemWriterBuilder;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.io.FileSystemResource;
import org.springframework.oxm.jaxb.Jaxb2Marshaller;
import org.springframework.transaction.PlatformTransactionManager;

import javax.sql.DataSource;
import java.util.Map;

/**
 * Step 2 configuration: reads all transactions from PostgreSQL (paginated)
 * and writes to an XML file using JAXB marshalling (chunk size = 1 000).
 *
 * <p>This is the Java equivalent of the Rust {@code run_step2} function.
 */
@Configuration
public class XmlExportConfig {

    @Value("${benchmark.xml.path:#{systemProperties['java.io.tmpdir']}/transactions_export.xml}")
    private String xmlPath;

    /**
     * Reads transactions from PostgreSQL using keyset-based pagination.
     *
     * <p>Page size 1 000 — same as the Rust benchmark's {@code with_page_size(1_000)}.
     */
    @Bean
    public JdbcPagingItemReader<Transaction> postgresReader(DataSource dataSource) throws Exception {
        return new JdbcPagingItemReaderBuilder<Transaction>()
            .name("postgresTransactionReader")
            .dataSource(dataSource)
            .selectClause("SELECT transaction_id, amount, currency, timestamp, " +
                          "account_from, account_to, status, amount_eur")
            .fromClause("FROM transactions")
            .sortKeys(Map.of("transaction_id", Order.ASCENDING))
            .rowMapper((rs, rowNum) -> {
                Transaction t = new Transaction();
                t.setTransactionId(rs.getString("transaction_id"));
                t.setAmount(rs.getDouble("amount"));
                t.setCurrency(rs.getString("currency"));
                t.setTimestamp(rs.getString("timestamp"));
                t.setAccountFrom(rs.getString("account_from"));
                t.setAccountTo(rs.getString("account_to"));
                t.setStatus(rs.getString("status"));
                t.setAmountEur(rs.getDouble("amount_eur"));
                return t;
            })
            .pageSize(1_000)
            .build();
    }

    /**
     * Configures the JAXB marshaller for {@link Transaction} XML serialisation.
     */
    @Bean
    public Jaxb2Marshaller jaxb2Marshaller() throws Exception {
        Jaxb2Marshaller marshaller = new Jaxb2Marshaller();
        marshaller.setClassesToBeBound(Transaction.class);
        marshaller.afterPropertiesSet();
        return marshaller;
    }

    /**
     * Writes transactions to an XML file with {@code <transactions>} root
     * and {@code <transaction>} item tags.
     */
    @Bean
    public StaxEventItemWriter<Transaction> xmlWriter(Jaxb2Marshaller marshaller) {
        return new StaxEventItemWriterBuilder<Transaction>()
            .name("transactionXmlWriter")
            .resource(new FileSystemResource(xmlPath))
            .marshaller(marshaller)
            .rootTagName("transactions")
            .build();
    }

    /**
     * Step 2: PostgreSQL → XML.
     *
     * <p>Chunk size 1 000 — same as the Rust benchmark for fair comparison.
     */
    @Bean
    public Step step2(JobRepository jobRepository,
                      PlatformTransactionManager transactionManager,
                      JdbcPagingItemReader<Transaction> postgresReader,
                      StaxEventItemWriter<Transaction> xmlWriter) {
        return new StepBuilder("postgrestoXmlStep", jobRepository)
            .<Transaction, Transaction>chunk(1_000, transactionManager)
            .reader(postgresReader)
            .writer(xmlWriter)
            .build();
    }
}
