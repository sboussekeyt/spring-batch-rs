package com.example.benchmark;

import org.springframework.batch.item.ItemProcessor;
import org.springframework.stereotype.Component;

import java.util.Map;

/**
 * Converts transaction amounts to EUR and normalises status values.
 *
 * <p>Conversion rates (fixed for benchmark reproducibility):
 * <ul>
 *   <li>USD → EUR: × 0.92</li>
 *   <li>GBP → EUR: × 1.17</li>
 *   <li>EUR → EUR: × 1.00</li>
 * </ul>
 *
 * <p>Status normalisation: {@code CANCELLED} is mapped to {@code FAILED}.
 */
@Component
public class TransactionProcessor implements ItemProcessor<Transaction, Transaction> {

    private static final Map<String, Double> RATES = Map.of(
        "USD", 0.92,
        "GBP", 1.17,
        "EUR", 1.0
    );

    @Override
    public Transaction process(Transaction item) {
        double rate = RATES.getOrDefault(item.getCurrency(), 1.0);
        double amountEur = Math.round(item.getAmount() * rate * 100.0) / 100.0;
        item.setAmountEur(amountEur);

        if ("CANCELLED".equals(item.getStatus())) {
            item.setStatus("FAILED");
        }

        return item;
    }
}
