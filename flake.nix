{
  description = "WarpgateSH development tools and builds";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      perSystem = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          inherit (pkgs) lib;
          toolchain = pkgs.rust-bin.stable."1.85.0".minimal.override {
            extensions = [ "clippy" "rustfmt" ];
            targets = lib.optionals pkgs.stdenv.isDarwin [
              "aarch64-apple-darwin" "x86_64-apple-darwin"
            ];
          };
          tools = [ toolchain pkgs.nodejs_22 pkgs.python3 pkgs.pkg-config pkgs.zsh pkgs.perl pkgs.git ];
          libraries = lib.optionals pkgs.stdenv.isLinux [ pkgs.openssl ];
          rustPlatform = pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; };
          cli = rustPlatform.buildRustPackage {
            pname = "warpgatesh";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                let relative = lib.removePrefix (toString ./. + "/") path;
                in relative == "Cargo.toml" || relative == "Cargo.lock"
                  || relative == "crates" || lib.hasPrefix "crates/" relative;
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config pkgs.perl pkgs.zsh ];
            buildInputs = libraries;
            strictDeps = true;
            # API contract tests use local mock servers, including in the sandbox.
            __darwinAllowLocalNetworking = true;
            meta.mainProgram = "warpgatesh";
          };
          shell = (if pkgs.stdenv.isDarwin then pkgs.mkShellNoCC else pkgs.mkShell) {
            packages = tools;
            buildInputs = libraries;
            shellHook = lib.optionalString pkgs.stdenv.isDarwin ''
              # Desktop distribution uses Apple's installed SDK and signing tools.
              export SDKROOT="$(/usr/bin/xcrun --sdk macosx --show-sdk-path)"
              export MACOSX_DEPLOYMENT_TARGET=13.0
              # Rust's debug stripping can produce dylibs rejected by macOS 27.
              # Keep build-time proc macros loadable; shipped binaries are unaffected.
              export CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP=none
              export CC=/usr/bin/clang
              export CXX=/usr/bin/clang++
              export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/clang
              export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=/usr/bin/clang
            '';
          };
          task = name: {
            type = "app";
            meta.description = "WarpgateSH ${name}";
            program = toString (pkgs.writeShellScript "warpgatesh-${name}" ''
              set -euo pipefail
              root="$(${pkgs.git}/bin/git rev-parse --show-toplevel)"
              cd "$root"
              exec ${pkgs.nix}/bin/nix develop "$root" --command bash "$root/nix/tasks.sh" ${name} "$@"
            '');
          };
        in {
          devShells.default = shell;
          packages = { default = cli; warpgatesh = cli; };
          checks.cli = cli;
          apps = {
            check = task "check";
            build-macos = task "build-macos";
            release-macos = task "release-macos";
            build-site = task "build-site";
          };
        };
    in {
      devShells = forAllSystems (system: (perSystem system).devShells);
      packages = forAllSystems (system: (perSystem system).packages);
      checks = forAllSystems (system: (perSystem system).checks);
      apps = forAllSystems (system: (perSystem system).apps);
    };
}
