# Rust_NTG

An early-stage Rust/Axum service for controlled file operations between a local working directory and a remote SFTP server. The project is being developed as part of a larger data workflow; it currently has no browser-based UI.

## Current scope

- Versioned JSON API under `/api/v1`, including a health endpoint.
- SFTP directory listing, directory creation and deletion, and file upload, download, move, archive, and deletion operations.
- A bounded SFTP session pool, server host-key verification, local/remote path checks, structured logging, and graceful shutdown.
- PostgreSQL connection configuration. CSV import and broader workflow endpoints are still in progress.

See [Docs/roadmap.md](Docs/roadmap.md) for planned phases and [Docs/codebase.md](Docs/codebase.md) for the current implementation map. The source and documentation are evolving; the roadmap distinguishes implemented work from planned work.

## Requirements

- Rust 1.94 or newer (edition 2024).
- A reachable PostgreSQL database and SFTP server.
- Environment variables configured from `.env.example`. The service requires SFTP credentials and a SHA-256 host-key fingerprint; do not commit your populated `.env` file.

A Docker Compose setup for local development is provided under `docker/` and can be started with `just docker-up` (requires Docker and `just`).

## Build and run

After configuring `.env` and starting the required services:

```sh
cargo build
cargo run
```

By default, the API binds to `127.0.0.1:8080`. For example, check its health endpoint with:

```sh
curl http://127.0.0.1:8080/api/v1/health
```

Keep the service bound to loopback unless it is protected by an appropriately configured authenticated TLS reverse proxy. See [Docs/deployment.md](Docs/deployment.md) before deploying.

## License

Apache-2.0. See [Cargo.toml](Cargo.toml).
