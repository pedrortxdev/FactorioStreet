-- Factorio2 Server Schema
-- Run: psql -U postgres -d factorio2 -f schema.sql

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS regions (
    id TEXT PRIMARY KEY,
    state JSONB DEFAULT '{}',
    throughput JSONB DEFAULT '{}',
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS player_state (
    user_id INT REFERENCES users(id),
    region_id TEXT DEFAULT 'rs_sul',
    money BIGINT DEFAULT 5000,
    inventory JSONB DEFAULT '{}',
    grid JSONB DEFAULT '{}',
    x REAL DEFAULT 0.0,
    y REAL DEFAULT 0.0,
    last_seen TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (user_id, region_id)
);

-- Seed regions
INSERT INTO regions (id) VALUES ('rs_sul'), ('sp_capital'), ('mg_gerais')
ON CONFLICT DO NOTHING;
