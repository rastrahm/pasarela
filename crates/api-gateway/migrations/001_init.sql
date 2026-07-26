-- API Gateway — esquema inicial (MERCHANT, TRANSACTION, SETTLEMENT, GATEWAY_AUDIT_LOG)

DO $$ BEGIN
    CREATE TYPE gateway_funding_type AS ENUM ('traditional_bank', 'binance_cex', 'solana_wallet');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE transaction_status AS ENUM (
        'pending',
        'authorized',
        'held',
        'settled',
        'failed',
        'reversed'
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE settlement_status AS ENUM ('completed', 'failed');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE api_key_env AS ENUM ('test', 'live');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS merchant (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL DEFAULT 'default',
    default_currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    api_key_hash VARCHAR(128) NOT NULL UNIQUE,
    api_key_env api_key_env NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gateway_transaction (
    id UUID PRIMARY KEY,
    merchant_id UUID NOT NULL REFERENCES merchant (id),
    status transaction_status NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    funding_type gateway_funding_type,
    settlement_proof TEXT,
    oracle_hold_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gateway_transaction_merchant_id
    ON gateway_transaction (merchant_id);

CREATE INDEX IF NOT EXISTS idx_gateway_transaction_status
    ON gateway_transaction (status);

CREATE TABLE IF NOT EXISTS settlement (
    id UUID PRIMARY KEY,
    transaction_id UUID NOT NULL REFERENCES gateway_transaction (id),
    rail_type gateway_funding_type NOT NULL,
    proof TEXT NOT NULL,
    status settlement_status NOT NULL,
    settled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_settlement_transaction_id
    ON settlement (transaction_id);

CREATE TABLE IF NOT EXISTS gateway_audit_log (
    id UUID PRIMARY KEY,
    transaction_id UUID REFERENCES gateway_transaction (id),
    event_type VARCHAR(64) NOT NULL,
    detail TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gateway_audit_log_transaction_id
    ON gateway_audit_log (transaction_id);

CREATE TABLE IF NOT EXISTS idempotency_record (
    merchant_id UUID NOT NULL REFERENCES merchant (id),
    idempotency_key VARCHAR(256) NOT NULL,
    request_fingerprint TEXT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'in_flight'
        CHECK (status IN ('in_flight', 'completed')),
    response_status SMALLINT NOT NULL DEFAULT 0,
    response_body JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (merchant_id, idempotency_key)
);
