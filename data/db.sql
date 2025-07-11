CREATE TABLE IF NOT EXISTS "users" (
    "id" INTEGER NOT NULL UNIQUE,
    "name" VARCHAR NOT NULL UNIQUE,
    "psw_hash" VARCHAR NOT NULL,
    "email" VARCHAR,
    "cpf" VARCHAR,
    PRIMARY KEY("id")
);
