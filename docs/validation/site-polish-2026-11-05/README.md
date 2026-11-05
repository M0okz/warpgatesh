# WarpgateSH website — design engineering pass, 5 November 2026

Requested follow-up to the Picta pass using [emil-design-eng](https://github.com/emilkowalski/skills/tree/main/skills/emil-design-eng). Scope: the landing page and seven generated guides. The product identity, authentic app captures, screenshot manifests and application version remain unchanged.

| Before | After | Why |
| --- | --- | --- |
| Copy controls: 44 × 36 px on desktop, 44 × 40 px on mobile, 9.76 px labels | 76 × 44 px controls, 12 px labels and stable confirmation width | Easier activation and readable feedback |
| Old confirmation timers can clear a newer copy result | Each copy restarts its own confirmation; concurrent requests on the same control are ignored | Feedback corresponds to the latest action |
| Clipboard failure only announced at the bottom of the page | Persistent Retry state and manual-copy guidance next to the command, also announced | The recovery action stays near the failure |
| 2 px horizontal overflow at a 768 px viewport | Decorative glow stays within the tablet container | Fixes the source of overflow without clipping content |
| Dense two-column companion section on tablet | Stacked copy and full-width app capture at 701–900 px | Preserves text readability and makes the app visible |
| Mobile metadata as small as 10.08 px | Important metadata at least 12 px | Compatibility and download context remain readable |
| Internal migration details interrupt the product explanation | Short explanation of native updates and the background/manual choice | Focuses the page on decisions useful to visitors |
| Mobile menu remains open across viewport changes | Reset on breakpoint change, Escape and outside-click dismissal | Menu state follows its visible layout |
| Mobile navigation hidden without JavaScript | Navigation stays expanded; inactive menu/copy controls are hidden | Documentation and downloads remain reachable |
| Skip/section links lack focusable destinations; all anchors scroll smoothly | Focusable main/section destinations and immediate anchor navigation | Keyboard users land on the intended content |
| Long documentation code blocks cannot be reached by keyboard | Overflowing code blocks/tables become focusable on resize | Horizontal content remains available without a mouse |
| Generic press animation also applies to keyboard interaction | Short 150 ms pointer feedback, 0.98 press scale, no focused-keyboard animation or reduced-motion transition | Feedback stays quick and respects input/accessibility preferences |
| Illustrative terminal contents repeat fictional data in the reading order | Illustration remains visible with its caption; decorative internals hidden from assistive technology | Keeps reading focused on the product and real instructions |

## Verification

- Reference build: `nix run .#build-site`, including checks of 8 pages and 187 local links/assets.
- Chromium: all 8 pages at 320, 375, 768 and 1440 px; no page overflow or runtime errors.
- 152 browser checks covering page responses, initial assets, skip destinations, mobile menu, keyboard navigation, exact clipboard contents, repeated copy, clipboard denial, FAQ, reduced motion and navigation without JavaScript.
- Visual review of desktop/mobile hero, mobile command controls and tablet companion capture. Evidence stored beside this note. `browser_checks.py` can repeat the campaign against a local preview or the public site with `--url` and `--output`; Python Playwright and Chromium are QA-only dependencies.
- A complete VoiceOver, touch-device and cross-browser campaign is not claimed. No system accessibility or privacy settings are changed.

Tracking: WarpgateSH project (24), Kanban view (96), task #46, medium priority (2), Doing (64) → Done (65) after validation and delivery. No new dates or colors.
