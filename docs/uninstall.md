# Uninstall WarpgateSH

Use the built-in uninstaller so the app can distinguish its own files from your
personal SSH configuration.

## Uninstall and keep local data

1. Open **WarpgateSH → Préférences**.
2. Select **Désinstaller…**.
3. Leave **Supprimer aussi mes données** unchecked.
4. Enter `DÉSINSTALLER` and continue.

WarpgateSH stops and unregisters the background agent, removes the CLI link
owned by the app, and moves the application to the Trash. Profiles, Keychain
tokens, the last synchronized snapshot, diagnostics, and managed SSH files are
kept for a later reinstall.

## Uninstall and delete all WarpgateSH data

Follow the same steps and enable **Supprimer aussi mes données** before confirming. In
addition to uninstalling the app, WarpgateSH removes:

- its personal API tokens from the macOS Keychain;
- `~/Library/Application Support/WarpgateSH/`;
- `~/Library/Logs/WarpgateSH/`;
- `~/.ssh/warpgatesh/`;
- only the `Include ~/.ssh/warpgatesh/config` directive it manages in
  `~/.ssh/config`.

Other `Host` entries and files in `~/.ssh` are left untouched. Deleting the data
cannot be undone; a future installation will require a new profile enrollment
and host-key approval.

## If the app cannot open

Move `/Applications/WarpgateSH.app` to the Trash in Finder, then remove only a
CLI link that resolves inside that application bundle. Avoid deleting an
unrelated `warpgatesh` executable installed by another package manager.

If you also want to remove all data manually, back up `~/.ssh/config` first and
delete only the paths listed above plus the single managed `Include` line.

For a damaged installation where ownership is unclear, open a GitHub issue
before deleting SSH or Keychain data.
