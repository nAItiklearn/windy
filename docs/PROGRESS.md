# Windy progress

Last updated: 2026-10-30, against `main` at `0a864a1`. `main` is GitHub's default branch. `feat/bootstrap` is deleted.
No tag, release, deploy or installer exists.

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
| CI | Read from the public GitHub API on 2026-10-30: `ci.yml` succeeded on `001a62e` and `ad578f7`. Confirm the latest run yourself |

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
- **Website claims that nothing has tested**: Windows 10, macOS 12 or newer, and the FAQ's statement that Wayland works with
  the data-control protocol (`DECISIONS.md` says v0.1 Linux is X11 and XWayland only). "Makes no network requests" rests on
  code review and having no updater, not on a packet capture.
- **Installers**: `release.yml` has never been run. The NSIS, DMG, AppImage and deb outputs, the version-free renaming step,
  and the macOS universal build are unverified. All local builds use the GNU/MinGW toolchain (linker warning: `.rsrc merge
  failure: multiple non-default manifests`); the installers will use MSVC on `windows-latest`.
- **Performance**: a first data point on one machine with an empty history. Search keystroke latency and any populated
  history are unmeasured; the app caps history at 999 items, so the ADR's 10,000-item condition cannot occur.
- **Security** (`SECURITY-REVIEW.md`, written with the author's own tooling, not an independent audit): no encryption at rest,
  no explicit Windows ACL on the history file, paste is a timed `Ctrl+V` that can land in the wrong window, releases are unsigned.
- **Not in v0.1**: launch at login, preview slide-out, item editor, hold-to-cycle hotkey, pin drag-reorder, window and status-item
  popup positions, encryption at rest.
- **Local state left by testing**: running the app created `%LOCALAPPDATA%\app.windy.clipboard` (history cleared through the
  app). The harness scripts are in `scripts/perf/`; they take over keyboard, mouse and clipboard while they run.

## Only the owner can do these

1. **Confirm CI** on the latest `main` run has finished green.
2. **Dry-run the release workflow**: Actions, "release", Run workflow on `main`. A manual run builds the installers as
   artifacts only and creates no Release. Install and try them, including on a Mac and a Linux machine.
3. **Tag and publish**: `git tag v0.1.0` then `git push origin v0.1.0`. A `v*` tag runs the `publish` job, which creates the
   GitHub Release with `SHA256SUMS.txt`. Check that repo Actions settings allow the workflow its `contents: write`
   permission. The site's download links use `releases/latest`, which only resolves once a release exists.
4. **Deploy the website**: connect the repository to a host (the config is `vercel.json` with `outputDirectory: site`) and set
   the domain. Nothing deploys automatically.
5. **Contact**: there is no email address anywhere. The site and `SECURITY.md` point to GitHub Issues and GitHub's private
   vulnerability reporting. Decide whether to add an email, and enable private vulnerability reporting in the repository's
   security settings, otherwise the `SECURITY.md` link does not work.
6. **Code signing**: a Windows code-signing certificate, and an Apple Developer ID with notarization for macOS. Add the secrets
   to the repository. `release.yml` has no signing step today, so builds are unsigned and SmartScreen and Gatekeeper will warn.
7. **Test on hardware I do not have**: Windows 10, macOS, Linux on X11 and on Wayland.
8. **Decide what the site may claim** (Wayland, Windows 10, macOS 12) until those platforms are tested.
9. **Open decisions**: encryption at rest in v1; a per-app paste key; the ADR gate's 10,000-item wording; which Linux distros to cover.
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
