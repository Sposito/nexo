-- sqlite3 db.sql

CREATE TABLE IF NOT EXISTS "users" (
    "id" INTEGER NOT NULL UNIQUE,
    "name" VARCHAR NOT NULL UNIQUE,
    "psw_hash" VARCHAR NOT NULL,
    "email" VARCHAR,
    "cpf" VARCHAR,
    PRIMARY KEY("id")
);

INSERT INTO "users" (
    "id", "name", "psw_hash", "email", "cpf")
SELECT
    1,
    'thiago',
    'ea32961dbd579ef5697c367f9267921ee07f14d77fb2d4fb9500d4221d615695',
    'thiago@thiago.com',
    '12345678909'
WHERE NOT EXISTS (
    SELECT 1 FROM "users" WHERE "name" = 'thiago'
);

CREATE TABLE IF NOT EXISTS "sessions" (
    "id" INTEGER NOT NULL UNIQUE,
    "user_id" INTEGER NOT NULL,
    "token" VARCHAR NOT NULL,
    "expires_at" INTEGER NOT NULL,
    PRIMARY KEY("id")
);
