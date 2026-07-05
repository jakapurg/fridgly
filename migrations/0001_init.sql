CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE items (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    quantity    TEXT NOT NULL DEFAULT '1',
    category    TEXT,
    expiry_date DATE,
    status      TEXT NOT NULL DEFAULT 'in_fridge',  -- in_fridge | used | tossed
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_items_status_expiry ON items (status, expiry_date);
