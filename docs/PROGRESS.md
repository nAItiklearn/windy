# Windy progress

Last updated: 2026-10-30, against `main` at `e5c7c2a`. `main` is GitHub's default branch. `feat/bootstrap` is deleted.
**v0.1.0 is tagged and published** (tag at `465f032`, Release published 13:04 UTC). The website is
deployed to production at <https://windy-beryl.vercel.app> (Vercel project `naitiklearns-projects/windy`).

## Verified, and where the evidence is

| What | Evidence |
|---|---|
| Rust unit tests | `cargo test -p windy-core -p windy-platform` on `0a864a1`: 78 + 3 passed, 0 failed |
| UI and website behaviour (demo mode) | 16 Playwright tests (11 popup, 5 website) pass locally on `0a864a1` |
| Real app, Windows 11, release build | Clipboard capture of copied text, the global hotkey, popup, arrow-key selection, pinning, search, the settings window (after the fix), the app's own "Clear all". `docs/screenshots/` are captures of those windows; method in `scripts/perf/README.md` |
| Start-up, latency, memory, CPU | `PERF.md` (one machine, empty history) |
| Page loads | The WebView2 debug probe shows both pages loading with no failed requests and no console errors (after the favicon fix) |
| Website, local | Lighthouse 100 on all four categories, desktop and mobile; LCP 223 ms, CLS 0.00 (`PERF.md`) |
| Dependencies | `cargo audit`: no vulnerabilities, 5 transitive warnings; `npm audit`: 0 (`SECURITY-REVIEW.md`) |
| CI | Read from the public GitHub API on 2026-10-30: `ci.yml` succeeded on `0a864a1` (all six jobs), on `465f032` (the tagged commit) and on `main` at `e5c7c2a` |
| Release | The dry run (`workflow_dispatch` on `0a864a1`) and the `v0.1.0` tag run both succeeded. The Release holds the four installers and `SHA256SUMS.txt`; the checksums in that file match GitHub's own digests of the hosted files (I did not hash the downloaded bytes myself). The site's three `releases/latest/download/...` links resolve |
| Windows installer | The owner installed the dry-run artifact: it installed cleanly, launched and worked normally (owner-reported, not independently observed) |

ADR-001 gate: popup latency and idle CPU pass. Idle RAM was 103.4 MB; the owner raised the budget from 80 MB to **120 MB**
(`DECISIONS.md`, dated note in `ADR-001-stack.md`). The headroom is small.

## Not verified, and known limitations

- **Never exercised in the real app**: selecting an item (copy back, and paste), `Alt+Backspace` delete, the tray icon and its
  menu, image and file items, ignore rules and the password-manager exclusion, pausing, "clear on quit", changing the hotkey,
  settings persistence. Some of the logic behind these may be covered by the core unit tests and the UI by the demo-mode
  tests (which tests cover which behaviour was not audited), but the real app was not driven through any of them. An earlier
  version of this file wrongly said the tray and delete had been checked.
- **Platforms**: only Windows 11 (build 26300) has run the app. macOS and Linux are compiled in CI and nothing more. Windows 10
  is untested. Linux is X11 and XWayland only. macOS and Linux report no foreground app, so app-based ignore rules do nothing there.
- **Website and README claims are now labelled for what is unverified.** Windows 10 and macOS 12 or newer say "should work, not
  yet verified". Linux says X11 and XWayland; the old FAQ claim of Wayland data-control support contradicted `DECISIONS.md` and
  is gone (native Wayland is a known limitation, with no date promised). Speed is stated as about 20 ms with an empty history,
  and unmeasured with a full one. The theme claim is limited to the system light/dark setting (seen in dark mode only), and says
  the accent colour is fixed, not the system's. "No network requests" is stated as by design and based on code review, not a
  packet capture. The password-manager exclusion and app ignoring are marked Windows-only. Generic adjectives ("fast, private")
  remain in the page meta description and the bundle description.
- **Installers**: only the Windows installer has been installed and run (by the owner, from the dry-run artifact). The macOS
  `.dmg`, the Linux AppImage and the `.deb` were built and published, but nobody has installed or run them. All local builds
  use the GNU/MinGW toolchain (linker warning: `.rsrc merge failure: multiple non-default manifests`); the published installers
  come from the MSVC build on `windows-latest`. They are unsigned.
- **The v0.1.0 Release page**: its notes are auto-generated (a list of pull requests, some titled just "fix") and carry none of
  the caveats the site and README now carry. The tag points at `465f032`; `main` has since changed only `README.md`,
  `site/index.html` and `docs/PROGRESS.md`, so the app code is identical, but the Release's source archives contain the README
  from before the claims were softened.
- **Third-party license notices** are not bundled with the installers. `NOTICE` tells readers to run `cargo about`. MIT and
  Apache-2.0 dependencies ask that their notices accompany binary distributions, and the usual practice is to ship a generated
  third-party licenses file. This is not legal advice.
- **Performance**: a first data point on one machine with an empty history. Search keystroke latency and any populated
  history are unmeasured; the app caps history at 999 items, so the ADR's 10,000-item condition cannot occur.
- **Security** (`SECURITY-REVIEW.md`, written with the author's own tooling, not an independent audit): no encryption at rest,
  no explicit Windows ACL on the history file, paste is a timed `Ctrl+V` that can land in the wrong window, releases are unsigned.
- **Not in v0.1**: launch at login, preview slide-out, item editor, hold-to-cycle hotkey, pin drag-reorder, window and status-item
  popup positions, encryption at rest.
- **Local state left by testing**: running the app created `%LOCALAPPDATA%\app.windy.clipboard` (history cleared through the
  app). The harness scripts are in `scripts/perf/`; they take over keyboard, mouse and clipboard while they run.

## Only the owner can do these

1. **Edit the v0.1.0 Release notes** (only you can; I have no write access to GitHub). Suggested content: the builds are
   unsigned so SmartScreen and Gatekeeper warn; the app has been developed and tested on Windows 11 only; Windows 10, macOS and
   Linux are unverified; Linux is X11 and XWayland only; the password-manager exclusion and app ignoring are Windows-only.
2. **Decide whether the tag stays on `465f032`.** For the app it makes no difference. Moving it means deleting a published
   tag and Release, which should only happen on your say-so.
3. **Install the macOS and Linux assets, or say plainly that they are untested.** They are published next to the tested
   Windows one.
4. **Domain and Vercel settings**: the site is live on the `windy-beryl.vercel.app` alias (the plain `windy.vercel.app` name was
   taken). Choose a custom domain in the Vercel project settings if you want one. The Vercel project is connected to this
   GitHub repository, so every push to `main` redeploys production and other branches get previews that sit behind a Vercel
   login. Disconnect it under the project's Git settings if you do not want that. The download links work.
5. **Contact**: there is no email address anywhere. The site and `SECURITY.md` point to GitHub Issues and GitHub's private
   vulnerability reporting. Decide whether to add an email, and enable private vulnerability reporting in the repository's
   security settings, otherwise the `SECURITY.md` link does not work.
6. **Code signing**: a Windows code-signing certificate, and an Apple Developer ID with notarization for macOS. Add the secrets
   to the repository. `release.yml` has no signing step today, so builds are unsigned and SmartScreen and Gatekeeper will warn.
7. **Test on hardware I do not have**: Windows 10, macOS, Linux on X11 and on Wayland.
8. **Remove the caveats only after the testing**: take "should work, not yet verified" off Windows 10 and macOS 12, and lift the
   Linux and password-manager caveats, once each has been tested on real hardware. Native Wayland support is not promised.
9. **Open decisions**: encryption at rest in v1; a per-app paste key; the ADR gate's 10,000-item wording; which Linux distros to
   cover; whether to ship third-party license notices with the installers.
10. **Optional**: delete the merged branches on origin (`copilot/fix-failing-github-actions-job`, `docs/perf-and-screenshots`,
    `docs/ram-decision-and-perf-harness`, `feat/security-audit`, `fix/clear-shortcut`, `fix/favicon-500`,
    `fix/settings-window-blank`); run the 999-item measurement by copying the `tools/seed` output into the app data folder
    (this replaces any real history there); get `gh` installed (`winget install GitHub.cli` failed here with an empty log).

## Carried over from earlier notes (not revisited)

- Snowflake Cortex backend for the agents: `SNOWFLAKE_ACCOUNT` and `SNOWFLAKE_CORTEX_MODEL` were not set, and routing the
  agents there needs a small translating proxy. Until then the agents run on the session's own model.
- `rea` MCP: install is `npx rea-agents setup`, interactive, to be run by you from a neutral directory.

## Decisions

- Tauri 2 stack (ADR-001); idle-RAM budget 120 MB (see `DECISIONS.md`).
- Not ported: paste stack (disabled in Maccy: `multiSelectionEnabled = false`), App Intents, Vision OCR, Sparkle auto-update.
- `CanUploadToCloudClipboard=0` alone does not hide an item from local history (Microsoft docs say it only affects sync).
