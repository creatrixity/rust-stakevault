-- Create Subscriptions table

CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at timestamptz NOT NULL
)
