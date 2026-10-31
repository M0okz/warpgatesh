# Native macOS companion

The WarpgateSH companion uses SwiftUI with the sidebar, translucent macOS
material, application logo, and hover feedback shown below. It remains optional:
the synchronization agent and OpenSSH access work independently of its window.

**Availability:** the native interface is available in **v0.1.18 and later**.
The screenshots are actual captures of the running SwiftUI application in its
isolated demo mode. Every profile, target, counter and diagnostic entry shown
is fictional. The refreshed v0.1.23 captures use English labels; the three unchanged profile/sign-in/token sheets retain their v0.1.22 captures, identified in the asset manifest. From v0.1.20 the
interface supports French, English and automatic language selection.

## First launch (v0.1.23 and later)

New installations explain background synchronization before installing an agent.
Choose **Enable synchronization** to keep SSH targets current and renew managed
tokens while the window is closed. **Later** saves your choice and keeps profile
management and **Sync now** available without installing a background service.
Automatic token renewal requires the agent. **Preferences → Synchronization**
lets you enable or disable it later. An existing active or already registered
agent skips this first-launch window, including on upgrades.

![Manual synchronization preferences with an enable button, after choosing Later](assets/swiftui/preferences-manual.png)

![Access remains available in manual mode without a background agent](assets/swiftui/manual-mode.png)

## Access

![SwiftUI Access screen with demo targets, profile count and search](assets/swiftui/access.png)

Check the agent state, synchronized target count, configured profiles, and the
age of the last successful synchronization. Use **Synchronize now**
for an immediate sync, or **⌘K** to search by name, alias or profile. Selecting
a target in the connected app opens its qualified alias in the terminal.
The last successful snapshot remains available if a later synchronization fails.

## Profiles

![SwiftUI Profiles screen with a fictional homelab profile](assets/swiftui/profiles.png)

From **v0.1.22**, select **Add a profile**, enter the local name and HTTPS
instance URL, then **Sign in to Warpgate**. The official instance page opens
inside an ephemeral sign-in window; the instance still controls SSO and MFA.
Use **Check profile**, compare the SSH host-key fingerprints with a trusted
source, approve them, then **Save**. Only saving creates and stores the token.

![Profile enrollment and SSH approval in demo mode](assets/swiftui/profile-add.png)

The demo sign-in screen below simulates authentication only. Connected mode
loads your instance's official page. Providers that reject embedded sign-in can
use **Use an existing token**, which retains the manual secure-field flow.

![Isolated sign-in simulation; no real instance authentication](assets/swiftui/profile-login.png)

**SSH options** overrides a detected endpoint. Changed fingerprints between
inspection and saving are refused. Tokens remain in the native secret store.

### Managed token renewal

**Renew token → Renew through the API** creates an app-managed replacement
using your current valid token. Managed profiles show **Automatic renewal** and
the server's expiry date. The independent agent rotates the token before expiry,
with retries on failure, even when the companion is closed. Only app-created
tokens may be revoked after replacement; imported tokens are never revoked.

![Token renewal actions in the native demo app](assets/swiftui/token-renewal.png)

An expired or revoked token requires **Sign in to Warpgate again**. Authenticate
with the same identity as the profile. You can disable automatic renewal per
profile or import a token manually. See the [complete token lifecycle guide](getting-started.md#automatic-renewal-and-reconnecting)
for policy limits, ownership and manual-token behavior.

## Preferences

![SwiftUI Preferences screen with synchronization and terminal settings](assets/swiftui/preferences.png)

From **v0.1.20**, preferences have three tabs:

- **General**: choose Automatic, Français or English; select Warpgate, Light,
  Black or System appearance; configure [desktop notifications](#notifications).
  These local preferences save immediately. System follows macOS light/dark mode.
- **Synchronization**: set the synchronization interval and default profile.
- **System**: open at login, install the terminal command, read its guide or
  uninstall the application. Existing conflicting commands are preserved.

Select **Save preferences** to apply changes under Synchronization or System.
The independent agent continues working when the companion window is closed.

![Synchronization preferences in the running app](assets/swiftui/preferences-sync.png)

![System preferences and terminal command guide in the running app](assets/swiftui/preferences-system.png)

### Light and Black appearances

The original **Warpgate** theme remains the default. Light and Black provide
explicit alternatives; System switches between them with the macOS appearance.

![Light appearance with desktop notification controls](assets/swiftui/preferences-light.png)

![Black appearance with desktop notification controls](assets/swiftui/preferences-black.png)

## Synchronization

![SwiftUI Synchronization screen with demo agent state and last snapshot](assets/swiftui/synchronization.png)

Inspect the agent state, configured interval, last successful snapshot and
synchronization alerts. Follow a profile-related alert to **Profiles** when a
token or SSH key requires attention. Closing the window leaves the agent running.

## Logs

![SwiftUI Logs screen showing fictional diagnostic file totals before export](assets/swiftui/logs.png)

Review the local file list, event counts, total size and seven-day retention
before selecting **Export sanitized diagnostics…**. Export creates a local
sanitized ZIP; it never uploads anything. Review the archive before sharing it.

## About

![SwiftUI About screen with version and update controls](assets/swiftui/about.png)

The app logo, identity, version and update card are centered. Links use cyan
with an underline and color change on hover. Check the installed version and update status. DMG installations check for new
releases every six hours; **Check for updates** requests a manual check.
An available signed update requires confirmation before replacing the app, CLI
and agent together. Homebrew installations instead show the upgrade command.
The version shown corresponds to the v0.1.23 source build used for these captures.

## Menu bar, keyboard and accessibility

From **v0.1.22**, the menu-bar item uses the monochrome WarpgateSH logo. Closing
the window hides its Dock icon while leaving the companion in the menu bar;
reopening restores the window and Dock. **Quit WarpgateSH** exits the companion.
The Dock logo has native margins, and sidebar labels and icons are larger.

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

**Availability:** desktop notifications are included from **v0.1.20**.

Open **Preferences → General → Desktop notifications** to enable macOS alerts.
macOS asks for permission the first time. If permission is denied, use **macOS
settings…** to change it. **Send a test notification** checks delivery.

You can choose which alerts to receive:

- **Expired or expiring tokens:** rejected authentication opens the profile list.
  Managed profiles use their server-reported expiry and report automatic renewal
  failures. For an imported token without a known expiry, enter a date under
  **Profiles → Token expiry reminder…**. Reminders occur within seven days,
  within one day and after expiry. WarpgateSH never guesses an imported token’s
  expiry; successful replacement clears an old manual reminder.
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
