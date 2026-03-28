package com.example.benchmark;

import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;

/**
 * Generates a CSV file with random financial transactions.
 *
 * <p>Uses the same LCG algorithm as the Rust generator to produce an
 * equivalent distribution of currencies, statuses, and amounts.
 */
public class DataGenerator {

    private static final String[] CURRENCIES = {"USD", "EUR", "GBP"};
    private static final String[] STATUSES   = {"PENDING", "COMPLETED", "FAILED", "CANCELLED"};

    /**
     * Writes {@code count} transaction rows to {@code path}.
     *
     * @param path  output file path
     * @param count number of rows to generate (excluding header)
     * @throws IOException if the file cannot be created or written
     */
    public static void generate(String path, long count) throws IOException {
        try (BufferedWriter writer = new BufferedWriter(new FileWriter(path), 256 * 1024)) {
            writer.write("transaction_id,amount,currency,timestamp,account_from,account_to,status");
            writer.newLine();

            // Same LCG constants as Rust generator for reproducibility
            long seed = 0xDEADBEEFCAFEBABEL;

            for (long i = 0; i < count; i++) {
                seed = seed * 6364136223846793005L + 1442695040888963407L;
                long r1 = (seed >>> 33) & 0xFFFFFFFFL;
                seed = seed * 6364136223846793005L + 1442695040888963407L;
                long r2 = (seed >>> 33) & 0xFFFFFFFFL;

                String currency = CURRENCIES[(int)(r1 % 3)];
                String status   = STATUSES[(int)(r2 % 4)];
                double amount   = ((r1 % 9_999_999) + 100) / 100.0;
                long month = r1 % 12 + 1;
                long day   = r2 % 28 + 1;
                long hour  = r1 % 24;
                long min   = r2 % 60;
                long sec   = r1 % 60;
                long from  = r1 % 1_000_000;
                long to    = r2 % 1_000_000;

                writer.write(String.format(
                    "TXN-%010d,%.2f,%s,2024-%02d-%02dT%02d:%02d:%02dZ,ACC-%08d,ACC-%08d,%s",
                    i + 1, amount, currency,
                    month, day, hour, min, sec,
                    from, to, status
                ));
                writer.newLine();
            }
        }
    }
}
