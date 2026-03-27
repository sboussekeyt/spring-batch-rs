package com.example.benchmark;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

class DataGeneratorTest {

    @Test
    void shouldGenerateCorrectHeaderAndRowCount(@TempDir Path tempDir) throws IOException {
        Path csv = tempDir.resolve("test.csv");
        DataGenerator.generate(csv.toString(), 5);

        try (BufferedReader reader = new BufferedReader(new FileReader(csv.toFile()))) {
            String header = reader.readLine();
            assertThat(header).isEqualTo(
                "transaction_id,amount,currency,timestamp,account_from,account_to,status"
            );
            long rows = reader.lines().count();
            assertThat(rows).isEqualTo(5L);
        }
    }

    @Test
    void shouldGenerateValidCurrencyAndStatusValues(@TempDir Path tempDir) throws IOException {
        Path csv = tempDir.resolve("curr_test.csv");
        DataGenerator.generate(csv.toString(), 100);

        List<String> validCurrencies = Arrays.asList("USD", "EUR", "GBP");
        List<String> validStatuses   = Arrays.asList("PENDING", "COMPLETED", "FAILED", "CANCELLED");

        try (BufferedReader reader = new BufferedReader(new FileReader(csv.toFile()))) {
            reader.readLine(); // skip header
            reader.lines().forEach(line -> {
                String[] fields = line.split(",");
                assertThat(fields[2])
                    .as("currency must be one of USD/EUR/GBP")
                    .isIn(validCurrencies);
                assertThat(fields[6])
                    .as("status must be one of PENDING/COMPLETED/FAILED/CANCELLED")
                    .isIn(validStatuses);
            });
        }
    }
}
