-- Add migration script here
-- Add missing column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS title VARCHAR(255);

-- Create missing media table
CREATE TABLE IF NOT EXISTS media (
    id UUID PRIMARY KEY,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_data BYTEA NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);