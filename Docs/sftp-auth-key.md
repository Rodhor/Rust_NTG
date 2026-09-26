# SFTP Authentication — Key-Based Auth

> **Status: Planned — not currently relevant, revisit at a later date.**
> Password auth is what ships first. This doc records the intended upgrade.

## The Goal

Replace the SFTP password in `.env` with a private key, so the internet-facing
SFTP server has no password surface at all.

## Context

The SFTP server is hosted **outside our network**. Password auth on such a server
means running a password-guessing endpoint that is reachable from anywhere,
and it means the credential in `SFTP_PASSWORD` is a reusable plaintext secret
that travels wherever the config is copied.

Trigger to revisit: we want the server to stop accepting passwords, or we want
per-client revocation without rotating a shared password.

## Decision (2026-09-26): ship password auth first

Password auth is implemented now; key auth is deferred. Reasoning:

- The app is a local tool under active development; password auth is what the
  test servers accept (`atmoz/sftp` is password-only).
- The real security win of key auth lives in **server-side `authorized_keys`
  configuration**, not in Rust code. Nothing is lost by deferring the client side.
- Avoid building a credential path we cannot test end-to-end yet.

Cost accepted: the server has a password surface until this is done.

## Intended Path

1. **Now:** `SFTP_PASSWORD` + `SFTP_USERNAME`, host key pinned via
   `SFTP_HOST_KEY_SHA256`.
2. **Later:** generate an ed25519 key pair with `ssh-keygen` (a one-time
   documented setup step — *not* application code), install the public half in
   the server's `authorized_keys`, point config at the private key.
3. **Then:** set `PasswordAuthentication no` on the server. The key becomes the
   only way in.

## Implementation Notes (for when this is built)

Config additions — both optional, key preferred when present:

```rust
pub private_key_path: Option<PathBuf>,
pub private_key_passphrase: Option<String>,
```

Auth becomes a two-branch selection in `sftp::connection::connect`:

- `ssh2::Session::userauth_pubkey_file(&username, None, &key_path, passphrase.as_deref())`
- else `userauth_password(&username, &password)` if a password is set
- else error: neither credential configured

`ssh2` accepts `None` for the public-key path, which makes libssh2 derive the
public key from the private key — so only **one** file needs to exist locally.

`ssh2::Session::userauth_pubkey_memory` is gated behind
`cfg(any(unix, feature = "vendored-openssl", feature = "openssl-on-win32"))` and
this crate enables no default features, so it is **not** available to us. The
file-based call is the only portable option.

## Things to Get Right

- **Do not generate keys in application code.** That needs a crypto dependency
  (`ed25519-dalek` / `ring`) plus hand-written OpenSSH `openssh-key-v1`
  serialization. `ssh2` reads keys, it never writes them.
- **Never auto-generate a missing key.** A "generate if absent" branch would
  silently rotate the credential and break the server-side `authorized_keys`
  entry, failing only at the next unattended run. A missing key must stop the
  app with a clear message.
- **Do not passphrase-protect the key.** A Windows Task Scheduler job runs
  unattended with nobody to type a passphrase, so the passphrase would end up
  in `.env` anyway — a plaintext secret with extra steps. Rely on filesystem
  ACLs instead.
- **Keep the key outside the repo.** It is a plaintext file; being in a
  gitignored `.env` versus a gitignored keyfile makes no difference. Location
  and ACL are the entire mitigation. Use `~/.ssh/id_rust_ntg`.
- **Add `private_key_passphrase` to the redacting `Debug` impl** when the field
  is added. It is the same class of secret as the password.
- **Relax the `password.is_none()` config check** at the same time, or a
  key-only setup will fail startup validation.

## Server-Side Restrictions (the actual win)

This is what key auth buys that password auth cannot express. All server config,
nothing to write in Rust:

```
restrict,from="<internal-network>" command="internal-sftp -d /incoming" ssh-ed25519 AAAA... user@pc
```

- `command="internal-sftp -d /incoming"` pins every session to one directory,
  **enforced by the server** — it holds even if the API is later exposed.
- `restrict` disables pty, port forwarding, agent forwarding, X11, and `~/.ssh/rc`.
- `from="<cidr>"` restricts source networks.

## Alternatives Considered

- **SSH agent (`userauth_agent`)** — the private key never touches the app. But
  it requires an agent running as a service account, which is its own
  operational burden on Windows. Revisit if the key file's ACLs become a
  concern.
- **Certificate-based auth** — better revocation at scale. Unnecessary for one
  client on one server.
- **`ssh2::Session::userauth_password` kept permanently** — acceptable only if
  the server is unreachable from the open internet. Revisit if that changes.

## References

- `ssh2::Session::userauth_pubkey_file` — <https://docs.rs/ssh2/0.9.6/ssh2/struct.Session.html#method.userauth_pubkey_file>
- OpenSSH `authorized_keys` options — <https://man.openbsd.org/sshd#AUTHORIZED_KEYS_FILE>
- `internal-sftp` — <https://man.openbsd.org/sftp#internal-sftp>
