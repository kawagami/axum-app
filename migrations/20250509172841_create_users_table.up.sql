-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE, -- 這裡可以維持 UNIQUE，但考慮未來擴展可改為非必填
    display_name TEXT,
    avatar_url TEXT,
    is_active BOOLEAN DEFAULT TRUE, -- 方便暫停帳戶
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_identities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL, 
    provider_user_id TEXT NOT NULL,
    -- 建議增加儲存 OAuth 提供的 Email，有助於帳號救援或身分識別
    provider_email TEXT, 
    access_token TEXT,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

-- 建立索引提升查詢效率
CREATE INDEX idx_user_identities_user_id ON user_identities(user_id);
