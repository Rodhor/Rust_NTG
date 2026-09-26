# API Bind Address — Exposure and Credential Delegation

> **Status: Reconsider — not currently relevant, revisit if the frontend moves.**
> The API is loopback-only today. This doc records the tripwire.

## The Decision (2026-09-26): bind to loopback

`APP_BIND_ADDR=127.0.0.1:8080` and it stays there. Loopback is the app's entire
access-control story: no API auth, no TLS, because only processes on this machine
can reach the socket.

## Open Contradiction

Two statements cannot both hold:

- the API binds `127.0.0.1`, **and**
- a frontend runs on a *different internal server*.

`127.0.0.1` accepts connections from the local machine only. A browser or client
on another internal host cannot reach it. One of the following must be true, and
which one is not yet decided:

1. The frontend is always **served from this machine** (internal servers only
   host its source or build artifacts), or
2. When the frontend moves, the API also moves to a machine that serves it.

If the answer is ever "the frontend is on another box, so I widened the bind",
that is the trigger for this doc — and for real auth on the API.

## Why Widening the Bind Is Not a Small Change

The API holds the SFTP credential and performs operations on an
**internet-facing** server on the caller's behalf. It is a credentialed proxy.

With `0.0.0.0`:

- Anyone on the internal network can drive the outside SFTP server using our
  credentials, with our `authorized_keys` restrictions as the only remaining
  control.
- There is no authentication, so there is no way to tell a legitimate CLI or
  Task Scheduler job from anyone else who reached the port.
- The frontend becomes an attack surface: anything running in it, including
  browser-side script, inherits API authority.

Loopback-only keeps all of this closed. Widening it means adding auth, deciding
on a transport, and deciding which client is allowed to do which operation.

## Tripwire

**If `APP_BIND_ADDR` is ever changed from `127.0.0.1`, the following become
required in the same change** — not as a follow-up:

- API authentication (at minimum, a shared secret in a header; the Windows Task
  Scheduler job is a legitimate client and needs a way to present it)
- Transport protection if the frontend is served over anything but loopback
- A decision on which operations a given client may perform — in particular,
  anything that writes or deletes on the outside server

## Related

- [sftp-auth-key.md](sftp-auth-key.md) — the other half of the credential story.
  Server-side `command="internal-sftp -d /incoming"` is the mitigation that holds
  even if this API is later exposed.
