# Contributing to WarpgateSH

Thank you for helping improve WarpgateSH. Bug reports, compatibility feedback,
documentation fixes, tests, and focused pull requests are welcome.

WarpgateSH is an unofficial community client. Server-side Warpgate issues
belong in the upstream [Warpgate repository](https://github.com/warp-tech/warpgate).

## Before opening a change

- Search existing issues and pull requests.
- Open an issue before a large feature or architectural change.
- Keep pull requests focused on one user-visible problem.
- Never include real API tokens, SSH keys, private hostnames, diagnostic
  archives, signing material, or other personal infrastructure data.

Security vulnerabilities must be reported privately through
[GitHub Security Advisories](https://github.com/M0okz/warpgatesh/security/advisories/new)
instead of a public issue.

## Development setup

Requirements:

- Rust 1.85 or newer;
- Node.js 22 and npm for the graphical companion;
- macOS 13 or newer for the complete desktop application and LaunchAgent flow.

Build the Rust workspace:

```sh
git clone https://github.com/M0okz/warpgatesh.git
cd warpgatesh
cargo build --workspace
```

Build the companion:

```sh
cd apps/warpgatesh-companion
npm ci
npm run build
```

## Required checks

Run these before submitting a pull request:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

cd apps/warpgatesh-companion
npm ci
npm run build
npm run test:sidecars
npm run test:updater-manifest
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Add or update tests for behavior changes. Documentation-only changes do not
need synthetic code tests, but all Markdown links and commands should still be
checked.

## Project boundaries

- `warpgatesh` remains the primary interface.
- Connections are delegated to the system OpenSSH client.
- The background agent is the single writer of synchronized state.
- User-owned SSH configuration must remain untouched outside the managed
  include directive.
- API tokens belong in the platform secret store and must never enter logs,
  fixtures, generated configuration, commits, or issue attachments.
- macOS is the supported desktop target until the daily beta is validated;
  Linux work should preserve the shared Rust behavior.

Read the [architecture decisions](docs/adr/) before changing one of these
boundaries. A deliberate change should include a new or updated ADR.

## Pull requests

Describe the user problem, the chosen behavior, validation performed, and any
known limitation. Include screenshots for visible companion changes and avoid
mixing unrelated formatting or refactors into the same pull request.

Maintainers publish signed macOS artifacts using the private process described
in [docs/releasing-macos.md](docs/releasing-macos.md). Contributors do not need
Apple signing credentials for normal builds and tests.
