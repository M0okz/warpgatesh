# SwiftUI screenshot provenance

Captured on 31 October 2026 from the running native macOS application in
`apps/warpgatesh-macos`, using its `--demo` mode and native app screenshot API.
These are application screenshots, not rendered HTML or generated mockups.

The fifteen images show the approved interface and real native controls. Every
profile, hostname, URL, counter and diagnostic entry comes from fictional demo
fixtures; no live infrastructure data or tokens are included. The English UI
labels are captured as displayed. Twelve screenshots were refreshed from the v0.1.23 source build. The three unchanged profile/sign-in/token sheets retain their v0.1.22 captures; `capturedVersion` identifies these entries in the manifest. The native SwiftUI
interface is distributed from v0.1.18.

`manifest.json` records dimensions and SHA-256 checksums. The project README,
`docs/desktop-app.md` and website share these exact files. The site build copies
them to `dist/assets/swiftui/`; do not maintain a second set of screenshots.

To refresh, build the native preview, quit an existing connected preview, launch
with `--demo`, verify the Demo badge and `example.com` profile, and
capture each section at the same 881 × 713 point window size. Confirm that no
live profiles, private hostnames or personal paths appear before publication.

The preferences captures include General, Synchronization, System and the Light
and Black variants. The original Warpgate theme is used for the other screens.

The three new sheets (`profile-add`, `profile-login`, `token-renewal`) retain
their actual native sheet dimensions. The login capture is a labeled simulation,
not evidence of authentication against a real instance or identity provider.

The v0.1.23 `manual-mode` and `preferences-manual` captures show the result of choosing Later in the first-launch sheet. Enabling background synchronization is simulated; no LaunchAgent or personal data is touched.
