-- Migration: Add x, y columns to player_state if they do not exist yet
-- Run this once on the VPS Postgres database

ALTER TABLE player_state
    ADD COLUMN IF NOT EXISTS x REAL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS y REAL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS grid TEXT DEFAULT '{}';

-- Confirm
SELECT column_name, data_type FROM information_schema.columns
WHERE table_name = 'player_state'
ORDER BY ordinal_position;
