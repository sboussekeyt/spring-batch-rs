CREATE TABLE IF NOT EXISTS transactions (
    transaction_id  VARCHAR(36)       PRIMARY KEY,
    amount          DOUBLE PRECISION  NOT NULL,
    currency        VARCHAR(3)        NOT NULL,
    timestamp       VARCHAR(25)       NOT NULL,
    account_from    VARCHAR(15)       NOT NULL,
    account_to      VARCHAR(15)       NOT NULL,
    status          VARCHAR(15)       NOT NULL,
    amount_eur      DOUBLE PRECISION  NOT NULL DEFAULT 0.0
);
