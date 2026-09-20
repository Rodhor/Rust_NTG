CREATE TABLE IF NOT EXISTS sftp_config
(
    config_id           INTEGER PRIMARY KEY,
    name                TEXT    NOT NULL,
    host                TEXT    NOT NULL,
    port                INTEGER NOT NULL DEFAULT 22,
    username            TEXT    NOT NULL,
    password_ciphertext TEXT    NOT NULL,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    deleted             BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at          TEXT
);

CREATE UNIQUE INDEX ui_sftp_config_name
    ON sftp_config (name) WHERE deleted = FALSE;
