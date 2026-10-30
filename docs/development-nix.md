# Reference development and build environment

Nix is the reference environment for local development, CI and macOS release
builds. `flake.lock` pins Nixpkgs and rust-overlay; `flake.nix` selects Rust
1.85.0 (the minimum supported version), rustfmt, Clippy, Node.js 22, npm,
Python, zsh and native Linux dependencies. Both Apple Rust targets are included
on macOS. OpenSSH is declared for configuration tests; GitHub CLI and jq are provided for
release tasks. No global rustup or Node installation is required. On macOS,
proxy directories containing a host `rustup` are removed from the shell PATH so
Tauri does not validate Nix's targets against an unrelated Rustup toolchain.
`RUSTUP_TOOLCHAIN` also points to the Nix toolchain so Cargo's rustfmt/Clippy
proxies in `CARGO_HOME/bin` cannot select a different compiler version.

## Setup

Install Nix using <https://nix.dev/install-nix>. Enable its command and flake
interfaces in `~/.config/nix/nix.conf` if your installer has not enabled them:

```ini
experimental-features = nix-command flakes
```

On macOS, install Xcode and accept its license. Desktop builds use Apple's
installed SDK, clang, lipo, codesign and hdiutil. The shell sets the deployment
target to macOS 13.0; it does not install or select Xcode. The build host itself
must meet the pinned Nixpkgs release’s macOS requirements. Consequently, pinning
Nix does not pin Xcode or guarantee byte-identical signed desktop artifacts.

Run all commands below from the repository root:

```sh
nix develop
cargo build --workspace --locked
```

Cargo/npm remain the language build tools. Their manifests and lockfiles remain
required. Ordinary Cargo/npm commands can also be used without Nix when the
corresponding toolchain and native dependencies have been installed manually.

On macOS, the shell disables release debug stripping for build dependencies
(proc macros). This avoids the [misaligned Mach-O string pool bug](https://github.com/rust-lang/rust/issues/157750)
when older Rust tooling is used on macOS 27. It does not disable stripping of
shipped application executables.

## Commands

| Command | Output or checks |
| --- | --- |
| `nix develop` | Interactive development shell |
| `nix build` | CLI and agent in `result/bin/`, built by Nix |
| `nix flake check` | Evaluate the flake, build and test the CLI/agent package |
| `nix run .#check` | Rust formatting, tests and Clippy; on macOS also frontend, sidecar and native companion checks |
| `nix run .#build-macos` | Unsigned universal `.app` and DMG, bundled CLI/agent checks and DMG verification |
| `nix run .#release-macos` | Universal signed release build with updater artifacts; requires release credentials |
| `nix run .#build-site` | Static website build and link checks |

Sandbox isolation follows the local Nix daemon configuration.

The CLI/agent package and shell are defined for Apple silicon/Intel macOS and
ARM64/x86-64 Linux. The graphical companion is built only on macOS. The package
does not install or start the synchronization agent. Existing runtime platform
requirements, including system OpenSSH and a native secret store, still apply.

The macOS app commands deliberately build in the checkout, outside the Nix
sandbox: Tauri packaging and Apple signing/notarization need native host tools
and, for releases, external services. The unsigned validation command enables
neither notarization nor updater signing. Release secrets remain process
variables and must never be written to Nix expressions or the Nix store.

Desktop outputs are in
`apps/warpgatesh-companion/src-tauri/target/universal-apple-darwin/release/bundle/`.
The website output is in `apps/warpgatesh-site/dist/`.

## Updating and cleaning

Update pinned inputs deliberately with `nix flake update`, then run
`nix flake check`, `nix run .#check`, `nix run .#build-site` and, on a Mac,
`nix run .#build-macos`. Review the lockfile diff before committing. A Rust
version change also requires editing the selection in `flake.nix`; keep the
minimum version documented in `Cargo.toml` consistent with compatibility tests.

`result` is only a link into the Nix store and can be removed after inspecting
the package. Cargo target directories, frontend `dist`, Tauri `binaries` and
`node_modules` remain ignored generated outputs. Preserve local build caches
unless you intentionally want a clean rebuild. Nix store garbage collection is
a machine-wide maintenance operation, separate from cleaning this project.
