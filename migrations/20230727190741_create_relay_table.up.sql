CREATE TABLE relays (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    name TEXT,
    description TEXT,
    pubkey TEXT,
    contact TEXT,
    supported_nips JSONB,
    software TEXT,
    version TEXT,
    limitation JSONB,
    retention TEXT,
    relay_countries JSONB,
    language_tags JSONB,
    tags JSONB,
    posting_policy TEXT,
    payments_url TEXT,
    fees JSONB,
    icon TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);