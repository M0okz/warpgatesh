# Warpgate API compatibility

Reviewed on 2026-10-02 against Warpgate **0.27.5**, **0.28.6** and **0.29.1**.
The 0.29.0 release warns about breaking API changes. The user API contracts
consumed by WarpgateSH remain compatible in these tags, so this review does
not require a client endpoint or authentication change.

The source review is backed by synthetic contract tests and an operational
spot check on a deployed 0.29.1 instance on 2026-10-02/03: authenticated
synchronization, unchanged RSA/Ed25519 host keys and a browser-approved
OpenSSH connection succeeded. This does not certify every server configuration,
a fresh SSO login, MFA enrollment or the new administrator approval policy.
Compatibility remains capability-based, as described in
[ADR 0025](adr/0025-detect-warpgate-api-capabilities.md).

## Contracts used by the client

| Contract | What WarpgateSH needs | 0.29.1 assessment |
| --- | --- | --- |
| Personal token | `X-Warpgate-Token` request header | Unchanged |
| `GET /@warpgate/api/info` | `username`, optional `version`, `external_hosts.ssh` or `external_host`, `ports.ssh` | Fields and types retained |
| `GET /@warpgate/api/targets` | JSON array with `id`, `name`, `kind`; SSH kind is `Ssh` | Shape and SSH kind retained |
| Personal token browser page | `/@warpgate/#/profile/api-tokens` | Route retained |

The deprecated `minimize_password_login` field is removed in 0.29.
WarpgateSH never used it. Added MFA fields and admin permissions do not affect
the small subset of `/info` the client reads. Extra target descriptions,
groups and database fields are ignored. The user target list continues to be
filtered by server-side access rules.

WarpgateSH does not consume the admin session or recording APIs affected by
the user/target session split. It checks HTTP status before decoding successful
responses, so authentication failures and server errors do not depend on the
error body's wording or serialization.

## Regression coverage

The [contract tests](../crates/warpgatesh-runtime/tests/warpgate_api_contract.rs)
use synthetic fixtures built from the OpenAPI schemas committed in each
upstream tag. [Fixture provenance](../crates/warpgatesh-runtime/tests/fixtures/README.md)
records their source commits and limits.

They verify:

- both user API paths and the personal token header;
- username, version and external SSH endpoint extraction;
- SSH filtering with all seven currently declared protocol kinds;
- synchronization, stable target IDs and browser authentication configuration;
- preservation of the snapshot, managed SSH config and saved profiles when a
  subsequent target response is malformed, incomplete, or returns 401, 403 or
  500.

Run them with:

```sh
cargo test -p warpgatesh-runtime --test warpgate_api_contract
```

The normal workspace CI runs these integration tests on macOS and Linux.
These fixtures do not run the upstream server, enforce its authorization
logic, or exercise database migrations, MFA, session approval or SSH host-key
import.

## SSH scan comments during an upgrade

System `ssh-keyscan` can include the server banner in `known_hosts` comments.
Warpgate 0.29.1 changes the SSH library banner. Earlier WarpgateSH clients
incorrectly compared those comments as key material and could report changed
host keys even though the approved keys were identical. The comparison now
ignores comment lines while still rejecting changed key material. Regression
tests cover both cases.

For an installed client without this fix, compare the actual approved and
presented key fingerprints through a trusted administrative path before
refreshing the managed scan metadata. A banner change alone never authorizes
a new key. Do not discard pins or disable host-key verification.

## Before upgrading a server to 0.29

1. Back up the database, configuration and data directory, including existing
   SSH host keys. Keep a tested restoration path: switching an image back is
   not sufficient to undo database migrations.
2. Review every admin API integration separately. The 0.29.0 release names
   Terraform provider **1.2.0** and Kubernetes operator **0.4.11** as compatible
   versions. Update provider constraints and lock files deliberately and
   inspect a plan against the upgraded test server before applying changes.
3. Exercise `warpgatesh sync` with an existing non-admin personal token on a
   test instance. Compare its visible SSH targets, target IDs, username and
   advertised SSH endpoint with the pre-upgrade snapshot.
4. Verify that existing pinned SSH host keys still match. In 0.29, Warpgate
   imports the existing key files into its database. Investigate unexpected
   fingerprint changes before approving replacement keys.
5. Connect with the system OpenSSH client using the configured authentication
   method. Test browser/SSO authentication, MFA enrollment policy and any new
   admin session approval requirements independently. User web authentication
   and administrator session approval are separate server-controlled steps.
6. Revalidate locally maintained server patches against the new version,
   especially patches to browser approval behavior.

## Reviewing the next release

Compare the user OpenAPI schema and handler source for `/info` and `/targets`,
the token security scheme, HTTP route mounting and the personal token page.
Add a new synthetic fixture only after confirming the upstream contract and
record its immutable source commit. Unknown extra fields are tolerated;
missing required client fields must fail without replacing saved state.

For a live test instance, the user schema is exposed at
`/@warpgate/api/openapi.json`. Inspect admin API consumers independently;
compatibility of the user API does not establish admin API compatibility.

## Upstream references

- [Warpgate 0.29.0 release notes](https://github.com/warp-tech/warpgate/releases/tag/v0.29.0)
- [Warpgate 0.29.1 release notes](https://github.com/warp-tech/warpgate/releases/tag/v0.29.1)
- [0.29.1 user OpenAPI schema](https://github.com/warp-tech/warpgate/blob/54f93c807be2c161a94c0df764242849125161a8/warpgate-web/src/gateway/lib/openapi-schema.json)
- [0.29.1 info handler](https://github.com/warp-tech/warpgate/blob/54f93c807be2c161a94c0df764242849125161a8/warpgate-protocol-http/src/api/info.rs)
- [0.29.1 targets handler](https://github.com/warp-tech/warpgate/blob/54f93c807be2c161a94c0df764242849125161a8/warpgate-protocol-http/src/api/targets_list.rs)
- [0.29.1 token security scheme](https://github.com/warp-tech/warpgate/blob/54f93c807be2c161a94c0df764242849125161a8/warpgate-protocol-http/src/api/auth_scheme.rs)
