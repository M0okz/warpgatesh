# SwiftUI screenshot provenance

Captured on 30 October 2026 from the running native macOS application in
`apps/warpgatesh-macos`, using its `--demo` mode and native app screenshot API.
These are application screenshots, not rendered HTML or generated mockups.

The six images show the approved interface and real native controls. Every
profile, hostname, URL, counter and diagnostic entry comes from fictional demo
fixtures; no live infrastructure data or tokens are included. The French UI
labels are captured as displayed. The source build keeps version 0.1.17 until
release preparation; this does not make the SwiftUI preview part of that
published release.

`manifest.json` records dimensions and SHA-256 checksums. The project README,
`docs/desktop-app.md` and website share these exact files. The site build copies
them to `dist/assets/swiftui/`; do not maintain a second set of screenshots.

To refresh, build the native preview, quit an existing connected preview, launch
with `--demo`, verify the Démonstration badge and `example.com` profile, and
capture each section at the same 860 × 620 point window size. Confirm that no
live profiles, private hostnames or personal paths appear before publication.
