package com.example.benchmark;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import jakarta.xml.bind.annotation.XmlAccessType;
import jakarta.xml.bind.annotation.XmlAccessorType;
import jakarta.xml.bind.annotation.XmlRootElement;

/**
 * Financial transaction entity used for both database persistence (JPA)
 * and XML serialisation (JAXB).
 */
@Entity
@Table(name = "transactions")
@XmlRootElement(name = "transaction")
@XmlAccessorType(XmlAccessType.FIELD)
public class Transaction {

    @Id
    @Column(name = "transaction_id")
    private String transactionId;

    private double amount;
    private String currency;
    private String timestamp;

    @Column(name = "account_from")
    private String accountFrom;

    @Column(name = "account_to")
    private String accountTo;

    private String status;

    @Column(name = "amount_eur")
    private double amountEur;

    // --- Getters and setters ---

    public String getTransactionId() { return transactionId; }
    public void setTransactionId(String transactionId) { this.transactionId = transactionId; }

    public double getAmount() { return amount; }
    public void setAmount(double amount) { this.amount = amount; }

    public String getCurrency() { return currency; }
    public void setCurrency(String currency) { this.currency = currency; }

    public String getTimestamp() { return timestamp; }
    public void setTimestamp(String timestamp) { this.timestamp = timestamp; }

    public String getAccountFrom() { return accountFrom; }
    public void setAccountFrom(String accountFrom) { this.accountFrom = accountFrom; }

    public String getAccountTo() { return accountTo; }
    public void setAccountTo(String accountTo) { this.accountTo = accountTo; }

    public String getStatus() { return status; }
    public void setStatus(String status) { this.status = status; }

    public double getAmountEur() { return amountEur; }
    public void setAmountEur(double amountEur) { this.amountEur = amountEur; }
}
