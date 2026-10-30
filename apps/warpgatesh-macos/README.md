# Native macOS companion

SwiftUI application, macOS 13+, Swift 6.2+. The Rust companion remains the local
backend. Build or launch the isolated demo from the repository's Nix environment:

```sh
nix develop --command sh apps/warpgatesh-macos/scripts/run.sh
```

Use `--live` only to connect the preview to real local profiles. The normal demo
never reads credentials, changes SSH configuration or checks server releases.
The preview bundle is signed ad hoc so macOS can authorize local notifications.

## Preferences in the development version

These settings are new, unreleased changes after v0.1.18:

- **General**: automatic/French/English language; original Warpgate, Light, Black,
  or System appearance. System follows the macOS light/dark setting. Language and
  appearance update immediately, including the menu bar and dialogs.
- **Synchronization**: interval and default profile, saved through the existing
  backend. **System**: launch at login, CLI installation and documentation,
  application removal. These settings retain their explicit Save button.
- Local presentation settings persist in UserDefaults independently of backend
  configuration. Demo settings use `dev.warpgatesh.preview.settings` and never
  overwrite the installed application's presentation settings.

## Notifications

Notifications are opt-in. Enabling them requests the native macOS authorization;
denied permission is displayed with a link to System Settings. The test button
sends a real local notification after authorization. No push service is used.

The running companion evaluates the agent's existing persistent-error alerts
and client update state. Incidents are delivered once until resolution; updates
are delivered once per version. Clicking the action opens Profiles, Diagnostics,
About, or the official Warpgate release notes. The companion keeps monitoring
when its window is closed, while its menu-bar process is running. Quitting it
stops notifications; the independent Rust agent continues synchronizing.

Warpgate does not identify the expiry date of an already stored API token in the
responses used by this client. **Profiles → Token expiry reminder** accepts an
optional user-supplied date. Reminders occur within seven days, within one day,
and after expiry. No date is guessed. Rejected authentication is reported even
without a date. Successfully replacing a token clears its old expiry reminder.

Server release checks are optional per profile, every six hours, against the
public GitHub releases API. They compare stable numeric versions with the server
version observed during synchronization. No instance URL, username or token is
sent to GitHub. An optional release-check failure never blocks SSH or syncing.

## Validation

```sh
python3 apps/warpgatesh-macos/scripts/swift.py test --package-path apps/warpgatesh-macos
python3 apps/warpgatesh-macos/scripts/swift.py format lint --strict --recursive apps/warpgatesh-macos/Sources apps/warpgatesh-macos/Tests apps/warpgatesh-macos/Package.swift
```

Tests cover language resources, local persistence, appearance selection,
notification preferences and reminder thresholds, incident recovery and
version comparison, alongside the existing bridge contracts and stale-read
protection. Full VoiceOver verification and Intel hardware testing remain
manual checks.
