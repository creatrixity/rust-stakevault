-- Create Accounts table

CREATE TABLE accounts (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    username TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at timestamptz NOT NULL
)
