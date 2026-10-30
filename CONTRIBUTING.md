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

Use [the reference Nix environment](docs/development-nix.md). Install Nix
with `nix-command` and `flakes` enabled. macOS desktop builds also require Xcode
and its accepted license. Rust 1.85.0, both macOS Rust targets, Node.js 22 and
the remaining development tools are supplied by Nix.

```sh
git clone https://github.com/M0okz/warpgatesh.git
cd warpgatesh
nix develop
cargo build --workspace --locked
```

For interactive companion development, run `npm ci` and `npm run tauri dev`
from `apps/warpgatesh-companion` inside that shell. Manual Cargo/npm workflows
remain available with an equivalent toolchain.

## Required checks

Run these from the repository root before submitting a pull request:

```sh
nix flake check
nix run .#check
nix run .#build-site
```

On macOS, also validate the complete universal companion, bundled CLI/agent
and disk image:

```sh
nix run .#build-macos
```

`nix run .#check` runs workspace formatting, tests and Clippy. On macOS it also
runs frontend checks, sidecar/updater tests, and native companion formatting,
tests and Clippy. CI uses the same commands and pinned inputs.

For changes to the macOS update/relaunch flow, run `npm run test:relaunch` from
`apps/warpgatesh-companion` inside `nix develop` in a graphical macOS session. This test creates two
temporary bundles signed ad hoc with the hardened runtime, replaces the first
with a different executable, and verifies that a visible window opens after the
old process exits. It uses a separate bundle identifier and does not load user
profiles, install agents, or modify `/Applications/WarpgateSH.app`. Release
signing, notarization, and the updater download/signature check still require
their release validations.

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
