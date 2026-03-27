package com.example.benchmark;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.within;

class TransactionProcessorTest {

    private TransactionProcessor processor;

    @BeforeEach
    void setUp() {
        processor = new TransactionProcessor();
    }

    private Transaction txn(String currency, double amount, String status) {
        Transaction t = new Transaction();
        t.setTransactionId("TXN-0000000001");
        t.setCurrency(currency);
        t.setAmount(amount);
        t.setStatus(status);
        t.setTimestamp("2024-06-15T12:00:00Z");
        t.setAccountFrom("ACC-00000001");
        t.setAccountTo("ACC-00000002");
        return t;
    }

    @Test
    void shouldConvertUsdToEur() throws Exception {
        Transaction result = processor.process(txn("USD", 1000.0, "COMPLETED"));
        assertThat(result.getAmountEur())
            .as("USD 1000 * 0.92 = EUR 920")
            .isCloseTo(920.0, within(1e-9));
    }

    @Test
    void shouldConvertGbpToEur() throws Exception {
        Transaction result = processor.process(txn("GBP", 100.0, "COMPLETED"));
        assertThat(result.getAmountEur())
            .as("GBP 100 * 1.17 = EUR 117")
            .isCloseTo(117.0, within(1e-9));
    }

    @Test
    void shouldKeepEurUnchanged() throws Exception {
        Transaction result = processor.process(txn("EUR", 500.0, "PENDING"));
        assertThat(result.getAmountEur())
            .as("EUR passthrough: rate = 1.0")
            .isCloseTo(500.0, within(1e-9));
    }

    @Test
    void shouldNormaliseCancelledToFailed() throws Exception {
        Transaction result = processor.process(txn("EUR", 100.0, "CANCELLED"));
        assertThat(result.getStatus())
            .as("CANCELLED must be mapped to FAILED")
            .isEqualTo("FAILED");
    }

    @Test
    void shouldPreserveOtherStatuses() throws Exception {
        for (String s : new String[]{"PENDING", "COMPLETED", "FAILED"}) {
            Transaction result = processor.process(txn("EUR", 100.0, s));
            assertThat(result.getStatus())
                .as("status '%s' must not be changed", s)
                .isEqualTo(s);
        }
    }

    @Test
    void shouldRoundAmountEurToTwoDecimals() throws Exception {
        // 333.33 * 0.92 = 306.6636 → 306.66
        Transaction result = processor.process(txn("USD", 333.33, "COMPLETED"));
        assertThat(result.getAmountEur())
            .as("amount_eur must be rounded to 2 decimals")
            .isCloseTo(306.66, within(1e-9));
    }
}
