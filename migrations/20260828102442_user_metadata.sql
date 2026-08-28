CREATE TABLE user_metadata (
    mantle_user_id UUID NOT NULL REFERENCES mantle_users(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value JSONB NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (mantle_user_id, key)
);
