# Install WarpgateSH on Linux

Linux support currently covers the `warpgatesh` CLI and its per-user background
agent. The graphical companion remains macOS-only during this first Linux beta.

## Requirements

- a desktop session managed by systemd;
- OpenSSH client tools (`ssh`, `ssh-keyscan`, and `ssh-keygen`);
- `xdg-open` for opening the Warpgate token page;
- a Secret Service provider, such as GNOME Keyring or KWallet.

On `x86_64`, the current Homebrew/Linuxbrew runtime requires SSSE3. Proxmox
guests using the legacy `kvm64` CPU model need a newer model such as
`x86-64-v2-AES` before Homebrew itself can run.

WarpgateSH stores API tokens through the freedesktop Secret Service API. Tokens
are not written to its JSON configuration or generated SSH files.

## Install from the official tap

Add the project tap and install the latest stable version:

```sh
brew tap M0okz/warpgatesh
brew install M0okz/warpgatesh/warpgatesh
warpgatesh agent install
```

The last command writes
`~/.config/systemd/user/warpgatesh-agent.service`, enables it for the current
user, and starts it immediately. It never installs a system-wide daemon and
does not require `sudo`.

Verify the installation:

```sh
warpgatesh agent status
warpgatesh doctor
```

Then continue with [Create a token and connect](getting-started.md).

## Upgrade

```sh
brew update
brew upgrade M0okz/warpgatesh/warpgatesh
warpgatesh agent install
```

Running `agent install` again is safe. It refreshes the systemd unit when the
agent executable path changes and restarts the service when required.

## Uninstall

Stop and remove the user service before uninstalling the Formula:

```sh
warpgatesh agent uninstall
brew uninstall warpgatesh
```

This keeps profiles, Secret Service tokens, synchronized aliases, and local
diagnostics so a later reinstall can reuse them. Linux data is stored below
`~/.local/share/warpgatesh`, `~/.local/state/warpgatesh`, and
`~/.ssh/warpgatesh`.
