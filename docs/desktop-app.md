# Native macOS companion — SwiftUI preview

The next WarpgateSH companion uses SwiftUI with the sidebar, translucent macOS
material, application logo, and hover feedback shown below. It remains optional:
the synchronization agent and OpenSSH access work independently of its window.

**Availability:** this interface is implemented on `main` for the next release.
The latest published **v0.1.17** DMG still contains the previous companion.
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
The screenshot's version refers to the development build, not a new published release.

## Menu bar, keyboard and accessibility

The menu-bar companion shows agent status and counts and provides shortcuts to
the main window, synchronization, profiles and preferences. **⌘,** opens
Preferences; **Escape** dismisses sheets. Native controls have accessibility
labels, hover motion respects Reduce Motion, and the sidebar becomes opaque
when Reduce Transparency is enabled. Full assistive-technology coverage still
requires manual verification before publication.

## Try the preview from source

See the [native companion development guide](../apps/warpgatesh-macos/README.md).
Its preview script starts in isolated demo mode by default. The public signed
DMG remains available from the [latest release](https://github.com/M0okz/warpgatesh/releases/latest).
