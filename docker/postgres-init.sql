-- Runs once, on first start, inside the database named by POSTGRES_DB.
-- Docs: https://github.com/docker-library/docs/blob/master/postgres/README.md#initialization-and-startup-process
--
-- ── Why this file exists ──────────────────────────────────────────────────────
-- The obvious setup is to point the app at POSTGRES_USER and be done. That makes
-- the app user a database SUPERUSER in development, and the app's migration
-- privileges are then never actually tested — a migration could use a
-- superuser-only feature and pass here, then fail in production where the role is
-- ordinary. A fixture that cannot distinguish those two cases is the same mistake
-- Docs/docker/README.md calls out for the SFTP fixture: making a broken thing and
-- a working thing look identical.
--
-- So the app gets an ordinary role here, exactly as Docs/data-model.md says it
-- should in production, and the container's superuser is left for administration.

-- The app's login role. Ordinary: no SUPERUSER, no CREATEDB, no BYPASSRLS.
CREATE ROLE rust_ntg LOGIN PASSWORD 'rust-ntg-dev-password'
    NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT;

-- ── The line that is easy to miss ─────────────────────────────────────────────
-- PostgreSQL 15 REMOVED the default CREATE grant on the public schema. A fresh
-- role that owns its database but lacks this grant fails its first migration with
--
--     ERROR: permission denied for schema public
--
-- which reads like a permissions bug in the migration and is actually a
-- PostgreSQL 15 default. Granted here so development matches production, where
-- this is a real DBA decision rather than a default.
GRANT ALL ON SCHEMA public TO rust_ntg;

-- The migrations run as this user, so it needs to create and alter its own
-- objects. That is ordinary DDL, not superuser power.
GRANT ALL ON ALL TABLES IN SCHEMA public TO rust_ntg;
