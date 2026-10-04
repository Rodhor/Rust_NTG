set shell := ["bash", "-uc"]

source := "src/db/migrations"
compose := "docker compose -f docker/docker-compose.yml"
dc := "newgrp docker -c"

default:
    @just --list

# ── Docker ────────────────────────────────────────────────────────────────────
docker-up:
    @echo "Starting Docker..."
    {{ dc }} "{{ compose }} up --wait"
    @echo "Docker is up!"

docker-down:
    @echo "Stopping Docker..."
    {{ dc }} "{{ compose }} stop"
    @echo "Docker is down!"

# ── Migrations ────────────────────────────────────────────────────────────────
new-migration name:
    @echo "Creating a reversible migration named '{{ name }}'"
    sqlx migrate add {{ name }} --source {{ source }} --sequential --reversible
    @echo
    @echo "Fill in the pair, then: just migrate"

new-migration-simple name:
    @echo "Creating an up-only migration named '{{ name }}' (NOT revertable)"
    sqlx migrate add {{ name }} --source {{ source }} --sequential --simple
    @echo
    @echo "Fill it in, then: just migrate"

migrate:
    sqlx migrate run --source {{ source }}

migrate-dry-run:
    sqlx migrate run --source {{ source }} --dry-run

migrate-info:
    sqlx migrate info --source {{ source }}

migrate-revert:
    @read -r -p "Revert the last migration? This destroys data. [y/N] " a; if [ "$a" = "y" ]; then sqlx migrate revert --source {{ source }}; else echo "aborted"; fi

# ── Database lifecycle ────────────────────────────────────────────────────────
db-up:
    {{ dc }} "{{ compose }} up -d --wait postgres"

db-down:
    {{ dc }} "{{ compose }} stop postgres"

db-reset: db-down
    {{ dc }} "{{ compose }} down -v"
    @just db-up
    sqlx migrate run --source {{ source }}
    @echo "Rebuilt from scratch. Now run: just prepare"

db-shell:
    {{ dc }} "{{ compose }} exec postgres psql -U rust_ntg -d rust_ntg"

#   just logs                # both services, interleaved
#   just logs postgres       # one service
# just logs sftp --tail 20
logs *args:
    {{ dc }} "{{ compose }} logs -f {{ args }}"

# ── SQLx compile-time queries ─────────────────────────────────────────────────
prepare:
    cargo sqlx prepare -- --all-targets

check-sqlx:
    SQLX_OFFLINE=true cargo sqlx prepare --check -- --all-targets

sqlx *args:
    cargo sqlx {{ args }}

# ── Build ─────────────────────────────────────────────────────────────────────
check: check-sqlx
    cargo clippy

run: check
    cargo run

build: check
    cargo build

test: check
    cargo test
