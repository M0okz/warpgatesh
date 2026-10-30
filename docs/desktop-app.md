# Native macOS companion

The WarpgateSH companion uses SwiftUI with the sidebar, translucent macOS
material, application logo, and hover feedback shown below. It remains optional:
the synchronization agent and OpenSSH access work independently of its window.

**Availability:** the native interface is available in **v0.1.18 and later**.
The screenshots are actual captures of the running SwiftUI application in its
isolated demo mode. Every profile, target, counter and diagnostic entry shown
is fictional. The interface currently uses French labels.

## Access

![SwiftUI Access screen with demo targets, profile count and search](assets/swiftui/access.png)

Check the agent state, synchronized target count, configured profiles, and the
age of the last successful synchronization. Use **Synchroniser maintenant**
for an immediate sync, or **⌘K** to search by name, alias or profile. Selecting
a target in the connected app opens its qualified alias in the terminal.
The last successful snapshot remains available if a later synchronization fails.

## Profiles

![SwiftUI Profiles screen with a fictional homelab profile](assets/swiftui/profiles.png)

Use **Ajouter un profil**, enter the local name, instance URL and personal API
token, then select **Vérifier le profil**. Compare the displayed SSH host-key
fingerprints with a trusted source. Enable **J’ai vérifié ces empreintes par un
canal de confiance** and select **Enregistrer** only after they match.

**Options SSH** lets you override a detected hostname or port. If the keys change
between inspection and saving, enrollment is refused and must be verified again.
Existing profiles offer **Renouveler le jeton**, **Définir par défaut** and a
confirmed **Supprimer** action. Tokens stay in the platform secret store.

## Preferences

![SwiftUI Preferences screen with synchronization and terminal settings](assets/swiftui/preferences.png)

Choose a synchronization interval, a default profile, and whether the companion
opens at login. The agent is independent of that preference. Under **Commande
du terminal**, **Installer la commande warpgatesh** exposes the bundled CLI.
Existing conflicting commands are preserved. **Enregistrer les préférences**
applies changed settings through the agent.

The built-in uninstaller remains under **Application**. It requires typing
`DÉSINSTALLER`; removal of profiles, tokens and managed SSH data is optional.

## Synchronization

![SwiftUI Synchronization screen with demo agent state and last snapshot](assets/swiftui/synchronization.png)

Inspect the agent state, configured interval, last successful snapshot and
synchronization alerts. Follow a profile-related alert to **Profils** when a
token or SSH key requires attention. Closing the window leaves the agent running.

## Logs

![SwiftUI Logs screen showing fictional diagnostic file totals before export](assets/swiftui/logs.png)

Review the local file list, event counts, total size and seven-day retention
before selecting **Exporter les diagnostics nettoyés…**. Export creates a local
sanitized ZIP; it never uploads anything. Review the archive before sharing it.

## About

![SwiftUI About screen with version and update controls](assets/swiftui/about.png)

Check the installed version and update status. DMG installations check for new
releases every six hours; **Vérifier les mises à jour** requests a manual check.
An available signed update requires confirmation before replacing the app, CLI
and agent together. Homebrew installations instead show the upgrade command.
The version shown corresponds to the v0.1.18 source build used for these captures.

## Menu bar, keyboard and accessibility

The menu-bar companion shows agent status and counts and provides shortcuts to
the main window, synchronization, profiles and preferences. **⌘,** opens
Preferences; **Escape** dismisses sheets. Native controls have accessibility
labels, hover motion respects Reduce Motion, and the sidebar becomes opaque
when Reduce Transparency is enabled. Full assistive-technology coverage still
requires manual verification before publication.

## Run from source

See the [native companion development guide](../apps/warpgatesh-macos/README.md).
Its preview script starts in isolated demo mode by default. The public signed
DMG remains available from the [latest release](https://github.com/M0okz/warpgatesh/releases/latest).


## Notifications

**Availability:** these settings are part of the upcoming release after v0.1.18.
They are available in the development preview; the v0.1.18 screenshots above
show the earlier preferences layout.

Open **Preferences → General → Desktop notifications** to enable macOS alerts.
macOS asks for permission the first time. If permission is denied, use **macOS
settings…** to change it. **Send a test notification** checks delivery.

You can choose which alerts to receive:

- **Expired or expiring tokens:** rejected authentication opens the profile list.
  For advance reminders, enter a known expiry date under **Profiles → Token expiry
  reminder…**. Reminders occur within seven days, within one day, and after expiry.
  WarpgateSH does not guess the date of an existing token. Successfully replacing
  the token clears its old reminder date.
- **Persistent synchronization failures:** an alert opens diagnostics when the
  agent reports repeated failures or overdue synchronization.
- **WarpgateSH updates:** an alert opens About when a client update is available.
- **Warpgate server releases:** optional checks compare each selected instance's
  observed version with the latest stable public Warpgate release. Use **Instances
  to monitor** to exclude profiles. Checks run every six hours and send no instance
  URL, username or token to GitHub. The notification opens the official release
  notes; it does not update the server.

An ongoing incident is notified once and becomes eligible again after recovery.
Each update is notified once per version. Notifications continue with the window
closed while WarpgateSH remains in the menu bar. Quitting the companion stops its
notifications; the independent synchronization agent continues running.

The interface is available in French and English. The same settings are under
**Préférences → Général → Notifications du bureau** in French.
