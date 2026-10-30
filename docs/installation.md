# Install WarpgateSH on macOS

WarpgateSH currently supports macOS 13 or newer on Apple silicon and Intel
Macs. The public installation method is a signed and notarized universal DMG.

## Native macOS interface

From **v0.1.18**, the companion uses SwiftUI with a sidebar and six sections.
See the [native companion guide and screenshots](desktop-app.md). Earlier
versions use the previous interface; update before following the new labels.

## Install from the DMG

1. Open the [latest GitHub release](https://github.com/M0okz/warpgatesh/releases/latest).
2. Download `WarpgateSH_<version>_universal.dmg`.
3. Optionally download the matching `.sha256` file and verify it from the same
   directory:

   ```sh
   shasum -a 256 -c WarpgateSH_<version>_universal.dmg.sha256
   ```

4. Open the DMG and drag **WarpgateSH** into **Applications**.
5. Eject the DMG, then start WarpgateSH from `/Applications`.

Do not run the app permanently from the mounted DMG. The background agent must
refer to the stable application path in `/Applications`.

The first launch registers a per-user background agent. It runs independently
of the graphical window and keeps synchronizing after the window is closed.

## Install the command-line interface

The app already generates standard OpenSSH aliases, so `ssh <target>` works
without installing an extra SSH client. To also use the `warpgatesh` command:

1. Open **WarpgateSH → Préférences**.
2. Find **Commande du terminal**.
3. Select **Installer la commande warpgatesh** and approve the macOS administrator prompt.

The app creates `/usr/local/bin/warpgatesh` as a link to the signed executable
inside the application bundle. It does not modify your shell configuration or
`PATH`.

Verify the installation:

```sh
command -v warpgatesh
warpgatesh status
```

## Homebrew status

A Homebrew Cask is planned, but `warpgatesh` is not currently published in the
official Homebrew Cask repository. Until this page announces otherwise, use the
signed DMG and do not rely on `brew install --cask warpgatesh`.

## Updates

DMG installations check for new GitHub releases every six hours. You can
also check manually from **À propos → Vérifier les mises à jour**. WarpgateSH downloads
an update only after your confirmation and verifies its updater signature
before replacing the companion, CLI, and agent together.

If macOS rejects an official release, do not bypass Gatekeeper. Verify that the
download came from this repository, check the SHA-256 file, and report the
problem with the release version and the exact macOS message.

Continue with [Create a token and connect](getting-started.md).
