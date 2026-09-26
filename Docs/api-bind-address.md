# API Bind Address — Exposure and Credential Delegation

> **Status: In force. The contradiction this doc used to carry is resolved.**
> The API is loopback-only. The tripwire below is unchanged and still applies.

## The Decision (2026-09-26): bind to loopback

`APP_BIND_ADDR=127.0.0.1:8080` and it stays there. Loopback is the app's entire
access-control story: no API auth, no TLS, because only processes on this machine
can reach the socket.

## Contradiction: Resolved

This doc previously recorded an open contradiction:

- the API binds `127.0.0.1`, **and**
- a frontend runs on a *different internal server*.

**Resolved 2026-09-26 by [deployment.md](deployment.md).** Everything runs on the
Windows VM — backend, database, files and UI. There is no second machine serving
a frontend, so `127.0.0.1` is reachable by everything that needs it and by
nothing else. See option 1 of the original list: the frontend is served from this
machine.

The UI is additionally confirmed to be server-rendered from the same binary
([frontend.md](frontend.md)), so it is same-origin by construction — no CORS, no
cross-host credential delegation, nothing for a browser-side script to inherit.

The tripwire below therefore stays exactly as written. It is now a guard against
a future change rather than a live ambiguity.

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

The realistic triggers, should they ever occur:

- the app is moved to a Linux server, or
- the UI is ever run from another host (a hosted frontend, a phone, a second
  workstation).

Neither is planned. If either becomes real, this tripwire fires before the bind
is widened, not after.

## Related

- [deployment.md](deployment.md) — the decision that resolved the contradiction.
- [frontend.md](frontend.md) — why the UI is same-origin with the API.
- [sftp-auth-key.md](sftp-auth-key.md) — the other half of the credential story.
  Server-side `command="internal-sftp -d /incoming"` is the mitigation that holds
  even if this API is later exposed.
