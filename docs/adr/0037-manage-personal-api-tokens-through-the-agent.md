# 0037 — Manage personal API tokens through the agent

Status: accepted; included from WarpgateSH 0.1.22.

## Decision

Offer sign-in to the official Warpgate page inside an ephemeral WebKit window.
The instance URL must use HTTPS (loopback HTTP is allowed for local tests).
Password, SSO and MFA remain controlled by the instance. No password is collected
by the native interface, no DOM scraping is used, and native API requests never
follow redirects. Providers that forbid embedded authentication can use the
existing manual personal-token flow.

After authentication, the companion checks `/@warpgate/api/info` using only the
instance's `warpgate-http-session` cookie. A profile is enrolled only after the
user approves its SSH host fingerprints and saves it. The agent creates a token
with `POST /@warpgate/api/profile/api-tokens`, validates the returned identity,
and stores the secret and its server-reported expiry in the native secret store.
The ordinary primary token entry remains compatible with existing CLI clients.

The agent owns credential writes and rotates managed tokens before expiry,
independently of the companion window. It requests a maximum lifetime of 30 days,
bounded by the instance's `max_api_token_duration_seconds`, with a safety margin.
A server policy below six minutes is rejected rather than promising unreliable
background renewal. An expired or revoked credential requires a fresh sign-in;
there is no unauthenticated refresh grant.

## Failure and ownership rules

A native-secret-store journal records the creation intent before any API mutation,
then the candidate before replacing the primary credential. Lost responses are
reconciled using an unpredictable, exact token label. Recovery never searches by
a generic app prefix. Only app-created token IDs are eligible for revocation;
imported credentials are never revoked. Owned old credentials receive a five-minute
grace period after durable replacement. Manual replacement clears management.

A nonblocking cross-process lock serializes configuration mutations. Network
credential operations run outside the agent's IPC status loop. Failed automatic
renewals preserve the last usable token, surface an alert and retry with bounded
exponential backoff. A non-secret status cache contains expiry, renewal preference
and generic failures only. Pending authorization and candidate secrets remain in
the native secret store; ordinary successful sign-in cookies are ephemeral.

Disabling renewal prevents future creation and cancels pending replacement when
it has not already committed. Removing a profile or deleting app data also removes
its local credential journal. This does not promise to revoke every server token
when offline; Warpgate remains the place to review and revoke personal tokens.

## Validation

Rust tests cover the HTTP issuance/deletion contract, redirect isolation, malformed
and oversized responses, first enrollment, imported-token ownership, wrong identity,
concurrent mutations, failures at journal/primary commits, recovery, cancellation,
old-token grace and retry backoff. Swift tests cover origin and cookie isolation,
localization and an in-memory demo flow. Real SSO/MFA behavior depends on the
instance and identity provider and still requires an interactive instance check.
