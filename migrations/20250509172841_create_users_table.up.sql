-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE,  -- 可 null，但如果有要唯一
    name TEXT NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- linked_accounts table
CREATE TABLE linked_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    provider TEXT NOT NULL,  -- 'google', 'github', etc
    provider_user_id TEXT NOT NULL,

    email TEXT,  -- 該 provider 的 email (optional)
    access_token TEXT,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (provider, provider_user_id)
);

-- Optional: 加速 user_id 查詢 (建議加)
CREATE INDEX idx_linked_accounts_user_id ON linked_accounts(user_id);
