CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE mantle_users (
  id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TYPE identity_provider AS ENUM (
    'discord',
    'external'
);

CREATE TABLE identities (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    mantle_user_id UUID NOT NULL REFERENCES mantle_users(id) ON DELETE CASCADE,

    provider identity_provider NOT NULL,
    provider_user_id TEXT NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE (provider, provider_user_id)
);

CREATE TABLE oauth_tokens (
    identity_id UUID PRIMARY KEY REFERENCES identities(id),
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE identity_cache (
    identity_id UUID NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,

    PRIMARY KEY (identity_id, key)
);
