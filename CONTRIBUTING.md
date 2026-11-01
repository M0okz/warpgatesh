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

The macOS production interface is SwiftUI in `apps/warpgatesh-macos` (Swift 6.2+).
See its README for an isolated preview and native tests. The private backend is
`crates/warpgatesh-companion`; it runs without a GUI framework. Cargo and Swift
can also be used with an equivalent manually installed toolchain. Node/npm are
used only for the website.

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
runs Swift tests/formatting and the standalone pipe protocol smoke test. The
universal bundle check verifies all four executables, minimum macOS, resource
loading and absence of Nix-store dependencies.

Changes to the updater require legacy Minisign compatibility, native Sparkle
verification/installation checks, and both old-to-new and new-to-next updates. Use disposable fixtures and the official signed artifacts for release
validation; personal installations are not destructive test fixtures.

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
