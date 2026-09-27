# API Bind Address — Exposure and Credential Delegation

> **Status: In force. Revised 2026-09-27 for the Linux deployment target.**
> The API is still loopback-only, but the reason has changed. The tripwire below
> **fired** when the app moved to a Linux server, and is now **satisfied at the
> nginx proxy** rather than being hypothetical.

## The Decision: the app binds loopback, nginx is the trust boundary

`APP_BIND_ADDR=127.0.0.1:8080` and it stays there.

This is now a *different* decision from the 2026-09-26 one, and the difference
matters. Previously loopback was sufficient because everything ran on one
machine and there was nothing else to reach the socket. That is no longer true:
the operator's browser is on Windows and the app is on a Linux server, so
something **must** be able to reach the app across a network.

The resolution is that **the proxy is the network-facing surface and the app is
not**:

```
Windows browser ──HTTPS──▶ nginx (Linux) ──HTTP, 127.0.0.1──▶ rust-ntg
                          ▲
                          └── authenticates the client, terminates TLS
```

`127.0.0.1:8080` is therefore still correct, and now means something stronger than
before: the app is not merely "unexposed", it is **unreachable by anything except
the proxy on the same host**. A LAN scanner finds an HTTPS endpoint owned by
nginx, and nothing else.

## The Tripwire Fired, and How It Was Satisfied

The tripwire says: *if `APP_BIND_ADDR` is ever changed from `127.0.0.1`, then
authentication, transport protection and a per-client authorisation decision all
become required **in the same change**.*

Moving the app to a Linux server reachable from a Windows browser is exactly that
trigger. So the requirements are now real rather than hypothetical, and they are
met as follows:

| Requirement | Where it is met |
| --- | --- |
| Transport protection | nginx terminates TLS. The browser never speaks plain HTTP to the app. |
| API authentication | nginx rejects unauthenticated requests before they reach the app. |
| Per-client authorisation | The only legitimate client is the operator's own browser session. |
| Credentialed-proxy containment | The app holds the SFTP and ERP credentials and stays on loopback, so those credentials are not reachable by anything that merely found the port. |

**The bind does not change.** The tripwire is satisfied by *not widening the
bind* and putting the authenticated surface in front of it. That is the cheapest
way to meet the requirement, and it is why `APP_BIND_ADDR` still says loopback.

## Rejected: binding to the LAN address

"Browser reaches the server by IP, no proxy" was considered and **rejected**. It
is the one option that violates the tripwire rather than satisfying it:

- The API holds the SFTP credential and drives an **internet-facing** server on
  the caller's behalf. It is a credentialed proxy. On `0.0.0.0` anyone on the
  network can drive the outside SFTP server using our credentials.
- No authentication means no way to distinguish the operator's browser from
  anything else that reached the port.
- Plain HTTP means the session is observable, and the app is on a different host
  from the browser now, so this is a real network rather than a loopback socket.
- The UI inherits API authority: anything running in the browser page, including
  browser-side script, can call the API.

If nginx is unavailable and a direct path is genuinely wanted, use an **SSH
tunnel** — `ssh -L 8080:127.0.0.1:8080 server`, then browse
`http://127.0.0.1:8080`. That preserves the invariant, because the socket is
still loopback and traffic is still encrypted. A raw `0.0.0.0` bind does not.

## Why This Is Still A Security Decision, Not Plumbing

nginx is easy to misconfigure, and the failure mode is silent: an
`auth_basic` block in a server block that never includes it leaves a working,
wide-open endpoint that returns 200s. Two consequences:

- **The nginx config is part of the security boundary**, and should be reviewed
  as carefully as anything in `src/sftp/connection.rs`. The host-key pinning
  check there is the same *kind* of decision.
- Authentication is the *only* thing standing between a network scanner and the
  SFTP credential. If it is ever dropped, the bind must go back to loopback in
  the same change.

## Related

- [deployment.md](deployment.md) — the decision that made the proxy mandatory.
- [frontend.md](frontend.md) — the UI is same-origin with the API behind nginx,
  so there is no CORS and no cross-host credential delegation.
- [sftp-auth-key.md](sftp-auth-key.md) — the other half of the credential story.
  Server-side `command="internal-sftp -d /incoming"` is the mitigation that holds
  even if this API is later exposed.
