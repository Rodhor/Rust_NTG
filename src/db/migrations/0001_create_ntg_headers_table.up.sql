CREATE TABLE IF NOT EXISTS ntg_column_specs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    ntg_header VARCHAR(50) NOT NULL,
    original_header VARCHAR(255),
    content_type TEXT NOT NULL DEFAULT 'an' CHECK (content_type IN ('an', 'n')),
    content_length INT,
    column_index INT NOT NULL,
    required BOOLEAN NOT NULL DEFAULT FALSE,
    deprecated BOOLEAN NOT NULL DEFAULT FALSE,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
)
