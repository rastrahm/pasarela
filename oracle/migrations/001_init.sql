-- Oracle de autorización — esquema inicial (HOLD, AUTHORIZATION_REQUEST, ORACLE_AUDIT_LOG)

DO $$ BEGIN
    CREATE TYPE funding_type AS ENUM ('traditional_bank', 'binance_cex', 'solana_wallet');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE hold_status AS ENUM ('active', 'consumed', 'released', 'expired');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE auth_result AS ENUM ('approved', 'rejected');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE card_brand AS ENUM ('visa', 'mastercard', 'amex');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS authorization_request (
    id UUID PRIMARY KEY,
    gateway_request_id UUID NOT NULL,
    funding_type funding_type NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    brand card_brand NOT NULL,
    brand_code SMALLINT NOT NULL,
    card_token_hash VARCHAR(128) NOT NULL,
    result auth_result NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_authorization_request_gateway_id
    ON authorization_request (gateway_request_id);

CREATE TABLE IF NOT EXISTS hold (
    id UUID PRIMARY KEY,
    authorization_request_id UUID NOT NULL REFERENCES authorization_request (id),
    funding_type funding_type NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status hold_status NOT NULL DEFAULT 'active',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_hold_status_expires
    ON hold (status, expires_at)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS idx_hold_funding_active
    ON hold (funding_type)
    WHERE status = 'active';

CREATE TABLE IF NOT EXISTS oracle_audit_log (
    id UUID PRIMARY KEY,
    hold_id UUID REFERENCES hold (id),
    authorization_request_id UUID REFERENCES authorization_request (id),
    event_type VARCHAR(64) NOT NULL,
    detail TEXT NOT NULL,
    caller_ip INET,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_oracle_audit_log_hold_id ON oracle_audit_log (hold_id);
CREATE INDEX IF NOT EXISTS idx_oracle_audit_log_occurred_at ON oracle_audit_log (occurred_at);
